//! Registration facts share the existing validated source coverage owner.
use super::{SemanticFidelity, WorkspaceIndex};
use crate::layers::meta::resolver_registrations::ResolverRegistrations;
use crate::layers::meta::semantic::{SemanticDependency, SemanticEdge};

/// Owned publication input, not a second store or an admitted coverage entry.
/// Keep the workspace boundary independent of MCP/session snapshot types.
pub(crate) struct SemanticProjectionInput {
    pub edges: Vec<SemanticEdge>,
    pub fidelity: SemanticFidelity,
    pub source_hash: String,
    pub dependencies: Vec<SemanticDependency>,
    #[cfg(feature = "typescript")]
    pub typescript_authority: Option<crate::ir::typescript_resolution::TypeScriptAuthority>,
    pub resolver_registrations: Option<ResolverRegistrations>,
    pub resolver_registration_scope: Option<bool>,
}

impl WorkspaceIndex {
    pub(crate) fn replace_semantic_projection_with_complete_authority(
        &mut self,
        file: &str,
        input: SemanticProjectionInput,
        registrations_enabled: bool,
    ) -> Result<(), &'static str> {
        let SemanticProjectionInput {
            edges,
            fidelity,
            source_hash,
            dependencies,
            resolver_registrations,
            resolver_registration_scope,
            #[cfg(feature = "typescript")]
            typescript_authority,
        } = input;
        if resolver_registrations.is_some() && resolver_registration_scope != Some(true) {
            return Err("Registration production scope is absent or inapplicable");
        }
        if resolver_registration_scope == Some(true)
            && (!registrations_enabled || resolver_registrations.is_none())
        {
            return Err("Required registration projection is missing or disabled");
        }
        if !registrations_enabled && resolver_registrations.is_none() {
            return self.replace_semantic_projection_with_retained_authority(
                file,
                edges,
                fidelity,
                source_hash,
                dependencies,
                #[cfg(feature = "typescript")]
                typescript_authority,
            );
        }
        if !registrations_enabled && resolver_registrations.is_some() {
            return Err("Registration production is disabled");
        }
        #[cfg(feature = "typescript")]
        let admitted = if let Some(proof) = &typescript_authority {
            proof.source_hash == source_hash
                && proof.capture_current
                && proof.disposition.is_none()
                && crate::dictionary::path::canonical_identity_key(&proof.source.to_string_lossy())
                    == crate::dictionary::path::canonical_identity_key(file)
                && proof.witnesses.iter().all(|witness| witness.current())
                && proof.admits_registration_projection(
                    &edges,
                    resolver_registrations.as_ref(),
                    resolver_registration_scope == Some(true),
                )
        } else {
            resolver_registrations.is_none()
                && edges
                    .iter()
                    .all(|edge| edge.subject.canonical.is_none() && edge.object.canonical.is_none())
        };
        #[cfg(not(feature = "typescript"))]
        let admitted = resolver_registrations.is_none()
            && edges
                .iter()
                .all(|edge| edge.subject.canonical.is_none() && edge.object.canonical.is_none());
        if !admitted {
            return Err("Complete source-owned semantic authority is not admitted");
        }
        self.replace_semantic_projection_with_dependencies(
            file,
            edges,
            fidelity,
            source_hash,
            dependencies,
        );
        let coverage = self
            .semantic_coverage
            .get_mut(file)
            .expect("inserted coverage");
        coverage.resolver_registration_admitted = false;
        coverage.resolver_registrations = resolver_registrations;
        #[cfg(feature = "typescript")]
        {
            coverage.resolver_registration_admitted = registrations_enabled
                && typescript_authority.is_some()
                && resolver_registration_scope.is_some();
            coverage.typescript_authority = typescript_authority;
        }
        Ok(())
    }

    /// Callers validate the complete consumed-owner footprint at final evaluation.
    #[cfg(feature = "angular")]
    pub(crate) fn resolver_registrations(&self, file: &str) -> Option<&ResolverRegistrations> {
        self.semantic_coverage
            .get(file)?
            .resolver_registrations
            .as_ref()
    }

    #[cfg(feature = "angular")]
    pub(crate) fn has_resolver_registration_admission(&self, file: &str) -> bool {
        self.semantic_coverage
            .get(file)
            .is_some_and(|coverage| coverage.resolver_registration_admitted)
    }
}
