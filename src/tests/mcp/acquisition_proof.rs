use super::{
    AcquisitionProofClaim, AcquisitionRequirement, AcquisitionSatisfaction, CoverageUnit,
    ProofContext, ProofProvenance, ProofRejection, SemanticPublicationStatus, SourceKind,
    validate_proof_claim,
};

fn unit() -> CoverageUnit {
    CoverageUnit::builtin_type_ref("root-a", SourceKind::CSharp, "producer-1")
}

fn requirement() -> AcquisitionRequirement {
    AcquisitionRequirement::incoming_type_ref("ContextA", "scope-1", [unit()])
}

fn context() -> ProofContext {
    ProofContext {
        inventory_generation: "inventory-1".into(),
        admission_fingerprint: "admission-1".into(),
        predicate_version: 1,
        proof_contract_version: 1,
    }
}

fn claim() -> AcquisitionProofClaim {
    AcquisitionProofClaim {
        unit: unit(),
        target: super::AcquisitionTarget::exact_type_ref("ContextA"),
        scope_fingerprint: "scope-1".into(),
        inventory_generation: "inventory-1".into(),
        admission_fingerprint: "admission-1".into(),
        predicate_version: 1,
        proof_contract_version: 1,
        exhaustive: true,
        truncated: false,
        limitations: Vec::new(),
        candidate_owners: vec!["b.cs".into(), "a.cs".into(), "a.cs".into()],
        provenance: ProofProvenance::Provider("provider-a".into()),
    }
}

#[test]
fn acquisition_proof_validation_accepts_only_an_exact_current_claim() {
    let proof = validate_proof_claim(&requirement(), &context(), claim()).expect("valid proof");
    assert_eq!(proof.candidate_owners, vec!["a.cs", "b.cs"]);

    let mut satisfaction = AcquisitionSatisfaction::new(&requirement());
    assert!(satisfaction.record_after_publication(proof, SemanticPublicationStatus::Complete));
    assert!(satisfaction.is_source_complete());
}

#[test]
fn validated_proof_cannot_satisfy_a_unit_after_publication_failure() {
    let proof = validate_proof_claim(&requirement(), &context(), claim()).expect("valid proof");
    let mut satisfaction = AcquisitionSatisfaction::new(&requirement());

    assert!(!satisfaction.record_after_publication(proof, SemanticPublicationStatus::Failed));
    assert!(!satisfaction.is_source_complete());
    assert_eq!(
        satisfaction.residual_units(),
        [unit()].into_iter().collect()
    );
}

#[test]
fn acquisition_proof_validation_fails_closed_for_every_authority_dimension() {
    let cases = [
        (
            {
                let mut value = claim();
                value.unit =
                    CoverageUnit::builtin_type_ref("root-b", SourceKind::CSharp, "producer-1");
                value
            },
            ProofRejection::UnitNotRequired,
        ),
        (
            {
                let mut value = claim();
                value.target = super::AcquisitionTarget::exact_type_ref("Other");
                value
            },
            ProofRejection::TargetMismatch,
        ),
        (
            {
                let mut value = claim();
                value.scope_fingerprint = "scope-2".into();
                value
            },
            ProofRejection::ScopeMismatch,
        ),
        (
            {
                let mut value = claim();
                value.inventory_generation = "inventory-2".into();
                value
            },
            ProofRejection::InventoryGenerationMismatch,
        ),
        (
            {
                let mut value = claim();
                value.admission_fingerprint = "admission-2".into();
                value
            },
            ProofRejection::AdmissionPolicyMismatch,
        ),
        (
            {
                let mut value = claim();
                value.predicate_version = 2;
                value
            },
            ProofRejection::PredicateVersionMismatch,
        ),
        (
            {
                let mut value = claim();
                value.proof_contract_version = 2;
                value
            },
            ProofRejection::ContractVersionMismatch,
        ),
        (
            {
                let mut value = claim();
                value.exhaustive = false;
                value
            },
            ProofRejection::NotExhaustive,
        ),
        (
            {
                let mut value = claim();
                value.truncated = true;
                value
            },
            ProofRejection::Truncated,
        ),
        (
            {
                let mut value = claim();
                value.limitations.push("missing producer".into());
                value
            },
            ProofRejection::Limited,
        ),
    ];

    for (claim, expected) in cases {
        assert_eq!(
            validate_proof_claim(&requirement(), &context(), claim),
            Err(expected)
        );
    }
}
