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
use std::path::{Path, PathBuf};

use crate::config::CleanCtxConfig;
use crate::mcp::McpState;
use crate::mcp::dispatcher::Dispatcher;
use crate::protocol::JsonRpcRequest;
use crate::protocol::send_response;

/// Cached project root, resolved once per process lifetime.
/// Walks up from the executable's directory looking for `.clean-ctx.json`
/// or `Cargo.toml` to anchor all relative paths (config,, DB, debug log).
static PROJECT_ROOT: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();

/// Resolve the project root directory.
///
/// Resolution order (P1-4):
///   1. `CLEAN_CTX_PROJECT_ROOT` env var (if set)
///   2. Walk up from CWD looking for `.clean-ctx.json` or `Cargo.toml`
///   3. Walk up from the executable's directory (for dev / local builds)
///   4. Fallback to CWD
///
/// Previously the search started from the executable directory, which fails
/// when the binary is installed system-wide (e.g., `/usr/local/bin/`).
/// Now we check CWD first (the MCP client's working directory) which is the
/// most reliable indicator of the project root in production.
pub(crate) fn find_project_root() -> &'static PathBuf {
    PROJECT_ROOT.get_or_init(|| {
        // 1. Check CLEAN_CTX_PROJECT_ROOT env var
        if let Ok(root) = std::env::var("CLEAN_CTX_PROJECT_ROOT") {
            let p = PathBuf::from(root);
            if p.exists() {
                return p;
            }
            eprintln!(
                "[clean-ctx] Warning: CLEAN_CTX_PROJECT_ROOT set but not found: {}",
                p.display()
            );
        }

        // 2. Walk up from CWD (most reliable in production — MCP client's CWD)
        if let Ok(cwd) = std::env::current_dir() {
            if let Some(found) = walk_up_for_project_root(&cwd) {
                return found;
            }
        }

        // 3. Walk up from executable directory (works for dev/cargo run)
        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent() {
                if let Some(found) = walk_up_for_project_root(parent) {
                    return found;
                }
            }
        }

        // 4. Fallback to CWD
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    })
}

