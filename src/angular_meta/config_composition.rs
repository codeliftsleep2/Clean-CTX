//! Written native merge inputs; no provider flattening, merge order or runtime values.
use crate::ir::typescript_resolution::declaration_facets::{
    DeclarationFacet, ExpressionFacet, ExpressionValue,
};
use crate::ir::typescript_resolution::{Namespace, Resolution, TypeScriptAuthority};
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};

pub(super) fn result_args<'a>(
    authority: &TypeScriptAuthority,
    facet: &'a DeclarationFacet,
) -> Option<&'a [ExpressionFacet]> {
    if !super::typed_provider_root::admits_qualified(
        authority,
        facet,
        "@angular/core",
        "ApplicationConfig",
    ) {
        return None;
    }
    let initializer = facet.initializer.as_ref()?;
    let ExpressionValue::Call(callee, args) = &initializer.value else {
        return None;
    };
    let ExpressionValue::Reference(Resolution::Resolved(api)) = &callee.value else {
        return None;
    };
    let id = &facet.declaration;
    if authority.demanded_reference_at(&id.source, &id.source_hash, callee.start, callee.end)
        != Some(api)
        || !super::class_role_authority::public_entry_api(
            authority,
            api,
            "@angular/core",
            "@angular/core",
            "mergeApplicationConfig",
            Namespace::Value,
        )
    {
        return None;
    }
    Some(args)
}
pub(crate) fn has_owner(authority: &TypeScriptAuthority) -> bool {
    authority.declaration_facets.iter().any(|facet| {
        facet.declaration.source == authority.source && result_args(authority, facet).is_some()
    })
}
pub(crate) fn project(authority: &TypeScriptAuthority) -> Vec<SemanticEdge> {
    let mut edges = Vec::new();
    for facet in &authority.declaration_facets {
        let id = &facet.declaration;
        if id.source != authority.source
            || id.source_hash != authority.source_hash
            || !authority.declarations.contains(id)
        {
            continue;
        }
        let mut bindings = authority
            .references
            .iter()
            .filter(|(name, namespace, _)| name == &id.name && *namespace == Namespace::Value);
        if !matches!(bindings.next(),Some((_,_,Resolution::Resolved(actual))) if actual==id)
            || bindings.next().is_some()
        {
            continue;
        }
        let Some(args) = result_args(authority, facet) else {
            continue;
        };
        for arg in args {
            let ExpressionValue::Reference(Resolution::Resolved(input)) = &arg.value else {
                continue;
            };
            if authority.demanded_reference_at(&id.source, &id.source_hash, arg.start, arg.end)
                != Some(input)
            {
                continue;
            }
            let Some(leaf) = authority
                .declaration_facets
                .iter()
                .find(|f| f.declaration == *input)
            else {
                continue;
            };
            if !super::typed_provider_root::admits_qualified(
                authority,
                leaf,
                "@angular/core",
                "ApplicationConfig",
            ) || !(leaf
                .initializer
                .as_ref()
                .is_some_and(|v| matches!(v.value, ExpressionValue::Object(_)))
                || result_args(authority, leaf).is_some())
            {
                continue;
            }
            let edge = SemanticEdge {
                subject: super::string_token_authority::owner(id, "ApplicationConfig"),
                object: super::string_token_authority::owner(input, "ApplicationConfig"),
                relation: SemanticRelation::IncludesApplicationConfig,
                layer: "angular",
                call_evidence: None,
            };
            // Public pair semantics; all original argument facets remain private.
            if !edges.iter().any(|e: &SemanticEdge| {
                e.subject.canonical == edge.subject.canonical
                    && e.object.canonical == edge.object.canonical
            }) {
                edges.push(edge);
            }
        }
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
    edge.call_evidence.is_none()
        && project(authority).iter().any(|expected| {
            expected.relation == edge.relation
                && expected.layer == edge.layer
                && same(&expected.subject, &edge.subject)
                && same(&expected.object, &edge.object)
        })
}
