//! Provider-neutral acquisition requirements and completeness proofs.
//!
//! These types are introduced before the hydration migration so the authority
//! contract can be pinned independently. The allowance is migration-only: each
//! item becomes production-used as hydration, caching, and coverage move onto
//! this boundary in the following Issue #144 slices.

#![allow(dead_code, unused_imports)]

mod cache;
mod clean;
mod proof;
mod requirement;
mod satisfaction;

pub(crate) use clean::{
    CleanProofRejection, clean_full_unit_type_ref_proof_claim, clean_type_ref_proof_claim,
    has_certified_type_ref_name_bearing_predicate, type_ref_candidate_owners,
};
pub(crate) use proof::{
    AcquisitionProofClaim, InventoryEntry, InventoryManifest, ProofContext, ProofProvenance,
    ProofRejection, ValidatedAcquisitionProof, scope_policy_fingerprint, validate_proof_claim,
};
pub(crate) use requirement::{
    AcquisitionRequirement, AcquisitionTarget, CoverageUnit, EvidenceRequirement,
    ProducerCapability, ProducerCapabilityDisposition, RelationshipFamily, SourceKind,
    TYPE_REF_CAPABILITY_VERSION, builtin_type_ref_capability, producer_generation_fingerprint,
    requirement_fingerprint,
};
pub(crate) use satisfaction::{
    AcquisitionOutcome, AcquisitionProcessingState, AcquisitionSatisfaction,
    CoverageProofDisposition, ProofBarrier, SemanticPublicationStatus,
};

#[cfg(all(test, feature = "rust"))]
#[path = "../tests/mcp/acquisition_requirement.rs"]
mod requirement_tests;

#[cfg(all(test, feature = "rust"))]
#[path = "../tests/mcp/acquisition_proof.rs"]
mod proof_tests;

#[cfg(all(test, feature = "rust"))]
#[path = "../tests/mcp/acquisition_inventory.rs"]
mod inventory_tests;

#[cfg(all(test, feature = "rust"))]
#[path = "../tests/mcp/acquisition_cache.rs"]
mod cache_tests;

#[cfg(all(test, feature = "rust"))]
#[path = "../tests/mcp/acquisition_clean.rs"]
mod clean_tests;
pub(crate) use cache::{AcquisitionCacheKey, AcquisitionCompletionCache};
