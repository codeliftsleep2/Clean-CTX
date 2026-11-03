//! Exact unqualified resolve written use. Key-specific public answers remain open.
use super::route_objects::{contains, unique};
use crate::ir::typescript_resolution::TypeScriptAuthority;
use crate::ir::typescript_resolution::declaration_facets::ExpressionValue;
use crate::layers::meta::semantic::{SemanticEdge, SemanticRelation};
use std::collections::BTreeSet;
#[cfg(test)]
pub(crate) mod replay_metrics;
pub(crate) fn project(a: &TypeScriptAuthority) -> Vec<SemanticEdge> {
    #[cfg(test)]
    super::admission_work_metrics::resolver();
    #[cfg(test)]
    replay_metrics::projection();
    let mut edges = Vec::new();
    for route in super::route_objects::project(a) {
        let Some(object) = unique(route.fields, "resolve") else {
            continue;
        };
        if !contains(route.object, object)
            || !a.admits_demanded_structure(&a.source, &a.source_hash, object)
        {
            continue;
        }
        let ExpressionValue::Object(fields) = &object.value else {
            continue;
        };
        // Unknown keys from spread/computed/shorthand structure cannot establish
        // independently unique named mappings. Ordinary refused values stay local.
        if fields.iter().any(|(name, _)| name.is_empty()) {
            continue;
        }
        let mut targets = BTreeSet::new();
        for (name, value) in fields {
            if fields.iter().filter(|(n, _)| n == name).count() != 1 || !contains(object, value) {
                continue;
            }
            let candidates = a
                .resolve_mappings
                .iter()
                .filter(|m| {
                    m.source == a.source
                        && m.source_hash == a.source_hash
                        && m.owner == (route.object.start, route.object.end)
                        && m.object == (object.start, object.end)
                        && m.value == (value.start, value.end)
                        && m.name == *name
                })
                .collect::<Vec<_>>();
            if candidates.len() != 1 {
                continue;
            }
            let Some((_, id)) = a.resolve_mapping_at(candidates[0].pair) else {
                continue;
            };
            let Ok(span) = crate::ir::invocation_evidence::SourceSpan::new(value.start, value.end)
            else {
                continue;
            };
            // resolve_mapping_at already verified the exact demanded value/id.
            // Preserve the accessor's extra crossed-hash same-value refusal
            // without invoking the entire current mapping proof a second time.
            if a.resolve_mappings
                .iter()
                .filter(|m| m.source == a.source && m.value == (value.start, value.end))
                .count()
                != 1
                || a.value_reference_at(span) != Some(id)
                || !targets.insert(id.clone())
            {
                continue;
            }
            edges.push(SemanticEdge {
                subject: route.subject(a),
                object: super::string_token_authority::owner(id, "Resolver"),
                relation: SemanticRelation::ResolvedBy,
                layer: "angular",
                call_evidence: None,
            });
        }
    }
    edges
}
/// Compare only with the complete projection owned by this admission call.
pub(crate) fn admits_projected(projected: &[SemanticEdge], edge: &SemanticEdge) -> bool {
    let same = |x: &crate::layers::meta::semantic::EntityRef,
                y: &crate::layers::meta::semantic::EntityRef| {
        x.domain == y.domain
            && x.entity_type == y.entity_type
            && x.name == y.name
            && x.file == y.file
            && x.canonical == y.canonical
    };
    edge.relation == SemanticRelation::ResolvedBy
        && edge.call_evidence.is_none()
        && projected.iter().any(|expected| {
            edge.layer == expected.layer
                && same(&edge.subject, &expected.subject)
                && same(&edge.object, &expected.object)
        })
}
/// Field ownership is independent of duplicate/value proof; no owned downgrade.
pub(crate) fn owned_objects(a: &TypeScriptAuthority) -> Vec<(usize, usize)> {
    super::route_objects::project(a)
        .iter()
        .filter(|r| r.fields.iter().any(|(n, _)| n == "resolve"))
        .map(|r| (r.object.start, r.object.end))
        .collect()
}
