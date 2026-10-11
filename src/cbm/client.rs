// src/cbm/client.rs
//
// JSON-RPC 2.0 subprocess client for codebase-memory-mcp.
// Self-contained — no knowledge of Clean-CTX internals.

use serde_json::Value;
use std::io::BufWriter;
use std::process::{Child, ChildStdin};
use std::sync::atomic::AtomicU64;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crate::cbm::config::CbmStatus;

mod framing;
mod transport;
mod wire;

#[derive(Debug)]
pub enum CbmError {
    LaunchError(String),
    ConnectionLost(String),
    RpcError {
        code: i64,
        message: String,
    },
    Timeout(Duration),
    ParseError(String),
    /// CBM reported a tool-level failure inside a successful JSON-RPC
    /// envelope (`result.isError = true` + error payload). See
    /// [`check_soft_error`].
    ToolError {
        tool: String,
        message: String,
    },
}

impl std::fmt::Display for CbmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CbmError::LaunchError(msg) => write!(f, "CBM launch failed: {msg}"),
            CbmError::ConnectionLost(msg) => write!(f, "CBM connection lost: {msg}"),
            CbmError::RpcError { code, message } => write!(f, "CBM RPC error ({code}): {message}"),
            CbmError::Timeout(d) => write!(f, "CBM query timed out after {d:?}"),
            CbmError::ParseError(msg) => write!(f, "CBM parse error: {msg}"),
            CbmError::ToolError { tool, message } => {
                write!(f, "CBM {tool} failed: {message}")
            }
        }
    }
}

impl std::error::Error for CbmError {}

/// Maximum response size from CBM (4 MB safety bound).
pub const MAX_RESPONSE_BYTES: usize = 4 * 1024 * 1024;

/// The full `query_graph` wire table: echoed projection `columns` plus the
/// row matrix.
///
/// CBM-WIRE-002 (verified live 2026-08-24): `columns` echo the RETURN
/// expressions verbatim — including whitespace (`"type( r )"`), but with
/// any `AS` alias REPLACING the expression entirely (`type(r) AS rel_kind`
/// echoes as `"rel_kind"`, erasing the semantic marker). Callers that
/// interpret projection shapes must consume this struct, never rows alone.
#[derive(Debug, Clone, Default)]
pub struct QueryRows {
    /// Projection expressions as echoed by CBM, in RETURN order.
    pub columns: Vec<String>,
    /// Row cells; every cell of a column corresponds to `columns[i]`.
    pub rows: Vec<Vec<Value>>,
}

/// JSON-RPC 2.0 client over stdin/stdout subprocess.
pub struct CbmClient {
    child: Child,
    stdin: BufWriter<ChildStdin>,
    /// C-2 fix: stdout is read exclusively by a background reader thread.
    /// The thread sends each line through this channel so the main thread
    /// can drain lines with a timeout and honour the per-request deadline
    /// even when `read_line` would otherwise block indefinitely.
    stdout_rx: std::sync::mpsc::Receiver<Result<String, String>>,
    /// Background thread draining CBM's stdout. Held to keep it alive.
    _stdout_reader: Option<JoinHandle<()>>,
    /// Background thread draining CBM's stderr to avoid pipe-buffer deadlock.
    _stderr_drainer: Option<JoinHandle<()>>,
    request_id: AtomicU64,
    status: CbmStatus,
    timeout: Duration,
    /// Circuit breaker: consecutive transient failures.
    consecutive_failures: u32,
    /// When the circuit opened (set to Degraded). None when circuit is closed.
    degraded_since: Option<Instant>,
    /// Maximum consecutive failures before circuit opens.
    max_consecutive_failures: u32,
    /// Cooldown in seconds before circuit resets to half-open.
    circuit_cooldown_secs: u64,
}

