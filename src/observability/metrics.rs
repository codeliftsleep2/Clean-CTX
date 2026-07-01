// src/observability/metrics.rs
//
// A-04: Metrics registry for key operational signals.
//
// Provides counters, histograms, and gauges for:
//   - Compression latency (histogram, ms)
//   - Delta hit rate (counter: hits vs misses)
//   - Cache efficiency (counter: hits vs misses)
//   - CBM query latency (histogram, ms)
//   - Error rates by category (counter)
//   - File sizes processed (histogram, bytes)
//   - Active worker count (gauge)
//   - Queue depth (gauge)
//
// The registry is thread-safe (DashMap + atomics) and designed to be
// OTLP-exportable when an OpenTelemetry SDK is wired in. For now, it
// stores metrics in-memory and exposes them via the `context_stats`
// MCP tool and a `metrics_snapshot()` method.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use dashmap::DashMap;

/// A histogram bucket for latency/ size measurements.
/// Uses a fixed set of buckets (in milliseconds or bytes) for
/// O(1) insertion and O(buckets) snapshot.
#[derive(Debug)]
pub struct Histogram {
    /// Bucket upper bounds (inclusive). Sorted ascending.
    pub bounds: Vec<u64>,
    /// Per-bucket counts.
    pub counts: Vec<AtomicU64>,
    /// Total observations.
    pub total: Arc<AtomicU64>,
    /// Sum of all observed values (for mean calculation).
    pub sum: Arc<AtomicU64>,
}

impl Histogram {
    /// Create a new histogram with the given bucket bounds.
    /// Bounds must be sorted ascending.
    pub fn new(bounds: Vec<u64>) -> Self {
        let counts = bounds.iter().map(|_| AtomicU64::new(0)).collect();
        Self {
            bounds,
            counts,
            total: Arc::new(AtomicU64::new(0)),
            sum: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Default latency buckets (ms): 1, 5, 10, 25, 50, 100, 250, 500, 1000, 5000
    pub fn latency_default() -> Self {
        Self::new(vec![1, 5, 10, 25, 50, 100, 250, 500, 1000, 5000])
    }

    /// Exponential latency buckets (ms): powers of 2 from 1 to 16384.
    /// Provides better tail resolution for large file compression.
    pub fn latency_exponential() -> Self {
        Self::new(vec![1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384])
    }

    /// Default size buckets (bytes): 1K, 10K, 100K, 500K, 1M, 5M, 10M
    pub fn size_default() -> Self {
        Self::new(vec![1024, 10_240, 102_400, 512_000, 1_048_576, 5_242_880, 10_485_760])
    }

    /// Record a value into the histogram.
    pub fn record(&self, value: u64) {
        self.total.fetch_add(1, Ordering::Relaxed);
        self.sum.fetch_add(value, Ordering::Relaxed);
        for (i, bound) in self.bounds.iter().enumerate() {
            if value <= *bound {
                self.counts[i].fetch_add(1, Ordering::Relaxed);
                return;
            }
        }
        // Value exceeds all bounds — count in the last bucket
        if let Some(last) = self.counts.last() {
            last.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Record a `Duration` into the histogram (converts to milliseconds).
    pub fn record_duration(&self, duration: std::time::Duration) {
        self.record(duration.as_millis() as u64);
    }

    /// Snapshot the current histogram state.
    #[must_use]
    pub fn snapshot(&self) -> HistogramSnapshot {
        let counts: Vec<u64> = self.counts.iter().map(|c| c.load(Ordering::Relaxed)).collect();
        let total = self.total.load(Ordering::Relaxed);
        let sum = self.sum.load(Ordering::Relaxed);
        HistogramSnapshot {
            bounds: self.bounds.clone(),
            counts,
            total,
            mean: if total > 0 { sum as f64 / total as f64 } else { 0.0 },
        }
    }
}

/// A point-in-time snapshot of a histogram.
#[derive(Debug, Clone, serde::Serialize)]
pub struct HistogramSnapshot {
    pub bounds: Vec<u64>,
    pub counts: Vec<u64>,
    pub total: u64,
    pub mean: f64,
}

/// Normalized error categories for metrics.
///
/// Using an enum instead of free-form strings prevents unbounded
/// cardinality growth from error messages containing unique strings
/// (file paths, symbol names, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCategory {
    /// Compression pipeline failure.
    CompressionFail,
    /// Delta application failure.
    DeltaApplyError,
    /// CBM subprocess timeout.
    CbmTimeout,
    /// CBM query failure.
    CbmQueryFail,
    /// I/O error (file read/write).
    IoError,
    /// Parse error (JSON, config, etc.).
    ParseError,
    /// Internal/unexpected error.
    Internal,
}

impl ErrorCategory {
    /// Return a short, stable string key for this category.
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCategory::CompressionFail => "compression_fail",
            ErrorCategory::DeltaApplyError => "delta_apply_error",
            ErrorCategory::CbmTimeout => "cbm_timeout",
            ErrorCategory::CbmQueryFail => "cbm_query_fail",
            ErrorCategory::IoError => "io_error",
            ErrorCategory::ParseError => "parse_error",
            ErrorCategory::Internal => "internal",
        }
    }
}

impl std::fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A counter that can be incremented and snapped.
#[derive(Debug, Clone)]
pub struct Counter {
    value: Arc<AtomicU64>,
}

impl Counter {
    pub fn new() -> Self {
        Self { value: Arc::new(AtomicU64::new(0)) }
    }

