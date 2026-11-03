//! Written lazy selection from exact shared native route-object ownership.
use super::route_objects::{contains, unique};
use crate::ir::typescript_resolution::TypeScriptAuthority;
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};
/// Framework roles remain distinct even though neutral selection is shared.
fn target_role(
    authority: &TypeScriptAuthority,
    target: &crate::ir::typescript_resolution::DeclarationId,
) -> Option<&'static str> {
    let module = super::bootstrap_references::native_class(
        authority,
        target,
        super::decorators::ClassKind::Module,
    );
    let mut facets = authority
        .declaration_facets
        .iter()
        .filter(|facet| facet.declaration == *target);
    let routes = facets.next().is_some_and(|facet| {
        facets.next().is_none()
            && matches!(
                facet.initializer.as_ref().map(|v| &v.value),
                Some(
                    crate::ir::typescript_resolution::declaration_facets::ExpressionValue::Array(_)
                )
            )
            && super::typed_provider_root::admits_qualified(
                authority,
                facet,
                "@angular/router",
                "Routes",
            )
    });
    match (module, routes) {
        (true, false) => Some("Module"),
        (false, true) => Some("RouteConfig"),
        _ => None,
    }
}
pub(crate) fn project(authority: &TypeScriptAuthority) -> Vec<SemanticEdge> {
    #[cfg(test)]
    super::admission_work_metrics::projection(super::admission_work_metrics::Consumer::Children);
    super::route_objects::project(authority)
        .into_iter()
        .filter_map(|route| {
            let value = unique(route.fields, "loadChildren")?;
            if !contains(route.object, value) {
                return None;
            }
            let selection = authority.arrow_selection_at((value.start, value.end))?;
            let role = target_role(authority, &selection.target)?;
            Some(SemanticEdge {
                subject: route.subject(authority),
                object: super::string_token_authority::owner(&selection.target, role),
                relation: SemanticRelation::LoadsChildren,
                layer: "angular",
                call_evidence: None,
            })
        })
        .collect()
}
pub(crate) fn admits_projected(projected: &[SemanticEdge], edge: &SemanticEdge) -> bool {
    let same = |a: &EntityRef, b: &EntityRef| {
        a.domain == b.domain
            && a.entity_type == b.entity_type
            && a.name == b.name
            && a.file == b.file
            && a.canonical == b.canonical
    };
    projected.iter().any(|expected| {
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
        .filter(|route| route.fields.iter().any(|(name, _)| name == "loadChildren"))
        .map(|route| (route.object.start, route.object.end))
        .collect()
}
