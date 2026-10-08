use super::super::AuthorityError;
use serde::Serialize;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CancellationSource {
    User,
    Host,
}

/// A first-request-wins cancellation handle. It grants no execution authority.
#[derive(Debug, Clone, Default)]
pub struct CargoCheckCancellation(Arc<Mutex<Option<(CancellationSource, Instant)>>>);

impl CargoCheckCancellation {
    pub fn request(&self, source: CancellationSource) {
        let mut request = self.0.lock().unwrap_or_else(|error| error.into_inner());
        if request.is_none() {
            *request = Some((source, Instant::now()));
        }
    }

    pub(super) fn observed(&self) -> Option<(CancellationSource, Instant)> {
        *self.0.lock().unwrap_or_else(|error| error.into_inner())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ProcessOutcome {
    pub exit_code: Option<i64>,
    pub signal: Option<i32>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct StreamCaptureFacts {
    pub observed_bytes: u64,
    pub admitted_bytes: u64,
    pub discarded_bytes: u64,
    pub capture_limit_activated: bool,
    pub frames_observed: u64,
    pub complete_frames: u64,
    pub over_limit_frames: u64,
    pub admission_cut_frames: u64,
    pub unterminated_frames: u64,
    pub maximum_observed_frame_bytes: u64,
    pub eof_observed: bool,
    pub read_error_os_code: Option<i32>,
    pub read_failed: bool,
    pub stopped_before_eof: bool,
    pub post_limit_stderr_frame_candidates: u64,
    pub post_limit_stderr_samples_replaced: u64,
    pub post_limit_stderr_frames_withheld: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CaptureFacts {
    pub stdout: StreamCaptureFacts,
    pub stderr: StreamCaptureFacts,
    pub aggregate_admitted_bytes: u64,
    pub aggregate_discarded_bytes: u64,
    pub spool_used: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OwnershipFacts {
    pub mechanism: &'static str,
    pub ownership_established_before_execution: bool,
    pub controller_death_cleanup_guaranteed: bool,
    pub deliberate_detachment_excluded: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct CleanupFacts {
    pub cancellation_source: Option<CancellationSource>,
    pub cancellation_requested_after_millis: Option<u64>,
    pub forced_termination: bool,
    pub termination_request_succeeded: bool,
    pub forced_mechanism: Option<&'static str>,
    pub descendant_quiescence_observed: bool,
    pub control_failed: bool,
    pub control_error_os_code: Option<i32>,
    pub cleanup_uncertain: bool,
}

#[derive(Debug, Error)]
pub enum ExecutionError {
    #[error("CargoCheck authority changed before execution")]
    Authority(#[from] AuthorityError),
    #[error("CargoCheck owned process could not be started (OS code {os_code:?})")]
    Start { os_code: Option<i32> },
    #[error("CargoCheck process ownership is unsupported on this platform")]
    UnsupportedPlatform,
    #[error("CargoCheck was cancelled before process creation")]
    CancelledBeforeStart {
        cancellation_source: CancellationSource,
    },
}

impl From<std::io::Error> for ExecutionError {
    fn from(error: std::io::Error) -> Self {
        Self::Start {
            os_code: error.raw_os_error(),
        }
    }
}
