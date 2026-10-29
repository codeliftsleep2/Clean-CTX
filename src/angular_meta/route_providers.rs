//! Exact native Routes-object provider membership through shared ownership.
use super::route_objects::{contains, unique};
use crate::ir::typescript_resolution::TypeScriptAuthority;
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};

pub(crate) fn project(authority: &TypeScriptAuthority) -> Vec<SemanticEdge> {
    super::route_objects::project(authority)
        .into_iter()
        .flat_map(|route| {
            unique(route.fields, "providers")
                .filter(|value| contains(route.object, value))
                .map(|providers| {
                    super::provider_imported::project_entries(
                        authority,
                        &route.root.declaration,
                        route.subject(authority),
                        providers,
                        false,
                        SemanticRelation::RegistersProvider,
                        true,
                    )
                })
                .unwrap_or_default()
        })
        .collect()
}

/// Reproduce the exact root/object/field/entry join, including subject identity.
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
