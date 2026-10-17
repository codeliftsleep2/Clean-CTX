use super::{
    AcquisitionRequirement, AcquisitionSatisfaction, CoverageUnit, SourceKind,
};

fn unit(root: &str, source_kind: SourceKind, generation: &str) -> CoverageUnit {
    CoverageUnit::builtin_type_ref(root, source_kind, generation)
}

#[test]
fn acquisition_coverage_unit_identity_is_semantic_and_provider_neutral() {
    let csharp = unit("root-a", SourceKind::CSharp, "producer-1");
    let same = unit("root-a", SourceKind::CSharp, "producer-1");
    assert_eq!(csharp, same);

    assert_ne!(csharp, unit("root-b", SourceKind::CSharp, "producer-1"));
    assert_ne!(
        csharp,
        unit("root-a", SourceKind::TypeScript, "producer-1")
    );
    assert_ne!(csharp, unit("root-a", SourceKind::CSharp, "producer-2"));
}

#[test]
fn acquisition_partial_proof_leaves_only_exact_residual_units() {
    let root_a_csharp = unit("root-a", SourceKind::CSharp, "cs-1");
    let root_a_typescript = unit("root-a", SourceKind::TypeScript, "ts-1");
    let root_b_csharp = unit("root-b", SourceKind::CSharp, "cs-1");
    let requirement = AcquisitionRequirement::incoming_type_ref(
        "ContextA",
        "scope-1",
        [
            root_a_csharp.clone(),
            root_a_typescript.clone(),
            root_b_csharp.clone(),
        ],
    );
    let mut satisfaction = AcquisitionSatisfaction::new(&requirement);

    for unit in [root_a_csharp, root_b_csharp] {
        assert!(satisfaction.record(super::ValidatedAcquisitionProof {
            unit,
            inventory_generation: "inventory-1".into(),
            candidate_owners: Vec::new(),
            provenance: super::ProofProvenance::Provider("provider-a".into()),
        }));
    }

    assert_eq!(
        satisfaction.residual_units(),
        [root_a_typescript].into_iter().collect()
    );
    assert!(!satisfaction.is_source_complete());
}