/// Determines whether a CBM error is transient and should be retried.
///
/// Retryable errors:
/// - `ConnectionLost` — transient pipe failure, CBM may recover
/// - `Timeout` — CBM was busy, retry may succeed
/// - `RpcError` with code -32603 (Internal error) — transient server issue
///
/// Non-retryable errors:
/// - `LaunchError` — binary missing, retry won't help
/// - `ParseError` — malformed response, retry won't help
/// - `ToolError` — CBM rejected the request semantically, retry won't help
/// - `RpcError` with code -32601 (Method not found) — programming error
pub(crate) fn is_retryable(error: &CbmError) -> bool {
    matches!(
        error,
        CbmError::ConnectionLost(_)
            | CbmError::Timeout(_)
            | CbmError::RpcError { code: -32603, .. }
    )
}

/// Soft-error gate for parsed CBM tool results.
///
/// CBM signals tool-level failures inside a SUCCESSFUL JSON-RPC result:
///
/// ```json
/// {"isError": true,
///  "content": [{"type": "text", "text": "<body>"}]}
/// ```
///
/// where `<body>` is either a JSON object carrying an `"error"` string key
/// (e.g. `"project not found or not indexed"`) or a plain-text diagnostic
/// (e.g. Cypher parser output). Without this gate such failures would be
/// indistinguishable from valid empty results.
///
/// Pure function so tests can pin the exact wire shapes.
pub(crate) fn check_soft_error(tool_name: &str, result: &Value) -> Result<(), CbmError> {
    if result.get("isError").and_then(Value::as_bool) != Some(true) {
        return Ok(());
    }
    let text = result["content"][0]["text"].as_str().unwrap_or("");
    let message = serde_json::from_str::<Value>(text)
        .ok()
        .and_then(|v| v.get("error").and_then(Value::as_str).map(str::to_string))
        .unwrap_or_else(|| {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                "unknown CBM tool failure".to_string()
            } else {
                trimmed.to_string()
            }
        });
    Err(CbmError::ToolError {
        tool: tool_name.to_string(),
        message,
    })
}
// ── trace_path wire contract (CBM 0.8.1) ──────────────────────────────
//
// Verbatim live captures (fresh subprocess, 2026-08-24):
//
//   inbound  → {"function":"<fn>","direction":"inbound","callers":[…]}
//   outbound → {"function":"<fn>","direction":"outbound","callees":[…]}
//   both     → {"function":"<fn>","direction":"both",
//               "callees":[…],"callers":[…]}   // both arrays coexist
//
// There is NO "edges" key — the former parser read a phantom one, so every
// typed graph_trace silently collapsed to zero edges while the raw proxy
// path worked. Relationships arrive as caller/callee ARRAYS whose entries
// carry exactly `name`, `qualified_name`, and `hop` (a JSON number).
// A missing array is a valid zero-result half (e.g. `callees: []`).
//
// Depth semantics (pinned by the depth-3 capture): entries are FLAT BFS
// discoveries tagged only with their hop count — there is NO parent
// linkage. A hop-1 entry is a direct call of the traced function; for a
// hop-N (N ≥ 2) entry the immediate caller is not identifiable, so turning
// it into an edge pair would invent relationships. Only hop-1 entries are
// converted; deeper hops remain available through the raw cbm_proxy path.

/// Endpoint identity of a caller/callee entry: `qualified_name`, falling
/// back to the bare `name` — same normalization precedent as
/// `map_search_result`. `None` when the entry carries neither key.
fn trace_entry_endpoint(entry: &Value) -> Option<String> {
    let name = entry.get("name").and_then(Value::as_str)?;
    Some(
        entry
            .get("qualified_name")
            .and_then(Value::as_str)
            .unwrap_or(name)
            .to_string(),
    )
}

/// Only `hop: 1` entries describe DIRECT call relationships (see the
/// depth-semantics note above); deeper hops are skipped rather than
/// mis-attributed to an unidentifiable intermediate caller.
fn trace_entry_is_direct(entry: &Value) -> bool {
    entry.get("hop").and_then(Value::as_u64) == Some(1)
}

