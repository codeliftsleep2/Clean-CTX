//! Exact field-specific guard written use, not callable or active applicability.
use super::route_objects::{contains, unique};
use crate::ir::typescript_resolution::declaration_facets::ExpressionValue;
use crate::ir::typescript_resolution::{Resolution, TypeScriptAuthority, Witness};
use crate::layers::meta::semantic::{SemanticEdge, SemanticRelation};
use std::collections::BTreeSet;

pub(crate) const KINDS: [(&str, SemanticRelation); 4] = [
    ("canMatch", SemanticRelation::GuardedByCanMatch),
    ("canDeactivate", SemanticRelation::GuardedByCanDeactivate),
    ("canLoad", SemanticRelation::GuardedByCanLoad),
    (
        "canActivateChild",
        SemanticRelation::GuardedByCanActivateChild,
    ),
];

pub(crate) fn project(authority: &TypeScriptAuthority) -> Vec<SemanticEdge> {
    memberships(authority, &[("canActivate", SemanticRelation::GuardedBy)])
}
pub(crate) fn project_kinds(authority: &TypeScriptAuthority) -> Vec<SemanticEdge> {
    memberships(authority, &KINDS)
}
fn memberships(
    authority: &TypeScriptAuthority,
    kinds: &[(&str, SemanticRelation)],
) -> Vec<SemanticEdge> {
    #[cfg(test)]
    for &(field, _) in kinds {
        use super::admission_work_metrics::{Consumer, projection};
        projection(match field {
            "canActivate" => Consumer::Activate,
            "canMatch" => Consumer::Match,
            "canDeactivate" => Consumer::Deactivate,
            "canLoad" => Consumer::Load,
            "canActivateChild" => Consumer::ActivateChild,
            _ => unreachable!("fixed explicit guard kinds"),
        });
    }
    let mut edges = Vec::new();
    for route in super::route_objects::project(authority) {
        for &(field, relation) in kinds {
            let Some(array) = unique(route.fields, field) else {
                continue;
            };
            if !contains(route.object, array)
                || !authority.admits_demanded_structure(
                    &authority.source,
                    &authority.source_hash,
                    array,
                )
            {
                continue;
            }
            let ExpressionValue::Array(slots) = &array.value else {
                continue;
            };
            let mut targets = BTreeSet::new();
            for slot in slots {
                if !contains(array, slot) {
                    continue;
                }
                let ExpressionValue::Reference(Resolution::Resolved(id)) = &slot.value else {
                    continue;
                };
                let Ok(span) =
                    crate::ir::invocation_evidence::SourceSpan::new(slot.start, slot.end)
                else {
                    continue;
                };
                if authority.demanded_reference_at(&authority.source, &authority.source_hash, slot.start, slot.end) != Some(id)
                || authority.value_reference_at(span) != Some(id)
                || id.start >= id.end || id.source_hash.is_empty()
                || !(id.source == authority.source && id.source_hash == authority.source_hash
                    || authority.witnesses.iter().any(|w| matches!(w, Witness::Content { path, hash } if path == &id.source && hash == &id.source_hash)))
                || !targets.insert(id.clone())
            { continue; }
                edges.push(SemanticEdge {
                    subject: route.subject(authority),
                    object: super::string_token_authority::owner(id, "Guard"),
                    relation,
                    layer: "angular",
                    call_evidence: None,
                });
            }
        }
    }
    edges
}

pub(crate) fn admits_projected(projected: &[SemanticEdge], edge: &SemanticEdge) -> bool {
    let same = |a: &crate::layers::meta::semantic::EntityRef,
                b: &crate::layers::meta::semantic::EntityRef| {
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

/// Ownership is independent of field uniqueness, container or target proof.
pub(crate) fn owned_objects(authority: &TypeScriptAuthority) -> Vec<(usize, usize)> {
    super::route_objects::project(authority)
        .iter()
        .filter(|route| route.fields.iter().any(|(name, _)| name == "canActivate"))
        .map(|route| (route.object.start, route.object.end))
        .collect()
}

/// Exact per-field ownership prevents fallback even for duplicate/refused mappings.
pub(crate) fn owned_kinds(authority: &TypeScriptAuthority) -> Vec<((usize, usize), &'static str)> {
    let mut owned = Vec::new();
    for route in super::route_objects::project(authority) {
        for &(field, _) in &KINDS {
            if route.fields.iter().any(|(name, _)| name == field) {
                owned.push(((route.object.start, route.object.end), field));
            }
        }
    }
    owned
}

/// Complete membership for exactly the requested explicit guard relation.
pub(crate) fn project_relation(
    authority: &TypeScriptAuthority,
    relation: SemanticRelation,
) -> Vec<SemanticEdge> {
    let field = if relation == SemanticRelation::GuardedBy {
        "canActivate"
    } else if let Some((field, _)) = KINDS.iter().find(|(_, candidate)| *candidate == relation) {
        *field
    } else {
        return Vec::new();
    };
    memberships(authority, &[(field, relation)])
}
