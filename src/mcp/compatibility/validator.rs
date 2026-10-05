//! Pure validation boundary between structurally decoded persistence and live
//! canonical or semantic authority.

use super::derive_identities;
use super::identity::{CompatibilityIdentities, PersistedCompatibilityIdentities};
use crate::compression::Fidelity;
use crate::config::CleanCtxConfig;
use crate::ir::compiler::CompiledIR;
use crate::layers::meta::semantic::SemanticEdge;
use sha2::{Digest, Sha256};
use std::path::Path;

#[derive(Debug, Clone)]
pub(crate) struct UntrustedDurableContext {
    pub ir: CompiledIR,
    pub semantic_edges: Vec<SemanticEdge>,
    pub source_hash: String,
    pub fidelity: Fidelity,
    pub compatibility: PersistedCompatibilityIdentities,
}

#[derive(Debug, Clone)]
pub(crate) struct CompatibleCanonicalState {
    pub ir: CompiledIR,
    pub source_hash: String,
    pub fidelity: Fidelity,
}

#[derive(Debug, Clone)]
pub(crate) struct CompatibleSemanticProjection {
    pub semantic_edges: Vec<SemanticEdge>,
    pub source_hash: String,
    pub fidelity: Fidelity,
}

#[derive(Debug, Clone)]
pub(crate) struct CompatibleDurableContext {
    pub canonical: CompatibleCanonicalState,
    pub semantic: CompatibleSemanticProjection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CompatibilityComponent {
    CanonicalConfiguration,
    CanonicalProducers,
    SemanticConfiguration,
    SemanticProducers,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub(crate) enum CompatibilityFailure {
    #[error("physical or schema incompatibility: {detail}")]
    PhysicalSchemaIncompatibility { detail: String },
    #[error("structural snapshot incoherence: {detail}")]
    StructuralSnapshotIncoherence { detail: String },
    #[error("durable source mismatch: persisted={persisted}, current={current}")]
    SourceMismatch { persisted: String, current: String },
    #[error("persisted fidelity {persisted:?} is below required {required:?}")]
    InsufficientFidelity {
        persisted: Fidelity,
        required: Fidelity,
    },
    #[error("canonical configuration is incompatible")]
    CanonicalConfigurationIncompatible,
    #[error("semantic configuration is incompatible")]
    SemanticConfigurationIncompatible,
    #[error("canonical producer set or generation is incompatible")]
    CanonicalProducerIncompatible,
    #[error("semantic producer set or generation is incompatible")]
    SemanticProducerIncompatible,
    #[error("legacy persistence is missing {component:?} identity")]
    MissingLegacyIdentity { component: CompatibilityComponent },
    #[error("historical source is not current: historical={historical}, current={current}")]
    // The frozen compatibility taxonomy reserves this for publication paths
    // that distinguish valid historical authority from current-source authority.
    #[allow(dead_code)]
    HistoricalSourceNotCurrent { historical: String, current: String },
}

impl CompatibilityFailure {
    pub(crate) fn reason(&self) -> &'static str {
        match self {
            Self::PhysicalSchemaIncompatibility { .. } => "physical_schema_incompatibility",
            Self::StructuralSnapshotIncoherence { .. } => "structural_snapshot_incoherence",
            Self::SourceMismatch { .. } => "source_mismatch",
            Self::InsufficientFidelity { .. } => "insufficient_fidelity",
            Self::CanonicalConfigurationIncompatible => {
                "canonical_configuration_incompatible"
            }
            Self::SemanticConfigurationIncompatible => "semantic_configuration_incompatible",
            Self::CanonicalProducerIncompatible => "canonical_producer_incompatible",
            Self::SemanticProducerIncompatible => "semantic_producer_incompatible",
            Self::MissingLegacyIdentity { .. } => "missing_legacy_identity",
            Self::HistoricalSourceNotCurrent { .. } => "historical_source_not_current",
        }
    }
}

pub(crate) fn validate_current_context(
    loaded: UntrustedDurableContext,
    source: &str,
    path: &Path,
    config: &CleanCtxConfig,
    required_fidelity: Fidelity,
) -> Result<CompatibleDurableContext, CompatibilityFailure> {
    let current_hash = format!("{:x}", Sha256::digest(source.as_bytes()));
    if loaded.source_hash != current_hash {
        return Err(CompatibilityFailure::SourceMismatch {
            persisted: loaded.source_hash,
            current: current_hash,
        });
    }
    let current = derive_identities(source, path, config).map_err(|error| {
        CompatibilityFailure::StructuralSnapshotIncoherence {
            detail: format!("compatibility identity derivation failed: {error}"),
        }
    })?;
    validate_with_identities(loaded, &current, required_fidelity)
}

pub(crate) fn validate_historical_context(
    loaded: UntrustedDurableContext,
    current_source: &str,
    path: &Path,
    config: &CleanCtxConfig,
    required_fidelity: Fidelity,
) -> Result<CompatibleDurableContext, CompatibilityFailure> {
    let current = derive_identities(current_source, path, config).map_err(|error| {
        CompatibilityFailure::StructuralSnapshotIncoherence {
            detail: format!("compatibility identity derivation failed: {error}"),
        }
    })?;
    validate_with_identities(loaded, &current, required_fidelity)
}

fn validate_with_identities(
    loaded: UntrustedDurableContext,
    current: &CompatibilityIdentities,
    required_fidelity: Fidelity,
) -> Result<CompatibleDurableContext, CompatibilityFailure> {
    if fidelity_rank(loaded.fidelity) < fidelity_rank(required_fidelity) {
        return Err(CompatibilityFailure::InsufficientFidelity {
            persisted: loaded.fidelity,
            required: required_fidelity,
        });
    }
    let semantic = validate_semantic(&loaded, current)?;
    let canonical = validate_canonical(&loaded, current)?;
    Ok(CompatibleDurableContext { canonical, semantic })
}

pub(crate) fn validate_canonical(
    loaded: &UntrustedDurableContext,
    current: &CompatibilityIdentities,
) -> Result<CompatibleCanonicalState, CompatibilityFailure> {
    let persisted_config = loaded.compatibility.canonical_config.as_ref().ok_or(
        CompatibilityFailure::MissingLegacyIdentity {
            component: CompatibilityComponent::CanonicalConfiguration,
        },
    )?;
    if persisted_config != &current.canonical_config {
        return Err(CompatibilityFailure::CanonicalConfigurationIncompatible);
    }
    let persisted_producers = loaded.compatibility.canonical_producers.as_ref().ok_or(
        CompatibilityFailure::MissingLegacyIdentity {
            component: CompatibilityComponent::CanonicalProducers,
        },
    )?;
    if persisted_producers != &current.canonical_producers {
        return Err(CompatibilityFailure::CanonicalProducerIncompatible);
    }
    Ok(CompatibleCanonicalState {
        ir: loaded.ir.clone(),
        source_hash: loaded.source_hash.clone(),
        fidelity: loaded.fidelity,
    })
}

pub(crate) fn validate_semantic(
    loaded: &UntrustedDurableContext,
    current: &CompatibilityIdentities,
) -> Result<CompatibleSemanticProjection, CompatibilityFailure> {
    let persisted_config = loaded.compatibility.semantic_config.as_ref().ok_or(
        CompatibilityFailure::MissingLegacyIdentity {
            component: CompatibilityComponent::SemanticConfiguration,
        },
    )?;
    if persisted_config != &current.semantic_config {
        return Err(CompatibilityFailure::SemanticConfigurationIncompatible);
    }
    let persisted_producers = loaded.compatibility.semantic_producers.as_ref().ok_or(
        CompatibilityFailure::MissingLegacyIdentity {
            component: CompatibilityComponent::SemanticProducers,
        },
    )?;
    if persisted_producers != &current.semantic_producers {
        return Err(CompatibilityFailure::SemanticProducerIncompatible);
    }
    Ok(CompatibleSemanticProjection {
        semantic_edges: loaded.semantic_edges.clone(),
        source_hash: loaded.source_hash.clone(),
        fidelity: loaded.fidelity,
    })
}

fn fidelity_rank(fidelity: Fidelity) -> u8 {
    match fidelity {
        Fidelity::Low => 0,
        Fidelity::Medium => 1,
        Fidelity::High => 2,
        Fidelity::Edit => 3,
        Fidelity::Verbatim => 4,
    }
}

#[cfg(test)]
#[path = "../../tests/mcp/durable_compatibility_validation.rs"]
mod tests;
