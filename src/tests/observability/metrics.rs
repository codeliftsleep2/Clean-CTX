// src/tests/observability/metrics.rs
//
// A-04: Tests for the metrics registry.

use crate::observability::metrics::{Counter, ErrorCategory, Gauge, Histogram, MetricsRegistry};

#[test]
fn test_histogram_new() {
    let h = Histogram::new(vec![10, 20, 30]);
    let snap = h.snapshot();
    assert_eq!(snap.bounds, vec![10, 20, 30]);
    assert_eq!(snap.total, 0);
    assert_eq!(snap.mean, 0.0);
}

#[test]
fn test_histogram_record() {
    let h = Histogram::new(vec![10, 20, 30]);
    h.record(5);
    h.record(15);
    h.record(25);
    h.record(35); // exceeds all bounds — goes into last bucket
    let snap = h.snapshot();
    assert_eq!(snap.total, 4);
    // Buckets: [0..10], [11..20], [21..30], overflow (last bucket catches all)
    assert_eq!(snap.counts, vec![1, 1, 2]);
    assert!(snap.mean > 0.0);
}

#[test]
fn test_histogram_latency_default() {
    let h = Histogram::latency_default();
    assert_eq!(h.bounds, vec![1, 5, 10, 25, 50, 100, 250, 500, 1000, 5000]);
}

#[test]
fn test_histogram_size_default() {
    let h = Histogram::size_default();
    assert_eq!(h.bounds.len(), 7);
    assert_eq!(h.bounds[0], 1024);
}

#[test]
fn test_counter() {
    let c = Counter::new();
    assert_eq!(c.snapshot(), 0);
    c.increment(1);
    assert_eq!(c.snapshot(), 1);
    c.increment(5);
    assert_eq!(c.snapshot(), 6);
}

#[test]
fn test_gauge() {
    let g = Gauge::new(10);
    assert_eq!(g.snapshot(), 10);
    g.set(25);
    assert_eq!(g.snapshot(), 25);
    g.add(5);
    assert_eq!(g.snapshot(), 30);
    g.add(-3);
    assert_eq!(g.snapshot(), 27);
}

#[test]
fn test_metrics_registry_new() {
    let reg = MetricsRegistry::new();
    let snap = reg.snapshot();
    assert_eq!(snap.total_compressions, 0);
    assert_eq!(snap.delta_hits, 0);
    assert_eq!(snap.cache_hits, 0);
    assert!(snap.errors.is_empty());
}

#[test]
fn test_metrics_registry_record_error() {
    let reg = MetricsRegistry::new();
    reg.record_error(ErrorCategory::IoError);
    reg.record_error(ErrorCategory::IoError);
    reg.record_error(ErrorCategory::ParseError);
    let snap = reg.snapshot();
    assert_eq!(snap.errors.len(), 2);
    // Should be sorted by count descending
    assert_eq!(snap.errors[0].0, "io_error");
    assert_eq!(snap.errors[0].1, 2);
    assert_eq!(snap.errors[1].0, "parse_error");
    assert_eq!(snap.errors[1].1, 1);
}

#[test]
fn test_metrics_registry_time() {
    let h = Histogram::latency_default();
    let result = MetricsRegistry::time(&h, || {
        std::thread::sleep(std::time::Duration::from_millis(10));
        42
    });
    assert_eq!(result, 42);
    let snap = h.snapshot();
    assert_eq!(snap.total, 1);
    assert!(
        snap.mean >= 10.0,
        "mean should be >= 10ms, got {}",
        snap.mean
    );
}

#[test]
fn test_metrics_registry_delta_counters() {
    let reg = MetricsRegistry::new();
    reg.delta_hits.increment(3);
    reg.delta_misses.increment(2);
    reg.cache_hits.increment(10);
    reg.cache_misses.increment(1);
    reg.total_compressions.increment(5);
    reg.total_deltas.increment(5);
    let snap = reg.snapshot();
    // Use the snapshot to verify counters were recorded
    assert_eq!(snap.delta_hits, 3);
    assert_eq!(snap.delta_misses, 2);
    assert_eq!(snap.cache_hits, 10);
    assert_eq!(snap.cache_misses, 1);
    assert_eq!(snap.total_compressions, 5);
    assert_eq!(snap.total_deltas, 5);
}

