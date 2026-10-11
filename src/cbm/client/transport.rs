//! Frontend launch and shared correlated, bounded pipe receive path.

use super::framing::{is_response, read_frame};
use super::lifecycle::{FrontendLaunchSpec, FrontendSession};
use super::{CbmClient, CbmError, MAX_RESPONSE_BYTES};
use crate::cbm::config::CbmStatus;
use serde_json::Value;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

impl CbmClient {
    /// Launch an owned frontend transport at the authoritative primary root.
    /// Successful construction is not evidence of provider admission (A3).
    ///
    /// **Stderr handling:** A background reader thread drains CBM's stderr
    /// to the host diagnostic stream. Without this, if CBM writes enough diagnostic output
    /// (~64KB), the pipe buffer fills and CBM blocks — causing a deadlock
    /// since we only read stdout. (H-1 regression guard).
    pub fn try_launch(
        binary_path: &Path,
        project_root: &Path,
        timeout: Duration,
        max_consecutive_failures: u32,
        circuit_cooldown_secs: u64,
    ) -> Result<Option<Self>, CbmError> {
        let Some(spec) = FrontendLaunchSpec::resolve(binary_path, project_root)? else {
            return Ok(None);
        };
        let mut command = Command::new(&spec.binary);
        #[cfg(test)]
        super::a2_support::configure_command(&mut command);
        let child = command
            .current_dir(&spec.root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| CbmError::LaunchError(format!("spawn: {e}")))?;
        #[cfg(test)]
        super::a2_support::record_child(child.id());
        let mut frontend = FrontendSession::new(child);
        frontend.stdin = Some(BufWriter::new(
            frontend
                .child
                .stdin
                .take()
                .ok_or_else(|| CbmError::LaunchError("no stdin".into()))?,
        ));
        let stdout = frontend
            .child
            .stdout
            .take()
            .ok_or_else(|| CbmError::LaunchError("no stdout".into()))?;
        let stderr = frontend
            .child
            .stderr
            .take()
            .ok_or_else(|| CbmError::LaunchError("no stderr".into()))?;

        // H-1 fix: drain stderr in a background thread to prevent deadlock.
        frontend.stderr_drainer = Some(
            spawn_worker("cbm-stderr-drain", move || {
                let reader = BufReader::new(stderr);
                for l in reader.lines().map_while(Result::ok) {
                    eprintln!("[cbm-stderr] {l}");
                }
            })
            .map_err(|e| CbmError::LaunchError(format!("stderr drainer setup: {e}")))?,
        );

        // C-2 fix: read stdout in a background thread and forward each line
        // over a channel. The main thread can then use recv_timeout() to
        // honour the per-request deadline without blocking indefinitely in
        // read_line().
        let (stdout_tx, stdout_rx) =
            std::sync::mpsc::sync_channel::<Result<String, std::io::Error>>(256);
        frontend.stdout_rx = Some(stdout_rx);
        #[cfg(test)]
        let queue_marker = super::a2_support::queue_marker();
        frontend.stdout_reader = Some(
            spawn_worker("cbm-stdout-reader", move || {
                let mut reader = BufReader::new(stdout);
                #[cfg(test)]
                let mut observed_frames = 0;
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
                            #[cfg(test)]
                            super::a2_support::observe_frame(&queue_marker, &mut observed_frames);
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
            .map_err(|e| CbmError::LaunchError(format!("stdout reader setup: {e}")))?,
        );

        Ok(Some(Self {
            frontend: Some(frontend),
            usable: false,
            admitted_once: false,
            launch_spec: spec,
            recovery_pending: false,
            last_retirement: None,
            request_id: AtomicU64::new(1),
            status: CbmStatus::Degraded("admission_required".into()),
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
        self.ensure_usable()?;
        self.send_rpc_request(
            "tools/call",
            serde_json::json!({ "name": tool_name, "arguments": args }),
            self.timeout,
        )
    }

    /// Initialization and ordinary calls share the identical A1 receive boundary.
    pub(super) fn send_rpc_request(
        &mut self,
        method: &str,
        params: Value,
        timeout: Duration,
    ) -> Result<String, CbmError> {
        self.ensure_frontend_with_deadline(Instant::now() + timeout)?;
        let id = self.request_id.fetch_add(1, Ordering::SeqCst);
        let request =
            serde_json::json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params });
        let req_line =
            serde_json::to_string(&request).map_err(|e| CbmError::ParseError(e.to_string()))?;
        let write_result = {
            let stdin = self
                .frontend
                .as_mut()
                .and_then(|frontend| frontend.stdin.as_mut())
                .ok_or_else(|| CbmError::ConnectionLost("frontend stdin absent".into()))?;
            writeln!(stdin, "{req_line}").and_then(|_| stdin.flush())
        };
        if let Err(error) = write_result {
            self.retire_frontend("stdin write/flush failed");
            return Err(CbmError::ConnectionLost(error.to_string()));
        }

        // The frontend mirrors our newline framing. Notifications do not complete
        // this call and never reset its existing receive deadline.
        let deadline = std::time::Instant::now() + timeout;
        loop {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if remaining.is_zero() {
                self.retire_frontend("timeout");
                return Err(CbmError::Timeout(timeout));
            }
            let received = self
                .frontend
                .as_ref()
                .and_then(|frontend| frontend.stdout_rx.as_ref())
                .ok_or_else(|| CbmError::ConnectionLost("frontend stdout absent".into()))?
                .recv_timeout(remaining);
            match received {
                Ok(Ok(line)) => match is_response(line.trim(), id) {
                    Ok(true) => return Ok(line),
                    Ok(false) => continue,
                    Err(error) => return Err(self.reject_protocol(error)),
                },
                Ok(Err(error)) => {
                    if error.kind() == std::io::ErrorKind::InvalidData {
                        return Err(self.reject_protocol(CbmError::ParseError(error.to_string())));
                    }
                    self.retire_frontend("stdout exited or failed");
                    return Err(CbmError::ConnectionLost(error.to_string()));
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    self.retire_frontend("timeout");
                    return Err(CbmError::Timeout(timeout));
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    self.retire_frontend("stdout exited or failed");
                    return Err(CbmError::ConnectionLost("CBM exited".into()));
                }
            }
        }
    }

    pub(super) fn send_initialized(&mut self, deadline: Instant) -> Result<(), CbmError> {
        self.ensure_frontend_with_deadline(deadline)?;
        let stdin = self
            .frontend
            .as_mut()
            .and_then(|frontend| frontend.stdin.as_mut())
            .ok_or_else(|| CbmError::ConnectionLost("frontend stdin absent".into()))?;
        let result = writeln!(
            stdin,
            "{{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}}"
        )
        .and_then(|_| stdin.flush());
        if let Err(error) = result {
            self.retire_frontend("initialized notification write failed");
            return Err(CbmError::ConnectionLost(error.to_string()));
        }
        Ok(())
    }

    #[cfg(test)]
    pub(super) fn retire_for_a2_test(&mut self) {
        self.retire_frontend("explicit retirement fixture");
    }

    // Protocol poison uses the same ownership transition as all transport failures.
    fn reject_protocol(&mut self, error: CbmError) -> CbmError {
        self.retire_frontend("protocol failure");
        error
    }
}

fn spawn_worker<F>(name: &str, work: F) -> std::io::Result<std::thread::JoinHandle<()>>
where
    F: FnOnce() + Send + 'static,
{
    #[cfg(test)]
    super::a2_support::check_worker(name)?;
    std::thread::Builder::new().name(name.into()).spawn(work)
}
