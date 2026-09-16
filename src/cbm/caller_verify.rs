//! Clean-CTX verification for CBM-derived inbound caller evidence.
//!
//! CBM supplies candidate relationships. This module checks the structural
//! argument shape in candidate C# source without changing CBM's graph or
//! inserting any relationship into `WorkspaceIndex`, and it *retains* the
//! identity of every candidate it verified so a consumer never has to ask CBM
//! again which candidate passed. Arity extraction and the request-local parse
//! memo live in [`crate::cbm::caller_verify_arity`].

use crate::cbm::caller_verify_arity::{
    Declaration, ParseMemo, arities_overlap, declarations_of, invocation_arities_of,
};
use serde::Serialize;
use serde_json::Value;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CallerVerificationStatus {
    VerifiedCompatible,
    RejectedArityMismatch,
    Ambiguous,
    Unverifiable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CsharpTargetSelector<'a> {
    qualified_name: &'a str,
    declared_arity: Option<usize>,
}

impl<'a> CsharpTargetSelector<'a> {
    pub(crate) fn new(qualified_name: &'a str) -> Self {
        Self {
            qualified_name,
            declared_arity: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn with_declared_arity(mut self, arity: usize) -> Self {
        self.declared_arity = Some(arity);
        self
    }
}

/// One candidate under verification.
///
/// Carries CBM's proposed caller file (the advisory candidate identity), the
/// path Clean-CTX actually read after the trusted-path gate, and that file's
/// text. The CBM identity travels *through* verification instead of being
/// re-derived from source afterwards, so a verified result can name the
/// candidate it verified without a second CBM lookup. Authority is unchanged:
/// CBM proposed the candidate, and the verdict is computed from `text` alone.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CandidateSource<'a> {
    pub(crate) file: &'a str,
    pub(crate) resolved_file: &'a str,
    pub(crate) text: &'a str,
}

/// A candidate whose identity survived verification, with the evidence the
/// verifier established for it.
///
/// Retained beside the summary counters (which are unchanged) so a consumer
/// learns *which* candidate passed, and with what call-site shape, instead of
/// only how many did.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VerifiedCallerCandidate {
    /// Caller file exactly as CBM projected it — an advisory candidate
    /// identity, never promoted to a Clean-CTX semantic fact.
    pub(crate) cbm_file: String,
    /// Path Clean-CTX read for verification (post trusted-path gate).
    pub(crate) file: String,
    /// Distinct explicit argument counts Clean-CTX observed and accepted,
    /// ascending.
    pub(crate) argument_counts: Vec<usize>,
    /// Verdict this candidate received.
    pub(crate) status: CallerVerificationStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct CallerVerificationSummary {
    pub(crate) target_qualified_name: Option<String>,
    pub(crate) target_explicit_arity: Option<usize>,
    pub(crate) raw_candidates: usize,
    pub(crate) verified: usize,
    pub(crate) rejected_arity: usize,
    pub(crate) ambiguous: usize,
    pub(crate) unverifiable: usize,
    pub(crate) verified_caller_files: usize,
    pub(crate) compatible_caller_files: usize,
    pub(crate) resolution: CallerVerificationStatus,
    /// Identity evidence for every candidate that verified. The counters above
    /// remain the compatibility contract; this is the structured evidence
    /// beside them, and it is what removes the consumer's second CBM lookup.
    pub(crate) verified_candidates: Vec<VerifiedCallerCandidate>,
    /// Identity evidence for candidates whose call sites are compatible with a
    /// target that stayed ambiguous — proof that no unique resolution was
    /// claimed for them.
    pub(crate) ambiguous_candidates: Vec<VerifiedCallerCandidate>,
}

impl CallerVerificationSummary {
    pub(crate) fn unverifiable(target: Option<&str>) -> Self {
        Self {
            target_qualified_name: target.map(str::to_string),
            target_explicit_arity: None,
            raw_candidates: 0,
            verified: 0,
            rejected_arity: 0,
            ambiguous: 0,
            unverifiable: 1,
            verified_caller_files: 0,
            compatible_caller_files: 0,
            resolution: CallerVerificationStatus::Unverifiable,
            verified_candidates: Vec::new(),
            ambiguous_candidates: Vec::new(),
        }
    }

    /// Neutral zero summary — the starting point when a surface aggregates
    /// several independently verified targets into one summary.
    pub(crate) fn empty() -> Self {
        Self {
            target_qualified_name: None,
            target_explicit_arity: None,
            raw_candidates: 0,
            verified: 0,
            rejected_arity: 0,
            ambiguous: 0,
            unverifiable: 0,
            verified_caller_files: 0,
            compatible_caller_files: 0,
            resolution: CallerVerificationStatus::Unverifiable,
            verified_candidates: Vec::new(),
            ambiguous_candidates: Vec::new(),
        }
    }

