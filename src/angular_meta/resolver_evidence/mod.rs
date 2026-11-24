//! Gated internal E6-B operation. No MCP handler, index or durable owner calls this.
mod claims;
mod dto;
mod inventory;
mod response;
use super::{AngularRole, Prepared};
use crate::ir::source_observations::{
    Admitted, WrittenExpression, bounded,
    direct_identity::{self, Envelopes, Evidence, Obligation},
    request_budget::{Cost, Limits, Permit, Request, Stop},
};
pub use dto::OwnedDto;
pub use response::Output;
use std::{path::PathBuf, sync::Arc, time::Instant};

#[derive(Clone, Debug)]
pub struct Query {
    pub workspace_root: PathBuf,
    pub contract_version: u32,
    pub mode: String,
    pub canonical_policy: String,
    pub direction: String,
    pub selector_basis: String,
    pub selector: String,
    pub source_files: Option<Vec<PathBuf>>,
    pub within_path: Option<String>,
}
impl Query {
    pub fn forward(root: PathBuf, selector: String, source_files: Option<Vec<PathBuf>>) -> Self {
        Self {
            workspace_root: root,
            contract_version: 2,
            mode: "interactive".into(),
            canonical_policy: "none".into(),
            direction: "forward".into(),
            selector_basis: "route_literal".into(),
            selector,
            source_files,
            within_path: None,
        }
    }
    fn validate(&self, roots: &[PathBuf]) -> Result<(), Failure> {
        if self.contract_version != 2
            || self.mode != "interactive"
            || self.canonical_policy != "none"
            || roots.first() != Some(&self.workspace_root)
            || !matches!(
                (self.direction.as_str(), self.selector_basis.as_str()),
                ("forward", "route_literal") | ("reverse", "written_reference")
            )
            || (self.direction == "reverse" && self.selector.is_empty())
            || self.source_files.as_ref().is_some_and(Vec::is_empty)
        {
            return Err(Failure::Invalid(
                "unsupported or incompatible resolver-evidence request".into(),
            ));
        }
        Ok(())
    }
    fn selects(&self, observation: &crate::ir::source_observations::MappingObservation) -> bool {
        if self.direction == "forward" {
            observation.route_path == self.selector
        } else {
            matches!(&observation.expression, WrittenExpression::ReferenceText { text, .. } if text == &self.selector)
        }
    }
}
#[derive(Debug, PartialEq, Eq)]
pub enum Failure {
    Invalid(String),
    Budget(Stop),
    Integrity(String),
}

