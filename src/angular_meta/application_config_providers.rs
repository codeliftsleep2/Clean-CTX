//! Native ApplicationConfig declaration ownership; no application selection.
use crate::ir::typescript_resolution::declaration_facets::ExpressionValue;
use crate::ir::typescript_resolution::{Namespace, Resolution, TypeScriptAuthority};
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};

/// Native root nomination only. Fields and terminal membership are still replayed
/// by project/admission; no written type name or import text grants authority.
pub(crate) fn has_owner(authority: &TypeScriptAuthority) -> bool {
    authority.declaration_facets.iter().any(|facet| {
        super::typed_provider_root::admits(authority, facet, "@angular/core", "ApplicationConfig")
            && facet
                .initializer
                .as_ref()
                .is_some_and(|value| matches!(value.value, ExpressionValue::Object(_)))
    })
}

pub(crate) fn project(authority: &TypeScriptAuthority) -> Vec<SemanticEdge> {
    let mut edges = Vec::new();
    for facet in &authority.declaration_facets {
        if !super::typed_provider_root::admits(
            authority,
            facet,
            "@angular/core",
            "ApplicationConfig",
        ) {
            continue;
        }
        // A competing same-source binding cannot be resolved by display name.
        let mut bindings = authority
            .references
            .iter()
            .filter(|(name, ns, _)| name == &facet.declaration.name && *ns == Namespace::Value);
        let Some((_, _, Resolution::Resolved(id))) = bindings.next() else {
            continue;
        };
        if bindings.next().is_some() || id != &facet.declaration {
            continue;
        }
        let Some(object) = &facet.initializer else {
            continue;
        };
        let ExpressionValue::Object(fields) = &object.value else {
            continue;
        };
        if fields.iter().any(|(key, _)| key.is_empty()) {
            continue;
        }
        let mut providers = fields.iter().filter(|(key, _)| key == "providers");
        let Some((_, provider)) = providers.next() else {
            continue;
        };
        if providers.next().is_some()
            || provider.start < object.start
            || provider.start >= provider.end
            || provider.end > object.end
        {
            continue;
        }
        let subject = super::string_token_authority::owner(&facet.declaration, "ApplicationConfig");
        edges.extend(super::provider_imported::project_entries(
            authority,
            &facet.declaration,
            subject,
            provider,
            false,
            SemanticRelation::RegistersProvider,
            true,
        ));
    }
    edges
}

pub(crate) fn admits(authority: &TypeScriptAuthority, edge: &SemanticEdge) -> bool {
    let same = |a: &EntityRef, b: &EntityRef| {
        a.domain == b.domain
            && a.entity_type == b.entity_type
            && a.name == b.name
            && a.file == b.file
            && a.canonical == b.canonical
    };
    project(authority).iter().any(|expected| {
        expected.relation == edge.relation
            && expected.layer == edge.layer
            && same(&expected.subject, &edge.subject)
            && same(&expected.object, &edge.object)
    })
}
