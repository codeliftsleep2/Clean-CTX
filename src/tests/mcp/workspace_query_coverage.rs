use super::{CapabilityDirection, acquisition_coverage_detail, exact_identity_coverage};
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};
use crate::mcp::acquisition::{
    AcquisitionOutcome, AcquisitionRequirement, AcquisitionSatisfaction, CoverageProofDisposition,
    CoverageUnit, ProofBarrier, SemanticPublicationStatus, SourceKind,
};
use crate::mcp::tool_handlers::hydration::HydrationReport;
use crate::mcp::tool_handlers::query::identity::ResolvedIdentity;
use crate::workspace::index::WorkspaceIndex;
use serde_json::json;
use std::collections::BTreeMap;

fn indexed_identity() -> (WorkspaceIndex, ResolvedIdentity) {
    let identity = ResolvedIdentity {
        domain: "builtin".into(),
        entity_type: "TypeRef".into(),
        name: "IFooService".into(),
    };
    let mut index = WorkspaceIndex::new();
    index.add_edges(
        "consumer.cs",
        vec![SemanticEdge {
            relation: SemanticRelation::HasConstructorParameterType,
            subject: EntityRef::new("builtin", "Class", "Consumer")
                .with_file("consumer.cs".to_string()),
            object: EntityRef::new("builtin", "TypeRef", "IFooService")
                .with_file("consumer.cs".to_string()),
            layer: "builtin",
            call_evidence: None,
        }],
    );
    (index, identity)
}

#[test]
fn workspace_query_exact_identity_coverage_partial_hydration() {
    let (index, identity) = indexed_identity();
    let hydration = HydrationReport {
        hydration_attempted: true,
        discovery_status: "partial",
        candidates_discovered: 12,
        candidates_compiled: 4,
        ..HydrationReport::default()
    };
    let coverage = exact_identity_coverage(
        &index,
        &identity,
        None,
        CapabilityDirection::Reverse,
        &hydration,
        false,
    );
    assert_eq!(coverage["status"], "indexed_evidence_only");
    assert_eq!(coverage["identity_indexed"], true);
    assert_eq!(coverage["capability_established"], true);
    assert_eq!(coverage["source_complete"], false);
    assert_eq!(coverage["result_semantics"], "lower_bound");
    assert_eq!(coverage["omitted_possible"], true);
    assert_eq!(coverage["discovered_not_compiled_this_cycle"], 8);
    assert_eq!(
        coverage["coverage_detail"],
        json!({
            "processing_state": "not_attempted",
            "proof_state": "indeterminate",
            "fact_authority": "authoritative_observed",
            "retryability": "indeterminate",
            "reasons": ["processing_not_attempted"]
        }),
        "an unscoped TypeRef query has no workspace acquisition requirement and must say so"
    );
    assert!(
        coverage["coverage_detail"]
            .get("residual_unit_count")
            .is_none(),
        "no acquisition requirement means there is no residual-unit count"
    );
}

#[test]
fn interface_reverse_coverage_points_to_csharp_constructor_type_query() {
    let mut index = WorkspaceIndex::new();
    let identity = ResolvedIdentity {
        domain: "builtin".into(),
        entity_type: "Interface".into(),
        name: "IFooService".into(),
    };
    let interface = EntityRef::new("builtin", "Interface", "IFooService")
        .with_file("IFooService.cs".to_string());
    index.add_edges(
        "IFooService.cs",
        vec![SemanticEdge {
            relation: SemanticRelation::Defines,
            subject: interface.clone(),
            object: interface,
            layer: "builtin",
            call_evidence: None,
        }],
    );

    let hydration = HydrationReport {
        hydration_attempted: true,
        discovery_status: "partial",
        candidates_discovered: 14,
        ..HydrationReport::default()
    };
    let coverage = exact_identity_coverage(
        &index,
        &identity,
        None,
        CapabilityDirection::Reverse,
        &hydration,
        false,
    );
    assert_eq!(coverage["status"], "capability_not_established");
    assert_eq!(coverage["discovered_not_compiled_this_cycle"], 14);
    assert_eq!(coverage["alternative_query"]["type"], "reverse_edges");
    assert_eq!(coverage["alternative_query"]["domain"], "builtin");
    assert_eq!(coverage["alternative_query"]["entity_type"], "TypeRef");
    assert_eq!(coverage["alternative_query"]["name"], "IFooService");
    assert_eq!(coverage["alternative_query"]["relation"], "HasBaseType");
    assert_eq!(
        coverage["coverage_detail"],
        json!({
            "processing_state": "not_attempted",
            "proof_state": "unsupported",
            "fact_authority": "authoritative_observed",
            "retryability": "indeterminate",
            "reasons": ["capability_not_supported"]
        }),
        "an unsupported exact query family must explain why no completeness acquisition ran"
    );
    assert!(
        coverage["coverage_detail"]
            .get("residual_unit_count")
            .is_none(),
        "unsupported families have no acquisition requirement from which to derive a count"
    );
}