/// One allowance and cancellation domain. Cloning/sharing never resets capacity.
#[derive(Clone)]
pub struct Session {
    request: Request,
    roots: Arc<Vec<PathBuf>>,
    _context: Arc<Permit>,
}
impl Session {
    pub fn new(authorized_roots: Vec<PathBuf>, mut limits: Limits) -> Result<Self, Failure> {
        let roots_size = authorized_roots
            .iter()
            .try_fold(0u64, |n, p| n.checked_add(bounded::path_cost(p).ok()?))
            .ok_or(Stop::RetainedBytes)?;
        let floor = roots_size
            .checked_add(16384)
            .and_then(|v| v.checked_mul(8))
            .ok_or(Stop::RetainedBytes)?;
        if floor > limits.total.retained_bytes {
            return Err(Failure::Budget(Stop::RetainedBytes));
        }
        limits.finalization.retained_bytes = limits.finalization.retained_bytes.max(floor);
        Self::with_ledger(
            authorized_roots,
            Request::integrated(limits).map_err(|e| Failure::Invalid(e.into()))?,
        )
    }
    pub fn with_ledger(authorized_roots: Vec<PathBuf>, request: Request) -> Result<Self, Failure> {
        if !request.is_integrated()
            || authorized_roots.is_empty()
            || authorized_roots
                .iter()
                .any(|p| !p.is_absolute() || p.to_str().is_none())
        {
            return Err(Failure::Invalid(
                "integration requires explicit roots and an integrated ledger".into(),
            ));
        }
        let bytes = authorized_roots
            .iter()
            .try_fold(4096u64, |n, p| n.checked_add(bounded::path_cost(p).ok()?))
            .ok_or(Stop::RetainedBytes)?;
        let permit = request.reserve_input(Cost {
            retained_bytes: bytes,
            ..Cost::default()
        })?;
        Ok(Self {
            request,
            roots: Arc::new(authorized_roots),
            _context: Arc::new(permit),
        })
    }
    pub fn ledger(&self) -> Request {
        self.request.clone()
    }
    pub fn execute(&self, query: Query) -> Result<Output, Failure> {
        self.execute_inner(query, || {}, None)
    }
    #[cfg(test)]
    fn before_serving(&self, query: Query, hook: impl FnOnce()) -> Result<Output, Failure> {
        self.execute_inner(query, hook, None)
    }
    /// Borrow the immutable session source-exclusion policy; no new traversal.
    #[cfg(feature = "resolver_evidence_v2")]
    pub(crate) fn execute_with_policy(
        &self,
        query: Query,
        policy: &crate::config::CleanCtxConfig,
    ) -> Result<Output, Failure> {
        self.execute_inner(query, || {}, Some(policy))
    }
    fn execute_inner(
        &self,
        query: Query,
        before_serving: impl FnOnce(),
        policy: Option<&crate::config::CleanCtxConfig>,
    ) -> Result<Output, Failure> {
        query.validate(&self.roots)?;
        let started = Instant::now();
        let base_reservation = self.request.reserve_nested(
            Cost {
                retained_bytes: dto::base_cost(&query, &self.roots)?,
                ..Cost::default()
            },
            true,
        )?;
        response::preflight(&self.request, &query, &self.roots)?;
        let inventory_started = Instant::now();
        let mut timings = serde_json::Map::new();
        timings.insert(
            "preflight_us".into(),
            serde_json::json!(inventory_started.duration_since(started).as_micros()),
        );
        let inventory = match inventory::collect(&self.request, &self.roots, &query, policy) {
            Ok(inventory) => inventory,
            Err(Failure::Budget(stop)) => {
                return response::empty(&self.request, &query, &self.roots, stop, base_reservation);
            }
            Err(e) => return Err(e),
        };
        timings.insert(
            "inventory_us".into(),
            serde_json::json!(inventory_started.elapsed().as_micros()),
        );
        let mut sources = Vec::new();
        let mut failures = inventory.omissions.clone();
        let mut stopped = self.request.snapshot().last_stop;
        let capture_time = Instant::now();
        let mut source_late = 0u64;
        if stopped.is_none() {
            for file in &inventory.files {
                match bounded::capture(&self.request, &self.roots, file) {
                    Ok(capture) => {
                        stopped = capture.completed_after_stop;
                        source_late += u64::from(stopped.is_some());
                        let (candidate, retained) = capture.into_parts();
                        match candidate {
                            Ok(candidate) => {
                                let admitted = match self.request.admit(candidate) {
                                    Ok(admitted) => admitted,
                                    Err(stop) => {
                                        stopped = Some(stop);
                                        failures.push(format!(
                                            "{} admission: {stop:?}",
                                            file.display()
                                        ));
                                        break;
                                    }
                                };
                                if let Some(stop) = admitted.completed_after_stop {
                                    return Err(Failure::Budget(stop));
                                }
                                let (admitted, validation_permit) = admitted.into_parts();
                                drop(validation_permit);
                                match admitted {
                                    Ok(source) => {
                                        let dto_reservation = match self.request.reserve_nested(
                                            Cost {
                                                retained_bytes: bounded::dto_cost(&source)?,
                                                ..Cost::default()
                                            },
                                            true,
                                        ) {
                                            Ok(permit) => permit,
                                            Err(stop) => {
                                                stopped = Some(stop);
                                                failures.push(format!(
                                                    "{} response ownership: {stop:?}",
                                                    file.display()
                                                ));
                                                break;
                                            }
                                        };
                                        sources.push(Source {
                                            source,
                                            _retained: retained,
                                            dto_reservation,
                                            support: Vec::new(),
                                        });
                                    }
                                    Err(error) => {
                                        failures.push(format!("{}: {error:?}", file.display()))
                                    }
                                }
                            }
                            Err(error) => failures.push(format!("{}: {error:?}", file.display())),
                        }
                    }
                    Err(stop) => {
                        stopped = Some(stop);
                        failures.push(format!("{}: {stop:?}", file.display()));
                    }
                }
                if stopped.is_some() {
                    break;
                }
            }
        }
        timings.insert(
            "source_acquisition_us".into(),
            serde_json::json!(capture_time.elapsed().as_micros()),
        );
        timings.insert(
            "source_completed_after_stop".into(),
            serde_json::json!(source_late),
        );
        let optional_time = Instant::now();
        let (mut direct_us, mut role_us, mut direct_attempts, mut role_attempts) =
            (0u128, 0u128, 0u64, 0u64);
        for source in &mut sources {
            for index in 0..source.source.observations().len() {
                if !query.selects(&source.source.observations()[index]) {
                    continue;
                }
                let envelopes = Envelopes {
                    parse_retained_bytes: 0,
                    result_retained_bytes: bounded::envelope(
                        bounded::source_bytes(&source.source),
                        8,
                    )?,
                };
                let direct_start = Instant::now();
                let direct = if let Some(stop) = stopped {
                    Err(Obligation::Budget(stop))
                } else {
                    direct_attempts += 1;
                    direct_identity::capture(
                        &self.request,
                        &source.source,
                        index,
                        &inventory.roots,
                        envelopes,
                    )
                };
                direct_us += direct_start.elapsed().as_micros();
                stopped = stopped.or(self.request.snapshot().last_stop);
                let role_start = Instant::now();
                let role = if let Some(stop) = stopped {
                    Err(Obligation::Budget(stop))
                } else {
                    role_attempts += 1;
                    AngularRole::acquire(
                        &self.request,
                        &source.source,
                        index,
                        &inventory.roots,
                        envelopes,
                    )
                };
                role_us += role_start.elapsed().as_micros();
                stopped = stopped.or(self.request.snapshot().last_stop);
                source.support.push(Support {
                    index,
                    direct,
                    role,
                });
            }
        }
        timings.insert(
            "direct_and_role_us".into(),
            serde_json::json!(optional_time.elapsed().as_micros()),
        );
        timings.insert("direct_us".into(), serde_json::json!(direct_us));
        timings.insert("role_us".into(), serde_json::json!(role_us));
        timings.insert("direct_attempts".into(), serde_json::json!(direct_attempts));
        timings.insert("role_attempts".into(), serde_json::json!(role_attempts));
        #[cfg(test)]
        bounded::test_hooks::before_serving();
        before_serving();
        response::finish(
            &self.request,
            &query,
            WorkResult {
                inventory,
                sources,
                failures,
                stopped,
                timings,
                base_reservation,
            },
        )
    }
}
struct Source {
    source: Admitted,
    _retained: Permit,
    dto_reservation: Permit,
    support: Vec<Support>,
}
struct Support {
    index: usize,
    direct: Result<Evidence, Obligation>,
    role: Result<AngularRole, Obligation>,
}
struct WorkResult {
    inventory: inventory::Inventory,
    sources: Vec<Source>,
    failures: Vec<String>,
    stopped: Option<Stop>,
    timings: serde_json::Map<String, serde_json::Value>,
    base_reservation: Permit,
}

#[cfg(test)]
#[path = "../../tests/ir/resolver_evidence_e6b.rs"]
mod tests;

#[cfg(feature = "resolver_evidence_v2")]
pub(crate) fn execution_snapshot(request: &Request) -> serde_json::Value {
    dto::execution(request, request.snapshot().last_stop)
}
