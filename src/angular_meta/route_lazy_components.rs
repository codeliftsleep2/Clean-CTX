//! Written lazy selection from exact shared native route-object ownership.
use super::route_objects::{contains, unique};
use crate::ir::typescript_resolution::TypeScriptAuthority;
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};
pub(crate) fn project(authority: &TypeScriptAuthority) -> Vec<SemanticEdge> {
    super::route_objects::project(authority)
        .into_iter()
        .filter_map(|route| {
            let value = unique(route.fields, "loadComponent")?;
            if !contains(route.object, value) {
                return None;
            }
            let selection = authority.arrow_selection_at((value.start, value.end))?;
            if !super::bootstrap_references::native_class(
                authority,
                &selection.target,
                super::decorators::ClassKind::Component,
            ) {
                return None;
            }
            Some(SemanticEdge {
                subject: route.subject(authority),
                object: super::string_token_authority::owner(&selection.target, "Component"),
                relation: SemanticRelation::RouteMapsTo,
                layer: "angular",
                call_evidence: None,
            })
        })
        .collect()
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
            && edge.call_evidence.is_none()
    })
}
/// Ownership precedes proof. Refused/duplicate/unsupported owned fields never
/// fall back to the legacy spelling/file producer.
pub(crate) fn owned_objects(authority: &TypeScriptAuthority) -> Vec<(usize, usize)> {
    super::route_objects::project(authority)
        .iter()
        .filter(|route| route.fields.iter().any(|(name, _)| name == "loadComponent"))
        .map(|route| (route.object.start, route.object.end))
        .collect()
}
