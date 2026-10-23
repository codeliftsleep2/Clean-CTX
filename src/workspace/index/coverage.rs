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

#[derive(Debug, Clone)]
pub(super) struct SemanticCoverage {
    #[cfg(feature = "typescript")]
    pub(super) typescript_authority: Option<crate::ir::typescript_resolution::TypeScriptAuthority>,
    pub(super) fidelity: SemanticFidelity,
    pub(super) source_hash: String,
    pub(super) dependencies: Vec<crate::layers::meta::semantic::SemanticDependency>,
}

impl WorkspaceIndex {
    /// Known owners are revalidation candidates, never discovery/completeness
    /// proof. Cached discovery cannot certify their consumed TS dependencies.
    #[cfg(feature = "typescript")]
    pub(crate) fn retained_typescript_owners_in_roots(
        &self,
        roots: &[std::path::PathBuf],
    ) -> Vec<String> {
        self.semantic_coverage
            .iter()
            .filter(|(owner, coverage)| {
                coverage.typescript_authority.is_some()
                    && roots.iter().any(|root| {
                        crate::workspace::path_identity::is_within_root(
                            owner,
                            &root.to_string_lossy(),
                        )
                    })
            })
            .map(|(owner, _)| owner.clone())
            .collect()
    }

    #[cfg(all(test, feature = "typescript"))]
    pub(crate) fn retained_typescript_authority(
        &self,
        file: &str,
    ) -> Option<&crate::ir::typescript_resolution::TypeScriptAuthority> {
        let coverage = self.semantic_coverage.get(file)?;
        if !self.has_current_semantic_projection(file, coverage.fidelity, &coverage.source_hash) {
            return None;
        }
        coverage.typescript_authority.as_ref()
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn replace_semantic_projection_with_retained_authority(
        &mut self,
        file: &str,
        edges: Vec<SemanticEdge>,
        fidelity: SemanticFidelity,
        source_hash: String,
        dependencies: Vec<crate::layers::meta::semantic::SemanticDependency>,
        #[cfg(feature = "typescript")] authority: Option<
            crate::ir::typescript_resolution::TypeScriptAuthority,
        >,
    ) -> Result<(), &'static str> {
        #[cfg(feature = "typescript")]
        if let Some(proof) = &authority
            && (proof.source_hash != source_hash
                || !proof.capture_current
                || proof.disposition.is_some()
                || !proof.witnesses.iter().all(|witness| witness.current()))
        {
            return Err("TypeScript authority changed before index publication");
        }
        #[cfg(feature = "typescript")]
        let relation_evidence_admitted = authority.as_ref().map_or_else(
            || {
                edges
                    .iter()
                    .all(|edge| edge.subject.canonical.is_none() && edge.object.canonical.is_none())
            },
            |proof| proof.admits_relation_evidence(&edges),
        );
        #[cfg(not(feature = "typescript"))]
        let relation_evidence_admitted = edges
            .iter()
            .all(|edge| edge.subject.canonical.is_none() && edge.object.canonical.is_none());
        if !relation_evidence_admitted {
            return Err("Canonical relation evidence has no matching retained authority");
        }
        self.replace_semantic_projection_with_dependencies(
            file,
            edges,
            fidelity,
            source_hash,
            dependencies,
        );
        #[cfg(feature = "typescript")]
        {
            self.semantic_coverage
                .get_mut(file)
                .expect("inserted source coverage")
                .typescript_authority = authority;
        }
        Ok(())
    }

    #[cfg(feature = "angular")]
    pub(crate) fn semantic_dependencies_for_file(
        &self,
        file: &str,
    ) -> Vec<crate::layers::meta::semantic::SemanticDependency> {
        self.semantic_coverage
            .get(file)
            .map(|coverage| coverage.dependencies.clone())
            .unwrap_or_default()
    }

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
                #[cfg(feature = "typescript")]
                if let Some(authority) = &coverage.typescript_authority
                    && (authority.source_hash != source_hash
                        || !authority.capture_current
                        || authority.disposition.is_some()
                        || !authority.witnesses.iter().all(|witness| witness.current()))
                {
                    return false;
                }
                coverage.source_hash == source_hash
                    && coverage.fidelity >= fidelity
                    && coverage
                        .dependencies
                        .iter()
                        .all(|dependency| dependency.is_current())
            })
    }

    pub(crate) fn semantic_projection_fidelity(&self, file_path: &str) -> Option<SemanticFidelity> {
        self.semantic_coverage
            .get(file_path)
            .map(|coverage| coverage.fidelity)
    }

    #[cfg(test)]
    pub(crate) fn semantic_dependency_count(&self, file_path: &str) -> usize {
        self.semantic_coverage
            .get(file_path)
            .map_or(0, |coverage| coverage.dependencies.len())
    }

    pub fn replace_semantic_projection(
        &mut self,
        file_path: &str,
        edges: Vec<SemanticEdge>,
        fidelity: SemanticFidelity,
        source_hash: String,
    ) {
        self.replace_semantic_projection_with_dependencies(
            file_path,
            edges,
            fidelity,
            source_hash,
            Vec::new(),
        );
    }

    pub fn replace_semantic_projection_with_dependencies(
        &mut self,
        file_path: &str,
        edges: Vec<SemanticEdge>,
        fidelity: SemanticFidelity,
        source_hash: String,
        dependencies: Vec<crate::layers::meta::semantic::SemanticDependency>,
    ) {
        self.remove_file(file_path);
        self.add_edges(file_path, edges);
        self.semantic_coverage.insert(
            file_path.to_string(),
            SemanticCoverage {
                #[cfg(feature = "typescript")]
                typescript_authority: None,
                fidelity,
                source_hash,
                dependencies,
            },
        );
    }

    pub fn semantic_dependents_of_root(&self, root: &str) -> Vec<String> {
        self.semantic_coverage
            .iter()
            .filter(|(owner, coverage)| {
                !crate::workspace::path_identity::is_within_root(owner, root)
                    && coverage.dependencies.iter().any(|dependency| {
                        crate::workspace::path_identity::is_within_root(
                            &crate::dictionary::path::canonical_identity_key(&dependency.file_path),
                            root,
                        )
                    })
            })
            .map(|(owner, _)| owner.clone())
            .collect()
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
