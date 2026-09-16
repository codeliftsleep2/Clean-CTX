//! Per-result caller verification for the `search_graph` surface.
//!
//! `search_graph` answers with a *set* of symbols, so caller verification
//! belongs to each returned symbol instead of to the response as a whole. Every
//! result carrying `in_degree` either runs the shared C# arity verifier on its
//! own (one [`VerificationRequest`] per eligible result, reusing the exact
//! request/result types the single-target surfaces use) or is annotated with
//! explicit raw-unverified evidence. A broad search therefore can never expose a
//! bare CBM `in_degree` that is indistinguishable from a verified count.
//!
//! Invariants this module owns:
//!
//! * search breadth never decides whether verification runs — one result is a
//!   batch of one;
//! * one unverifiable result never suppresses verification of its neighbours;
//! * evidence is attached to the result it was computed for (the planned
//!   position, confirmed by that entry's stable `qualified_name`), never to a
//!   guessed overload;
//! * the arity algorithm stays in [`crate::cbm::caller_verify`] — this module is
//!   orchestration only.

use crate::cbm::caller_verify::{
    CallerVerificationStatus, CallerVerificationSummary, aggregate_resolution,
    caller_evidence_metadata,
};
use crate::cbm::caller_verify_arity::ParseMemo;
use crate::cbm::caller_verify_proxy::{VerificationAttempt, VerificationRequest};
use serde_json::Value;
use std::collections::HashMap;

/// CBM node labels whose `in_degree` does not describe inbound call sites.
///
/// Caller verification compares call-site argument shape against a method
/// declaration, so only callable labels are eligible. A label this module has
/// never seen stays eligible: the shared verifier then declines with explicit
/// evidence rather than the orchestration silently skipping an unknown kind.
const NON_CALLABLE_LABELS: &[&str] = &[
    "Class",
    "Struct",
    "Interface",
    "Enum",
    "Field",
    "Variable",
    "Module",
    "File",
    "Folder",
    "Project",
    "Route",
];

/// Evidence value for a result whose `verified_in_degree` was established from
/// candidate source — the existing single-target convention.
const EVIDENCE_RAW: &str = "cbm_raw";

/// Evidence value for a result whose raw CBM `in_degree` was *not* verified.
const EVIDENCE_RAW_UNVERIFIED: &str = "cbm_raw_unverified";

/// One `search_graph` result carrying `in_degree`, with the verification work it
/// requires.
#[derive(Debug)]
pub(crate) struct SearchTarget {
    /// Position of the entry in the response `results` array. Used only to
    /// locate an entry that carries no correlation identity.
    index: usize,
    /// Stable correlation identity, when the result carries one.
    qualified_name: Option<String>,
    /// Raw CBM `in_degree` value, preserved verbatim on the result.
    raw_in_degree: Value,
    plan: SearchPlan,
}

#[derive(Debug)]
enum SearchPlan {
    /// Eligible: the shared verifier runs once for this symbol.
    Verify(VerificationRequest),
    /// Not eligible: annotated with explicit raw-unverified evidence.
    Ineligible(IneligibleReason),
}

/// Why a result that carries `in_degree` did not enter caller verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IneligibleReason {
    /// No usable `qualified_name`: the symbol has no verification identity.
    MissingQualifiedName,
    /// Several results share one identity, so the raw count of either entry
    /// cannot be attributed to a specific overload.
    DuplicateQualifiedName,
    /// The label is not a callable symbol kind.
    NotCallableSymbolKind,
    /// The symbol's own file is not C# source, which the shared verifier cannot
    /// read.
    NonCsharpTarget,
}

impl IneligibleReason {
    /// Machine-readable reason reported next to the unverified result.
    fn reason(self) -> &'static str {
        match self {
            Self::MissingQualifiedName => "missing_qualified_name",
            Self::DuplicateQualifiedName => "duplicate_qualified_name",
            Self::NotCallableSymbolKind => "not_callable_symbol_kind",
            Self::NonCsharpTarget => "non_csharp_target",
        }
    }

    /// Verdict for a result that never entered verification: an unidentifiable
    /// entry is unverifiable, while a duplicated identity is ambiguous because
    /// Clean-CTX refuses to guess which overload the raw count belongs to.
    fn status(self) -> CallerVerificationStatus {
        match self {
            Self::DuplicateQualifiedName => CallerVerificationStatus::Ambiguous,
            _ => CallerVerificationStatus::Unverifiable,
        }
    }
}

