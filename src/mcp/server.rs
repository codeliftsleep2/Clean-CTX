// src/mcp/server.rs
//
// MCP server main loop: reads JSON-RPC requests from stdin, dispatches
// them via the thread-pool dispatcher, and writes responses to stdout.
//
// A-09 (FAANG architectural review): The original loop dispatched each
// request synchronously on the reader thread. A slow CBM query or large
// file compression would block ALL subsequent requests. The loop now
// enqueues each request to a thread-pool dispatcher, so the reader
// thread never blocks.
//
// Phase 1 (FAANG audit F-02): the previous loop used `BufRead::read_line`
// with no upper bound. A 4 GB line on stdin would OOM the process. The
// loop now caps each request at [`MAX_LINE_BYTES`] and answers with a
// JSON-RPC `-32600 Invalid Request` if a client tries to send a request
// that exceeds the cap.
//
// Phase 2 (FAANG audit F-05): the server now constructs a single
// [`McpState`] (path dict + cache + CleanCtxConfig) and threads it
// through the dispatch chain so tool handlers can consult the user's
// `exclude_patterns`, `fidelity_overrides`, and `type_aliases`.

use std::io::{self, BufRead};
use std::path::PathBuf;

use crate::config::CleanCtxConfig;
use crate::mcp::McpState;
use crate::mcp::dispatcher::Dispatcher;
use crate::protocol::send_response;
use crate::protocol::JsonRpcRequest;

/// Cached project root, resolved once per process lifetime.
/// Walks up from the executable's directory looking for `.clean-ctx.json`
/// or `Cargo.toml` to anchor all relative paths (config,, DB, debug log).
static PROJECT_ROOT: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();

/// Resolve the project root directory. Walks up from the executable's
/// location looking for `.clean-ctx.json` or `Cargo.toml`. Falls back
/// to `current_dir()` if nothing is found.
pub(crate) fn find_project_root() -> &'static PathBuf {
    PROJECT_ROOT.get_or_init(|| {
        // Start from the executable's directory (e.g. target/release/)
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));

        let mut current = exe_dir;
        loop {
            if current.join(".clean-ctx.json").exists() || current.join("Cargo.toml").exists() {
                return current;
            }
            if !current.pop() {
                break;
            }
        }
        // Fallback to CWD
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    })
}

/// Maximum size, in bytes, of a single JSON-RPC request line. Anything
/// larger is rejected with a `-32600` error rather than OOMing the
/// process. 16 MiB is comfortably above any legitimate request (the
/// largest `compress_code_context` arguments are a path + a few
/// keywords) while still keeping memory bounded.
pub(crate) const MAX_LINE_BYTES: usize = 16 * 1024 * 1024;

/// Run the MCP server loop, processing incoming JSON-RPC requests until
/// stdin is exhausted.
///
/// A-09: Uses a thread-pool Dispatcher so the stdin reader never blocks
/// on slow requests. Each parsed request is enqueued to a worker thread;
/// the reader immediately returns to reading the next line.
pub(crate) fn run() -> Result<(), Box<dyn std::error::Error>> {
    // A-04: Initialize structured tracing. Configures the tracing
    // subscriber from environment variables:
    //   CLEAN_CTX_LOG       — log level (default: info)
    //   CLEAN_CTX_LOG_FORMAT — output format (json or text, default: text)
    //   CLEAN_CTX_LOG_FILTER — fine-grained filter (e.g., warn,clean_ctx=debug)
    // This is a no-op if the subscriber is already set (e.g., in tests).
    crate::observability::init_tracing();

    // Eagerly initialise the BPE engine. A failure here (e.g. the BPE
    // data file is missing or unreadable) used to take the server
    // down on the *first* compression call. We now surface it at
    // startup with a clear error and a non-zero exit code.
    if let Err(e) = crate::analytics::bpe_or_init() {
        eprintln!("[clean-ctx] fatal: {}", e);
        std::process::exit(2);
    }

    // F-05: load the project config and bundle it into the
    // per-session state. The config is no longer bound to `_` and
    // thrown away; tool handlers consult it via `state.config`.
    let project_root = find_project_root();
    eprintln!("[clean-ctx] Project root: {}", project_root.display());
    let config = CleanCtxConfig::load(project_root);
    let state = McpState::new(config);

    // A-PRODUCTION: Wrap state in the production-grade dispatcher.
    let dispatcher = Dispatcher::new(state);

    let stdin = io::stdin();
    let mut handle = stdin.lock();

    while let Some(line_result) = read_request_line(&mut handle) {
        let line = match line_result {
            Ok(line) => line,
            Err(OversizeRequest) => {
                send_response(&serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": serde_json::Value::Null,
                    "error": {
                        "code": -32600,
                        "message": format!(
                            "Request too large (limit: {} bytes)",
                            MAX_LINE_BYTES
                        )
                    }
                }));
                continue;
            }
        };

        if line.is_empty() {
            continue;
        }

        // A-PRODUCTION: Parse the request and enqueue it to the dispatcher.
        // The closure captures the parsed request data and dispatches
        // it on a worker thread. The reader thread returns immediately.
        if let Ok(req) = serde_json::from_str::<JsonRpcRequest>(&line) {
            let req_for_handler = req.clone();
            if let Err(e) = dispatcher.spawn(&req, move |state| {
                crate::mcp::router::dispatch(req_for_handler, state);
            }) {
                eprintln!("[clean-ctx] ERROR: Failed to enqueue request: {}", e);
                send_response(&serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": serde_json::Value::Null,
                    "error": { "code": -32603, "message": "Internal error: queue full" }
                }));
            }
        }
    }

    // A-09: Wait for all queued work to complete before exiting.
    // The Dispatcher's drop impl (via rayon) will block until all
    // spawned tasks finish.
    eprintln!("[clean-ctx] Stdin exhausted, waiting for pending work...");
    drop(dispatcher);

    Ok(())
}