/// Normalize CBM 0.8.1's `trace_path` response into edge objects
/// (`{"from","to","label"}`) consumable by `GraphBridge::trace_path`.
///
/// Direction semantics: every `callers[i]` calls `function_name`, and
/// `function_name` calls every `callees[i]`. Edges are always oriented
/// caller → callee regardless of which array produced them, so consumers
/// see one consistent direction.
///
/// Pure function so tests can pin the exact wire shapes against verbatim
/// captures (`src/tests/cbm/trace_wire.rs`).
pub(crate) fn extract_trace_edges(inner: &Value, function_name: &str) -> Vec<Value> {
    let mut pairs: Vec<(String, String)> = Vec::new();
    if let Some(callers) = inner.get("callers").and_then(Value::as_array) {
        for entry in callers {
            if trace_entry_is_direct(entry) {
                if let Some(id) = trace_entry_endpoint(entry) {
                    pairs.push((id, function_name.to_string()));
                }
            }
        }
    }
    if let Some(callees) = inner.get("callees").and_then(Value::as_array) {
        for entry in callees {
            if trace_entry_is_direct(entry) {
                if let Some(id) = trace_entry_endpoint(entry) {
                    pairs.push((function_name.to_string(), id));
                }
            }
        }
    }
    // Dedupe EXACT duplicate edges only, preserving first-seen CBM order.
    // The same symbol may legitimately appear at several hops; repeated
    // nodes are not relationships and are never merged away.
    let mut edges: Vec<Value> = Vec::with_capacity(pairs.len());
    for (from, to) in pairs {
        let dup = edges.iter().any(|e| {
            e["from"].as_str() == Some(from.as_str()) && e["to"].as_str() == Some(to.as_str())
        });
        if dup {
            continue;
        }
        edges.push(serde_json::json!({"from": from, "to": to, "label": "calls"}));
    }
    edges
}

impl CbmClient {
    pub fn status(&self) -> &CbmStatus {
        &self.status
    }

    // ── Circuit breaker ─────────────────────────────────────────

    /// Check if the circuit is open (too many recent failures).
    /// Returns true if the circuit allows the call, false if it's tripped.
    ///
    /// If the circuit is open but the cooldown has elapsed, it resets
    /// automatically and allows the next call through (half-open state).
    pub(crate) fn circuit_allows(&mut self) -> bool {
        if self.consecutive_failures < self.max_consecutive_failures {
            return true;
        }
        // Circuit is open — check if cooldown has elapsed
        if let Some(since) = self.degraded_since {
            if since.elapsed() >= Duration::from_secs(self.circuit_cooldown_secs) {
                // Cooldown elapsed — reset to half-open, allow one try
                self.consecutive_failures = 0;
                self.degraded_since = None;
                self.status = CbmStatus::Available;
                eprintln!("[clean-ctx-cbm] Circuit cooldown elapsed, trying again…");
                return true;
            }
        }
        false
    }

    /// Record a successful query — resets the failure counter.
    pub(crate) fn record_success(&mut self) {
        self.consecutive_failures = 0;
        self.degraded_since = None;
        // Don't overwrite status if it was explicitly set to Available
        if matches!(self.status, CbmStatus::Degraded(_)) {
            self.status = CbmStatus::Available;
        }
    }

    /// Record a transient failure — increments the counter, degrades if threshold reached.
    pub(crate) fn record_failure(&mut self) {
        self.consecutive_failures += 1;
        if self.consecutive_failures >= self.max_consecutive_failures {
            self.status = CbmStatus::Degraded(format!(
                "circuit_open_after_{}_failures",
                self.max_consecutive_failures
            ));
            self.degraded_since = Some(Instant::now());
            eprintln!(
                "[clean-ctx-cbm] Circuit opened after {} consecutive failures",
                self.max_consecutive_failures
            );
        }
    }

