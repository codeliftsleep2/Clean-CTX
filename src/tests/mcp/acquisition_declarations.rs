use super::{
    HydrationReport, HydrationRequirement, TEST_PROJECT_HYDRATION_SERIALIZE,
    fail_test_publication_once, hydrate_workspace_index_for_acquisition,
    invalidate_discovery_for_root,
};
use crate::mcp::acquisition::{
    AcquisitionCacheKey, AcquisitionCompletionCache, AcquisitionRequirement,
    AcquisitionSatisfaction, AcquisitionTarget, CoverageUnit, EvidenceRequirement, SourceKind,
    requirement_fingerprint,
};

fn acquire(state: &crate::mcp::McpState, root: &std::path::Path) -> HydrationReport {
    hydrate_workspace_index_for_acquisition(
        state,
        "Wanted",
        Some(&root.to_string_lossy()),
        HydrationRequirement::Semantic(crate::workspace::index::SemanticFidelity::High),
        AcquisitionTarget::declarations("Wanted"),
    )
    .expect("declaration acquisition")
}

#[test]
fn declaration_full_unit_publication_failure_is_retryable_and_cache_invalidates() {
    let _serial = TEST_PROJECT_HYDRATION_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = tempfile::tempdir().unwrap();
    let other = root.path().join("other.rs");
    std::fs::write(
        &other,
        "struct Other; impl Other { fn unrelated(&self) {} }",
    )
    .unwrap();
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    let state = crate::mcp::McpState::new(config);
    fail_test_publication_once(&other.to_string_lossy());
    let failed = acquire(&state, root.path());
    assert!(
        failed
            .acquisition_satisfaction
            .as_ref()
            .is_some_and(|proof| !proof.is_source_complete())
    );
    let complete = acquire(&state, root.path());
    assert!(
        complete
            .acquisition_satisfaction
            .as_ref()
            .is_some_and(AcquisitionSatisfaction::is_source_complete)
    );
    assert!(
        state
            .workspace_index_read()
            .declarations_by_name("unrelated")
            .len()
            == 1
    );
    let warm = acquire(&state, root.path());
    assert_eq!(
        complete
            .acquisition_satisfaction
            .unwrap()
            .completeness_generation(),
        warm.acquisition_satisfaction
            .unwrap()
            .completeness_generation(),
    );
    invalidate_discovery_for_root(&state, &root.path().to_string_lossy());
    state
        .workspace_index_lock()
        .remove_file(&state.semantic_owner_path(&other.to_string_lossy()));
    fail_test_publication_once(&other.to_string_lossy());
    assert!(
        acquire(&state, root.path())
            .acquisition_satisfaction
            .as_ref()
            .is_some_and(|proof| !proof.is_source_complete())
    );
}

#[test]
fn incomplete_authorized_root_cannot_earn_declaration_completeness() {
    let _serial = TEST_PROJECT_HYDRATION_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = tempfile::tempdir().unwrap();
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config.additional_roots = vec![root.path().join("missing").to_string_lossy().into_owned()];
    let state = crate::mcp::McpState::new(config);
    let report = acquire(&state, root.path());
    assert!(
        !report
            .acquisition_satisfaction
            .as_ref()
            .is_some_and(AcquisitionSatisfaction::is_source_complete)
    );
}

#[test]
fn declaration_and_type_ref_obligations_are_distinct_in_the_shared_proof_model() {
    let declaration = CoverageUnit::declarations("root", SourceKind::Rust, "generation");
    let incoming = CoverageUnit::builtin_type_ref("root", SourceKind::Rust, "generation");
    assert_ne!(declaration, incoming);
    let declaration_requirement = AcquisitionRequirement::for_target(
        AcquisitionTarget::declarations("Wanted"),
        "scope",
        [declaration],
    );
    let incoming_requirement =
        AcquisitionRequirement::incoming_type_ref("Wanted", "scope", [incoming]);
    assert_ne!(
        requirement_fingerprint(&declaration_requirement),
        requirement_fingerprint(&incoming_requirement)
    );
    let mut cache = AcquisitionCompletionCache::new();
    let proof = AcquisitionSatisfaction::new(&declaration_requirement);
    assert!(!cache.commit(
        AcquisitionCacheKey {
            target: declaration_requirement.target.clone(),
            roots: vec!["root".into()],
            admission_fingerprint: "scope".into(),
            producer_generations: Vec::new(),
            capability_version: 1,
            predicate_version: 1,
            proof_contract_version: 2,
            evidence: EvidenceRequirement::CurrentHighSemanticProjection,
        },
        proof
    ));
}
