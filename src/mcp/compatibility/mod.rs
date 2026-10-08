//! Compatibility vocabulary for durable canonical and semantic authority.
//!
//! Identity derivation and pure validation remain separate from persistence
//! decoding and live-state publication.

pub(crate) mod identity;
pub(crate) mod producer;
pub(crate) mod validator;

use crate::config::{CleanCtxConfig, MetaLayerConfig};
use crate::layers::meta::MetaLayer;
use identity::{
    AngularMarkerConfigIdentity, AngularSemanticConfigIdentity, CanonicalConfigIdentity,
    CanonicalProducerIdentity, CompatibilityIdentities, ComponentDigest,
    DotNetMarkerConfigIdentity, DotNetSemanticConfigIdentity, IDENTITY_SCHEMA_VERSION, ProducerKey,
    SemanticConfigIdentity, SemanticProducerIdentity,
};
use producer::GenerationCatalog;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub(crate) enum IdentityDerivationError {
    #[error("no active canonical language producer for {0}")]
    UnsupportedSource(String),
    #[error("failed to serialize deterministic compatibility input: {0}")]
    Serialization(String),
    #[error("active producer generation is missing for {0:?}")]
    MissingGeneration(ProducerKey),
}

pub(crate) fn derive_identities(
    source: &str,
    path: &Path,
    config: &CleanCtxConfig,
) -> Result<CompatibilityIdentities, IdentityDerivationError> {
    derive_with_catalog(source, path, config, &GenerationCatalog::current())
}

fn derive_with_catalog(
    source: &str,
    path: &Path,
    config: &CleanCtxConfig,
    catalog: &GenerationCatalog,
) -> Result<CompatibilityIdentities, IdentityDerivationError> {
    let language = language_keys(path)?;
    let applicability = FrameworkApplicability::derive(source, path, config);

    Ok(CompatibilityIdentities {
        canonical_config: canonical_config_identity(config, &applicability)?,
        canonical_producers: canonical_producer_identity(
            config,
            language,
            &applicability,
            catalog,
        )?,
        semantic_config: semantic_config_identity(config, &applicability),
        semantic_producers: semantic_producer_identity(language, &applicability, catalog)?,
    })
}

#[derive(Debug, Clone, Copy)]
struct LanguageKeys {
    canonical: ProducerKey,
    semantic_input: ProducerKey,
    semantic_projection: Option<ProducerKey>,
}

fn language_keys(path: &Path) -> Result<LanguageKeys, IdentityDerivationError> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    let layer = crate::layers::LayerRegistry::global()
        .language_layer_for_extension(extension)
        .filter(|layer| layer.language_ptr().is_some())
        .ok_or_else(|| IdentityDerivationError::UnsupportedSource(path.display().to_string()))?;
    match layer.name() {
        "typescript" => Ok(LanguageKeys {
            canonical: ProducerKey::TypeScriptCanonical,
            semantic_input: ProducerKey::TypeScriptSemanticInput,
            semantic_projection: None,
        }),
        "csharp" => Ok(LanguageKeys {
            canonical: ProducerKey::CSharpCanonical,
            semantic_input: ProducerKey::CSharpSemanticInput,
            semantic_projection: Some(ProducerKey::CSharpSemanticProjection),
        }),
        "rust" => Ok(LanguageKeys {
            canonical: ProducerKey::RustCanonical,
            semantic_input: ProducerKey::RustSemanticInput,
            semantic_projection: None,
        }),
        "java" => Ok(LanguageKeys {
            canonical: ProducerKey::JavaCanonical,
            semantic_input: ProducerKey::JavaSemanticInput,
            semantic_projection: None,
        }),
        _ => Err(IdentityDerivationError::UnsupportedSource(
            path.display().to_string(),
        )),
    }
}

#[derive(Debug, Default)]
struct FrameworkApplicability {
    angular: bool,
    dotnet: bool,
    spring: bool,
}

impl FrameworkApplicability {
    fn derive(source: &str, path: &Path, config: &CleanCtxConfig) -> Self {
        let registry = crate::layers::LayerRegistry::global();
        Self {
            angular: framework_applies(registry.meta_layers(), "angular", source, path, config),
            dotnet: framework_applies(registry.meta_layers(), "dotnet", source, path, config),
            spring: framework_applies(registry.meta_layers(), "spring_boot", source, path, config),
        }
    }
}

fn framework_applies(
    layers: &[Box<dyn MetaLayer>],
    name: &str,
    source: &str,
    path: &Path,
    config: &CleanCtxConfig,
) -> bool {
    layers
        .iter()
        .find(|layer| layer.name() == name)
        .is_some_and(|layer| layer.is_applicable(source, path, Some(config)))
}