/// What happened to one target in this response.
enum Disposition {
    /// The shared verifier ran for this symbol.
    Attempted(VerificationAttempt),
    /// Verification was not attempted; the reason carries the evidence.
    Ineligible(IneligibleReason),
}

impl Disposition {
    fn verified(&self) -> usize {
        match self {
            Self::Attempted(attempt) => attempt.summary.verified,
            Self::Ineligible(_) => 0,
        }
    }

    fn status(&self) -> CallerVerificationStatus {
        match self {
            Self::Attempted(attempt) => attempt.summary.resolution,
            Self::Ineligible(reason) => reason.status(),
        }
    }

    /// `cbm_raw` only when candidate source established at least one compatible
    /// caller; every other outcome leaves the raw value explicitly unverified.
    fn evidence(&self) -> &'static str {
        if self.verified() > 0 {
            EVIDENCE_RAW
        } else {
            EVIDENCE_RAW_UNVERIFIED
        }
    }

    /// Reason reported for every result that is not verified. `None` means the
    /// result's `verified_in_degree` is established evidence.
    fn reason(&self) -> Option<&'static str> {
        match self {
            Self::Ineligible(reason) => Some(reason.reason()),
            Self::Attempted(attempt) => match attempt.summary.resolution {
                CallerVerificationStatus::VerifiedCompatible => None,
                CallerVerificationStatus::Ambiguous => Some("ambiguous_arity_evidence"),
                CallerVerificationStatus::RejectedArityMismatch => Some("arity_mismatch"),
                CallerVerificationStatus::Unverifiable => Some(
                    attempt
                        .gap
                        .map_or("arity_evidence_unresolved", |gap| gap.reason()),
                ),
            },
        }
    }
}

/// Build the per-result verification plan for a `search_graph` payload.
///
/// Returns one target per result carrying `in_degree`, in response order.
/// Results without `in_degree` hold nothing to verify and stay untouched.
pub(crate) fn search_targets(payload: &Value, project: Option<&str>) -> Vec<SearchTarget> {
    let Some(results) = payload["results"].as_array() else {
        return Vec::new();
    };
    let identities = identity_counts(results);
    let mut targets = Vec::new();
    for (index, result) in results.iter().enumerate() {
        let Some(raw_in_degree) = result.get("in_degree").cloned() else {
            continue;
        };
        let qualified_name = result["qualified_name"]
            .as_str()
            .filter(|name| !name.is_empty())
            .map(str::to_string);
        let plan = match qualified_name.as_deref() {
            None => SearchPlan::Ineligible(IneligibleReason::MissingQualifiedName),
            Some(name) if identities.get(name).copied().unwrap_or(0) > 1 => {
                SearchPlan::Ineligible(IneligibleReason::DuplicateQualifiedName)
            }
            Some(name) => match ineligibility(result) {
                Some(reason) => SearchPlan::Ineligible(reason),
                None => SearchPlan::Verify(VerificationRequest {
                    target: Some(name.to_string()),
                    raw_candidates: numeric_usize(&raw_in_degree).unwrap_or(0),
                    project: project.map(str::to_string),
                }),
            },
        };
        targets.push(SearchTarget {
            index,
            qualified_name,
            raw_in_degree,
            plan,
        });
    }
    targets
}

/// Verify every eligible result independently, annotate every `in_degree` result
/// with its own truth status, and aggregate the surface-level summary.
///
/// `verify` is the shared verification path (the same resolver the single-target
/// surfaces use), invoked once per eligible result. Verification stays scoped to
/// results with `in_degree` plus an identity plus a callable C# symbol:
/// ineligible results cost no CBM query and no source read.
///
/// The batch owns exactly one [`ParseMemo`] and hands it to every attempt, so a
/// file referenced by several results (overloads declared together, or one
/// caller file reached from more than one result) is parsed once for the whole
/// response. The memo is created here and dropped with the response — it is
/// batch-local reuse, never a cache.
pub(crate) fn verify_search_results(
    payload: &mut Value,
    targets: &[SearchTarget],
    verify: &mut dyn FnMut(&VerificationRequest, &mut ParseMemo) -> VerificationAttempt,
) -> SearchAggregate {
    let results_total = payload["results"].as_array().map_or(0, Vec::len);
    let mut aggregate = SearchAggregate::new();
    let mut memo = ParseMemo::new();
    for target in targets {
        let disposition = match &target.plan {
            SearchPlan::Verify(request) => {
                let mut attempt = verify(request, &mut memo);
                if request.raw_candidates > 0 {
                    attempt.summary.raw_candidates = request.raw_candidates;
                }
                Disposition::Attempted(attempt)
            }
            SearchPlan::Ineligible(reason) => Disposition::Ineligible(*reason),
        };
        if annotate_target(payload, target, &disposition) {
            aggregate.record(&disposition);
            continue;
        }
        // Defensive: the entry this target described is no longer locatable
        // (this module never reorders or removes results). Never annotate a
        // different symbol — report the gap through the aggregate instead.
        aggregate.results_not_eligible += 1;
    }
    aggregate.results_total = results_total;
    aggregate.results_not_eligible += results_total.saturating_sub(targets.len());
    aggregate
}

