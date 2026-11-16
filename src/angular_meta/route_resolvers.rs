//! Exact written membership and key registrations; no executable eligibility.
use super::route_objects::{contains, unique};
use crate::ir::typescript_resolution::TypeScriptAuthority;
use crate::ir::typescript_resolution::declaration_facets::ExpressionValue;
use crate::layers::meta::resolver_registrations::{
    ResolverMappingEvidence, ResolverOmissionReason, ResolverRegistration,
    ResolverRegistrationOmission, ResolverRegistrations,
};
use crate::layers::meta::semantic::{SemanticEdge, SemanticRelation};
use std::collections::BTreeSet;
#[cfg(test)]
pub(crate) mod replay_metrics;
#[derive(Default)]
pub(crate) struct ResolverProjection {
    pub edges: Vec<SemanticEdge>,
    pub registrations: ResolverRegistrations,
}

#[cfg(test)]
pub(crate) fn project(a: &TypeScriptAuthority) -> Vec<SemanticEdge> {
    project_with_registrations(a).edges
}

/// One compilation/admission-local traversal produces both written projections.
/// No result is retained across source, witness or publication generations.
pub(crate) fn project_with_registrations(a: &TypeScriptAuthority) -> ResolverProjection {
    #[cfg(test)]
    super::admission_work_metrics::resolver();
    #[cfg(test)]
    replay_metrics::projection();
    let mut result = ResolverProjection::default();
    for route in super::route_objects::project(a) {
        let coordinate = route.occurrence(a);
        let mut omit = |start, end, reason| {
            result
                .registrations
                .semantic_omissions
                .push(ResolverRegistrationOmission {
                    source: a.source.clone(),
                    source_hash: a.source_hash.clone(),
                    start,
                    end,
                    reason,
                    route: Some(coordinate.clone()),
                    route_path: Some(route.path.clone()),
                    resolver: None,
                });
        };
        let Some(object) = unique(route.fields, "resolve") else {
            if route
                .fields
                .iter()
                .filter(|(name, _)| name == "resolve")
                .count()
                > 1
            {
                omit(
                    route.object.start,
                    route.object.end,
                    ResolverOmissionReason::DuplicateResolveField,
                );
            }
            continue;
        };
        if !contains(route.object, object)
            || !a.admits_demanded_structure(&a.source, &a.source_hash, object)
        {
            omit(
                object.start,
                object.end,
                ResolverOmissionReason::MappingProofNotEstablished,
            );
            continue;
        }
        let ExpressionValue::Object(fields) = &object.value else {
            omit(
                object.start,
                object.end,
                ResolverOmissionReason::UnsupportedMappingValue,
            );
            continue;
        };
        // Unknown keys from spread/computed/shorthand structure cannot establish
        // independently unique named mappings. Ordinary refused values stay local.
        if fields.iter().any(|(name, _)| name.is_empty()) {
            omit(
                object.start,
                object.end,
                ResolverOmissionReason::UnsupportedMappingKey,
            );
            continue;
        }
        let mut targets = BTreeSet::new();
        for (name, value) in fields {
            if fields.iter().filter(|(n, _)| n == name).count() != 1 {
                omit(
                    value.start,
                    value.end,
                    ResolverOmissionReason::DuplicateMappingKey,
                );
                continue;
            }
            if !contains(object, value) {
                omit(
                    value.start,
                    value.end,
                    ResolverOmissionReason::MappingProofNotEstablished,
                );
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
                omit(
                    value.start,
                    value.end,
                    ResolverOmissionReason::MappingProofNotEstablished,
                );
                continue;
            }
            let Some((mapping, id)) = a.resolve_mapping_at(candidates[0].pair) else {
                omit(
                    value.start,
                    value.end,
                    ResolverOmissionReason::MappingProofNotEstablished,
                );
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
            {
                omit(
                    value.start,
                    value.end,
                    ResolverOmissionReason::MappingProofNotEstablished,
                );
                continue;
            }
            result.registrations.records.push(ResolverRegistration {
                route: coordinate.clone(),
                route_path: route.path.clone(),
                key: mapping.name.clone(),
                resolver: id.into(),
                evidence: ResolverMappingEvidence {
                    source: mapping.source.clone(),
                    source_hash: mapping.source_hash.clone(),
                    field_pair: mapping.field_pair,
                    field_key: mapping.field_key,
                    object: mapping.object,
                    pair: mapping.pair,
                    key: mapping.key,
                    value: mapping.value,
                },
            });
            if !targets.insert(id.clone()) {
                continue;
            }
            result.edges.push(SemanticEdge {
                subject: route.subject(a),
                object: super::string_token_authority::owner(id, "Resolver"),
                relation: SemanticRelation::ResolvedBy,
                layer: "angular",
                call_evidence: None,
            });
        }
    }
    result
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
