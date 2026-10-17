//! Provider-neutral acquisition requirements and completeness proofs.
//!
//! These types are introduced before the hydration migration so the authority
//! contract can be pinned independently. The allowance is migration-only: each
//! item becomes production-used as hydration, caching, and coverage move onto
//! this boundary in the following Issue #144 slices.

#![allow(dead_code, unused_imports)]

mod proof;
mod requirement;
mod satisfaction;

pub(crate) use proof::{
    AcquisitionProofClaim, ProofContext, ProofProvenance, ProofRejection,
    ValidatedAcquisitionProof, validate_proof_claim,
};
pub(crate) use requirement::{
    AcquisitionRequirement, AcquisitionTarget, CoverageUnit, EvidenceRequirement,
    ProducerCapability, RelationshipFamily, SourceKind,
};
pub(crate) use satisfaction::AcquisitionSatisfaction;

#[cfg(all(test, feature = "rust"))]
#[path = "../tests/mcp/acquisition_requirement.rs"]
mod requirement_tests;

#[cfg(all(test, feature = "rust"))]
#[path = "../tests/mcp/acquisition_proof.rs"]
mod proof_tests;