/// Write the surface-level verification block for a `search_graph` response.
///
/// The block keeps the single-target field set (identity and arity survive only
/// for the single-result case) and adds per-response disposition counters. The
/// authoritative status of any individual symbol stays on that result.
pub(crate) fn annotate_search_evidence(payload: &mut Value, aggregate: &SearchAggregate) {
    let mut metadata = caller_evidence_metadata("search_graph", &aggregate.summary);
    if let Some(object) = metadata.as_object_mut() {
        for (key, count) in [
            ("results_total", aggregate.results_total),
            ("results_verified", aggregate.results_verified),
            ("results_ambiguous", aggregate.results_ambiguous),
            (
                "results_rejected_or_adjusted",
                aggregate.results_rejected_or_adjusted,
            ),
            ("results_unverifiable", aggregate.results_unverifiable),
            ("results_not_eligible", aggregate.results_not_eligible),
        ] {
            object.insert(key.into(), Value::from(count as u64));
        }
    }
    if let Some(object) = payload.as_object_mut() {
        object.insert("clean_ctx_caller_verification".into(), metadata);
    }
}

/// Surface-level verification state for one `search_graph` response.
pub(crate) struct SearchAggregate {
    /// Counters summed over every result the shared verifier actually ran on.
    pub(crate) summary: CallerVerificationSummary,
    /// Number of entries in the response `results` array.
    pub(crate) results_total: usize,
    pub(crate) results_verified: usize,
    pub(crate) results_ambiguous: usize,
    pub(crate) results_rejected_or_adjusted: usize,
    pub(crate) results_unverifiable: usize,
    /// Results verification never attempted: no `in_degree`, no usable identity,
    /// a non-callable kind, or a non-C# target.
    pub(crate) results_not_eligible: usize,
    /// Per-result statuses, folded into `summary.resolution`.
    statuses: Vec<CallerVerificationStatus>,
}

impl SearchAggregate {
    fn new() -> Self {
        Self {
            summary: CallerVerificationSummary::empty(),
            results_total: 0,
            results_verified: 0,
            results_ambiguous: 0,
            results_rejected_or_adjusted: 0,
            results_unverifiable: 0,
            results_not_eligible: 0,
            statuses: Vec::new(),
        }
    }

    fn record(&mut self, disposition: &Disposition) {
        match disposition {
            Disposition::Attempted(attempt) => {
                self.summary.fold(&attempt.summary);
                self.statuses.push(attempt.summary.resolution);
                self.summary.resolution = aggregate_resolution(&self.statuses);
                match attempt.summary.resolution {
                    CallerVerificationStatus::VerifiedCompatible => self.results_verified += 1,
                    CallerVerificationStatus::Ambiguous => self.results_ambiguous += 1,
                    CallerVerificationStatus::RejectedArityMismatch => {
                        self.results_rejected_or_adjusted += 1;
                    }
                    CallerVerificationStatus::Unverifiable => self.results_unverifiable += 1,
                }
            }
            Disposition::Ineligible(_) => self.results_not_eligible += 1,
        }
    }
}

