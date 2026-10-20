use super::filesystem::RootInventoryManifest;
use crate::mcp::acquisition::{
    AcquisitionCacheKey, AcquisitionOutcome, AcquisitionRequirement, AcquisitionSatisfaction,
    AcquisitionTarget, CleanProofRejection, CoverageProofDisposition, CoverageUnit,
    EvidenceRequirement, InventoryManifest, ProducerCapability, ProducerCapabilityDisposition,
    ProofBarrier, ProofContext, SemanticPublicationStatus, SourceKind, TYPE_REF_CAPABILITY_VERSION,
    ValidatedAcquisitionProof, builtin_type_ref_capability, clean_full_unit_proof_claim,
    clean_type_ref_proof_claim, has_certified_type_ref_name_bearing_predicate,
    producer_generation_fingerprint, scope_policy_fingerprint, validate_proof_claim,
};
use crate::mcp::compatibility::producer::GenerationCatalog;
use std::collections::BTreeMap;

const TYPE_REF_PREDICATE_VERSION: u32 = 1;
const ACQUISITION_PROOF_CONTRACT_VERSION: u32 = 2;

pub(super) fn acquisition_cache_key(
    target: AcquisitionTarget,
    roots: Vec<String>,
    exclusions: &[String],
) -> AcquisitionCacheKey {
    let capability = target.capability();
    let catalog = GenerationCatalog::current();
    let admission_fingerprint =
        scope_policy_fingerprint(roots.iter().cloned(), exclusions.iter().cloned());
    let producer_generations = [
        SourceKind::CSharp,
        SourceKind::TypeScript,
        SourceKind::Rust,
        SourceKind::Java,
    ]
    .into_iter()
    .map(|source_kind| {
        let generation = capability_generation(source_kind, &catalog, capability);
        (source_kind, generation)
    })
    .collect();
    AcquisitionCacheKey {
        target,
        roots,
        admission_fingerprint,
        producer_generations,
        capability_version: TYPE_REF_CAPABILITY_VERSION,
        predicate_version: TYPE_REF_PREDICATE_VERSION,
        proof_contract_version: ACQUISITION_PROOF_CONTRACT_VERSION,
        evidence: EvidenceRequirement::CurrentHighSemanticProjection,
    }
}

pub(super) struct ProvisionalAcquisition {
    requirement: AcquisitionRequirement,
    proofs: Vec<ValidatedAcquisitionProof>,
    barriers: BTreeMap<CoverageUnit, ProofBarrier>,
    owners: Vec<String>,
    source_snapshots: Vec<(String, String)>,
}

impl ProvisionalAcquisition {
    pub(super) fn owners(&self) -> &[String] {
        &self.owners
    }

    pub(super) fn publication_is_current(&self, state: &crate::mcp::McpState) -> bool {
        self.source_snapshots.iter().all(|(owner, expected)| {
            state.read_source(owner).is_ok_and(|source| {
                crate::cache::content_hash(source.as_bytes()) == *expected
                    && state
                        .workspace_index_read()
                        .has_current_semantic_projection(
                            owner,
                            crate::workspace::index::SemanticFidelity::High,
                            expected,
                        )
            })
        })
    }
}

