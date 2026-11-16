//! Produced written registrations, owned with the existing semantic generation.
use super::canonical_endpoint::{CanonicalDeclaration, RouteOccurrence};
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub(crate) struct ResolverRegistrations {
    pub format_version: u32,
    pub records: Vec<ResolverRegistration>,
    pub semantic_omissions: Vec<ResolverRegistrationOmission>,
}

impl Default for ResolverRegistrations {
    fn default() -> Self {
        Self {
            format_version: 1,
            records: Vec::new(),
            semantic_omissions: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub(crate) struct ResolverRegistration {
    pub route: RouteOccurrence,
    pub route_path: String,
    pub key: String,
    pub resolver: CanonicalDeclaration,
    pub evidence: ResolverMappingEvidence,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub(crate) struct ResolverMappingEvidence {
    pub source: PathBuf,
    pub source_hash: String,
    pub field_pair: (usize, usize),
    pub field_key: (usize, usize),
    pub object: (usize, usize),
    pub pair: (usize, usize),
    pub key: (usize, usize),
    pub value: (usize, usize),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ResolverOmissionReason {
    DuplicateResolveField,
    DuplicateMappingKey,
    UnsupportedMappingKey,
    UnsupportedMappingValue,
    UnresolvedMappingTarget,
    AmbiguousMappingTarget,
    TypeOnlyMappingTarget,
    MappingProofNotEstablished,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub(crate) struct ResolverRegistrationOmission {
    pub source: PathBuf,
    pub source_hash: String,
    pub start: usize,
    pub end: usize,
    pub reason: ResolverOmissionReason,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route: Option<RouteOccurrence>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolver: Option<CanonicalDeclaration>,
}

impl ResolverRegistrations {
    pub(crate) fn matches(&self, expected: &Self) -> bool {
        fn exact<T: Eq + std::hash::Hash>(actual: &[T], expected: &[T]) -> bool {
            let a: std::collections::HashSet<_> = actual.iter().collect();
            let e: std::collections::HashSet<_> = expected.iter().collect();
            a.len() == actual.len() && e.len() == expected.len() && a == e
        }
        self.format_version == 1
            && expected.format_version == 1
            && exact(&self.records, &expected.records)
            && exact(&self.semantic_omissions, &expected.semantic_omissions)
    }
}

pub(crate) fn enabled(config: &crate::config::CleanCtxConfig) -> bool {
    cfg!(all(feature = "angular", feature = "typescript"))
        && config
            .meta_layers
            .get("angular")
            .is_none_or(|layer| layer.enabled && layer.routing.enabled)
}
