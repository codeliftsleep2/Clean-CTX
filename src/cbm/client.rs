// src/cbm/client.rs
//
// JSON-RPC 2.0 subprocess client for codebase-memory-mcp.
// Self-contained — no knowledge of Clean-CTX internals.

use serde_json::Value;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crate::cbm::config::CbmStatus;

#[derive(Debug)]
pub enum CbmError {
    LaunchError(String),
    ConnectionLost(String),
    RpcError { code: i64, message: String },
    Timeout(Duration),
    ParseError(String),
}

impl std::fmt::Display for CbmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CbmError::LaunchError(msg) => write!(f, "CBM launch failed: {msg}"),
            CbmError::ConnectionLost(msg) => write!(f, "CBM connection lost: {msg}"),
            CbmError::RpcError { code, message } => write!(f, "CBM RPC error ({code}): {message}"),
            CbmError::Timeout(d) => write!(f, "CBM query timed out after {d:?}"),
            CbmError::ParseError(msg) => write!(f, "CBM parse error: {msg}"),
        }
    }
}

impl std::error::Error for CbmError {}

/// Maximum response size from CBM (4 MB safety bound).
pub const MAX_RESPONSE_BYTES: usize = 4 * 1024 * 1024;

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
/// - `RpcError` with code -32601 (Method not found) — programming error
pub(crate) fn is_retryable(error: &CbmError) -> bool {
    matches!(
        error,
        CbmError::ConnectionLost(_)
            | CbmError::Timeout(_)
            | CbmError::RpcError { code: -32603, .. }
    )
}

impl CbmClient {
    /// Launch the CBM subprocess.
    ///
    /// **Stderr handling:** A background reader thread drains CBM's stderr
    /// into a log file. Without this, if CBM writes enough diagnostic output
    /// (~64KB), the pipe buffer fills and CBM blocks — causing a deadlock
    /// since we only read stdout. (H-1 regression guard).
    pub fn try_launch(
        binary_path: &Path,
        timeout: Duration,
        max_consecutive_failures: u32,
        circuit_cooldown_secs: u64,
    ) -> Result<Option<Self>, CbmError> {
        if !binary_path.exists() || !binary_path.is_file() {
            return Ok(None);
        }

        let mut child = Command::new(binary_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| CbmError::LaunchError(format!("spawn: {e}")))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| CbmError::LaunchError("no stdin".into()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| CbmError::LaunchError("no stdout".into()))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| CbmError::LaunchError("no stderr".into()))?;

        // H-1 fix: drain stderr in a background thread to prevent deadlock.
        let _stderr_drainer = std::thread::Builder::new()
            .name("cbm-stderr-drain".into())
            .spawn(move || {
                let reader = BufReader::new(stderr);
                for l in reader.lines().map_while(Result::ok) {
                    eprintln!("[cbm-stderr] {l}");
                }
            })
            .ok();

        // C-2 fix: read stdout in a background thread and forward each line
        // over a channel. The main thread can then use recv_timeout() to
        // honour the per-request deadline without blocking indefinitely in
        // read_line().
        let (stdout_tx, stdout_rx) = std::sync::mpsc::sync_channel::<Result<String, String>>(256);
        let _stdout_reader = std::thread::Builder::new()
            .name("cbm-stdout-reader".into())
            .spawn(move || {
                let mut reader = BufReader::new(stdout);
                loop {
                    let mut line = String::new();
                    match reader.read_line(&mut line) {
                        Ok(0) => {
                            // EOF — child exited.
                            let _ = stdout_tx.send(Err("CBM exited".to_string()));
                            break;
                        }
                        Ok(_) => {
                            if stdout_tx.send(Ok(line)).is_err() {
                                break; // Main thread dropped the receiver — stop reading.
                            }
                        }
                        Err(e) => {
                            let _ = stdout_tx.send(Err(e.to_string()));
                            break;
                        }
                    }
                }
            })
            .ok();