/// Annotate one result with its own verification truth.
///
/// Returns `false` when the entry cannot be located unambiguously, in which case
/// the caller must not record a status for it.
fn annotate_target(payload: &mut Value, target: &SearchTarget, disposition: &Disposition) -> bool {
    let Some(results) = payload.get_mut("results").and_then(Value::as_array_mut) else {
        return false;
    };
    let Some(entry) = locate(results, target) else {
        return false;
    };
    let Some(object) = entry.as_object_mut() else {
        return false;
    };
    object.insert("raw_in_degree".into(), target.raw_in_degree.clone());
    object.insert(
        "verified_in_degree".into(),
        Value::from(disposition.verified() as u64),
    );
    object.insert(
        "in_degree_evidence".into(),
        Value::String(disposition.evidence().to_string()),
    );
    object.insert(
        "in_degree_resolution".into(),
        serde_json::to_value(disposition.status()).unwrap_or(Value::Null),
    );
    if let Some(reason) = disposition.reason() {
        object.insert("in_degree_reason".into(), Value::String(reason.to_string()));
    }
    if let Disposition::Attempted(attempt) = disposition {
        // Identity evidence for the symbol this result describes: which candidate
        // verified, and (when the target stayed ambiguous) which candidates were
        // compatible but unresolved. Rejected candidates are counted, never
        // enumerated — a broad search must not inflate the response with them.
        if !attempt.summary.verified_candidates.is_empty() {
            object.insert(
                "verified_candidates".into(),
                serde_json::to_value(&attempt.summary.verified_candidates).unwrap_or(Value::Null),
            );
        }
        if !attempt.summary.ambiguous_candidates.is_empty() {
            object.insert(
                "ambiguous_candidates".into(),
                serde_json::to_value(&attempt.summary.ambiguous_candidates).unwrap_or(Value::Null),
            );
        }
    }
    true
}

/// Locate the response entry a target belongs to.
///
/// The recorded position is the entry the plan was built from — this module
/// never reorders or removes response entries — and it is confirmed against the
/// target's stable `qualified_name`, so a reshaped array fails closed instead of
/// landing evidence on a different symbol. Only when that positional check fails
/// (defensive: the array was rebuilt underneath us) does a unique identity match
/// take over; a duplicated identity cannot be attributed to one overload and is
/// never guessed.
fn locate<'a>(results: &'a mut [Value], target: &SearchTarget) -> Option<&'a mut Value> {
    let identity = target.qualified_name.as_deref();
    let anchored = results
        .get(target.index)
        .is_some_and(|entry| entry.get("in_degree").is_some() && identity_matches(entry, identity));
    if anchored {
        return results.get_mut(target.index);
    }
    let identity = identity?;
    let matches = results
        .iter()
        .enumerate()
        .filter(|(_, entry)| {
            entry.get("in_degree").is_some() && identity_matches(entry, Some(identity))
        })
        .map(|(index, _)| index)
        .collect::<Vec<usize>>();
    match matches.as_slice() {
        [index] => results.get_mut(*index),
        _ => None,
    }
}

/// Does the entry still carry the identity its target was planned with?
fn identity_matches(entry: &Value, identity: Option<&str>) -> bool {
    entry.get("qualified_name").and_then(Value::as_str) == identity
}

/// Count `qualified_name` occurrences among the results carrying `in_degree`.
fn identity_counts(results: &[Value]) -> HashMap<&str, usize> {
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for result in results
        .iter()
        .filter(|result| result.get("in_degree").is_some())
    {
        if let Some(name) = result.get("qualified_name").and_then(Value::as_str) {
            if !name.is_empty() {
                *counts.entry(name).or_insert(0) += 1;
            }
        }
    }
    counts
}

/// `None` when the shared C# verifier may be run for this result.
fn ineligibility(result: &Value) -> Option<IneligibleReason> {
    if result["label"]
        .as_str()
        .is_some_and(|label| NON_CALLABLE_LABELS.contains(&label))
    {
        return Some(IneligibleReason::NotCallableSymbolKind);
    }
    match result["file_path"].as_str() {
        Some(path) if !path.ends_with(".cs") => Some(IneligibleReason::NonCsharpTarget),
        _ => None,
    }
}

/// Read a CBM degree cell: search results carry `in_degree` as a JSON number,
/// while CBM answers numeric projections as strings elsewhere.
fn numeric_usize(value: &Value) -> Option<usize> {
    value
        .as_u64()
        .and_then(|number| usize::try_from(number).ok())
        .or_else(|| value.as_str()?.parse().ok())
}

#[cfg(test)]
#[path = "../tests/cbm/caller_verify_search.rs"]
mod tests;
