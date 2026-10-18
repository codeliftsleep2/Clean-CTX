//! Truthfulness metadata for exact-identity graph answers.

use super::identity::ResolvedIdentity;
use crate::mcp::acquisition::{
    AcquisitionOutcome, AcquisitionProcessingState, AcquisitionSatisfaction,
    CoverageProofDisposition, ProofBarrier,
};
use crate::mcp::tool_handlers::hydration::HydrationReport;
use crate::workspace::index::WorkspaceIndex;
use crate::workspace::scope::WorkspaceScope;
use serde_json::{Value, json};

#[derive(Clone, Copy)]
pub(super) enum CapabilityDirection {
    Forward,
    Reverse,
}

/// Proof that one authoritative operation satisfied its declared evidence
/// requirement against a compatible current-source generation. Construction is
/// intentionally narrow: callers cannot toggle public completeness fields.
#[derive(Clone)]
pub(super) struct CompletenessCertificate {
    scope_kind: &'static str,
    scope_value: String,
    source_generation: String,
    evidence_requirements: &'static [&'static str],
}

impl CompletenessCertificate {
    pub(super) fn current_file(path: String, source_generation: String) -> Self {
        Self {
            scope_kind: "file",
            scope_value: path,
            source_generation,
            evidence_requirements: &[
                "authorized_source_acquired",
                "producer_supported",
                "semantic_projection_current",
                "dependencies_current",
            ],
        }
    }

    fn incoming_relationships(satisfaction: &AcquisitionSatisfaction) -> Option<Self> {
        Some(Self {
            scope_kind: "workspace",
            scope_value: satisfaction.scope_fingerprint().to_string(),
            source_generation: satisfaction.completeness_generation()?,
            evidence_requirements: &[
                "incoming_relationship_units_proven",
                "semantic_projection_current",
                "dependencies_current",
                "publication_complete",
            ],
        })
    }

    pub(super) fn project(&self) -> Value {
        json!({
            "status": "authoritative_complete",
            "scope": {
                "kind": self.scope_kind,
                "value": self.scope_value,
            },
            "source_generation": self.source_generation,
            "evidence_requirements": self.evidence_requirements,
            "source_complete": true,
            "result_semantics": "exact",
            "omitted_possible": false,
            "reasons": [],
        })
    }
}

pub(super) fn exact_identity_coverage(
    index: &WorkspaceIndex,
    identity: &ResolvedIdentity,
    scope: Option<&WorkspaceScope>,
    direction: CapabilityDirection,
    hydration: &HydrationReport,
    response_has_typed_evidence: bool,
) -> Value {
    let identity_indexed = index.has_identity_in_scope(
        &identity.domain,
        &identity.entity_type,
        &identity.name,
        scope,
    );
    let stored_capability_established = match direction {
        CapabilityDirection::Forward => {
            index.has_forward_capability_evidence(&identity.domain, &identity.entity_type, scope)
        }
        CapabilityDirection::Reverse => {
            index.has_reverse_capability_evidence(&identity.domain, &identity.entity_type, scope)
        }
    };
    let capability_established = stored_capability_established || response_has_typed_evidence;
    let incoming_type_ref = matches!(direction, CapabilityDirection::Reverse)
        && identity.domain == "builtin"
        && identity.entity_type == "TypeRef";
    let exact_incoming_type_ref = incoming_type_ref && scope.is_some();
    if exact_incoming_type_ref
        && let Some(certificate) = hydration
            .acquisition_satisfaction
            .as_ref()
            .and_then(CompletenessCertificate::incoming_relationships)
    {
        let mut coverage = certificate.project();
        coverage["identity_indexed"] = identity_indexed.into();
        coverage["capability_established"] = capability_established.into();
        if let Some(outcome) = hydration.acquisition_outcome.as_ref() {
            coverage["coverage_detail"] = acquisition_coverage_detail(outcome);
        }
        return coverage;
    }
    let status = if !identity_indexed {
        "identity_not_indexed"
    } else if !capability_established {
        "capability_not_established"
    } else if hydration.hydration_attempted && hydration.discovery_status != "completed" {
        "indexed_evidence_only"
    } else {
        "established_indexed_capability"
    };
    let mut coverage = json!({
        "status": status,
        "identity_indexed": identity_indexed,
        "capability_established": capability_established,
        "source_complete": false,
        "result_semantics": "lower_bound",
        "omitted_possible": true,
    });
    let discovered_not_compiled_this_cycle = hydration
        .candidates_discovered
        .saturating_sub(hydration.candidates_compiled);
    if discovered_not_compiled_this_cycle > 0 {
        coverage["discovered_not_compiled_this_cycle"] = discovered_not_compiled_this_cycle.into();
    }
    if exact_incoming_type_ref
        && let Some(outcome) = hydration.acquisition_outcome.as_ref()
        && !outcome.satisfaction().is_source_complete()
    {
        coverage["coverage_detail"] = acquisition_coverage_detail(outcome);
    }
    if incoming_type_ref && hydration.acquisition_outcome.is_none() {
        coverage["coverage_detail"] = json!({
            "processing_state": "not_attempted",
            "proof_state": "indeterminate",
            "fact_authority": "authoritative_observed",
            "retryability": "indeterminate",
            "reasons": ["processing_not_attempted"],
        });
    }
    if matches!(direction, CapabilityDirection::Reverse)
        && identity.domain == "builtin"
        && matches!(identity.entity_type.as_str(), "Class" | "Interface")
        && !response_has_typed_evidence
    {
        coverage["coverage_detail"] = json!({
            "processing_state": "not_attempted",
            "proof_state": "unsupported",
            "fact_authority": "authoritative_observed",
            "retryability": "indeterminate",
            "reasons": ["capability_not_supported"],
        });
        coverage["alternative_query"] = json!({
            "type": "reverse_edges",
            "domain": "builtin",
            "entity_type": "TypeRef",
            "name": identity.name,
            "relation": "HasBaseType",
            "meaning": "C# classes naming this written type in an unresolved base list"
        });
    }
    coverage
}

