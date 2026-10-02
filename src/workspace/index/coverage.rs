use super::WorkspaceIndex;
use super::{
    EntityKey, StoredEdge, entity_occurrence_admitted, identity_key, selected_occurrences,
};
use crate::layers::meta::semantic::SemanticEdge;
use crate::workspace::scope::WorkspaceScope;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SemanticFidelity {
    Low,
    Medium,
    High,
}

impl SemanticFidelity {
    pub fn for_request(fidelity: crate::compression::Fidelity) -> Self {
        use crate::compression::Fidelity;

        match fidelity {
            Fidelity::Low => Self::Low,
            Fidelity::Medium => Self::Medium,
            Fidelity::High | Fidelity::Edit | Fidelity::Verbatim => Self::High,
        }
    }

    pub fn from_compilation(fidelity: crate::compression::Fidelity) -> Option<Self> {
        use crate::compression::Fidelity;

        match fidelity {
            Fidelity::Low => Some(Self::Low),
            Fidelity::Medium => Some(Self::Medium),
            Fidelity::High => Some(Self::High),
            Fidelity::Edit | Fidelity::Verbatim => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SemanticCoverage {
    pub(super) fidelity: SemanticFidelity,
    pub(super) source_hash: String,
}

impl WorkspaceIndex {
    /// Whether this exact semantic identity has an admitted indexed occurrence.
    pub fn has_identity_in_scope(
        &self,
        domain: &str,
        entity_type: &str,
        name: &str,
        scope: Option<&WorkspaceScope>,
    ) -> bool {
        self.entities
            .get(&identity_key(domain, entity_type, name))
            .is_some_and(|occurrences| {
                occurrences
                    .iter()
                    .any(|occurrence| entity_occurrence_admitted(occurrence, scope))
            })
    }

    /// Whether the scoped index contains outgoing projection evidence for this
    /// domain/entity-type family. This establishes only indexed capability; it
    /// does not claim source completeness or define a global support matrix.
    pub fn has_forward_capability_evidence(
        &self,
        domain: &str,
        entity_type: &str,
        scope: Option<&WorkspaceScope>,
    ) -> bool {
        has_adjacency_evidence(&self.forward, domain, entity_type, scope)
    }

    /// Incoming counterpart of [`WorkspaceIndex::has_forward_capability_evidence`].
    pub fn has_reverse_capability_evidence(
        &self,
        domain: &str,
        entity_type: &str,
        scope: Option<&WorkspaceScope>,
    ) -> bool {
        has_adjacency_evidence(&self.reverse, domain, entity_type, scope)
    }

    pub fn has_current_semantic_projection(
        &self,
        file_path: &str,
        fidelity: SemanticFidelity,
        source_hash: &str,
    ) -> bool {
        self.semantic_coverage
            .get(file_path)
            .is_some_and(|coverage| {
                coverage.source_hash == source_hash && coverage.fidelity >= fidelity
            })
    }

    pub fn replace_semantic_projection(
        &mut self,
        file_path: &str,
        edges: Vec<SemanticEdge>,
        fidelity: SemanticFidelity,
        source_hash: String,
    ) {
        self.remove_file(file_path);
        self.add_edges(file_path, edges);
        self.semantic_coverage.insert(
            file_path.to_string(),
            SemanticCoverage {
                fidelity,
                source_hash,
            },
        );
    }
}

fn has_adjacency_evidence(
    adjacency: &HashMap<EntityKey, Vec<StoredEdge>>,
    domain: &str,
    entity_type: &str,
    scope: Option<&WorkspaceScope>,
) -> bool {
    adjacency.iter().any(|(key, _)| {
        key.0 == domain
            && key.1 == entity_type
            && !selected_occurrences(adjacency, key, scope).is_empty()
    })
}
