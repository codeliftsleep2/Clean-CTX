//! Explicit request-local reservations. No default profile or public handler uses this API.
//! Declared byte reservations are not allocator interception or hard process ceilings.
use super::{Admitted, Candidate, Refusal};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Cumulative work except retained bytes, which are live reservations.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cost {
    pub source_bytes: u64,
    pub support_bytes: u64,
    pub retained_bytes: u64,
    pub units: u64,
    pub edges: u64,
    pub validation_bytes: u64,
    pub validation_checks: u64,
    pub response_bytes: u64,
}
impl Cost {
    fn values(self) -> [u64; 8] {
        [
            self.source_bytes,
            self.support_bytes,
            self.retained_bytes,
            self.units,
            self.edges,
            self.validation_bytes,
            self.validation_checks,
            self.response_bytes,
        ]
    }
    fn from_values(v: [u64; 8]) -> Self {
        Self {
            source_bytes: v[0],
            support_bytes: v[1],
            retained_bytes: v[2],
            units: v[3],
            edges: v[4],
            validation_bytes: v[5],
            validation_checks: v[6],
            response_bytes: v[7],
        }
    }
}
/// All values must be explicitly supplied; zero disallows that dimension.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub total: Cost,
    pub finalization: Cost,
    pub deadline: Duration,
    pub finalization_time: Duration,
    pub active_tasks: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stop {
    Cancelled,
    Deadline,
    ActiveTasks,
    SourceBytes,
    SupportBytes,
    RetainedBytes,
    Units,
    Edges,
    ValidationBytes,
    ValidationChecks,
    ResponseBytes,
}
const DIMENSIONS: [Stop; 8] = [
    Stop::SourceBytes,
    Stop::SupportBytes,
    Stop::RetainedBytes,
    Stop::Units,
    Stop::Edges,
    Stop::ValidationBytes,
    Stop::ValidationChecks,
    Stop::ResponseBytes,
];

#[derive(Clone, Debug)]
pub struct Snapshot {
    pub used: Cost,
    pub finalization_used: Cost,
    pub retained_peak: u64,
    pub active_tasks: usize,
    pub active_peak: usize,
    pub elapsed: Duration,
    pub deadline_overshoot: Duration,
    pub optional_cancelled: bool,
    pub last_stop: Option<Stop>,
    pub retained_released: u64,
    pub pending_response_bytes: u64,
}
struct State {
    used: Cost,
    finalization_used: Cost,
    retained_peak: u64,
    active: usize,
    active_peak: usize,
    cancelled: bool,
    last_stop: Option<Stop>,
    work: WorkCounts,
    retained_released: u64,
    response_reserved: u64,
}
#[derive(Clone, Copy, Debug, Default, serde::Serialize)]
pub struct WorkCounts {
    pub directory_enumerations: u64,
    pub metadata_operations: u64,
    pub physical_path_validations: u64,
    pub exclusion_checks: u64,
}
#[derive(Clone, Copy)]
pub(crate) enum Work {
    Directory,
    DirectoryEntry,
    Metadata,
    Physical,
    Exclusion,
}
struct Inner {
    limits: Limits,
    start: Instant,
    state: Mutex<State>,
    integrated: bool,
}
/// Clones and nested work share the same counters and cancellation state.
#[derive(Clone)]
pub struct Request {
    inner: Arc<Inner>,
}