#[test]
fn test_metrics_snapshot_format_dashboard() {
    let reg = MetricsRegistry::new();
    reg.total_compressions.increment(100);
    reg.delta_hits.increment(80);
    reg.delta_misses.increment(20);
    reg.cache_hits.increment(500);
    reg.cache_misses.increment(50);
    reg.record_error(ErrorCategory::CbmTimeout);

    let snap = reg.snapshot();
    let dashboard = snap.format_dashboard();
    assert!(
        dashboard.contains("100"),
        "dashboard should show 100 compressions"
    );
    assert!(
        dashboard.contains("80"),
        "dashboard should show 80 delta hits"
    );
    assert!(
        dashboard.contains("500"),
        "dashboard should show 500 cache hits"
    );
    assert!(
        dashboard.contains("timeout"),
        "dashboard should show timeout errors"
    );
    assert!(
        dashboard.contains("Clean-CTX Metrics Dashboard"),
        "dashboard should have header"
    );
}

#[test]
fn test_histogram_concurrent_safety() {
    use std::sync::Arc;
    let h = Arc::new(Histogram::new(vec![10, 100]));
    let mut handles = Vec::new();
    for _ in 0..4 {
        let h = Arc::clone(&h);
        handles.push(std::thread::spawn(move || {
            for _ in 0..100 {
                h.record(50);
            }
        }));
    }
    for handle in handles {
        handle.join().unwrap();
    }
    let snap = h.snapshot();
    assert_eq!(snap.total, 400);
    assert!(snap.mean > 0.0);
}

#[test]
fn test_histogram_exponential() {
    let h = Histogram::latency_exponential();
    assert_eq!(h.bounds.len(), 15);
    assert_eq!(h.bounds[0], 1);
    assert_eq!(h.bounds[14], 16384);
    // Record values across the range
    h.record(0); // underflows to first bucket
    h.record(1);
    h.record(100);
    h.record(10000);
    h.record(20000); // exceeds max
    let snap = h.snapshot();
    assert_eq!(snap.total, 5);
}

#[test]
fn test_histogram_record_duration() {
    let h = Histogram::new(vec![10, 100]);
    h.record_duration(std::time::Duration::from_millis(50));
    let snap = h.snapshot();
    assert_eq!(snap.total, 1);
    assert!(snap.mean >= 50.0);
}

#[test]
fn test_error_category_as_str() {
    assert_eq!(ErrorCategory::CompressionFail.as_str(), "compression_fail");
    assert_eq!(ErrorCategory::DeltaApplyError.as_str(), "delta_apply_error");
    assert_eq!(ErrorCategory::CbmTimeout.as_str(), "cbm_timeout");
    assert_eq!(ErrorCategory::CbmQueryFail.as_str(), "cbm_query_fail");
    assert_eq!(ErrorCategory::IoError.as_str(), "io_error");
    assert_eq!(ErrorCategory::ParseError.as_str(), "parse_error");
    assert_eq!(ErrorCategory::Internal.as_str(), "internal");
}

#[test]
fn test_error_category_display() {
    assert_eq!(
        format!("{}", ErrorCategory::CompressionFail),
        "compression_fail"
    );
    assert_eq!(format!("{}", ErrorCategory::Internal), "internal");
}

#[test]
fn test_metrics_snapshot_serialize() {
    let reg = MetricsRegistry::new();
    let snap = reg.snapshot();
    let json = serde_json::to_string(&snap).unwrap();
    assert!(json.contains("compression_latency"));
    assert!(json.contains("delta_hits"));
}

#[test]
fn test_high_contention_histogram() {
    use std::sync::Arc;
    let h = Arc::new(Histogram::latency_exponential());
    let mut handles = Vec::new();
    for _ in 0..8 {
        let h = Arc::clone(&h);
        handles.push(std::thread::spawn(move || {
            for _ in 0..500 {
                h.record(rand_latency());
            }
        }));
    }
    for handle in handles {
        handle.join().unwrap();
    }
    let snap = h.snapshot();
    assert_eq!(snap.total, 4000);
    assert!(snap.mean > 0.0);
}

/// Generate a random-ish latency value for contention testing.
fn rand_latency() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as u64;
    (nanos % 5000) + 1
}
