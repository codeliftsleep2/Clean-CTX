//! Typed payload carried with an existing source-owned semantic generation.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub(crate) struct RetainedAuthority {
    // None: unbound scope; Some(false): checked outside this producer scope;
    // Some(true): the actual configured evaluation requires the collection.
    // Rebind after decoding; never infer this from a conservative producer hint.
    #[serde(skip)]
    pub resolver_registration_scope: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolver_registrations:
        Option<crate::layers::meta::resolver_registrations::ResolverRegistrations>,
    #[cfg(feature = "typescript")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub typescript_authority: Option<crate::ir::typescript_resolution::TypeScriptAuthority>,
}

impl RetainedAuthority {
    /// Existing subset relationship API; no complete registration requirement.
    pub(crate) fn admits_relation_evidence(
        &self,
        edges: &[crate::layers::meta::semantic::SemanticEdge],
    ) -> bool {
        #[cfg(feature = "typescript")]
        if let Some(authority) = &self.typescript_authority {
            return authority.admits_relation_evidence(edges);
        }
        edges.iter().all(|edge| {
            edge.subject.entity_type != "ApplicationBootstrap"
                && !matches!(
                    edge.relation,
                    crate::layers::meta::semantic::SemanticRelation::DeclaresBootstrapRoot
                    | crate::layers::meta::semantic::SemanticRelation::DeclaresBootstrapModule
                        | crate::layers::meta::semantic::SemanticRelation::UsesApplicationConfig
                        | crate::layers::meta::semantic::SemanticRelation::IncludesApplicationConfig
                )
                && edge.relation
                    != crate::layers::meta::semantic::SemanticRelation::ProviderUsesExisting
                && edge.relation
                    != crate::layers::meta::semantic::SemanticRelation::RegistersViewProvider
                && edge.subject.canonical.is_none()
                && edge.object.canonical.is_none()
        })
    }

    /// Complete produced payload admission, with applicability from current config.
    pub(crate) fn admits_semantic_projection(
        &self,
        edges: &[crate::layers::meta::semantic::SemanticEdge],
        registrations_enabled: bool,
    ) -> bool {
        if self.resolver_registrations.is_some() && self.resolver_registration_scope != Some(true) {
            return false;
        }
        if !registrations_enabled {
            return self.resolver_registration_scope != Some(true)
                && self.resolver_registrations.is_none()
                && self.admits_relation_evidence(edges);
        }
        if self.resolver_registration_scope == Some(true) && self.resolver_registrations.is_none() {
            return false;
        }
        #[cfg(feature = "typescript")]
        if let Some(authority) = &self.typescript_authority {
            return authority.admits_registration_projection(
                edges,
                self.resolver_registrations.as_ref(),
                self.resolver_registration_scope == Some(true),
            );
        }
        self.resolver_registration_scope != Some(true)
            && self.resolver_registrations.is_none()
            && self.admits_relation_evidence(edges)
    }

    pub(crate) fn is_current(&self, source_hash: &str) -> bool {
        #[cfg(feature = "typescript")]
        if let Some(authority) = &self.typescript_authority {
            return authority.source_hash == source_hash
                && authority.capture_current
                && authority.disposition.is_none()
                && authority.witnesses.iter().all(|witness| witness.current());
        }
        let _ = source_hash;
        true
    }
}
