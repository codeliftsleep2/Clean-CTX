//! Frontend launch and shared correlated, bounded pipe receive path.

use super::framing::{is_response, read_frame};
use super::{CbmClient, CbmError, MAX_RESPONSE_BYTES};
use crate::cbm::config::CbmStatus;
use serde_json::Value;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

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
        let (stdout_tx, stdout_rx) =
            std::sync::mpsc::sync_channel::<Result<String, std::io::Error>>(256);
        let _stdout_reader = std::thread::Builder::new()
            .name("cbm-stdout-reader".into())
            .spawn(move || {
                let mut reader = BufReader::new(stdout);
                loop {
                    match read_frame(&mut reader, MAX_RESPONSE_BYTES) {
                        Ok(None) => {
                            let _ = stdout_tx.send(Err(std::io::Error::new(
                                std::io::ErrorKind::UnexpectedEof,
                                "CBM exited",
                            )));
                            break;
                        }
                        Ok(Some(line)) => {
                            if stdout_tx.send(Ok(line)).is_err() {
                                break;
                            }
                        }
                        Err(e) => {
                            let _ = stdout_tx.send(Err(e));
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
            protocol_failed: false,
            status: CbmStatus::Available,
            timeout,
            consecutive_failures: 0,
            degraded_since: None,
            max_consecutive_failures,
            circuit_cooldown_secs,
        }))
    }

    /// Send a JSON-RPC 2.0 request to CBM and read the raw response text.
    ///
    /// This is the **single shared transport pipeline** — both
    /// [`call_tool_inner`] and [`call_tool_raw_inner`] delegate here.
    ///
    /// Handles:
    /// - Request ID allocation
    /// - JSON-RPC envelope construction and serialization
    /// - stdin write + flush
    /// - Bounded newline frames via the background reader channel
    /// - Pending-request correlation and notification filtering
    /// - Timeout detection (kills the child on timeout)
    /// - Pipe death / disconnected-channel detection
    /// - Oversized-response enforcement
    /// - Degraded-status updates on transport errors
    ///
    /// Returns the complete raw JSON-RPC response text on success.
    /// Callers that need the parsed `result` field should call
    /// [`call_tool_inner`]; callers that need the raw intercepted text
    /// should call [`call_tool_raw_inner`].
    pub(super) fn send_and_receive_raw(
        &mut self,
        tool_name: &str,
        args: Value,
    ) -> Result<String, CbmError> {
        if self.protocol_failed {
            self.status = CbmStatus::Degraded("protocol_failed".into());
            return Err(CbmError::ParseError(
                "CBM frontend retired after protocol failure".into(),
            ));
        }
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

        // The frontend mirrors our newline framing. Notifications do not complete
        // this call and never reset its existing receive deadline.
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
                Ok(Ok(line)) => match is_response(line.trim(), id) {
                    Ok(true) => return Ok(line),
                    Ok(false) => continue,
                    Err(error) => return Err(self.reject_protocol(error)),
                },
                Ok(Err(error)) => {
                    if error.kind() == std::io::ErrorKind::InvalidData {
                        return Err(self.reject_protocol(CbmError::ParseError(error.to_string())));
                    }
                    self.status = CbmStatus::Degraded("exited".into());
                    return Err(CbmError::ConnectionLost(error.to_string()));
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

    // This narrow A1 fail-stop prevents uncertain framing/correlation from being
    // reused. Timeout recovery, replacement admission and retry policy are A2.
    fn reject_protocol(&mut self, error: CbmError) -> CbmError {
        self.protocol_failed = true;
        self.status = CbmStatus::Degraded("protocol_failed".into());
        let _ = self.child.kill();
        let _ = self.child.try_wait();
        error
    }
}
