use super::{
    AcquisitionRequirement, AcquisitionSatisfaction, CoverageUnit, SemanticPublicationStatus,
    SourceKind, producer_generation_fingerprint, requirement_fingerprint,
};
use crate::mcp::compatibility::identity::ProducerKey;
use crate::mcp::compatibility::producer::GenerationCatalog;

fn unit(root: &str, source_kind: SourceKind, generation: &str) -> CoverageUnit {
    CoverageUnit::builtin_type_ref(root, source_kind, generation)
}

#[test]
fn acquisition_coverage_unit_identity_is_semantic_and_provider_neutral() {
    let csharp = unit("root-a", SourceKind::CSharp, "producer-1");
    let same = unit("root-a", SourceKind::CSharp, "producer-1");
    assert_eq!(csharp, same);

    assert_ne!(csharp, unit("root-b", SourceKind::CSharp, "producer-1"));
    assert_ne!(csharp, unit("root-a", SourceKind::TypeScript, "producer-1"));
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
        assert!(satisfaction.record_after_publication(
            super::ValidatedAcquisitionProof {
                unit,
                inventory_generation: "inventory-1".into(),
                candidate_owners: Vec::new(),
                provenance: super::ProofProvenance::Provider("provider-a".into()),
            },
            SemanticPublicationStatus::Complete,
        ));
    }

    assert_eq!(
        satisfaction.residual_units(),
        [root_a_typescript].into_iter().collect()
    );
    assert!(!satisfaction.is_source_complete());
}

#[test]
fn requirement_fingerprint_is_order_independent_and_authority_complete() {
    let csharp = unit("root-a", SourceKind::CSharp, "cs-1");
    let typescript = unit("root-a", SourceKind::TypeScript, "ts-1");
    let left = AcquisitionRequirement::incoming_type_ref(
        "ContextA",
        "scope-1",
        [csharp.clone(), typescript.clone()],
    );
    let reordered =
        AcquisitionRequirement::incoming_type_ref("ContextA", "scope-1", [typescript, csharp]);
    assert_eq!(
        requirement_fingerprint(&left),
        requirement_fingerprint(&reordered)
    );

    for distinct in [
        AcquisitionRequirement::incoming_type_ref(
            "ContextB",
            "scope-1",
            left.units.iter().cloned(),
        ),
        AcquisitionRequirement::incoming_type_ref(
            "ContextA",
            "scope-2",
            left.units.iter().cloned(),
        ),
        AcquisitionRequirement::incoming_type_ref(
            "ContextA",
            "scope-1",
            [unit("root-b", SourceKind::CSharp, "cs-1")],
        ),
        AcquisitionRequirement::incoming_type_ref(
            "ContextA",
            "scope-1",
            [unit("root-a", SourceKind::CSharp, "cs-2")],
        ),
    ] {
        assert_ne!(
            requirement_fingerprint(&left),
            requirement_fingerprint(&distinct)
        );
    }
}

#[test]
fn producer_generation_fingerprint_uses_catalog_authorities_and_fails_closed() {
    let current = GenerationCatalog::current();
    let baseline = producer_generation_fingerprint(SourceKind::CSharp, &current)
        .expect("complete C# producer generations");
    let changed = producer_generation_fingerprint(
        SourceKind::CSharp,
        &current
            .clone()
            .with_generation(ProducerKey::GenericSemanticProjection, 99),
    )
    .expect("changed C# producer generations");
    assert_ne!(baseline, changed);

    let missing = current.without(ProducerKey::CSharpSemanticInput);
    assert_eq!(
        producer_generation_fingerprint(SourceKind::CSharp, &missing),
        None
    );
}