/// Sentinel returned by [`read_request_line`] when the next line on the
/// stream exceeds [`MAX_LINE_BYTES`]. The caller should send a
/// `-32600` JSON-RPC error and resume reading the following line.
#[derive(Debug)]
struct OversizeRequest;

/// Read the next newline-terminated request from `handle`, with a
/// per-line byte cap of [`MAX_LINE_BYTES`]. Returns:
///
/// - `Some(Ok(line))` for a normal line (line never includes the
///   trailing newline);
/// - `Some(Err(OversizeRequest))` if the line exceeded the cap — the
///   caller MUST drain the rest of that line before issuing the next
///   read or it will be mis-attributed to the next request;
/// - `None` on clean EOF.
fn read_request_line<R: BufRead>(handle: &mut R) -> Option<Result<String, OversizeRequest>> {
    let mut buffer = String::new();
    let mut total: usize = 0;

    loop {
        buffer.clear();
        // `read_line` returns the number of bytes read (including the
        // trailing newline, if any). A `0` return means EOF.
        //
        // Note: `read_line` may return the *entire* line in a single
        // call (e.g. with an in-memory `Cursor`), so the cap check has
        // to come *first* — a 20 MB line read in one call is still
        // 20 MB worth of bytes, even if it ends in `\n`.
        match handle.read_line(&mut buffer) {
            Ok(0) => {
                // EOF.
                if buffer.is_empty() && total == 0 {
                    return None;
                }
                // Partial line at EOF — treat the partial data as the
                // final request, if any.
                return Some(Ok(buffer));
            }
            Ok(n) => {
                total = total.saturating_add(n);

                // Cap check FIRST: a single `read_line` may have
                // returned the entire oversize line in one go. If so,
                // the cap has already been exceeded and we must
                // reject the request regardless of whether the line
                // is well-terminated.
                if total > MAX_LINE_BYTES {
                    // Only drain if the over-budget line is still
                    // "in flight" (no terminating newline yet). If
                    // `read_line` already consumed the whole line in
                    // one go, the cursor is already positioned at the
                    // start of the next request and there's nothing
                    // more to discard.
                    if !buffer.ends_with('\n') {
                        drain_line(handle);
                    }
                    return Some(Err(OversizeRequest));
                }

                if buffer.ends_with('\n') {
                    // Trim the trailing newline so downstream parsers
                    // see the request body only.
                    if buffer.ends_with('\n') {
                        buffer.pop();
                        if buffer.ends_with('\r') {
                            buffer.pop();
                        }
                    }
                    return Some(Ok(buffer));
                }
            }
            Err(_) => {
                // Treat any read error as EOF for the purposes of the
                // outer loop; the server should not crash on a
                // transient stdin error.
                return None;
            }
        }
    }
}

/// Consume bytes from `handle` until the next `\n` or EOF. Used to
/// recover from an oversize request — without this, the remainder of
/// the over-budget line would be prepended to the next legitimate
/// request, corrupting it.
fn drain_line<R: BufRead>(handle: &mut R) {
    let mut sink = String::new();
    let mut drained: usize = 0;
    while let Ok(n) = handle.read_line(&mut sink) {
        if n == 0 || sink.ends_with('\n') {
            break;
        }
        drained = drained.saturating_add(n);
        sink.clear();
    }
    if drained > 0 {
        eprintln!("[clean-ctx] WARNING: Drained {} oversize bytes from stdin", drained);
    }
}

#[cfg(test)]
#[path = "../tests/mcp/server.rs"]
mod tests;
