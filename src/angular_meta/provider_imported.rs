//! Imported terminals in the shared provider pipeline. No file reads or resolver.
use super::provider_pair_authority::{Strategy, pair_entry};
use super::provider_position::Position;
use crate::ir::typescript_resolution::TypeScriptAuthority;
use crate::ir::typescript_resolution::declaration_facets::ExpressionValue;
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};

pub(super) fn project(
    authority: &TypeScriptAuthority,
    name: &str,
    position: Position,
) -> Vec<SemanticEdge> {
    let Some((owner, root)) =
        super::string_token_authority::provider_array_at(authority, name, position)
    else {
        return Vec::new();
    };
    let Some(owner_type) = position.owner_type(authority, owner) else {
        return Vec::new();
    };
    if root.start >= root.end || root.end > owner.end {
        return Vec::new();
    }
    project_entries(
        authority,
        owner,
        super::string_token_authority::owner(owner, owner_type),
        root,
        true,
        position.relation(),
    )
}

/// Shared terminal policies; specialized owners supply only proved membership.
pub(super) fn project_entries(
    authority: &TypeScriptAuthority,
    owner: &crate::ir::typescript_resolution::DeclarationId,
    subject: EntityRef,
    root: &crate::ir::typescript_resolution::declaration_facets::ExpressionFacet,
    imported_only: bool,
    registration: SemanticRelation,
) -> Vec<SemanticEdge> {
    let mut edges = Vec::new();
    let strict = matches!(
        subject.canonical,
        Some(crate::layers::meta::canonical_endpoint::CanonicalEndpoint::RouteOccurrence { .. })
    );
    let occurrences = if strict {
        super::provider_composition::proved_entries(authority, root)
    } else {
        super::provider_composition::qualified_entries(authority, root)
    };
    for occurrence in occurrences {
        // Existing owner-source syntax/compatibility interpretation is preserved.
        if imported_only && !occurrence.crossed {
            continue;
        }
        if strict
            && let ExpressionValue::Object(fields) = &occurrence.facet.value
            && fields
                .iter()
                .filter(|(key, _)| key == "useValue")
                .any(|(_, value)| {
                    matches!(
                        value.value,
                        ExpressionValue::Literal
                            | ExpressionValue::Boolean(_)
                            | ExpressionValue::StringValue(_)
                    ) && occurrence.literal(authority, value).is_none()
                })
        {
            continue;
        }
        let Some(token) =
            super::provider_position::registration_entry(authority, owner, occurrence)
        else {
            continue;
        };
        edges.push(edge(registration, subject.clone(), token.clone()));
        let (strategy, relation, field) = match &occurrence.facet.value {
            ExpressionValue::Reference(_) => (
                Some(Strategy::Class),
                SemanticRelation::ProviderUsesClass,
                None,
            ),
            ExpressionValue::Object(fields) => {
                let Some((key, value)) = fields.iter().find(|(key, _)| {
                    matches!(
                        key.as_str(),
                        "useClass" | "useFactory" | "useExisting" | "useValue"
                    )
                }) else {
                    continue;
                };
                match key.as_str() {
                    "useClass" => (
                        Some(Strategy::Class),
                        SemanticRelation::ProviderUsesClass,
                        Some(value),
                    ),
                    "useFactory" => (
                        Some(Strategy::Factory),
                        SemanticRelation::ProviderUsesFactory,
                        Some(value),
                    ),
                    "useExisting" => (
                        Some(Strategy::Existing),
                        SemanticRelation::ProviderUsesExisting,
                        Some(value),
                    ),
                    _ => (None, SemanticRelation::ProviderUsesValue, Some(value)),
                }
            }
            _ => continue,
        };
        if let Some(strategy) = strategy
            && let Some((provided, target)) = pair_entry(authority, owner, occurrence, strategy)
        {
            edges.push(edge(relation, provided, target));
            continue;
        }
        // Only already-approved noncanonical value/callable-variable presentation.
        let Some(value) = field else {
            continue;
        };
        let label = occurrence
            .reference(authority, value)
            .map(|id| id.name.as_str())
            .or_else(|| occurrence.literal(authority, value));
        if let Some(label) = label {
            let mut subject = token;
            subject.canonical = None;
            let kind = if relation == SemanticRelation::ProviderUsesFactory {
                "ProviderFactory"
            } else {
                "ProviderValue"
            };
            edges.push(edge(
                relation,
                subject,
                EntityRef::new("angular", kind, label),
            ));
        }
    }
    edges
}

fn edge(relation: SemanticRelation, subject: EntityRef, object: EntityRef) -> SemanticEdge {
    SemanticEdge {
        relation,
        subject,
        object,
        layer: "angular",
        call_evidence: None,
    }
}

pub(super) fn admits(authority: &TypeScriptAuthority, edge: &SemanticEdge) -> bool {
    let same = |actual: &EntityRef, expected: &EntityRef| {
        actual.domain == expected.domain
            && actual.entity_type == expected.entity_type
            && actual.name == expected.name
            && actual.file == expected.file
            && actual.canonical == expected.canonical
    };
    authority
        .declaration_facets
        .iter()
        .filter(|facet| {
            Position::Ordinary
                .owner_type(authority, &facet.declaration)
                .is_some()
        })
        .any(|facet| {
            [Position::Ordinary, Position::View]
                .into_iter()
                .any(|position| {
                    project(authority, &facet.declaration.name, position)
                        .iter()
                        .any(|expected| {
                            edge.layer == expected.layer
                                && edge.relation == expected.relation
                                && same(&edge.subject, &expected.subject)
                                && same(&edge.object, &expected.object)
                        })
                })
        })
}