/// Keeps declared allocation ownership charged until the result is dropped.
/// Dropping the permit before dropping the actual allocation violates this API.
pub struct Reserved<T> {
    value: T,
    reservation: Permit,
    pub completed_after_stop: Option<Stop>,
}
impl<T> Reserved<T> {
    pub(crate) fn into_parts(self) -> (T, Permit) {
        (self.value, self.reservation)
    }
    /// Transform an owned result without releasing its allocation reservation.
    /// Additional work/allocations inside the transform need their own charges.
    pub fn map<U>(self, transform: impl FnOnce(T) -> U) -> Reserved<U> {
        Reserved {
            value: transform(self.value),
            reservation: self.reservation,
            completed_after_stop: self.completed_after_stop,
        }
    }
    pub fn value(&self) -> &T {
        &self.value
    }
    pub fn reserved_bytes(&self) -> u64 {
        self.reservation.bytes
    }
}
/// Reservation for an active operation. Never wait/retry implicitly on exhaustion.
pub struct Permit {
    request: Request,
    bytes: u64,
    active: bool,
    finalizing: bool,
}
impl Permit {
    pub(crate) fn reserved_bytes(&self) -> u64 {
        self.bytes
    }
    pub(crate) fn finish(mut self) -> Self {
        if self.active {
            let mut s = self.request.inner.state.lock().expect("budget lock");
            s.active -= 1;
            self.active = false;
        }
        self
    }
}
impl Drop for Permit {
    fn drop(&mut self) {
        let mut s = self.request.inner.state.lock().expect("budget lock");
        if self.active {
            s.active -= 1;
        }
        s.used.retained_bytes -= self.bytes;
        if let Some(total) = s.retained_released.checked_add(self.bytes) {
            s.retained_released = total;
        } else {
            s.last_stop = Some(Stop::RetainedBytes);
            s.retained_released = u64::MAX;
        }
        if self.finalizing && self.request.is_integrated() {
            s.finalization_used.retained_bytes -= self.bytes;
        }
    }
}
impl Request {
    pub(crate) fn hold<T>(&self, value: T, permit: Permit, late: Option<Stop>) -> Reserved<T> {
        Reserved {
            value,
            reservation: permit.finish(),
            completed_after_stop: late,
        }
    }
    pub fn new(limits: Limits) -> Result<Self, &'static str> {
        if limits.active_tasks == 0
            || limits.deadline.is_zero()
            || limits.finalization_time > limits.deadline
            || limits
                .finalization
                .values()
                .iter()
                .zip(limits.total.values())
                .any(|(r, t)| *r > t)
        {
            return Err("invalid explicit request limits");
        }
        Ok(Self {
            inner: Arc::new(Inner {
                limits,
                start: Instant::now(),
                state: Mutex::new(State {
                    used: Cost::default(),
                    finalization_used: Cost::default(),
                    retained_peak: 0,
                    active: 0,
                    active_peak: 0,
                    cancelled: false,
                    last_stop: None,
                    work: WorkCounts::default(),
                    retained_released: 0,
                    response_reserved: 0,
                }),
                integrated: false,
            }),
        })
    }
    pub(crate) fn integrated(limits: Limits) -> Result<Self, &'static str> {
        if Instant::now().checked_add(limits.deadline).is_none()
            || limits
                .total
                .source_bytes
                .checked_add(limits.total.support_bytes)
                .is_none()
            || limits
                .total
                .values()
                .iter()
                .any(|v| usize::try_from(*v).is_err())
        {
            return Err("unrepresentable integration limits");
        }
        let mut request = Self::new(limits)?;
        Arc::get_mut(&mut request.inner)
            .expect("new request")
            .integrated = true;
        Ok(request)
    }
    pub(crate) fn is_integrated(&self) -> bool {
        self.inner.integrated
    }
    pub(crate) fn stop(&self, reason: Stop) -> Stop {
        self.inner.state.lock().expect("budget lock").last_stop = Some(reason);
        reason
    }
    pub(crate) fn limits(&self) -> Limits {
        self.inner.limits
    }
    pub fn work_counts(&self) -> WorkCounts {
        self.inner.state.lock().expect("budget lock").work
    }
    pub(crate) fn checkpoint_final(&self) -> Result<(), Stop> {
        self.check_at(true, self.inner.start.elapsed())
    }
    pub(crate) fn reserve_nested(&self, cost: Cost, finalizing: bool) -> Result<Permit, Stop> {
        self.reserve_kind(cost, finalizing, false)
    }
    pub(crate) fn work(&self, kind: Work, finalizing: bool) -> Result<(), Stop> {
        if !self.is_integrated() {
            return Ok(());
        }
        let cost = if finalizing {
            Cost {
                validation_checks: 1,
                ..Cost::default()
            }
        } else {
            Cost {
                units: 1,
                ..Cost::default()
            }
        };
        let _charge = self.reserve_nested(cost, finalizing)?;
        let mut state = self.inner.state.lock().expect("budget lock");
        let count = match kind {
            Work::Directory => &mut state.work.directory_enumerations,
            Work::DirectoryEntry => return Ok(()),
            Work::Metadata => &mut state.work.metadata_operations,
            Work::Physical => &mut state.work.physical_path_validations,
            Work::Exclusion => &mut state.work.exclusion_checks,
        };
        *count = count.checked_add(1).ok_or(if finalizing {
            Stop::ValidationChecks
        } else {
            Stop::Units
        })?;
        Ok(())
    }
    pub fn cancel_optional(&self) {
        self.inner.state.lock().expect("budget lock").cancelled = true;
    }
    fn check_at(&self, finalization: bool, elapsed: Duration) -> Result<(), Stop> {
        let mut s = self.inner.state.lock().expect("budget lock");
        let stop = if !finalization && self.inner.integrated && s.last_stop.is_some() {
            s.last_stop
        } else if !finalization && s.cancelled {
            Some(Stop::Cancelled)
        } else if elapsed
            >= if finalization {
                self.inner.limits.deadline
            } else {
                self.inner.limits.deadline - self.inner.limits.finalization_time
            }
        {
            Some(Stop::Deadline)
        } else {
            None
        };
        if let Some(reason) = stop {
            s.last_stop = Some(reason);
            Err(reason)
        } else {
            Ok(())
        }
    }
    /// Cooperative checkpoint; no filesystem work, sleeping or retry.
    pub fn checkpoint(&self) -> Result<(), Stop> {
        self.check_at(false, self.inner.start.elapsed())
    }
    pub fn reserve_optional(&self, cost: Cost) -> Result<Permit, Stop> {
        self.reserve(cost, false)
    }
    pub fn reserve_finalization(&self, cost: Cost) -> Result<Permit, Stop> {
        self.reserve(cost, true)
    }
    fn reserve(&self, cost: Cost, finalization: bool) -> Result<Permit, Stop> {
        self.reserve_kind(cost, finalization, true)
    }
    /// Nested input I/O belongs to the already active quantum, not another worker.
    pub(crate) fn reserve_input(&self, cost: Cost) -> Result<Permit, Stop> {
        self.reserve_kind(cost, false, false)
    }
    fn reserve_kind(&self, cost: Cost, finalization: bool, active: bool) -> Result<Permit, Stop> {
        self.check_at(finalization, self.inner.start.elapsed())?;
        let mut s = self.inner.state.lock().expect("budget lock");
        // Recheck time/cancellation under the same lock as allocation admission.
        let elapsed = self.inner.start.elapsed();
        let stop = if !finalization && s.cancelled {
            Some(Stop::Cancelled)
        } else if elapsed
            >= if finalization {
                self.inner.limits.deadline
            } else {
                self.inner.limits.deadline - self.inner.limits.finalization_time
            }
        {
            Some(Stop::Deadline)
        } else if active && s.active >= self.inner.limits.active_tasks {
            Some(Stop::ActiveTasks)
        } else {
            None
        };
        if let Some(reason) = stop {
            s.last_stop = Some(reason);
            return Err(reason);
        }
        let used = s.used.values();
        let final_used = s.finalization_used.values();
        let total = self.inner.limits.total.values();
        let reserve = self.inner.limits.finalization.values();
        let additions = cost.values();
        let mut next = used;
        let mut next_final = final_used;
        for i in 0..8 {
            let Some(sum) = used[i].checked_add(additions[i]) else {
                s.last_stop = Some(DIMENSIONS[i]);
                return Err(DIMENSIONS[i]);
            };
            let protected = if finalization {
                0
            } else {
                reserve[i].saturating_sub(final_used[i])
            };
            if sum > total[i].saturating_sub(protected) {
                s.last_stop = Some(DIMENSIONS[i]);
                return Err(DIMENSIONS[i]);
            }
            next[i] = sum;
            if finalization {
                next_final[i] = final_used[i].saturating_add(additions[i]);
            }
        }
        s.used = Cost::from_values(next);
        s.finalization_used = Cost::from_values(next_final);
        s.retained_peak = s.retained_peak.max(s.used.retained_bytes);
        s.active += usize::from(active);
        s.active_peak = s.active_peak.max(s.active);
        Ok(Permit {
            request: self.clone(),
            bytes: cost.retained_bytes,
            active,
            finalizing: finalization,
        })
    }
    /// One non-interruptible quantum. Nested work must use the supplied same Request.
    /// Returned data stays charged; a late result is explicitly marked, not erased.
    pub fn run_optional<T>(
        &self,
        cost: Cost,
        work: impl FnOnce(&Request) -> T,
    ) -> Result<Reserved<T>, Stop> {
        let permit = self.reserve_optional(cost)?;
        let value = work(self);
        let completed_after_stop = self.checkpoint().err();
        Ok(Reserved {
            value,
            reservation: permit.finish(),
            completed_after_stop,
        })
    }
    /// Independent E2 admission. Optional cancellation does not skip source validation.
    /// Expected source bytes are reserved before reads; concurrent file growth can overrun
    /// the declaration, and is rejected by E2's hash proof rather than accepted as fresh.
    pub fn admit(&self, candidate: Candidate) -> Result<Reserved<Result<Admitted, Refusal>>, Stop> {
        if self.is_integrated() {
            let current = super::bounded::validate(self, &candidate.witnesses)?;
            return Ok(current.map(|valid| {
                if valid {
                    Ok(Admitted { candidate })
                } else {
                    Err(Refusal::StaleSource)
                }
            }));
        }
        // Conservative count avoids allocating a second proof set merely to budget
        // the existing validator. Actual validation still deduplicates typed witnesses.
        let checks = candidate.witnesses.len() as u64;
        let bytes = candidate.source_bytes as u64;
        let permit = self.reserve_finalization(Cost {
            validation_bytes: bytes,
            validation_checks: checks,
            ..Cost::default()
        })?;
        let value = candidate.admit();
        let completed_after_stop = self.check_at(true, self.inner.start.elapsed()).err();
        Ok(Reserved {
            value,
            reservation: permit.finish(),
            completed_after_stop,
        })
    }
    /// Charge a later use-boundary validation to this same request.
    pub fn validate(&self, admitted: &Admitted) -> Result<Reserved<bool>, Stop> {
        if self.is_integrated() {
            return super::bounded::validate(self, &admitted.candidate.witnesses);
        }
        let cost = Cost {
            validation_bytes: admitted.candidate.source_bytes as u64,
            validation_checks: admitted.candidate.witnesses.len() as u64,
            ..Cost::default()
        };
        let permit = self.reserve_finalization(cost)?;
        let value = admitted.is_current();
        let completed_after_stop = self.check_at(true, self.inner.start.elapsed()).err();
        Ok(Reserved {
            value,
            reservation: permit.finish(),
            completed_after_stop,
        })
    }
    pub(crate) fn finish_validation<T>(
        &self,
        permit: Permit,
        work: impl FnOnce() -> T,
    ) -> Result<Reserved<T>, Stop> {
        let value = work();
        let completed_after_stop = self.check_at(true, self.inner.start.elapsed()).err();
        Ok(Reserved {
            value,
            reservation: permit.finish(),
            completed_after_stop,
        })
    }
    pub(crate) fn response_available(&self) -> u64 {
        let s = self.inner.state.lock().expect("response capacity");
        self.inner
            .limits
            .total
            .response_bytes
            .saturating_sub(s.used.response_bytes)
    }
    pub fn snapshot(&self) -> Snapshot {
        let elapsed = self.inner.start.elapsed();
        let s = self.inner.state.lock().expect("budget lock");
        let mut used = s.used;
        used.response_bytes -= s.response_reserved;
        let mut finalization_used = s.finalization_used;
        finalization_used.response_bytes -= s.response_reserved;
        Snapshot {
            used,
            finalization_used,
            pending_response_bytes: s.response_reserved,
            retained_peak: s.retained_peak,
            active_tasks: s.active,
            active_peak: s.active_peak,
            elapsed,
            deadline_overshoot: elapsed.saturating_sub(self.inner.limits.deadline),
            optional_cancelled: s.cancelled,
            last_stop: s.last_stop,
            retained_released: s.retained_released,
        }
    }
}

#[cfg(test)]
#[path = "../../tests/ir/written_source_budget_e3.rs"]
mod tests;

#[cfg(feature = "resolver_evidence_v2")]
#[path = "response_capacity.rs"]
pub(crate) mod response_capacity;

#[cfg(feature = "resolver_evidence_v2")]
#[path = "evidence_telemetry.rs"]
pub(crate) mod telemetry;