    /// Fold one independently verified target into a surface-level aggregate.
    ///
    /// Counters (including the per-target file counts) are summed, and the
    /// retained candidate evidence accumulates across targets — every verified
    /// candidate in the response is reported once. The first
    /// folded target is adopted — so a batch of one keeps its target identity
    /// exactly like the single-target surfaces report it — and identity and
    /// arity then survive only while every following target agrees; a plural
    /// aggregate never reports one target's identity for a batch.
    /// `resolution` is **not** derived here: aggregating surfaces assign it once
    /// from every folded status via [`aggregate_resolution`].
    pub(crate) fn fold(&mut self, other: &CallerVerificationSummary) {
        self.target_qualified_name = match self.target_qualified_name.take() {
            None => other.target_qualified_name.clone(),
            Some(current) if other.target_qualified_name.as_ref() == Some(&current) => {
                Some(current)
            }
            Some(_) => None,
        };
        self.target_explicit_arity = match self.target_explicit_arity.take() {
            None => other.target_explicit_arity,
            Some(current) if other.target_explicit_arity == Some(current) => Some(current),
            Some(_) => None,
        };
        self.raw_candidates += other.raw_candidates;
        self.verified += other.verified;
        self.rejected_arity += other.rejected_arity;
        self.ambiguous += other.ambiguous;
        self.unverifiable += other.unverifiable;
        self.verified_caller_files += other.verified_caller_files;
        self.compatible_caller_files += other.compatible_caller_files;
        self.verified_candidates
            .extend(other.verified_candidates.iter().cloned());
        self.ambiguous_candidates
            .extend(other.ambiguous_candidates.iter().cloned());
    }
}

/// Weakest-link status across independently verified targets.
///
/// The aggregate is reported as verified only when every folded target is
/// verified; incomplete, ambiguous, and purely arity-rejected targets pull the
/// surface-level verdict down in that order — the same precedence the per-run
/// verifier uses. A batch with nothing verified stays `Unverifiable`. Applied to
/// a batch of one this reproduces that single target's status exactly, so a
/// single-result surface keeps its existing verdict.
pub(crate) fn aggregate_resolution(
    statuses: &[CallerVerificationStatus],
) -> CallerVerificationStatus {
    if statuses.contains(&CallerVerificationStatus::Unverifiable) || statuses.is_empty() {
        CallerVerificationStatus::Unverifiable
    } else if statuses.contains(&CallerVerificationStatus::Ambiguous) {
        CallerVerificationStatus::Ambiguous
    } else if statuses.contains(&CallerVerificationStatus::VerifiedCompatible) {
        CallerVerificationStatus::VerifiedCompatible
    } else if statuses.contains(&CallerVerificationStatus::RejectedArityMismatch) {
        CallerVerificationStatus::RejectedArityMismatch
    } else {
        CallerVerificationStatus::Unverifiable
    }
}

/// Verify CBM's proposed callers against the target's declaration and their own
/// call sites.
///
/// `target_key` and every candidate's `resolved_file` are the canonical paths
/// the sources were read from: the [`ParseMemo`] keys, so one batch parses a
/// file once however many symbols reference it. `memo` is request-local (no
/// cross-request verification state exists) and carries nothing between
/// requests.
///
/// Counters keep their established meaning; the retained
/// [`VerifiedCallerCandidate`] evidence is added *beside* them, exactly as the
/// verifier established it — a candidate CBM proposed that source verification
/// contradicted is never reported as verified.
pub(crate) fn verify_csharp_callers(
    target_source: &str,
    target_key: &str,
    selector: CsharpTargetSelector<'_>,
    candidates: &[CandidateSource<'_>],
    memo: &mut ParseMemo,
) -> CallerVerificationSummary {
    let method_name = selector
        .qualified_name
        .rsplit('.')
        .next()
        .unwrap_or(selector.qualified_name);
    let type_name = selector.qualified_name.rsplit('.').nth(1);
    let Some(target_tree) = memo.tree(target_key, target_source) else {
        return CallerVerificationSummary::unverifiable(Some(selector.qualified_name));
    };
    let declarations = declarations_of(&target_tree, target_source, method_name, type_name);
    let selected: Vec<Declaration> = declarations
        .iter()
        .copied()
        .filter(|decl| {
            selector
                .declared_arity
                .is_none_or(|arity| decl.arity.declared == arity)
        })
        .collect();
    if selected.is_empty() {
        return CallerVerificationSummary::unverifiable(Some(selector.qualified_name));
    }
    if selector.declared_arity.is_none() && selected.len() > 1 {
        let mut ambiguous = 0;
        let mut files = HashSet::new();
        let mut unverifiable = 0;
        let mut ambiguous_candidates = Vec::new();
        for candidate in candidates {
            match memo.tree(candidate.resolved_file, candidate.text) {
                Some(tree) => {
                    let invocations = invocation_arities_of(&tree, candidate.text, method_name);
                    if !invocations.is_empty() {
                        files.insert(candidate.file);
                        ambiguous_candidates.push(candidate_evidence(
                            candidate,
                            &invocations,
                            CallerVerificationStatus::Ambiguous,
                        ));
                    }
                    ambiguous += invocations.len();
                }
                None => unverifiable += 1,
            }
        }
        return CallerVerificationSummary {
            target_qualified_name: Some(selector.qualified_name.to_string()),
            target_explicit_arity: None,
            raw_candidates: ambiguous,
            verified: 0,
            rejected_arity: 0,
            ambiguous,
            unverifiable,
            verified_caller_files: 0,
            compatible_caller_files: files.len(),
            resolution: if unverifiable > 0 {
                CallerVerificationStatus::Unverifiable
            } else {
                CallerVerificationStatus::Ambiguous
            },
            verified_candidates: Vec::new(),
            ambiguous_candidates,
        };
    }

    let target = selected[0].arity;
    let same_shape_ambiguity = selected.len() > 1
        || declarations
            .iter()
            .filter(|decl| arities_overlap(target, decl.arity))
            .count()
            > 1;
    let mut summary = CallerVerificationSummary {
        target_qualified_name: Some(selector.qualified_name.to_string()),
        target_explicit_arity: target.exact_explicit(),
        raw_candidates: 0,
        verified: 0,
        rejected_arity: 0,
        ambiguous: 0,
        unverifiable: 0,
        verified_caller_files: 0,
        compatible_caller_files: 0,
        resolution: CallerVerificationStatus::Unverifiable,
        verified_candidates: Vec::new(),
        ambiguous_candidates: Vec::new(),
    };
    let mut verified_files = HashSet::new();
    let mut compatible_files = HashSet::new();

    for candidate in candidates {
        let Some(tree) = memo.tree(candidate.resolved_file, candidate.text) else {
            summary.unverifiable += 1;
            continue;
        };
        let mut accepted = Vec::new();
        let mut compatible_but_ambiguous = Vec::new();
        for observed in invocation_arities_of(&tree, candidate.text, method_name) {
            summary.raw_candidates += 1;
            if !target.accepts(observed) {
                summary.rejected_arity += 1;
            } else if target.flexible || same_shape_ambiguity {
                summary.ambiguous += 1;
                compatible_files.insert(candidate.file);
                compatible_but_ambiguous.push(observed);
            } else {
                summary.verified += 1;
                compatible_files.insert(candidate.file);
                verified_files.insert(candidate.file);
                accepted.push(observed);
            }
        }
        if !accepted.is_empty() {
            summary.verified_candidates.push(candidate_evidence(
                candidate,
                &accepted,
                CallerVerificationStatus::VerifiedCompatible,
            ));
        } else if !compatible_but_ambiguous.is_empty() {
            summary.ambiguous_candidates.push(candidate_evidence(
                candidate,
                &compatible_but_ambiguous,
                CallerVerificationStatus::Ambiguous,
            ));
        }
    }
    summary.verified_caller_files = verified_files.len();
    summary.compatible_caller_files = compatible_files.len();
    summary.resolution = if summary.unverifiable > 0 {
        CallerVerificationStatus::Unverifiable
    } else if summary.ambiguous > 0 {
        CallerVerificationStatus::Ambiguous
    } else if summary.verified > 0 {
        CallerVerificationStatus::VerifiedCompatible
    } else if summary.rejected_arity > 0 {
        CallerVerificationStatus::RejectedArityMismatch
    } else {
        CallerVerificationStatus::Unverifiable
    };
    summary
}

/// Retain one candidate's identity beside the verdict it earned.
///
/// The observed counts are deduplicated and sorted, so the evidence is the
/// deterministic *set* of accepted call-site shapes rather than a repetition of
/// every call site. No count is invented and none is lost: what a call site
/// actually passed is exactly what is reported.
fn candidate_evidence(
    candidate: &CandidateSource<'_>,
    observed: &[usize],
    status: CallerVerificationStatus,
) -> VerifiedCallerCandidate {
    let mut argument_counts = observed.to_vec();
    argument_counts.sort_unstable();
    argument_counts.dedup();
    VerifiedCallerCandidate {
        cbm_file: candidate.file.to_string(),
        file: candidate.resolved_file.to_string(),
        argument_counts,
        status,
    }
}

/// Build the surface-level verification metadata object for one summary.
///
/// This is the single definition of the `clean_ctx_caller_verification` shape:
/// every serialized summary field plus the surface/evidence provenance keys.
/// Surfaces that verify several targets extend the returned object with their
/// own aggregate counters instead of re-deriving these keys.
pub(crate) fn caller_evidence_metadata(
    surface: &str,
    summary: &CallerVerificationSummary,
) -> Value {
    let mut metadata = serde_json::to_value(summary).unwrap_or(Value::Null);
    if let Some(object) = metadata.as_object_mut() {
        object.insert("surface".into(), Value::String(surface.to_string()));
        object.insert("evidence".into(), Value::String("arity".into()));
        object.insert(
            "cbm_relationships".into(),
            Value::String("raw_candidates_only".into()),
        );
    }
    metadata
}

pub(crate) fn annotate_caller_evidence(
    payload: &mut Value,
    surface: &str,
    summary: &CallerVerificationSummary,
) {
    let metadata = caller_evidence_metadata(surface, summary);
    if let Some(object) = payload.as_object_mut() {
        object.insert("clean_ctx_caller_verification".into(), metadata);
    }
}

#[cfg(test)]
#[path = "../tests/cbm/caller_verify.rs"]
mod tests;
