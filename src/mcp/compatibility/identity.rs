//! Durable compatibility values. These describe authority provenance but do
//! not by themselves authorize adoption of persisted state.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub(crate) const IDENTITY_SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ComponentDigest {
    pub algorithm: String,
    pub digest: String,
    pub item_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct AngularMarkerConfigIdentity {
    pub enabled: bool,
    pub rxjs_enabled: bool,
    pub rxjs_min_pipe_operators: usize,
    pub ngrx_enabled: bool,
    pub ngrx_include_dispatch_sites: bool,
    pub ngrx_include_select_sites: bool,
    pub ngrx_entity_selectors: bool,
    pub signals_enabled: bool,
    pub reactive_forms_enabled: bool,
    pub formly_enabled: bool,
    pub routing_enabled: bool,
    pub testing_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct DotNetMarkerConfigIdentity {
    pub enabled: bool,
    pub testing_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct CanonicalConfigIdentity {
    pub schema_version: u16,
    pub type_aliases: ComponentDigest,
    pub angular: Option<AngularMarkerConfigIdentity>,
    pub dotnet: Option<DotNetMarkerConfigIdentity>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct AngularSemanticConfigIdentity {
    pub enabled: bool,
    pub ngrx_enabled: bool,
    pub routing_enabled: bool,
    pub testing_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct DotNetSemanticConfigIdentity {
    pub enabled: bool,
    pub testing_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SemanticConfigIdentity {
    pub schema_version: u16,
    pub angular: Option<AngularSemanticConfigIdentity>,
    pub dotnet: Option<DotNetSemanticConfigIdentity>,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ProducerKey {
    SharedCanonicalPipeline,
    TypeScriptCanonical,
    CSharpCanonical,
    RustCanonical,
    JavaCanonical,
    TypeAliasTransform,
    AngularMarkers,
    DotNetMarkers,
    SpringBootMarkers,
    TypeScriptSemanticInput,
    CSharpSemanticInput,
    RustSemanticInput,
    JavaSemanticInput,
    BuiltinSemantic,
    GenericSemanticProjection,
    AngularSemantic,
    DotNetSemantic,
    SpringBootSemantic,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct CanonicalProducerIdentity {
    pub schema_version: u16,
    pub relevant_producers: BTreeMap<ProducerKey, u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SemanticProducerIdentity {
    pub schema_version: u16,
    pub relevant_producers: BTreeMap<ProducerKey, u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CompatibilityIdentities {
    pub canonical_config: CanonicalConfigIdentity,
    pub canonical_producers: CanonicalProducerIdentity,
    pub semantic_config: SemanticConfigIdentity,
    pub semantic_producers: SemanticProducerIdentity,
}

impl CanonicalProducerIdentity {
    pub(crate) fn differs_bidirectionally(&self, other: &Self) -> bool {
        self.relevant_producers != other.relevant_producers
    }
}

impl SemanticProducerIdentity {
    pub(crate) fn differs_bidirectionally(&self, other: &Self) -> bool {
        self.relevant_producers != other.relevant_producers
    }
}
