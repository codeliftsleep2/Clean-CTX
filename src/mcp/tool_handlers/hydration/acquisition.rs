use super::filesystem::RootInventoryManifest;
use crate::mcp::acquisition::{
    AcquisitionCacheKey, AcquisitionOutcome, AcquisitionRequirement, AcquisitionSatisfaction,
    AcquisitionTarget, CleanProofRejection, CoverageProofDisposition, CoverageUnit,
    EvidenceRequirement, InventoryManifest, ProducerCapabilityDisposition, ProofBarrier,
    ProofContext, SemanticPublicationStatus, SourceKind, TYPE_REF_CAPABILITY_VERSION,
    ValidatedAcquisitionProof, builtin_type_ref_capability, clean_full_unit_type_ref_proof_claim,
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
        let generation = producer_generation_fingerprint(source_kind, &catalog)
            .unwrap_or_else(|| "missing-generation".into());
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
}

impl ProvisionalAcquisition {
    pub(super) fn owners(&self) -> &[String] {
        &self.owners
    }
}

pub(super) fn prepare_clean_type_ref_acquisition(
    target: AcquisitionTarget,
    inventories: &[RootInventoryManifest],
    exclusions: &[String],
) -> Option<ProvisionalAcquisition> {
    let roots = inventories.iter().map(|inventory| inventory.root.clone());
    let admission = scope_policy_fingerprint(roots, exclusions.iter().cloned());
    let catalog = GenerationCatalog::current();
    let grouped: Vec<_> = inventories
        .iter()
        .flat_map(|inventory| manifests_by_source_kind(inventory, &catalog))
        .collect();
    if grouped.is_empty()
        && inventories
            .iter()
            .any(|inventory| !inventory.manifest.complete)
    {
        return None;
    }

    let requirement = AcquisitionRequirement::incoming_type_ref(
        target.written_name,
        admission.clone(),
        grouped.iter().map(|(unit, _)| unit.clone()),
    );
    let mut proofs = Vec::new();
    let mut barriers = BTreeMap::new();
    let mut owners = Vec::new();
    for (unit, manifest) in grouped {
        let claim = match if has_certified_type_ref_name_bearing_predicate(unit.source_kind) {
            clean_type_ref_proof_claim(
                &requirement,
                &unit,
                &manifest,
                admission.clone(),
                TYPE_REF_PREDICATE_VERSION,
                ACQUISITION_PROOF_CONTRACT_VERSION,
            )
        } else {
            clean_full_unit_type_ref_proof_claim(
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
            builtin_type_ref_capability(*source_kind) != ProducerCapabilityDisposition::Incapable
        })
        .map(|(source_kind, entries)| {
            let generation = producer_generation_fingerprint(source_kind, catalog)
                .unwrap_or_else(|| "missing-generation".into());
            (
                CoverageUnit::builtin_type_ref(&root.root, source_kind, generation),
                InventoryManifest::new(entries, root.manifest.attempted, root.manifest.complete),
            )
        })
        .collect()
}
