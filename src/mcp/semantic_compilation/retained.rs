//! Typed payload carried with an existing source-owned semantic generation.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub(crate) struct RetainedAuthority {
    #[cfg(feature = "typescript")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub typescript_authority: Option<crate::ir::typescript_resolution::TypeScriptAuthority>,
}

impl RetainedAuthority {
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
