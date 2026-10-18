use super::proof::ValidatedAcquisitionProof;
use super::requirement::{AcquisitionRequirement, CoverageUnit, requirement_fingerprint};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SemanticPublicationStatus {
    Complete,
    Failed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AcquisitionProcessingState {
    Terminal,
    Incomplete,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProofBarrier {
    InventoryIncomplete,
    CandidatePredicateUncertified,
    ClaimIncompatible,
    PublicationFailed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CoverageProofDisposition {
    Proven,
    Unproven(ProofBarrier),
}

/// Request-local proof outcome. Unlike cached satisfaction, this retains why
/// each required coverage unit did or did not become authoritative.
#[derive(Clone, Debug)]
pub(crate) struct AcquisitionOutcome {
    satisfaction: AcquisitionSatisfaction,
    dispositions: BTreeMap<CoverageUnit, CoverageProofDisposition>,
    processing: AcquisitionProcessingState,
}

impl AcquisitionOutcome {
    pub(crate) fn new(
        satisfaction: AcquisitionSatisfaction,
        dispositions: BTreeMap<CoverageUnit, CoverageProofDisposition>,
        publication: SemanticPublicationStatus,
    ) -> Self {
        Self {
            satisfaction,
            dispositions,
            processing: match publication {
                SemanticPublicationStatus::Complete => AcquisitionProcessingState::Terminal,
                SemanticPublicationStatus::Failed => AcquisitionProcessingState::Incomplete,
            },
        }
    }

    pub(crate) fn satisfaction(&self) -> &AcquisitionSatisfaction {
        &self.satisfaction
    }

    pub(crate) fn from_cached_complete(satisfaction: AcquisitionSatisfaction) -> Option<Self> {
        satisfaction.is_source_complete().then(|| Self {
            satisfaction,
            dispositions: BTreeMap::new(),
            processing: AcquisitionProcessingState::Terminal,
        })
    }

    pub(crate) fn dispositions(&self) -> &BTreeMap<CoverageUnit, CoverageProofDisposition> {
        &self.dispositions
    }

    pub(crate) fn processing(&self) -> AcquisitionProcessingState {
        self.processing
    }
}

/// Proof coverage accumulated for one exact acquisition requirement.
#[derive(Clone, Debug)]
pub(crate) struct AcquisitionSatisfaction {
    required: BTreeSet<CoverageUnit>,
    proven: BTreeMap<CoverageUnit, ValidatedAcquisitionProof>,
    requirement_fingerprint: String,
    scope_fingerprint: String,
}

impl AcquisitionSatisfaction {
    pub(crate) fn new(requirement: &AcquisitionRequirement) -> Self {
        Self {
            required: requirement.units.clone(),
            proven: BTreeMap::new(),
            requirement_fingerprint: requirement_fingerprint(requirement),
            scope_fingerprint: requirement.scope_fingerprint.clone(),
        }
    }

    pub(crate) fn record_after_publication(
        &mut self,
        proof: ValidatedAcquisitionProof,
        publication: SemanticPublicationStatus,
    ) -> bool {
        if publication != SemanticPublicationStatus::Complete {
            return false;
        }
        if !self.required.contains(&proof.unit) {
            return false;
        }
        self.proven.insert(proof.unit.clone(), proof);
        true
    }

    pub(crate) fn residual_units(&self) -> BTreeSet<CoverageUnit> {
        self.required
            .iter()
            .filter(|unit| !self.proven.contains_key(*unit))
            .cloned()
            .collect()
    }

    pub(crate) fn is_source_complete(&self) -> bool {
        !self.required.is_empty() && self.residual_units().is_empty()
    }

    pub(crate) fn completeness_generation(&self) -> Option<String> {
        if !self.is_source_complete() {
            return None;
        }
        let mut hasher = Sha256::new();
        hash_part(&mut hasher, self.requirement_fingerprint.as_bytes());
        for proof in self.proven.values() {
            hash_part(&mut hasher, proof.inventory_generation.as_bytes());
        }
        Some(format!("{:x}", hasher.finalize()))
    }

    pub(crate) fn scope_fingerprint(&self) -> &str {
        &self.scope_fingerprint
    }
}

fn hash_part(hasher: &mut Sha256, value: &[u8]) {
    hasher.update((value.len() as u64).to_le_bytes());
    hasher.update(value);
}
