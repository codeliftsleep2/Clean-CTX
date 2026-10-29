//! Exact written eager membership; shared route ownership, no activation meaning.
use super::route_objects::{contains, unique};
use crate::ir::typescript_resolution::declaration_facets::ExpressionValue;
use crate::ir::typescript_resolution::{Resolution, TypeScriptAuthority};
use crate::layers::meta::semantic::{SemanticEdge, SemanticRelation};

pub(crate) fn project(authority: &TypeScriptAuthority) -> Vec<SemanticEdge> {
    super::route_objects::project(authority)
        .into_iter()
        .filter_map(|route| {
            let value = unique(route.fields, "component")?;
            if !contains(route.object, value) {
                return None;
            }
            let ExpressionValue::Reference(Resolution::Resolved(id)) = &value.value else {
                return None;
            };
            let span =
                crate::ir::invocation_evidence::SourceSpan::new(value.start, value.end).ok()?;
            if authority.demanded_reference_at(
                &authority.source,
                &authority.source_hash,
                value.start,
                value.end,
            ) != Some(id)
                || authority.value_reference_at(span) != Some(id)
                || !super::bootstrap_references::native_class(
                    authority,
                    id,
                    super::decorators::ClassKind::Component,
                )
            {
                return None;
            }
            Some(SemanticEdge {
                subject: route.subject(authority),
                object: super::string_token_authority::owner(id, "Component"),
                relation: SemanticRelation::RouteMapsTo,
                layer: "angular",
                call_evidence: None,
            })
        })
        .collect()
}

pub(crate) fn admits(authority: &TypeScriptAuthority, edge: &SemanticEdge) -> bool {
    let same = |a: &crate::layers::meta::semantic::EntityRef,
                b: &crate::layers::meta::semantic::EntityRef| {
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

/// Ownership, independent of target eligibility, prevents weak fallback on refusal.
pub(crate) fn owned_objects(authority: &TypeScriptAuthority) -> Vec<(usize, usize)> {
    super::route_objects::project(authority)
        .iter()
        .map(|route| (route.object.start, route.object.end))
        .collect()
}
