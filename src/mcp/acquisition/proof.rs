use super::requirement::SourceKind;
use super::requirement::{AcquisitionRequirement, AcquisitionTarget, CoverageUnit};
use sha2::{Digest, Sha256};
use std::sync::Arc;

/// One admitted canonical source owner and the exact snapshot hashed for proof.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InventoryEntry {
    pub(crate) owner: String,
    pub(crate) source_kind: SourceKind,
    pub(crate) source_hash: String,
    pub(crate) source: Arc<String>,
}

/// Deterministic admitted-source inventory for one acquisition pass.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InventoryManifest {
    pub(crate) entries: Vec<InventoryEntry>,
    pub(crate) digest: String,
    pub(crate) attempted: bool,
    pub(crate) complete: bool,
}

impl InventoryManifest {
    pub(crate) fn new(mut entries: Vec<InventoryEntry>, attempted: bool, complete: bool) -> Self {
        entries.sort_by(|left, right| left.owner.cmp(&right.owner));
        entries.dedup_by(|left, right| left.owner == right.owner);
        let mut hasher = Sha256::new();
        for entry in &entries {
            update_length_delimited(&mut hasher, entry.owner.as_bytes());
            update_length_delimited(&mut hasher, format!("{:?}", entry.source_kind).as_bytes());
            update_length_delimited(&mut hasher, entry.source_hash.as_bytes());
        }
        Self {
            entries,
            digest: format!("{:x}", hasher.finalize()),
            attempted,
            complete: attempted && complete,
        }
    }
}

fn update_length_delimited(hasher: &mut Sha256, value: &[u8]) {
    hasher.update((value.len() as u64).to_le_bytes());
    hasher.update(value);
}

pub(crate) fn scope_policy_fingerprint(
    roots: impl IntoIterator<Item = String>,
    exclusions: impl IntoIterator<Item = String>,
) -> String {
    let mut roots: Vec<_> = roots.into_iter().collect();
    roots.sort();
    roots.dedup();
    let mut exclusions: Vec<_> = exclusions.into_iter().collect();
    exclusions.sort();
    exclusions.dedup();

    let mut hasher = Sha256::new();
    update_length_delimited(&mut hasher, b"clean-admission-v1");
    for root in roots {
        update_length_delimited(&mut hasher, root.as_bytes());
    }
    hasher.update([0xff]);
    for exclusion in exclusions {
        update_length_delimited(&mut hasher, exclusion.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

/// Diagnostic/invalidation provenance. It is deliberately absent from
/// `CoverageUnit` identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ProofProvenance {
    Clean,
    Provider(String),
}

/// Untrusted claim supplied by an acquisition mechanism.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AcquisitionProofClaim {
    pub(crate) unit: CoverageUnit,
    pub(crate) target: AcquisitionTarget,
    pub(crate) scope_fingerprint: String,
    pub(crate) inventory_generation: String,
    pub(crate) admission_fingerprint: String,
    pub(crate) predicate_version: u32,
    pub(crate) proof_contract_version: u32,
    pub(crate) exhaustive: bool,
    pub(crate) truncated: bool,
    pub(crate) limitations: Vec<String>,
    pub(crate) candidate_owners: Vec<String>,
    pub(crate) provenance: ProofProvenance,
}

/// Current Clean-owned facts against which a claim is validated.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProofContext {
    pub(crate) inventory_generation: String,
    pub(crate) admission_fingerprint: String,
    pub(crate) predicate_version: u32,
    pub(crate) proof_contract_version: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProofRejection {
    UnitNotRequired,
    TargetMismatch,
    ScopeMismatch,
    InventoryGenerationMismatch,
    AdmissionPolicyMismatch,
    PredicateVersionMismatch,
    ContractVersionMismatch,
    NotExhaustive,
    Truncated,
    Limited,
}

/// A claim accepted by Clean-CTX for one exact compatible coverage unit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ValidatedAcquisitionProof {
    pub(crate) unit: CoverageUnit,
    pub(crate) inventory_generation: String,
    pub(crate) candidate_owners: Vec<String>,
    pub(crate) provenance: ProofProvenance,
}

pub(crate) fn validate_proof_claim(
    requirement: &AcquisitionRequirement,
    context: &ProofContext,
    claim: AcquisitionProofClaim,
) -> Result<ValidatedAcquisitionProof, ProofRejection> {
    if !requirement.units.contains(&claim.unit) {
        return Err(ProofRejection::UnitNotRequired);
    }
    if requirement.target != claim.target {
        return Err(ProofRejection::TargetMismatch);
    }
    if requirement.scope_fingerprint != claim.scope_fingerprint {
        return Err(ProofRejection::ScopeMismatch);
    }
    if context.inventory_generation != claim.inventory_generation {
        return Err(ProofRejection::InventoryGenerationMismatch);
    }
    if context.admission_fingerprint != claim.admission_fingerprint {
        return Err(ProofRejection::AdmissionPolicyMismatch);
    }
    if context.predicate_version != claim.predicate_version {
        return Err(ProofRejection::PredicateVersionMismatch);
    }
    if context.proof_contract_version != claim.proof_contract_version {
        return Err(ProofRejection::ContractVersionMismatch);
    }
    if !claim.exhaustive {
        return Err(ProofRejection::NotExhaustive);
    }
    if claim.truncated {
        return Err(ProofRejection::Truncated);
    }
    if !claim.limitations.is_empty() {
        return Err(ProofRejection::Limited);
    }

    let mut candidate_owners = claim.candidate_owners;
    candidate_owners.sort();
    candidate_owners.dedup();
    Ok(ValidatedAcquisitionProof {
        unit: claim.unit,
        inventory_generation: claim.inventory_generation,
        candidate_owners,
        provenance: claim.provenance,
    })
}
