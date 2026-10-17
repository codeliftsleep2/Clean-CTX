use super::proof::{AcquisitionProofClaim, InventoryManifest, ProofProvenance};
use super::requirement::{AcquisitionRequirement, CoverageUnit, SourceKind};
use std::path::Path;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CleanProofRejection {
    IncompleteInventory,
    UncertifiedPredicate,
    SourceKindMismatch,
    OwnerOutsideRoot,
}

/// Whether the active producer has a tracked guarantee that every emitted
/// builtin TypeRef target occurs verbatim in its authoritative source.
pub(crate) fn has_certified_type_ref_name_bearing_predicate(kind: SourceKind) -> bool {
    matches!(kind, SourceKind::CSharp)
}

/// Select owners whose captured source snapshot contains the exact written
/// TypeRef target. This is candidate selection only: until every participating
/// producer has a tested name-bearing guarantee, it cannot create a coverage
/// proof or exclude a coverage unit from authoritative acquisition.
pub(crate) fn type_ref_candidate_owners(
    manifest: &InventoryManifest,
    written_name: &str,
) -> Vec<String> {
    if written_name.is_empty() {
        return Vec::new();
    }

    manifest
        .entries
        .iter()
        .filter(|entry| entry.source.contains(written_name))
        .map(|entry| entry.owner.clone())
        .collect()
}

/// Construct a provisional Clean-owned claim for one certified coverage unit.
/// Validation and semantic publication must still succeed before the claim can
/// become completion authority.
pub(crate) fn clean_type_ref_proof_claim(
    requirement: &AcquisitionRequirement,
    unit: &CoverageUnit,
    manifest: &InventoryManifest,
    admission_fingerprint: impl Into<String>,
    predicate_version: u32,
    proof_contract_version: u32,
) -> Result<AcquisitionProofClaim, CleanProofRejection> {
    if !manifest.complete {
        return Err(CleanProofRejection::IncompleteInventory);
    }
    if !has_certified_type_ref_name_bearing_predicate(unit.source_kind) {
        return Err(CleanProofRejection::UncertifiedPredicate);
    }

    let root = Path::new(&unit.canonical_root);
    if manifest
        .entries
        .iter()
        .any(|entry| !Path::new(&entry.owner).starts_with(root))
    {
        return Err(CleanProofRejection::OwnerOutsideRoot);
    }
    if manifest
        .entries
        .iter()
        .any(|entry| entry.source_kind != unit.source_kind)
    {
        return Err(CleanProofRejection::SourceKindMismatch);
    }

    Ok(AcquisitionProofClaim {
        unit: unit.clone(),
        target: requirement.target.clone(),
        scope_fingerprint: requirement.scope_fingerprint.clone(),
        inventory_generation: manifest.digest.clone(),
        admission_fingerprint: admission_fingerprint.into(),
        predicate_version,
        proof_contract_version,
        exhaustive: true,
        truncated: false,
        limitations: Vec::new(),
        candidate_owners: type_ref_candidate_owners(manifest, &requirement.target.written_name),
        provenance: ProofProvenance::Clean,
    })
}
