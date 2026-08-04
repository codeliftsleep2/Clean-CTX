// src/observability/tracing.rs
//
// A-04: Structured tracing initialization and span helpers.
//
// Uses the `tracing` crate (already in Cargo.toml) with a
// `tracing-subscriber` layer that outputs structured JSON to stderr
// when `CLEAN_CTX_LOG_FORMAT=json` is set, or human-readable text
// by default.
//
// Key spans:
//   - `compress_file` — per-file compression with fidelity, extension
//   - `delta_compute` — delta computation with file alias, version
//   - `cbm_query` — CBM graph query with query type
//   - `workspace_scan` — workspace scan with file count
//   - `handler` — MCP handler dispatch with method name
//
// The subscriber is initialized once at server startup. If the
// `tracing` crate is not configured, all events are no-ops (zero
// overhead at runtime).

use tracing_subscriber::EnvFilter;

/// Writer that routes all tracing output to **stderr**.
///
/// **CRITICAL (MCP protocol):** `tracing_subscriber::fmt()` defaults to
/// stdout, which would corrupt the JSON-RPC 2.0 response stream that MCP
/// clients (Claude Code, VS Code, etc.) parse from the process's stdout.
/// All diagnostics must go to stderr; stdout is reserved exclusively for
/// `send_response()` payloads.
///
/// This is a named type (rather than an inline `std::io::stderr`) so the
/// regression test can assert at compile time that the writer is stderr —
/// if someone changes it to stdout, the test fails to compile.
#[derive(Debug, Clone, Copy)]
pub struct StderrWriter;

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for StderrWriter {
    type Writer = std::io::Stderr;

    fn make_writer(&'a self) -> Self::Writer {
        std::io::stderr()
    }
}

/// Initialize the tracing subscriber.
///
/// Called once at server startup. Configures:
///   - Log level from `CLEAN_CTX_LOG` env var (default: `info`)
///   - Output format from `CLEAN_CTX_LOG_FORMAT` env var
///     (`json` or `text`, default: `text`)
///   - Filtering via `CLEAN_CTX_LOG_FILTER` env var (e.g., `warn,clean_ctx=debug`)
///
/// All output is routed to **stderr** via [`StderrWriter`] — never stdout.
///
/// If the subscriber is already set (e.g., by a test harness), this
/// is a no-op.
pub fn init_tracing() {
    // Use EnvFilter for flexible filtering
    let filter = EnvFilter::try_from_env("CLEAN_CTX_LOG_FILTER")
        .unwrap_or_else(|_| {
            EnvFilter::try_from_env("CLEAN_CTX_LOG")
                .unwrap_or_else(|_| EnvFilter::new("info"))
        });

    let format = std::env::var("CLEAN_CTX_LOG_FORMAT").unwrap_or_default();

    match format.as_str() {
        "json" => {
            tracing_subscriber::fmt()
                .json()
                .with_writer(StderrWriter)
                .with_env_filter(filter)
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true)
                .try_init()
                .ok();
        }
        _ => {
            tracing_subscriber::fmt()
                .with_writer(StderrWriter)
                .with_env_filter(filter)
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true)
                .try_init()
                .ok();
        }
    }
}

/// Convenience macro for creating a tracing span with function name.
///
/// Usage:
/// ```ignore
/// let _span = observability_span!("compress_file", fidelity = %fidelity, ext = %extension);
/// ```
#[macro_export]
macro_rules! observability_span {
    ($name:expr $(, $key:ident = $val:expr)* $(,)?) => {
        tracing::info_span!($name, $($key = $val),*)
    };
}

/// Record a structured event at INFO level.
///
/// Usage:
/// ```ignore
/// observability_event!("compression_complete", file = %path, savings = savings_pct);
/// ```
#[macro_export]
macro_rules! observability_event {
    ($msg:expr $(, $key:ident = $val:expr)* $(,)?) => {
        tracing::info!($($key = $val),*, "{}", $msg)
    };
}

/// Record a structured event at WARN level.
#[macro_export]
macro_rules! observability_warn {
    ($msg:expr $(, $key:ident = $val:expr)* $(,)?) => {
        tracing::warn!($($key = $val),*, "{}", $msg)
    };
}

/// Record a structured event at ERROR level.
#[macro_export]
macro_rules! observability_error {
    ($msg:expr $(, $key:ident = $val:expr)* $(,)?) => {
        tracing::error!($($key = $val),*, "{}", $msg)
    };
}

#[cfg(test)]
#[path = "../tests/observability/tracing.rs"]
mod tests;