    /// Send a JSON-RPC 2.0 request and read the raw response text.
    ///
    /// **Pipe-level interception:** This returns the **raw text** that CBM
    /// wrote to stdout, NOT a parsed JSON value. The intention is that
    /// Clean-CTX catches this at the pipe level and compresses it before
    /// it reaches the agent. CBM's ~5000-token structural seed gets
    /// compressed down to ~1100 tokens.
    ///
    /// Returns the raw response text (the full JSON-RPC response body).
    /// The caller MUST validate that it starts with `{"jsonrpc":"2.0"...`
    /// (Callers that need parsed JSON should still use `call_tool`.)
    /// Send a JSON-RPC 2.0 request, read the raw response text, with retry.
    ///
    /// Retries transient errors (timeout, connection lost, internal error)
    /// once with exponential backoff before giving up. Delegates to
    /// `call_tool_raw_inner` for the actual pipe I/O.
    pub fn call_tool_raw(&mut self, tool_name: &str, args: Value) -> Result<String, CbmError> {
        if !self.circuit_allows() {
            return Err(CbmError::ConnectionLost("circuit breaker open".into()));
        }
        let max_retries = 1;
        let mut retry_count = 0;
        let mut backoff = Duration::from_millis(100);

        loop {
            // Value::clone() is cheap — a few ref-count bumps for JSON strings/arrays
            match self.call_tool_raw_inner(tool_name, args.clone()) {
                Ok(result) => {
                    self.record_success();
                    return Ok(result);
                }
                Err(e) if retry_count < max_retries && is_retryable(&e) => {
                    retry_count += 1;
                    self.record_failure();
                    std::thread::sleep(backoff);
                    backoff *= 2; // Exponential backoff
                }
                Err(e) => {
                    self.record_failure();
                    return Err(e);
                }
            }
        }
    }

    /// Inner implementation of call_tool_raw — delegates to the shared
    /// transport pipeline [`send_and_receive_raw`] and returns the raw
    /// response text.
    fn call_tool_raw_inner(&mut self, tool_name: &str, args: Value) -> Result<String, CbmError> {
        self.send_and_receive_raw(tool_name, args)
    }

    /// Send a JSON-RPC 2.0 request and read the parsed response.
    ///
    /// For **pipe-level interception** use `call_tool_raw` instead —
    /// it returns the raw text that gets compressed.
    ///
    /// Retries transient errors (timeout, connection lost, internal error)
    /// once with exponential backoff before giving up.
    pub fn call_tool(&mut self, tool_name: &str, args: Value) -> Result<Value, CbmError> {
        if !self.circuit_allows() {
            return Err(CbmError::ConnectionLost("circuit breaker open".into()));
        }
        let max_retries = 1;
        let mut retry_count = 0;
        let mut backoff = Duration::from_millis(100);

        loop {
            // Value::clone() is cheap (~a few ref-count bumps for JSON strings/arrays)
            match self.call_tool_inner(tool_name, args.clone()) {
                Ok(result) => {
                    self.record_success();
                    return Ok(result);
                }
                Err(e) if retry_count < max_retries && is_retryable(&e) => {
                    retry_count += 1;
                    self.record_failure();
                    std::thread::sleep(backoff);
                    backoff *= 2; // Exponential backoff
                }
                Err(e) => {
                    self.record_failure();
                    return Err(e);
                }
            }
        }
    }

    /// Inner implementation of call_tool — delegates to the shared
    /// transport pipeline [`send_and_receive_raw`], then parses and
    /// verifies the response. Returns the parsed `result` field.
    fn call_tool_inner(&mut self, tool_name: &str, args: Value) -> Result<Value, CbmError> {
        let buf = self.send_and_receive_raw(tool_name, args)?;
        let resp: Value =
            serde_json::from_str(buf.trim()).map_err(|e| CbmError::ParseError(e.to_string()))?;
        if let Some(error) = resp.get("error") {
            return Err(CbmError::RpcError {
                code: error["code"].as_i64().unwrap_or(-1),
                message: error["message"].as_str().unwrap_or("unknown").into(),
            });
        }
        // Soft-error gate: CBM signals tool failures inside a
        // SUCCESSFUL JSON-RPC result via `result.isError` +
        // an error payload in `content[0].text`. Map them onto
        // CbmError so callers never conflate "failed" with
        // "valid query, zero results".
        let result = resp
            .get("result")
            .cloned()
            .ok_or_else(|| CbmError::ParseError("missing result".into()))?;
        check_soft_error(tool_name, &result)?;
        Ok(result)
    }


}

impl Drop for CbmClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        // L-02 fix: don't hang indefinitely on unresponsive child.
        // Poll try_wait with a deadline instead of blocking forever.
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            match self.child.try_wait() {
                Ok(Some(_)) | Err(_) => break,
                Ok(None) if Instant::now() > deadline => break,
                Ok(None) => std::thread::sleep(Duration::from_millis(50)),
            }
        }
    }
}