fn canonical_config_identity(
    config: &CleanCtxConfig,
    applicability: &FrameworkApplicability,
) -> Result<CanonicalConfigIdentity, IdentityDerivationError> {
    let alias_bytes = serde_json::to_vec(&config.type_aliases)
        .map_err(|error| IdentityDerivationError::Serialization(error.to_string()))?;
    let angular = applicability.angular.then(|| {
        let value = framework_config(config, "angular");
        AngularMarkerConfigIdentity {
            enabled: value.enabled,
            rxjs_enabled: value.rxjs.enabled,
            rxjs_min_pipe_operators: value.rxjs.min_pipe_operators,
            ngrx_enabled: value.ngrx.enabled,
            ngrx_include_dispatch_sites: value.ngrx.include_dispatch_sites,
            ngrx_include_select_sites: value.ngrx.include_select_sites,
            ngrx_entity_selectors: value.ngrx.entity_selectors,
            signals_enabled: value.signals.enabled,
            reactive_forms_enabled: value.reactive_forms.enabled,
            formly_enabled: value.formly.enabled,
            routing_enabled: value.routing.enabled,
            testing_enabled: value.testing.enabled,
        }
    });
    let dotnet = applicability.dotnet.then(|| {
        let value = framework_config(config, "dotnet");
        DotNetMarkerConfigIdentity {
            enabled: value.enabled,
            testing_enabled: value.testing.enabled,
        }
    });
    Ok(CanonicalConfigIdentity {
        schema_version: IDENTITY_SCHEMA_VERSION,
        type_aliases: ComponentDigest {
            algorithm: "sha256".to_string(),
            digest: format!("{:x}", Sha256::digest(alias_bytes)),
            item_count: config.type_aliases.len(),
        },
        angular,
        dotnet,
    })
}

fn semantic_config_identity(
    config: &CleanCtxConfig,
    applicability: &FrameworkApplicability,
) -> SemanticConfigIdentity {
    let angular = applicability.angular.then(|| {
        let value = framework_config(config, "angular");
        AngularSemanticConfigIdentity {
            enabled: value.enabled,
            ngrx_enabled: value.ngrx.enabled,
            routing_enabled: value.routing.enabled,
            testing_enabled: value.testing.enabled,
        }
    });
    let dotnet = applicability.dotnet.then(|| {
        let value = framework_config(config, "dotnet");
        DotNetSemanticConfigIdentity {
            enabled: value.enabled,
            testing_enabled: value.testing.enabled,
        }
    });
    SemanticConfigIdentity {
        schema_version: IDENTITY_SCHEMA_VERSION,
        angular,
        dotnet,
    }
}

fn framework_config(config: &CleanCtxConfig, name: &str) -> MetaLayerConfig {
    config.meta_layers.get(name).cloned().unwrap_or_default()
}

fn canonical_producer_identity(
    config: &CleanCtxConfig,
    language: LanguageKeys,
    applicability: &FrameworkApplicability,
    catalog: &GenerationCatalog,
) -> Result<CanonicalProducerIdentity, IdentityDerivationError> {
    let mut relevant_producers = BTreeMap::new();
    insert_required(
        &mut relevant_producers,
        ProducerKey::SharedCanonicalPipeline,
        catalog,
    )?;
    insert_required(&mut relevant_producers, language.canonical, catalog)?;
    if !config.type_aliases.is_empty() {
        insert_required(
            &mut relevant_producers,
            ProducerKey::TypeAliasTransform,
            catalog,
        )?;
    }
    insert_framework_producers(&mut relevant_producers, applicability, catalog, true)?;
    Ok(CanonicalProducerIdentity {
        schema_version: IDENTITY_SCHEMA_VERSION,
        relevant_producers,
    })
}

fn semantic_producer_identity(
    language: LanguageKeys,
    applicability: &FrameworkApplicability,
    catalog: &GenerationCatalog,
) -> Result<SemanticProducerIdentity, IdentityDerivationError> {
    let mut relevant_producers = BTreeMap::new();
    insert_required(&mut relevant_producers, language.semantic_input, catalog)?;
    if let Some(projection) = language.semantic_projection {
        insert_required(&mut relevant_producers, projection, catalog)?;
    }
    insert_required(
        &mut relevant_producers,
        ProducerKey::BuiltinSemantic,
        catalog,
    )?;
    insert_required(
        &mut relevant_producers,
        ProducerKey::GenericSemanticProjection,
        catalog,
    )?;
    insert_framework_producers(&mut relevant_producers, applicability, catalog, false)?;
    Ok(SemanticProducerIdentity {
        schema_version: IDENTITY_SCHEMA_VERSION,
        relevant_producers,
    })
}

fn insert_framework_producers(
    target: &mut BTreeMap<ProducerKey, u32>,
    applicability: &FrameworkApplicability,
    catalog: &GenerationCatalog,
    markers: bool,
) -> Result<(), IdentityDerivationError> {
    for (applies, marker, semantic) in [
        (
            applicability.angular,
            ProducerKey::AngularMarkers,
            ProducerKey::AngularSemantic,
        ),
        (
            applicability.dotnet,
            ProducerKey::DotNetMarkers,
            ProducerKey::DotNetSemantic,
        ),
        (
            applicability.spring,
            ProducerKey::SpringBootMarkers,
            ProducerKey::SpringBootSemantic,
        ),
    ] {
        if applies {
            let key = if markers { marker } else { semantic };
            if let Some(generation) = catalog.generation(key) {
                target.insert(key, generation);
            }
        }
    }
    Ok(())
}

fn insert_required(
    target: &mut BTreeMap<ProducerKey, u32>,
    key: ProducerKey,
    catalog: &GenerationCatalog,
) -> Result<(), IdentityDerivationError> {
    let generation = catalog
        .generation(key)
        .ok_or(IdentityDerivationError::MissingGeneration(key))?;
    target.insert(key, generation);
    Ok(())
}

#[cfg(test)]
#[path = "../../tests/mcp/durable_compatibility_identity.rs"]
mod tests;
