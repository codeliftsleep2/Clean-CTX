// src/observability/mod.rs
//
// A-04: Observability upgrade — structured tracing spans and metrics.
//
// Architecture:
//   - Tracing: `tracing` crate with structured spans for every major
//     operation (compression, delta, CBM queries, workspace scans).
//     The subscriber is configured once at server startup via `init_tracing()`.
//   - Metrics: `MetricsRegistry` with counters, histograms, and gauges
//     for key operational signals. Designed to be OTLP-exportable when
//     an OpenTelemetry SDK is wired in.
//   - Config: `ObservabilityConfig` in .clean-ctx.json controls
//     sampling rate, log level, and optional OTLP endpoint.
//
// The module is entirely opt-in. If no config is present, tracing outputs
// to stderr at INFO level and metrics are stored in-memory (accessible
// via the `context_stats` MCP tool).

pub mod metrics;
pub mod tracing;

pub use metrics::MetricsRegistry;
pub use tracing::init_tracing;