    pub fn increment(&self, delta: u64) {
        self.value.fetch_add(delta, Ordering::Relaxed);
    }

    #[must_use]
    pub fn snapshot(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }
}

/// A gauge that holds a current value.
#[derive(Debug, Clone)]
pub struct Gauge {
    value: Arc<AtomicU64>,
}

impl Gauge {
    pub fn new(initial: u64) -> Self {
        Self { value: Arc::new(AtomicU64::new(initial)) }
    }

    pub fn set(&self, value: u64) {
        self.value.store(value, Ordering::Relaxed);
    }

    pub fn add(&self, delta: i64) {
        if delta >= 0 {
            self.value.fetch_add(delta as u64, Ordering::Relaxed);
        } else {
            self.value.fetch_sub(delta.unsigned_abs(), Ordering::Relaxed);
        }
    }

    #[must_use]
    pub fn snapshot(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }
}

/// Thread-safe metrics registry.
///
/// All metrics are stored in DashMap for concurrent access. The registry
/// is designed to be shared across the entire server via `Arc<MetricsRegistry>`.
#[derive(Debug)]
pub struct MetricsRegistry {
    /// Compression latency histogram (ms).
    pub compression_latency: Histogram,
    /// Delta computation latency histogram (ms).
    pub delta_latency: Histogram,
    /// CBM query latency histogram (ms).
    pub cbm_latency: Histogram,
    /// File size histogram (bytes).
    pub file_size: Histogram,
    /// Delta hit counter (successful delta computations).
    pub delta_hits: Counter,
    /// Delta miss counter (no baseline — first call).
    pub delta_misses: Counter,
    /// Cache hit counter (source_cache or baseline cache).
    pub cache_hits: Counter,
    /// Cache miss counter.
    pub cache_misses: Counter,
    /// Error counter by category.
    pub errors: DashMap<String, Counter>,
    /// Active worker gauge.
    pub active_workers: Gauge,
    /// Queue depth gauge.
    pub queue_depth: Gauge,
    /// Total compression operations.
    pub total_compressions: Counter,
    /// Total delta operations.
    pub total_deltas: Counter,
    /// Total CBM queries.
    pub total_cbm_queries: Counter,
    /// Total workspace scans.
    pub total_workspace_scans: Counter,
}

impl MetricsRegistry {
    /// Create a new metrics registry with default histograms.
    pub fn new() -> Self {
        Self {
            compression_latency: Histogram::latency_default(),
            delta_latency: Histogram::latency_default(),
            cbm_latency: Histogram::latency_default(),
            file_size: Histogram::size_default(),
            delta_hits: Counter::new(),
            delta_misses: Counter::new(),
            cache_hits: Counter::new(),
            cache_misses: Counter::new(),
            errors: DashMap::new(),
            active_workers: Gauge::new(0),
            queue_depth: Gauge::new(0),
            total_compressions: Counter::new(),
            total_deltas: Counter::new(),
            total_cbm_queries: Counter::new(),
            total_workspace_scans: Counter::new(),
        }
    }

    /// Record an error under the given category.
    pub fn record_error(&self, category: ErrorCategory) {
        let key = category.as_str().to_string();
        self.errors
            .entry(key)
            .or_insert_with(Counter::new)
            .increment(1);
    }

