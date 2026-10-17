use super::{
    AcquisitionCacheKey, AcquisitionCompletionCache, AcquisitionRequirement,
    AcquisitionSatisfaction, AcquisitionTarget, CoverageUnit, EvidenceRequirement,
    ProofProvenance, SemanticPublicationStatus, SourceKind, ValidatedAcquisitionProof,
};

fn key(root: &str) -> AcquisitionCacheKey {
    AcquisitionCacheKey {
        target: AcquisitionTarget::exact_type_ref("ContextA"),
        roots: vec![root.into()],
        admission_fingerprint: "admission-1".into(),
        producer_generations: vec![(SourceKind::CSharp, "producer-1".into())],
        predicate_version: 1,
        proof_contract_version: 1,
        evidence: EvidenceRequirement::CurrentHighSemanticProjection,
    }
}

fn satisfaction(root: &str, complete: bool) -> AcquisitionSatisfaction {
    let unit = CoverageUnit::builtin_type_ref(root, SourceKind::CSharp, "producer-1");
    let requirement = AcquisitionRequirement::incoming_type_ref(
        "ContextA",
        "admission-1",
        [unit.clone()],
    );
    let mut satisfaction = AcquisitionSatisfaction::new(&requirement);
    if complete {
        assert!(satisfaction.record_after_publication(
            ValidatedAcquisitionProof {
                unit,
                inventory_generation: "inventory-1".into(),
                candidate_owners: vec![format!("{root}/Consumer.cs")],
                provenance: ProofProvenance::Clean,
            },
            SemanticPublicationStatus::Complete,
        ));
    }
    satisfaction
}

#[test]
fn acquisition_cache_commits_only_complete_satisfaction() {
    let mut cache = AcquisitionCompletionCache::new();
    let cache_key = key("root-a");
    assert!(!cache.commit(cache_key.clone(), satisfaction("root-a", false)));
    assert!(cache.get(&cache_key).is_none());

    assert!(cache.commit(cache_key.clone(), satisfaction("root-a", true)));
    assert!(
        cache
            .get(&cache_key)
            .is_some_and(|value| value.is_source_complete())
    );
}

#[test]
fn acquisition_cache_key_rejects_every_changed_authority_dimension() {
    let mut cache = AcquisitionCompletionCache::new();
    let baseline = key("root-a");
    assert!(cache.commit(baseline.clone(), satisfaction("root-a", true)));

    let mut changed = Vec::new();
    let mut target = baseline.clone();
    target.target = AcquisitionTarget::exact_type_ref("ContextB");
    changed.push(target);
    changed.push(key("root-b"));
    let mut admission = baseline.clone();
    admission.admission_fingerprint = "admission-2".into();
    changed.push(admission);
    let mut producer = baseline.clone();
    producer.producer_generations[0].1 = "producer-2".into();
    changed.push(producer);
    let mut predicate = baseline.clone();
    predicate.predicate_version = 2;
    changed.push(predicate);
    let mut contract = baseline.clone();
    contract.proof_contract_version = 2;
    changed.push(contract);

    for distinct in changed {
        assert!(cache.get(&distinct).is_none());
    }
}

#[test]
fn acquisition_cache_root_generation_invalidates_only_affected_entries() {
    let mut cache = AcquisitionCompletionCache::new();
    let root_a = key("root-a");
    let root_b = key("root-b");
    assert!(cache.commit(root_a.clone(), satisfaction("root-a", true)));
    assert!(cache.commit(root_b.clone(), satisfaction("root-b", true)));

    cache.invalidate_root("root-a");

    assert!(cache.get(&root_a).is_none());
    assert!(cache.get(&root_b).is_some());
}
