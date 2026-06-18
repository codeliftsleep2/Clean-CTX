// src/cbm/client.rs
//
// JSON-RPC 2.0 subprocess client for codebase-memory-mcp.
// Self-contained — no knowledge of Clean-CTX internals.

use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;
use serde_json::Value;

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
    stdout: BufReader<ChildStdout>,
    /// Background thread draining CBM's stderr to avoid pipe-buffer deadlock.
    _stderr_drainer: Option<JoinHandle<()>>,
    request_id: AtomicU64,
    status: CbmStatus,
    timeout: Duration,
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
            | CbmError::RpcError {
                code: -32603,
                ..
            }
    )
}

impl CbmClient {
    /// Launch the CBM subprocess.
    ///
    /// **Stderr handling:** A background reader thread drains CBM's stderr
    /// into a log file. Without this, if CBM writes enough diagnostic output
    /// (~64KB), the pipe buffer fills and CBM blocks — causing a deadlock
    /// since we only read stdout. (H-1 regression guard).
    pub fn try_launch(binary_path: &Path, timeout: Duration) -> Result<Option<Self>, CbmError> {
        if !binary_path.exists() || !binary_path.is_file() {
            return Ok(None);
        }

        let mut child = Command::new(binary_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| CbmError::LaunchError(format!("spawn: {e}")))?;

        let stdin = child.stdin.take()
            .ok_or_else(|| CbmError::LaunchError("no stdin".into()))?;
        let stdout = child.stdout.take()
            .ok_or_else(|| CbmError::LaunchError("no stdout".into()))?;
        let stderr = child.stderr.take()
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

        Ok(Some(Self {
            child,
            stdin: BufWriter::new(stdin),
            stdout: BufReader::new(stdout),
            _stderr_drainer,
            request_id: AtomicU64::new(1),
            status: CbmStatus::Available,
            timeout,
        }))
    }

    pub fn status(&self) -> &CbmStatus { &self.status }

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
        let max_retries = 1;
        let mut retry_count = 0;
        let mut backoff = Duration::from_millis(100);

