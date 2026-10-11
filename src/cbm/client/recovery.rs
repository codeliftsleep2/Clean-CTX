//! One serialized frontend replacement; transport recovery never authorizes replay.
use super::{CbmClient, CbmError, is_retryable};
use crate::cbm::config::CbmStatus;
use serde_json::Value;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

/// Pinned handlers are query-only. Unknown or mutation-capable tools never replay.
fn replay_safe(tool: &str, args: &Value) -> bool {
    // The production cache caller uses only project. scope="tests" enters
    // a separate work-producing workflow; other shapes have no replay proof.
    if tool == "detect_changes" {
        return args.as_object().is_some_and(|args| {
            args.len() == 1 && args.get("project").is_some_and(Value::is_string)
        });
    }
    matches!(
        tool,
        "search_graph" | "query_graph" | "trace_path" | "get_architecture" | "list_projects"
    )
}
impl CbmClient {
    /// Pure demand eligibility: status requests never launch or poll a provider.
    pub(crate) fn recovery_due(&self) -> bool {
        self.admitted_once
            && (self.recovery_pending
                || (self.usable && self.consecutive_failures >= self.max_consecutive_failures))
            && self.degraded_since.is_some_and(|since| {
                since.elapsed() >= Duration::from_secs(self.circuit_cooldown_secs)
            })
    }

    fn prepare_call(&mut self) -> Result<bool, CbmError> {
        if self.recovery_due() {
            self.replace_frontend()?;
            // One half-open attempt now belongs to a fresh, admitted epoch.
            self.consecutive_failures = 0;
            self.degraded_since = None;
            return Ok(true);
        }
        self.ensure_usable()?;
        if !self.circuit_allows() {
            return Err(CbmError::ConnectionLost("circuit breaker open".into()));
        }
        Ok(false)
    }

    pub(super) fn invoke_with_recovery<T>(
        &mut self,
        tool: &str,
        args: Value,
        attempt: fn(&mut Self, &str, Value) -> Result<T, CbmError>,
    ) -> Result<T, CbmError> {
        // An entry-time half-open replacement consumes this operation's retry
        // budget too. No later error can trigger another launch in this call.
        self.check_collection_deadline()?;
        let mut retry_spent = self.prepare_call()?;
        if let Some(state) = self.collection.as_mut() {
            retry_spent |= state.retry_spent;
            state.retry_spent |= retry_spent;
        }
        let mut interrupted = None;
        loop {
            self.check_collection_deadline()?;
            match attempt(self, tool, args.clone()) {
                Ok(result) => {
                    self.record_success();
                    return Ok(result);
                }
                Err(error) => {
                    let terminal_recoverable = self.frontend.is_none()
                        && self.admitted_once
                        && matches!(error, CbmError::ConnectionLost(_) | CbmError::Timeout(_))
                        && self
                            .last_retirement
                            .is_some_and(|outcome| outcome.child_reaped && outcome.workers_joined);
                    self.record_failure();
                    if terminal_recoverable {
                        self.recovery_pending = true;
                        self.degraded_since = Some(Instant::now());
                    }
                    if !retry_spent && terminal_recoverable {
                        retry_spent = true;
                        self.retry_pause()?;
                        if let Err(replacement) = self.replace_frontend() {
                            self.status = CbmStatus::Degraded(format!(
                                "original operation failure: {error}; replacement failed: {replacement}; cleanup: {:?}",
                                self.status
                            ));
                            return Err(error);
                        }
                        if replay_safe(tool, &args) {
                            interrupted = Some(error);
                            continue;
                        }
                        // A usable replacement says nothing about acceptance of
                        // the original mutation. Do not send its arguments again.
                        if self.consecutive_failures >= self.max_consecutive_failures {
                            self.status = CbmStatus::Degraded(format!(
                                "circuit_open_after_{}_failures; original operation failure: {error}; replacement admitted",
                                self.max_consecutive_failures
                            ));
                        }
                        return Err(error);
                    }
                    if !retry_spent
                        && self.frontend.is_some()
                        && replay_safe(tool, &args)
                        && is_retryable(&error)
                    {
                        retry_spent = true;
                        self.retry_pause()?;
                        interrupted = Some(error);
                        continue;
                    }
                    if let Some(original) = interrupted {
                        self.status = CbmStatus::Degraded(format!(
                            "original operation failure: {original}; replay failed: {error}; transport: {:?}",
                            self.status
                        ));
                        return Err(original);
                    }
                    return Err(error);
                }
            }
        }
    }

    fn replace_frontend(&mut self) -> Result<(), CbmError> {
        // Single &mut owner / existing bridge mutex. A failed attempt disables
        // recovery; permanent refusal cannot become a demand-driven respawn loop.
        self.recovery_pending = false;
        self.retire_frontend("replacement of owned frontend epoch");
        if self
            .last_retirement
            .is_none_or(|outcome| !outcome.child_reaped || !outcome.workers_joined)
        {
            return Err(CbmError::ConnectionLost(
                "frontend cleanup uncertain; replacement withheld".into(),
            ));
        }
        self.status = CbmStatus::Degraded("recovering".into());
        let spec = self.launch_spec.clone();
        let result = (|| {
            let mut replacement = Self::try_launch(
                &spec.binary,
                &spec.root,
                self.timeout,
                self.max_consecutive_failures,
                self.circuit_cooldown_secs,
            )?
            .ok_or_else(|| CbmError::LaunchError("replacement executable missing".into()))?;
            replacement.collection = self.collection.clone();
            if let Err(error) = replacement.admit() {
                self.status = replacement.status().clone();
                return Err(error);
            }
            self.frontend = replacement.frontend.take();
            self.usable = replacement.usable;
            self.request_id.store(
                replacement.request_id.load(Ordering::SeqCst),
                Ordering::SeqCst,
            );
            self.status = CbmStatus::Available;
            Ok(())
        })();
        if let Err(error) = &result {
            self.status = CbmStatus::Degraded(format!(
                "replacement failed: {error}; transport: {:?}",
                self.status
            ));
        }
        result
    }
}