        Ok(Some(Self {
            child,
            stdin: BufWriter::new(stdin),
            stdout_rx,
            _stdout_reader,
            _stderr_drainer,
            request_id: AtomicU64::new(1),
            status: CbmStatus::Available,
            timeout,
            consecutive_failures: 0,
            degraded_since: None,
            max_consecutive_failures,
            circuit_cooldown_secs,
        }))
    }

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

    /// Inner implementation of call_tool_raw — the actual pipe-level I/O.
    fn call_tool_raw_inner(&mut self, tool_name: &str, args: Value) -> Result<String, CbmError> {
        let id = self.request_id.fetch_add(1, Ordering::SeqCst);
        let request = serde_json::json!({
            "jsonrpc": "2.0", "id": id,
            "method": "tools/call",
            "params": { "name": tool_name, "arguments": args }
        });
        let req_line =
            serde_json::to_string(&request).map_err(|e| CbmError::ParseError(e.to_string()))?;
        writeln!(self.stdin, "{req_line}").map_err(|e| {
            self.status = CbmStatus::Degraded(format!("write: {e}"));
            CbmError::ConnectionLost(e.to_string())
        })?;
        self.stdin.flush().map_err(|e| {
            self.status = CbmStatus::Degraded(format!("flush: {e}"));
            CbmError::ConnectionLost(e.to_string())
        })?;

        // C-2 fix: accumulate lines until we have complete JSON, honouring the
        // deadline on every receive call. `recv_timeout` returns immediately
        // when the deadline has passed instead of blocking in `read_line`.
        let mut buf = String::new();
        let deadline = std::time::Instant::now() + self.timeout;
        loop {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if remaining.is_zero() {
                let _ = self.child.kill();
                let _ = self.child.try_wait();
                self.status = CbmStatus::Degraded("timeout".into());
                return Err(CbmError::Timeout(self.timeout));
            }
            match self.stdout_rx.recv_timeout(remaining) {
                Ok(Ok(line)) => {
                    buf.push_str(&line);
                    if buf.len() > MAX_RESPONSE_BYTES {
                        self.status = CbmStatus::Degraded("oversized".into());
                        return Err(CbmError::ConnectionLost(format!(
                            "response >{}B",
                            MAX_RESPONSE_BYTES
                        )));
                    }
                    if serde_json::from_str::<Value>(buf.trim()).is_ok() {
                        // Valid complete JSON — return the raw intercepted text
                        return Ok(buf);
                    }
                }
                Ok(Err(msg)) => {
                    self.status = CbmStatus::Degraded("exited".into());
                    return Err(CbmError::ConnectionLost(msg));
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    let _ = self.child.kill();
                    let _ = self.child.try_wait();
                    self.status = CbmStatus::Degraded("timeout".into());
                    return Err(CbmError::Timeout(self.timeout));
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    self.status = CbmStatus::Degraded("exited".into());
                    return Err(CbmError::ConnectionLost("CBM exited".into()));
                }
            }
        }
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

    /// Inner implementation of call_tool — the actual JSON-RPC call.
    fn call_tool_inner(&mut self, tool_name: &str, args: Value) -> Result<Value, CbmError> {
        let id = self.request_id.fetch_add(1, Ordering::SeqCst);
        let request = serde_json::json!({
            "jsonrpc": "2.0", "id": id,
            "method": "tools/call",
            "params": { "name": tool_name, "arguments": args }
        });
        let req_line =
            serde_json::to_string(&request).map_err(|e| CbmError::ParseError(e.to_string()))?;
        writeln!(self.stdin, "{req_line}").map_err(|e| {
            self.status = CbmStatus::Degraded(format!("write: {e}"));
            CbmError::ConnectionLost(e.to_string())
        })?;
        self.stdin.flush().map_err(|e| {
            self.status = CbmStatus::Degraded(format!("flush: {e}"));
            CbmError::ConnectionLost(e.to_string())
        })?;

        // C-2 fix: accumulate lines until we have complete JSON, honouring the
        // deadline on every receive call.
        let mut buf = String::new();
        let deadline = std::time::Instant::now() + self.timeout;
        loop {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if remaining.is_zero() {
                let _ = self.child.kill();
                let _ = self.child.try_wait();
                self.status = CbmStatus::Degraded("timeout".into());
                return Err(CbmError::Timeout(self.timeout));
            }
            match self.stdout_rx.recv_timeout(remaining) {
                Ok(Ok(line)) => {
                    buf.push_str(&line);
                    if buf.len() > MAX_RESPONSE_BYTES {
                        self.status = CbmStatus::Degraded("oversized".into());
                        return Err(CbmError::ConnectionLost(format!(
                            "response >{}B",
                            MAX_RESPONSE_BYTES
                        )));
                    }
                    if let Ok(resp) = serde_json::from_str::<Value>(buf.trim()) {
                        if let Some(error) = resp.get("error") {
                            return Err(CbmError::RpcError {
                                code: error["code"].as_i64().unwrap_or(-1),
                                message: error["message"].as_str().unwrap_or("unknown").into(),
                            });
                        }
                        return resp
                            .get("result")
                            .cloned()
                            .ok_or_else(|| CbmError::ParseError("missing result".into()));
                    }
                    // Not yet complete JSON — continue reading lines
                }
                Ok(Err(msg)) => {
                    self.status = CbmStatus::Degraded("exited".into());
                    return Err(CbmError::ConnectionLost(msg));
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    let _ = self.child.kill();
                    let _ = self.child.try_wait();
                    self.status = CbmStatus::Degraded("timeout".into());
                    return Err(CbmError::Timeout(self.timeout));
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    self.status = CbmStatus::Degraded("exited".into());
                    return Err(CbmError::ConnectionLost("CBM exited".into()));
                }
            }
        }
    }

    // ── Response parsing ───────────────────────────────────────

    /// CBM wraps all tool responses in MCP content array format.
    /// The actual data is a JSON string inside `result.content[0].text`.
    /// This helper extracts and parses that inner JSON string.
    fn parse_cbm_response(&self, response: &Value) -> Result<Value, CbmError> {
        let content = response["content"]
            .as_array()
            .and_then(|a| a.first())
            .ok_or_else(|| CbmError::ParseError("missing content array".into()))?;
        let text = content["text"]
            .as_str()
            .ok_or_else(|| CbmError::ParseError("missing text field in content".into()))?;
        serde_json::from_str(text)
            .map_err(|e| CbmError::ParseError(format!("inner JSON parse: {e}")))
    }

    // ── Typed wrappers ──────────────────────────────────────────

    /// Search the CBM knowledge graph by name pattern and optional label filter.
    ///
    /// CBM params: `name_pattern`, `label`, `file_pattern`, `project`, `limit`, `offset`
    pub fn search_graph(
        &mut self,
        name_pattern: &str,
        project: &str,
        label: Option<&str>,
    ) -> Result<Vec<Value>, CbmError> {
        let mut args = serde_json::json!({"name_pattern": name_pattern, "project": project});
        if let Some(l) = label {
            args["label"] = serde_json::Value::String(l.to_string());
        }
        let r = self.call_tool("search_graph", args)?;
        let inner = self.parse_cbm_response(&r)?;
        Ok(inner["results"].as_array().cloned().unwrap_or_default())
    }

    /// Trace call paths in the CBM knowledge graph.
    ///
    /// CBM params: `function_name`, `direction` (inbound|outbound|both), `depth`, `project`
    pub fn trace_path(
        &mut self,
        function_name: &str,
        direction: &str,
        project: &str,
        depth: Option<usize>,
    ) -> Result<Vec<Value>, CbmError> {
        let mut args = serde_json::json!({"function_name": function_name, "direction": direction, "project": project});
        if let Some(d) = depth {
            args["depth"] = serde_json::Value::Number(serde_json::Number::from(d));
        }
        let r = self.call_tool("trace_path", args)?;
        let inner = self.parse_cbm_response(&r)?;
        Ok(inner["edges"].as_array().cloned().unwrap_or_default())
    }

    /// Get architecture overview from CBM.
    pub fn get_architecture(&mut self, project: &str) -> Result<Value, CbmError> {
        let r = self.call_tool("get_architecture", serde_json::json!({"project": project}))?;
        // get_architecture may or may not wrap in content array — try both
        if r.get("content").is_some() {
            self.parse_cbm_response(&r)
        } else {
            Ok(r)
        }
    }

    /// Execute a Cypher-like query against the CBM knowledge graph.
    ///
    /// Returns rows as `Vec<Vec<Value>>` where each inner vec is a row of column values.
    /// CBM wraps results in `{columns, rows}` format.
    pub fn query_graph(
        &mut self,
        cypher: &str,
        project: &str,
    ) -> Result<Vec<Vec<Value>>, CbmError> {
        let r = self.call_tool(
            "query_graph",
            serde_json::json!({"query": cypher, "project": project}),
        )?;
        let inner = self.parse_cbm_response(&r)?;
        Ok(inner["rows"]
            .as_array()
            .map(|rows| {
                rows.iter()
                    .filter_map(|row| row.as_array().cloned())
                    .collect()
            })
            .unwrap_or_default())
    }

    /// Get symbol importance (caller count) from CBM via Cypher query.
    ///
    /// CBM has no dedicated `get_symbol_importance` tool, but `in_degree`
    /// on function nodes provides the same information.
    pub fn get_symbol_importance(
        &mut self,
        project: &str,
        min_degree: Option<usize>,
    ) -> Result<Vec<Value>, CbmError> {
        let min = min_degree.unwrap_or(1);
        let cypher = format!(
            "MATCH (f:Function) WHERE f.in_degree >= {} RETURN f.name, f.file_path, f.in_degree, f.out_degree ORDER BY f.in_degree DESC",
            min
        );
        let rows = self.query_graph(&cypher, project)?;
        Ok(rows
            .into_iter()
            .map(|row| {
                let name = row
                    .first()
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let file = row
                    .get(1)
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let in_degree = row
                    .get(2)
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0);
                let out_degree = row
                    .get(3)
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0);
                serde_json::json!({
                    "name": name,
                    "file": file,
                    "in_degree": in_degree as u64,
                    "out_degree": out_degree as u64,
                    "importance": in_degree / 100.0,  // normalized score for blending
                })
            })
            .collect())
    }

    /// Get dead code candidates from CBM via Cypher query.
    ///
    /// CBM has no dedicated `get_dead_code` tool, but functions with
    /// `in_degree = 0` and `is_entry_point = false` are dead code.
    pub fn get_dead_code(&mut self, project: &str) -> Result<Vec<Value>, CbmError> {
        let cypher = "MATCH (f:Function) WHERE f.in_degree = 0 AND f.is_entry_point = false RETURN f.name, f.file_path".to_string();
        let rows = self.query_graph(&cypher, project)?;
        Ok(rows
            .into_iter()
            .map(|row| {
                serde_json::json!({
                    "name": row.first().and_then(|v| v.as_str()).unwrap_or(""),
                    "file": row.get(1).and_then(|v| v.as_str()).unwrap_or(""),
                    "reason": "no callers",
                })
            })
            .collect())
    }

    /// Trigger indexing of a repository in CBM 0.8.1+.
    /// Uses `index_repository` with `repo_path` and `mode` parameters.
    pub fn index_repository(&mut self, repo_path: &str, mode: &str) -> Result<Value, CbmError> {
        self.call_tool(
            "index_repository",
            serde_json::json!({"repo_path": repo_path, "mode": mode}),
        )
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