        loop {
            // Value::clone() is cheap — a few ref-count bumps for JSON strings/arrays
            match self.call_tool_raw_inner(tool_name, args.clone()) {
                Ok(result) => return Ok(result),
                Err(e) if retry_count < max_retries && is_retryable(&e) => {
                    retry_count += 1;
                    std::thread::sleep(backoff);
                    backoff *= 2; // Exponential backoff
                }
                Err(e) => return Err(e),
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
        let req_line = serde_json::to_string(&request)
            .map_err(|e| CbmError::ParseError(e.to_string()))?;
        writeln!(self.stdin, "{req_line}").map_err(|e| {
            self.status = CbmStatus::Degraded(format!("write: {e}"));
            CbmError::ConnectionLost(e.to_string())
        })?;
        self.stdin.flush().map_err(|e| {
            self.status = CbmStatus::Degraded(format!("flush: {e}"));
            CbmError::ConnectionLost(e.to_string())
        })?;

        // C-1 fix: accumulate lines until we have complete JSON.
        let mut buf = String::new();
        let deadline = std::time::Instant::now() + self.timeout;
        loop {
            if std::time::Instant::now() > deadline {
                let _ = self.child.kill();
                self.status = CbmStatus::Degraded("timeout".into());
                return Err(CbmError::Timeout(self.timeout));
            }
            let mut line = String::new();
            match self.stdout.read_line(&mut line) {
                Ok(0) => {
                    self.status = CbmStatus::Degraded("exited".into());
                    return Err(CbmError::ConnectionLost("CBM exited".into()));
                }
                Ok(_) => {
                    buf.push_str(&line);
                    if buf.len() > MAX_RESPONSE_BYTES {
                        self.status = CbmStatus::Degraded("oversized".into());
                        return Err(CbmError::ConnectionLost(
                            format!("response >{}B", MAX_RESPONSE_BYTES)
                        ));
                    }
                    if serde_json::from_str::<Value>(buf.trim()).is_ok() {
                        // Valid complete JSON — return the raw intercepted text
                        return Ok(buf);
                    }
                }
                Err(e) => {
                    self.status = CbmStatus::Degraded(format!("read: {e}"));
                    return Err(CbmError::ConnectionLost(e.to_string()));
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
        let max_retries = 1;
        let mut retry_count = 0;
        let mut backoff = Duration::from_millis(100);

        loop {
            // Value::clone() is cheap (~a few ref-count bumps for JSON strings/arrays)
            match self.call_tool_inner(tool_name, args.clone()) {
                Ok(result) => return Ok(result),
                Err(e) if retry_count < max_retries && is_retryable(&e) => {
                    retry_count += 1;
                    std::thread::sleep(backoff);
                    backoff *= 2; // Exponential backoff
                }
                Err(e) => return Err(e),
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
        let req_line = serde_json::to_string(&request)
            .map_err(|e| CbmError::ParseError(e.to_string()))?;
        writeln!(self.stdin, "{req_line}").map_err(|e| {
            self.status = CbmStatus::Degraded(format!("write: {e}"));
            CbmError::ConnectionLost(e.to_string())
        })?;
        self.stdin.flush().map_err(|e| {
            self.status = CbmStatus::Degraded(format!("flush: {e}"));
            CbmError::ConnectionLost(e.to_string())
        })?;

        // C-1 fix: accumulate lines until we have complete JSON.
        let mut buf = String::new();
        let deadline = std::time::Instant::now() + self.timeout;
        const MAX_RESPONSE_BYTES: usize = 4 * 1024 * 1024; // 4 MB safety bound
        loop {
            if std::time::Instant::now() > deadline {
                let _ = self.child.kill();
                self.status = CbmStatus::Degraded("timeout".into());
                return Err(CbmError::Timeout(self.timeout));
            }
            let mut line = String::new();
            match self.stdout.read_line(&mut line) {
                Ok(0) => {
                    self.status = CbmStatus::Degraded("exited".into());
                    return Err(CbmError::ConnectionLost("CBM exited".into()));
                }
                Ok(_) => {
                    buf.push_str(&line);
                    if buf.len() > MAX_RESPONSE_BYTES {
                        self.status = CbmStatus::Degraded("oversized".into());
                        return Err(CbmError::ConnectionLost(
                            format!("response >{}B", MAX_RESPONSE_BYTES)
                        ));
                    }
                    if let Ok(resp) = serde_json::from_str::<Value>(buf.trim()) {
                        if let Some(error) = resp.get("error") {
                            return Err(CbmError::RpcError {
                                code: error["code"].as_i64().unwrap_or(-1),
                                message: error["message"].as_str().unwrap_or("unknown").into(),
                            });
                        }
                        return resp.get("result").cloned()
                            .ok_or_else(|| CbmError::ParseError("missing result".into()));
                    }
                    // Not yet complete JSON — continue reading lines
                }
                Err(e) => {
                    self.status = CbmStatus::Degraded(format!("read: {e}"));
                    return Err(CbmError::ConnectionLost(e.to_string()));
                }
            }
        }
    }

    // ── Response parsing ───────────────────────────────────────

    /// CBM wraps all tool responses in MCP content array format.
    /// The actual data is a JSON string inside `result.content[0].text`.
    /// This helper extracts and parses that inner JSON string.
    fn parse_cbm_response(&self, response: &Value) -> Result<Value, CbmError> {
        let content = response["content"].as_array()
            .and_then(|a| a.first())
            .ok_or_else(|| CbmError::ParseError("missing content array".into()))?;
        let text = content["text"].as_str()
            .ok_or_else(|| CbmError::ParseError("missing text field in content".into()))?;
        serde_json::from_str(text)
            .map_err(|e| CbmError::ParseError(format!("inner JSON parse: {e}")))
    }

    // ── Typed wrappers ──────────────────────────────────────────

    /// Search the CBM knowledge graph by name pattern and optional label filter.
    ///
    /// CBM params: `name_pattern`, `label`, `file_pattern`, `project`, `limit`, `offset`
    pub fn search_graph(&mut self, name_pattern: &str, project: &str, label: Option<&str>) -> Result<Vec<Value>, CbmError> {
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
    pub fn trace_path(&mut self, function_name: &str, direction: &str, project: &str, depth: Option<usize>) -> Result<Vec<Value>, CbmError> {
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
    pub fn query_graph(&mut self, cypher: &str, project: &str) -> Result<Vec<Vec<Value>>, CbmError> {
        let r = self.call_tool("query_graph", serde_json::json!({"query": cypher, "project": project}))?;
        let inner = self.parse_cbm_response(&r)?;
        Ok(inner["rows"].as_array()
            .map(|rows| {
                rows.iter().filter_map(|row| {
                    row.as_array().map(|cols| cols.clone())
                }).collect()
            })
            .unwrap_or_default())
    }

    /// Get symbol importance (caller count) from CBM via Cypher query.
    ///
    /// CBM has no dedicated `get_symbol_importance` tool, but `in_degree`
    /// on function nodes provides the same information.
    pub fn get_symbol_importance(&mut self, project: &str, min_degree: Option<usize>) -> Result<Vec<Value>, CbmError> {
        let min = min_degree.unwrap_or(1);
        let cypher = format!(
            "MATCH (f:Function) WHERE f.in_degree >= {} RETURN f.name, f.file_path, f.in_degree, f.out_degree ORDER BY f.in_degree DESC",
            min
        );
        let rows = self.query_graph(&cypher, project)?;
        Ok(rows.into_iter().map(|row| {
            let name = row.get(0).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let file = row.get(1).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let in_degree = row.get(2).and_then(|v| v.as_str()).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
            let out_degree = row.get(3).and_then(|v| v.as_str()).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
            serde_json::json!({
                "name": name,
                "file": file,
                "in_degree": in_degree as u64,
                "out_degree": out_degree as u64,
                "importance": in_degree / 100.0,  // normalized score for blending
            })
        }).collect())
    }

    /// Get dead code candidates from CBM via Cypher query.
    ///
    /// CBM has no dedicated `get_dead_code` tool, but functions with
    /// `in_degree = 0` and `is_entry_point = false` are dead code.
    pub fn get_dead_code(&mut self, project: &str) -> Result<Vec<Value>, CbmError> {
        let cypher = "MATCH (f:Function) WHERE f.in_degree = 0 AND f.is_entry_point = false RETURN f.name, f.file_path".to_string();
        let rows = self.query_graph(&cypher, project)?;
        Ok(rows.into_iter().map(|row| {
            serde_json::json!({
                "name": row.get(0).and_then(|v| v.as_str()).unwrap_or(""),
                "file": row.get(1).and_then(|v| v.as_str()).unwrap_or(""),
                "reason": "no callers",
            })
        }).collect())
    }
}

impl Drop for CbmClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