#[test]
fn incompatible_proof_claim_projects_stable_structural_reason() {
    let unit = CoverageUnit::builtin_type_ref("C:/repo", SourceKind::CSharp, "generation");
    let requirement =
        AcquisitionRequirement::incoming_type_ref("IFooService", "scope", [unit.clone()]);
    let outcome = AcquisitionOutcome::new(
        AcquisitionSatisfaction::new(&requirement),
        BTreeMap::from([(
            unit,
            CoverageProofDisposition::Unproven(ProofBarrier::ClaimIncompatible),
        )]),
        SemanticPublicationStatus::Complete,
    );

    assert_eq!(
        acquisition_coverage_detail(&outcome),
        json!({
            "processing_state": "terminal",
            "proof_state": "structurally_unproven",
            "fact_authority": "authoritative_observed",
            "retryability": "change_required",
            "residual_unit_count": 1,
            "reasons": ["proof_incompatible"]
        }),
        "internal proof-rejection names must not leak through the bounded public vocabulary"
    );
}

#[test]
fn incomplete_inventory_projects_operational_processing_state() {
    let unit = CoverageUnit::builtin_type_ref("C:/repo", SourceKind::CSharp, "generation");
    let requirement =
        AcquisitionRequirement::incoming_type_ref("IFooService", "scope", [unit.clone()]);
    let outcome = AcquisitionOutcome::new(
        AcquisitionSatisfaction::new(&requirement),
        BTreeMap::from([(
            unit,
            CoverageProofDisposition::Unproven(ProofBarrier::InventoryIncomplete),
        )]),
        SemanticPublicationStatus::Complete,
    );

    assert_eq!(
        acquisition_coverage_detail(&outcome),
        json!({
            "processing_state": "incomplete",
            "proof_state": "operationally_unproven",
            "fact_authority": "authoritative_observed",
            "retryability": "same_request_may_help",
            "residual_unit_count": 1,
            "reasons": ["inventory_incomplete"]
        }),
        "an incomplete inventory is unfinished acquisition work even when candidate publication did not fail"
    );
}

#[test]
fn mixed_structural_and_operational_barriers_project_deterministically() {
    let inventory_unit =
        CoverageUnit::builtin_type_ref("C:/repo", SourceKind::CSharp, "generation");
    let predicate_unit =
        CoverageUnit::builtin_type_ref("C:/repo", SourceKind::TypeScript, "generation");
    let requirement = AcquisitionRequirement::incoming_type_ref(
        "IFooService",
        "scope",
        [inventory_unit.clone(), predicate_unit.clone()],
    );
    let outcome = AcquisitionOutcome::new(
        AcquisitionSatisfaction::new(&requirement),
        BTreeMap::from([
            (
                inventory_unit,
                CoverageProofDisposition::Unproven(ProofBarrier::InventoryIncomplete),
            ),
            (
                predicate_unit,
                CoverageProofDisposition::Unproven(ProofBarrier::CandidatePredicateUncertified),
            ),
        ]),
        SemanticPublicationStatus::Complete,
    );

    assert_eq!(
        acquisition_coverage_detail(&outcome),
        json!({
            "processing_state": "incomplete",
            "proof_state": "mixed_unproven",
            "fact_authority": "authoritative_observed",
            "retryability": "same_request_may_help",
            "residual_unit_count": 2,
            "reasons": ["inventory_incomplete", "candidate_predicate_uncertified"]
        }),
        "mixed barriers must retain both categories in stable public-vocabulary order"
    );
}
