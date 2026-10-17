use super::proof::ValidatedAcquisitionProof;
use super::requirement::{AcquisitionRequirement, CoverageUnit};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SemanticPublicationStatus {
    Complete,
    Failed,
}

/// Proof coverage accumulated for one exact acquisition requirement.
#[derive(Clone, Debug)]
pub(crate) struct AcquisitionSatisfaction {
    required: BTreeSet<CoverageUnit>,
    proven: BTreeMap<CoverageUnit, ValidatedAcquisitionProof>,
}

impl AcquisitionSatisfaction {
    pub(crate) fn new(requirement: &AcquisitionRequirement) -> Self {
        Self {
            required: requirement.units.clone(),
            proven: BTreeMap::new(),
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
}