fn acquisition_coverage_detail(outcome: &AcquisitionOutcome) -> Value {
    let residual = outcome.satisfaction().residual_units();
    let mut observed_reasons = Vec::new();
    for unit in &residual {
        let reason = match outcome.dispositions().get(unit) {
            Some(CoverageProofDisposition::Unproven(
                ProofBarrier::CandidatePredicateUncertified,
            )) => "candidate_predicate_uncertified",
            Some(CoverageProofDisposition::Unproven(ProofBarrier::InventoryIncomplete)) => {
                "inventory_incomplete"
            }
            Some(CoverageProofDisposition::Unproven(ProofBarrier::PublicationFailed)) => {
                "publication_failed"
            }
            Some(CoverageProofDisposition::Unproven(ProofBarrier::CandidateProcessingFailed)) => {
                "candidate_processing_failed"
            }
            Some(CoverageProofDisposition::Unproven(ProofBarrier::ClaimIncompatible)) => {
                "proof_incompatible"
            }
            Some(CoverageProofDisposition::Proven) | None => "indeterminate",
        };
        if !observed_reasons.contains(&reason) {
            observed_reasons.push(reason);
        }
    }
    let reasons: Vec<_> = [
        "inventory_incomplete",
        "candidate_predicate_uncertified",
        "proof_incompatible",
        "candidate_processing_failed",
        "publication_failed",
        "indeterminate",
    ]
    .into_iter()
    .filter(|reason| observed_reasons.contains(reason))
    .collect();
    let complete = outcome.satisfaction().is_source_complete();
    let has_structural = reasons.iter().any(|reason| {
        matches!(
            *reason,
            "candidate_predicate_uncertified" | "proof_incompatible"
        )
    });
    let has_operational = reasons.iter().any(|reason| {
        matches!(
            *reason,
            "inventory_incomplete" | "candidate_processing_failed" | "publication_failed"
        )
    });
    let has_indeterminate = reasons.contains(&"indeterminate");
    let structurally_unproven = outcome.processing() == AcquisitionProcessingState::Terminal
        && has_structural
        && !has_operational
        && !has_indeterminate;
    let operationally_unproven = outcome.processing() == AcquisitionProcessingState::Incomplete
        && has_operational
        && !has_structural
        && !has_indeterminate;
    let mixed_unproven = outcome.processing() == AcquisitionProcessingState::Incomplete
        && has_operational
        && has_structural
        && !has_indeterminate;
    json!({
        "processing_state": if outcome.processing() == AcquisitionProcessingState::Terminal {
            "terminal"
        } else {
            "incomplete"
        },
        "proof_state": if complete {
            "complete"
        } else if structurally_unproven {
            "structurally_unproven"
        } else if operationally_unproven {
            "operationally_unproven"
        } else if mixed_unproven {
            "mixed_unproven"
        } else {
            "indeterminate"
        },
        "fact_authority": "authoritative_observed",
        "retryability": if complete {
            "not_needed"
        } else if structurally_unproven {
            "change_required"
        } else if operationally_unproven || mixed_unproven {
            "same_request_may_help"
        } else {
            "indeterminate"
        },
        "residual_unit_count": residual.len(),
        "reasons": reasons,
    })
}

#[cfg(test)]
#[path = "../../../tests/mcp/workspace_query_coverage.rs"]
mod tests;
