use super::requirement::{AcquisitionRequirement, AcquisitionTarget, CoverageUnit};

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
