//! Ownership of one frontend/session. No account-daemon control lives here.
use super::{CbmClient, CbmError};
use crate::cbm::config::CbmStatus;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin};
use std::sync::mpsc::Receiver;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

/// Derived from the bridge's existing primary-root authority, before spawning.
/// Environment remains inherited, including an explicit account cache override.
pub(super) struct FrontendLaunchSpec {
    pub binary: PathBuf,
    pub root: PathBuf,
}
impl FrontendLaunchSpec {
    pub fn resolve(binary: &Path, root: &Path) -> Result<Option<Self>, CbmError> {
        if !binary.is_file() {
            return Ok(None);
        }
        let binary = binary
            .canonicalize()
            .map_err(|e| CbmError::LaunchError(format!("resolve executable: {e}")))?;
        let root = root
            .canonicalize()
            .map_err(|e| CbmError::LaunchError(format!("resolve frontend root: {e}")))?;
        if !root.is_dir() {
            return Err(CbmError::LaunchError(
                "frontend root is not a directory".into(),
            ));
        }
        Ok(Some(Self { binary, root }))
    }
}

#[derive(Clone, Copy)]
pub(super) struct RetirementOutcome {
    pub child_reaped: bool,
    pub workers_joined: bool,
}

/// Also serves as the partial-launch guard: every installed resource is owned
/// immediately, and any setup failure drops this owner through the same cleanup.
pub(super) struct FrontendSession {
    pub child: Child,
    pub stdin: Option<BufWriter<ChildStdin>>,
    pub stdout_rx: Option<Receiver<Result<String, std::io::Error>>>,
    pub stdout_reader: Option<JoinHandle<()>>,
    pub stderr_drainer: Option<JoinHandle<()>>,
    retired: Option<RetirementOutcome>,
}
impl FrontendSession {
    pub fn new(child: Child) -> Self {
        Self {
            child,
            stdin: None,
            stdout_rx: None,
            stdout_reader: None,
            stderr_drainer: None,
            retired: None,
        }
    }

    pub fn retire(&mut self) -> RetirementOutcome {
        if let Some(outcome) = self.retired {
            return outcome;
        }
        // Do not flush buffered requests while retiring an uncertain session.
        if let Some(stdin) = self.stdin.take() {
            drop(stdin.into_parts());
        }
        // Release a reader blocked sending into the bounded channel first.
        self.stdout_rx.take();
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut child_reaped = matches!(self.child.try_wait(), Ok(Some(_)));
        if !child_reaped {
            // Only the direct frontend handle: never taskkill /T, jobs, process
            // groups, daemon IPC, or installation/account shutdown operations.
            let _ = self.child.kill();
        }
        while !child_reaped && Instant::now() < deadline {
            match self.child.try_wait() {
                Ok(Some(_)) => child_reaped = true,
                Err(_) => break,
                Ok(None) => std::thread::sleep(Duration::from_millis(10)),
            }
        }
        let mut workers_joined = true;
        for worker in [&mut self.stdout_reader, &mut self.stderr_drainer] {
            if let Some(handle) = worker.take() {
                // Killing/reaping the frontend closes its pipes. A shared daemon
                // or foreign process retaining a pipe must not cause an unbounded
                // join or trigger a global kill. Report that uncertainty honestly.
                while !handle.is_finished() && Instant::now() < deadline {
                    std::thread::sleep(Duration::from_millis(10));
                }
                if handle.is_finished() {
                    workers_joined &= handle.join().is_ok();
                } else {
                    workers_joined = false;
                }
            }
        }
        let outcome = RetirementOutcome {
            child_reaped,
            workers_joined,
        };
        self.retired = Some(outcome);
        if !child_reaped || !workers_joined {
            eprintln!(
                "[clean-ctx-cbm] Frontend cleanup uncertain: reaped={child_reaped}, workers_joined={workers_joined}"
            );
        }
        outcome
    }
}
impl Drop for FrontendSession {
    fn drop(&mut self) {
        self.retire();
    }
}

impl CbmClient {
    /// Terminal transition. Replacement and admission are deliberately absent.
    pub(super) fn retire_frontend(&mut self, reason: &str) {
        self.usable = false;
        if let Some(mut frontend) = self.frontend.take() {
            let outcome = frontend.retire();
            self.last_retirement = Some(outcome);
            self.status = CbmStatus::Degraded(if outcome.child_reaped && outcome.workers_joined {
                format!("frontend_retired: {reason}")
            } else {
                format!("frontend_retired_cleanup_uncertain: {reason}")
            });
        }
    }

    /// Check direct process/reader liveness before allocating or writing a new
    /// request. Serialized &mut access prevents retirement/write ownership races.
    pub(super) fn ensure_frontend(&mut self) -> Result<(), CbmError> {
        self.ensure_frontend_with_deadline(Instant::now() + self.timeout)
    }

    pub(super) fn ensure_frontend_with_deadline(
        &mut self,
        deadline: Instant,
    ) -> Result<(), CbmError> {
        let failure = match self.frontend.as_mut() {
            None => return Err(CbmError::ConnectionLost("frontend retired".into())),
            Some(frontend) => match frontend.child.try_wait() {
                Ok(Some(_)) => Some("frontend exited".to_string()),
                Err(error) => Some(format!("frontend status: {error}")),
                Ok(None)
                    if frontend
                        .stdout_reader
                        .as_ref()
                        .is_none_or(JoinHandle::is_finished) =>
                {
                    Some("stdout reader terminated".into())
                }
                Ok(None) => None,
            },
        };
        if let Some(failure) = failure {
            let error = if !self.usable {
                self.terminal_startup_error(deadline)
            } else {
                CbmError::ConnectionLost(failure)
            };
            self.retire_frontend(&error.to_string());
            return Err(error);
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../tests/cbm/frontend_launch_spec.rs"]
mod tests;
