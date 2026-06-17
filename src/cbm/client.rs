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
                for line in reader.lines() {
                    if let Ok(l) = line {
                        eprintln!("[cbm-stderr] {l}");
                    }
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
    pub fn call_tool_raw(&mut self, tool_name: &str, args: Value) -> Result<String, CbmError> {
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
                    if let Ok(_) = serde_json::from_str::<Value>(buf.trim()) {
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
    pub fn call_tool(&mut self, tool_name: &str, args: Value) -> Result<Value, CbmError> {
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

    // ── Typed wrappers ──────────────────────────────────────────

    pub fn search_graph(&mut self, query: &str, project: &str) -> Result<Vec<Value>, CbmError> {
        let r = self.call_tool("search_graph", serde_json::json!({"query": query, "project": project}))?;
        Ok(r["results"].as_array().cloned().unwrap_or_default())
    }

    pub fn trace_path(&mut self, from: &str, to: &str, project: &str) -> Result<Vec<Value>, CbmError> {
        let r = self.call_tool("trace_path", serde_json::json!({"from": from, "to": to, "project": project}))?;
        Ok(r["edges"].as_array().cloned().unwrap_or_default())
    }

    pub fn get_architecture(&mut self, project: &str) -> Result<Value, CbmError> {
        self.call_tool("get_architecture", serde_json::json!({"project": project}))
    }

    pub fn query_graph(&mut self, cypher: &str, project: &str) -> Result<Vec<Value>, CbmError> {
        let r = self.call_tool("query_graph", serde_json::json!({"query": cypher, "project": project}))?;
        Ok(r["results"].as_array().cloned().unwrap_or_default())
    }

    pub fn get_symbol_importance(&mut self, project: &str) -> Result<Vec<Value>, CbmError> {
        let r = self.call_tool("get_symbol_importance", serde_json::json!({"project": project}))?;
        Ok(r["symbols"].as_array().cloned().unwrap_or_default())
    }

    pub fn get_dead_code(&mut self, project: &str) -> Result<Vec<Value>, CbmError> {
        let r = self.call_tool("get_dead_code", serde_json::json!({"project": project}))?;
        Ok(r["dead_code"].as_array().cloned().unwrap_or_default())
    }
}

impl Drop for CbmClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}