    /// Time a closure and record its duration in the given histogram.
    pub fn time<T>(histogram: &Histogram, f: impl FnOnce() -> T) -> T {
        let start = Instant::now();
        let result = f();
        let elapsed = start.elapsed().as_millis() as u64;
        histogram.record(elapsed);
        result
    }

    /// Take a snapshot of all metrics.
    pub fn snapshot(&self) -> MetricsSnapshot {
        let mut error_counts: Vec<(String, u64)> = self.errors
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().snapshot()))
            .collect();
        error_counts.sort_by(|a, b| b.1.cmp(&a.1));

        MetricsSnapshot {
            compression_latency: self.compression_latency.snapshot(),
            delta_latency: self.delta_latency.snapshot(),
            cbm_latency: self.cbm_latency.snapshot(),
            file_size: self.file_size.snapshot(),
            delta_hits: self.delta_hits.snapshot(),
            delta_misses: self.delta_misses.snapshot(),
            cache_hits: self.cache_hits.snapshot(),
            cache_misses: self.cache_misses.snapshot(),
            errors: error_counts,
            active_workers: self.active_workers.snapshot(),
            queue_depth: self.queue_depth.snapshot(),
            total_compressions: self.total_compressions.snapshot(),
            total_deltas: self.total_deltas.snapshot(),
            total_cbm_queries: self.total_cbm_queries.snapshot(),
            total_workspace_scans: self.total_workspace_scans.snapshot(),
        }
    }
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// A point-in-time snapshot of all metrics.
#[derive(Debug, Clone, serde::Serialize)]
pub struct MetricsSnapshot {
    pub compression_latency: HistogramSnapshot,
    pub delta_latency: HistogramSnapshot,
    pub cbm_latency: HistogramSnapshot,
    pub file_size: HistogramSnapshot,
    pub delta_hits: u64,
    pub delta_misses: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub errors: Vec<(String, u64)>,
    pub active_workers: u64,
    pub queue_depth: u64,
    pub total_compressions: u64,
    pub total_deltas: u64,
    pub total_cbm_queries: u64,
    pub total_workspace_scans: u64,
}

impl MetricsSnapshot {
    /// Format the snapshot as a human-readable string for the dashboard.
    pub fn format_dashboard(&self) -> String {
        let mut out = String::new();
        out.push_str("═══════════════════════════════════════════════════════════════\n");
        out.push_str("  Clean-CTX Metrics Dashboard\n");
        out.push_str("═══════════════════════════════════════════════════════════════\n\n");

        // Operations summary
        out.push_str(&format!("  Compressions:      {}\n", self.total_compressions));
        out.push_str(&format!("  Deltas:            {} (hits: {}, misses: {})\n",
            self.total_deltas, self.delta_hits, self.delta_misses));
        out.push_str(&format!("  CBM Queries:       {}\n", self.total_cbm_queries));
        out.push_str(&format!("  Workspace Scans:   {}\n", self.total_workspace_scans));
        out.push_str(&format!("  Cache:             {} hits, {} misses\n",
            self.cache_hits, self.cache_misses));
        out.push_str("\n");

        // Latency histograms
        out.push_str("── Latency (ms) ──\n");
        out.push_str(&format!("  Compression:       mean={:.1}ms  total={}\n",
            self.compression_latency.mean, self.compression_latency.total));
        out.push_str(&format!("  Delta:             mean={:.1}ms  total={}\n",
            self.delta_latency.mean, self.delta_latency.total));
        out.push_str(&format!("  CBM:               mean={:.1}ms  total={}\n",
            self.cbm_latency.mean, self.cbm_latency.total));
        out.push_str("\n");

        // File size distribution
        out.push_str("── File Sizes (bytes) ──\n");
        out.push_str(&format!("  Mean:              {:.0} bytes  total={}\n",
            self.file_size.mean, self.file_size.total));
        out.push_str("\n");

        // Resource gauges
        out.push_str("── Resources ──\n");
        out.push_str(&format!("  Active Workers:    {}\n", self.active_workers));
        out.push_str(&format!("  Queue Depth:       {}\n", self.queue_depth));
        out.push_str("\n");

        // Errors
        if !self.errors.is_empty() {
            out.push_str("── Errors by Category ──\n");
            for (category, count) in &self.errors {
                out.push_str(&format!("  {:<20} {}\n", category, count));
            }
            out.push_str("\n");
        }

        out.push_str("═══════════════════════════════════════════════════════════════\n");
        out
    }
}

#[cfg(test)]
#[path = "../tests/observability/metrics.rs"]
mod tests;