/// Walk up from `start` looking for `.clean-ctx.json` or `Cargo.toml`.
fn walk_up_for_project_root(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        if current.join(".clean-ctx.json").exists() || current.join("Cargo.toml").exists() {
            return Some(current);
        }
        if !current.pop() {
            break;
        }
    }
    None
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

    // P3-19: Eagerly initialise the BPE engine. A failure here (e.g. the BPE
    // data file is missing or unreadable) is non-fatal — the server continues
    // running and BPE will be re-attempted lazily on first compression call.
    // This allows CBM-only operations (graph_search, get_architecture, etc.)
    // to work even when BPE data is unavailable.
    if let Err(e) = crate::analytics::bpe_or_init() {
        eprintln!("[clean-ctx] WARNING: BPE initialization failed: {e}");
        eprintln!("[clean-ctx] WARNING: Compression will fall back to character-count estimates.");
        eprintln!("[clean-ctx] WARNING: CBM-only operations (graph_search, etc.) are unaffected.");
    }

    // F-05: load the project config and bundle it into the
    // per-session state. The config is no longer bound to `_` and
    // thrown away; tool handlers consult it via `state.config`.
    let project_root = find_project_root();
    eprintln!("[clean-ctx] Project root: {}", project_root.display());

    // Phase 3: If CLEAN_CTX_PROJECT_ROOT is set, log it prominently so
    // the user always sees which root won. This makes the override
    // explicit rather than a silent footgun.
    if let Ok(root) = std::env::var("CLEAN_CTX_PROJECT_ROOT") {
        eprintln!(
            "[clean-ctx] Using CLEAN_CTX_PROJECT_ROOT override: {}",
            root
        );
    }

    let config = CleanCtxConfig::load(project_root);
    let mut state = McpState::new(config.clone());

    // Auto-start the proxy if enabled in config. Non-fatal: if the
    // proxy binary is missing or fails to spawn, the MCP server
    // continues without it (logged above).
    //
    // Phase 4b: The proxy's working directory is the config file's
    // directory (not the project root). This ensures `filters/` and
    // `log_dir` resolve consistently regardless of which repo window
    // is open — critical for workspace-mode setups where the project
    // root may be a single repo but the config lives at the workspace
    // root.
    if config.proxy.auto_start {
        let proxy_cwd = CleanCtxConfig::find_config(project_root)
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
            .unwrap_or_else(|| project_root.clone());
        match crate::proxy_spawner::spawn_proxy(&config.proxy, &proxy_cwd, false) {
            Ok(Some(child)) => {
                *state.proxy_child_lock() = Some(child);
            }
            Ok(None) => {
                // auto_start disabled, binary not found, or proxy already
                // running (adopted) — already logged.
            }
            Err(e) => {
                eprintln!("[clean-ctx] WARNING: {e}");
            }
        }

        // Phase 5 (cache-hint transport): query the proxy's cache
        // configuration so the MCP server can align its `_meta.cache_hints`
        // breakers with the proxy's actual injection behavior. Non-fatal —
        // if the proxy isn't reachable, we simply skip the alignment.
        if let Some(cache_state) = crate::proxy_spawner::query_proxy_cache_state(config.proxy.port)
        {
            state.proxy_cache = Some(cache_state.clone());
            eprintln!(
                "[clean-ctx] Proxy cache state: auto_cache={} tail_ttl={}",
                cache_state.auto_cache, cache_state.tail_ttl
            );
        }

        // Crash monitor: if the auto-started proxy dies mid-session
        // (crash, OOM, external kill), log a prominent warning so the
        // operator knows the proxy is no longer intercepting requests.
        // We poll the port rather than the child handle — `Child` cannot
        // be cloned across threads, and port polling detects the proxy
        // being down regardless of how it died. The monitor thread is
        // detached; it exits when the port stops listening.
        if state.proxy_child_lock().is_some() {
            let monitor_port = config.proxy.port;
            std::thread::Builder::new()
                .name("proxy-crash-monitor".into())
                .spawn(move || {
                    // Give the proxy a moment to finish binding before we
                    // start watching (the spawner already verified it, but
                    // a slow disk/AV could still be mid-startup).
                    std::thread::sleep(std::time::Duration::from_secs(5));
                    if !crate::proxy_spawner::port_in_use(monitor_port) {
                        eprintln!(
                            "[clean-ctx] WARNING: Auto-started proxy on port {} is no longer \
                             listening. The proxy is not intercepting requests. Restart \
                             Clean-CTX to relaunch it.",
                            monitor_port
                        );
                    }
                })
                .ok();
        }
    }

    // A-PRODUCTION: Wrap state in the production-grade dispatcher.
    let dispatcher = Dispatcher::new(state);

    // A-04: Spawn periodic metrics exporter if enabled in config.
    if dispatcher.state().config.observability.export_metrics {
        let interval_secs = dispatcher.state().config.observability.export_interval_secs;
        let registry = std::sync::Arc::clone(&dispatcher.state().metrics_registry);
        eprintln!(
            "[clean-ctx] Metrics export enabled (every {}s)",
            interval_secs
        );
        std::thread::Builder::new()
            .name("metrics-exporter".into())
            .spawn(move || {
                let interval = std::time::Duration::from_secs(interval_secs);
                loop {
                    std::thread::sleep(interval);
                    let snapshot = registry.snapshot();
                    eprintln!("[metrics] {}", snapshot.format_otlp_json());
                }
            })
            .ok();
    }

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
        } else {
            // JSON-RPC 2.0 requires a -32700 Parse error response.
            // Without this, the client hangs indefinitely waiting for a reply.
            send_response(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": serde_json::Value::Null,
                "error": { "code": -32700, "message": "Parse error" }
            }));
        }
    }

    // A-09: Wait for all queued work to complete before exiting.
    // The Dispatcher's drop impl will block until all spawned tasks
    // finish, so terminate the auto-started proxy child FIRST (while
    // we still have access to the state via `dispatcher.state()`).
    eprintln!("[clean-ctx] Stdin exhausted, waiting for pending work...");

    // Terminate the auto-started proxy child (if any).
    if let Some(ref mut child) = *dispatcher.state().proxy_child_lock() {
        crate::proxy_spawner::shutdown_proxy(child);
        eprintln!("[clean-ctx] Proxy stopped.");
    }

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
        // `read_line` appends to its argument, so we must clear the
        // buffer at the start of each iteration to avoid concatenating
        // chunks from incomplete multi-read lines.
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
                    // Trim the trailing newline(s) so downstream parsers
                    // see the request body only. Use a loop to handle
                    // \r\n followed by an extra \n (malformed but possible).
                    while buffer.ends_with('\n') {
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
// P1-5: Fix underreporting by counting the final chunk
fn drain_line<R: BufRead>(handle: &mut R) {
    let mut sink = String::new();
    let mut drained: usize = 0;
    while let Ok(n) = handle.read_line(&mut sink) {
        if n == 0 || sink.ends_with('\n') {
            // P1-5: Add final chunk to drained total
            drained = drained.saturating_add(n);
            break;
        }
        drained = drained.saturating_add(n);
        sink.clear();
    }
    if drained > 0 {
        eprintln!(
            "[clean-ctx] WARNING: Drained {} oversize bytes from stdin",
            drained
        );
    }
}

#[cfg(test)]
#[path = "../tests/mcp/server.rs"]
mod tests;