pub(super) fn prepare_clean_acquisition(
    target: AcquisitionTarget,
    inventories: &[RootInventoryManifest],
    exclusions: &[String],
) -> Option<ProvisionalAcquisition> {
    let roots = inventories.iter().map(|inventory| inventory.root.clone());
    let admission = scope_policy_fingerprint(roots, exclusions.iter().cloned());
    let catalog = GenerationCatalog::current();
    let grouped: Vec<_> = inventories
        .iter()
        .flat_map(|inventory| manifests_by_source_kind(inventory, &catalog, target.capability()))
        .collect();
    // A missing root inventory cannot disappear from the obligation merely
    // because it contains no admitted producer entries.
    if inventories.is_empty()
        || (target.capability() == ProducerCapability::SourceDeclarations
            && inventories.iter().any(|root| !root.manifest.complete))
    {
        return None;
    }
    if grouped.is_empty()
        && inventories
            .iter()
            .any(|inventory| !inventory.manifest.complete)
    {
        return None;
    }

    let requirement = AcquisitionRequirement::for_target(
        target,
        admission.clone(),
        grouped.iter().map(|(unit, _)| unit.clone()),
    );
    let mut proofs = Vec::new();
    let mut barriers = BTreeMap::new();
    let mut owners = Vec::new();
    let mut source_snapshots = Vec::new();
    for (unit, manifest) in grouped {
        if unit.producer_generation == "missing-generation" {
            barriers.insert(unit, ProofBarrier::ClaimIncompatible);
            continue;
        }
        let claim = match if unit.producer == ProducerCapability::BuiltinTypeRefIncoming
            && has_certified_type_ref_name_bearing_predicate(unit.source_kind)
        {
            clean_type_ref_proof_claim(
                &requirement,
                &unit,
                &manifest,
                admission.clone(),
                TYPE_REF_PREDICATE_VERSION,
                ACQUISITION_PROOF_CONTRACT_VERSION,
            )
        } else {
            clean_full_unit_proof_claim(
                &requirement,
                &unit,
                &manifest,
                admission.clone(),
                TYPE_REF_PREDICATE_VERSION,
                ACQUISITION_PROOF_CONTRACT_VERSION,
            )
        } {
            Ok(claim) => claim,
            Err(rejection) => {
                barriers.insert(unit, barrier_for_clean_rejection(rejection));
                continue;
            }
        };
        match validate_proof_claim(
            &requirement,
            &ProofContext {
                inventory_generation: manifest.digest,
                admission_fingerprint: admission.clone(),
                predicate_version: TYPE_REF_PREDICATE_VERSION,
                proof_contract_version: ACQUISITION_PROOF_CONTRACT_VERSION,
            },
            claim,
        ) {
            Ok(proof) => {
                source_snapshots.extend(
                    manifest
                        .entries
                        .iter()
                        .filter(|entry| proof.candidate_owners.contains(&entry.owner))
                        .map(|entry| (entry.owner.clone(), entry.source_hash.clone())),
                );
                owners.extend(proof.candidate_owners.iter().cloned());
                proofs.push(proof);
            }
            Err(_) => {
                barriers.insert(unit, ProofBarrier::ClaimIncompatible);
            }
        }
    }
    Some(ProvisionalAcquisition {
        requirement,
        proofs,
        barriers,
        owners,
        source_snapshots,
    })
}

impl ProvisionalAcquisition {
    pub(super) fn finish(
        self,
        publication: SemanticPublicationStatus,
        failure_barrier: ProofBarrier,
    ) -> AcquisitionOutcome {
        let mut satisfaction = AcquisitionSatisfaction::new(&self.requirement);
        let mut dispositions = self
            .barriers
            .into_iter()
            .map(|(unit, barrier)| (unit, CoverageProofDisposition::Unproven(barrier)))
            .collect::<BTreeMap<_, _>>();
        for proof in self.proofs {
            let unit = proof.unit.clone();
            if satisfaction.record_after_publication(proof, publication) {
                dispositions.insert(unit, CoverageProofDisposition::Proven);
            } else {
                dispositions.insert(unit, CoverageProofDisposition::Unproven(failure_barrier));
            }
        }
        AcquisitionOutcome::new(satisfaction, dispositions, publication)
    }
}

fn barrier_for_clean_rejection(rejection: CleanProofRejection) -> ProofBarrier {
    match rejection {
        CleanProofRejection::IncompleteInventory => ProofBarrier::InventoryIncomplete,
        CleanProofRejection::UncertifiedPredicate => ProofBarrier::CandidatePredicateUncertified,
        CleanProofRejection::SourceKindMismatch | CleanProofRejection::OwnerOutsideRoot => {
            ProofBarrier::ClaimIncompatible
        }
    }
}

fn manifests_by_source_kind(
    root: &RootInventoryManifest,
    catalog: &GenerationCatalog,
    capability: ProducerCapability,
) -> Vec<(CoverageUnit, InventoryManifest)> {
    let mut grouped = BTreeMap::<SourceKind, Vec<_>>::new();
    for entry in &root.manifest.entries {
        grouped
            .entry(entry.source_kind)
            .or_default()
            .push(entry.clone());
    }
    grouped
        .into_iter()
        .filter(|(source_kind, _)| {
            capability == ProducerCapability::SourceDeclarations
                || builtin_type_ref_capability(*source_kind)
                    != ProducerCapabilityDisposition::Incapable
        })
        .map(|(source_kind, entries)| {
            let generation = capability_generation(source_kind, catalog, capability);
            (
                match capability {
                    ProducerCapability::BuiltinTypeRefIncoming => {
                        CoverageUnit::builtin_type_ref(&root.root, source_kind, generation)
                    }
                    ProducerCapability::SourceDeclarations => {
                        CoverageUnit::declarations(&root.root, source_kind, generation)
                    }
                },
                InventoryManifest::new(entries, root.manifest.attempted, root.manifest.complete),
            )
        })
        .collect()
}

fn capability_generation(
    source_kind: SourceKind,
    catalog: &GenerationCatalog,
    capability: ProducerCapability,
) -> String {
    let Some(native) = producer_generation_fingerprint(source_kind, catalog) else {
        return "missing-generation".into();
    };
    match capability {
        ProducerCapability::BuiltinTypeRefIncoming => native,
        ProducerCapability::SourceDeclarations => {
            format!("{native}:{}", catalog.declaration_generation())
        }
    }
}
