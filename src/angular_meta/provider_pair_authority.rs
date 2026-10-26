//! Shared exact provider-position mechanics with explicit strategy policies.
use crate::ir::typescript_resolution::declaration_facets::{ExpressionFacet, ExpressionValue};
use crate::ir::typescript_resolution::{Resolution, TypeScriptAuthority};
use crate::layers::meta::canonical_endpoint::CanonicalEndpoint;
use crate::layers::meta::semantic::{EntityRef, SemanticEdge};

#[derive(Clone, Copy)]
pub(super) enum Strategy {
    Class,
    Factory,
    Existing,
}

impl Strategy {
    fn field(self) -> &'static str {
        match self {
            Self::Class => "useClass",
            Self::Factory => "useFactory",
            Self::Existing => "useExisting",
        }
    }
    fn target_kind(self) -> Option<&'static str> {
        match self {
            Self::Class => Some("class_declaration"),
            Self::Factory => Some("function_declaration"),
            Self::Existing => None,
        }
    }
    fn entity_type(self) -> &'static str {
        match self {
            Self::Class => "ProviderClass",
            Self::Factory => "ProviderFactory",
            Self::Existing => "ProviderToken",
        }
    }
}

pub(super) fn pair_at(
    authority: &TypeScriptAuthority,
    owner_name: &str,
    start: usize,
    end: usize,
    strategy: Strategy,
    position: super::provider_position::Position,
) -> Option<(EntityRef, EntityRef)> {
    let (owner, array) =
        super::string_token_authority::provider_array_at(authority, owner_name, position)?;
    if owner.source != authority.source
        || owner.source_hash != authority.source_hash
        || array.start >= array.end
        || array.end > owner.end
    {
        return None;
    }
    let entry = super::provider_composition::entry_at(
        authority,
        array,
        position.owner_type(authority, owner).is_some(),
        start,
        end,
    )?;
    pair_entry(
        authority,
        owner,
        super::provider_composition::Entry {
            source: &authority.source,
            hash: &authority.source_hash,
            facet: entry,
            crossed: false,
        },
        strategy,
    )
}

pub(super) fn pair_entry(
    authority: &TypeScriptAuthority,
    owner: &crate::ir::typescript_resolution::DeclarationId,
    occurrence: super::provider_composition::Entry<'_>,
    strategy: Strategy,
) -> Option<(EntityRef, EntityRef)> {
    let entry = occurrence.facet;
    let (token, target) = match &entry.value {
        ExpressionValue::Reference(_) if matches!(strategy, Strategy::Class) => (entry, entry),
        ExpressionValue::Object(fields) => {
            if fields.iter().any(|(key, _)| {
                key.is_empty()
                    || (matches!(
                        key.as_str(),
                        "useClass" | "useFactory" | "useValue" | "useExisting"
                    ) && key != strategy.field())
            }) {
                return None;
            }
            let unique = |key: &str| -> Option<&ExpressionFacet> {
                let mut found = fields.iter().filter(|(name, _)| name == key);
                let (_, value) = found.next()?;
                found.next().is_none().then_some(value)
            };
            (unique("provide")?, unique(strategy.field())?)
        }
        _ => return None,
    };
    if !contains(entry, token) || !contains(entry, target) {
        return None;
    }
    if matches!(entry.value, ExpressionValue::Object(_))
        && token.start < target.end
        && target.start < token.end
    {
        return None;
    }
    if matches!(strategy, Strategy::Existing) {
        return Some((
            canonical_token_in(authority, owner, occurrence, token)?,
            canonical_token_in(authority, owner, occurrence, target)?,
        ));
    }
    let ExpressionValue::Reference(Resolution::Resolved(target_id)) = &target.value else {
        return None;
    };
    if Some(target_id.kind.as_str()) != strategy.target_kind() {
        return None;
    }
    occurrence.reference(authority, target)?;
    Some((
        canonical_token_in(authority, owner, occurrence, token)?,
        declaration_entity(target_id, strategy.entity_type()),
    ))
}

pub(super) fn canonical_token_in(
    authority: &TypeScriptAuthority,
    owner: &crate::ir::typescript_resolution::DeclarationId,
    occurrence: super::provider_composition::Entry<'_>,
    value: &ExpressionFacet,
) -> Option<EntityRef> {
    if occurrence.source == authority.source.as_path() && occurrence.hash == authority.source_hash {
        return canonical_token(authority, owner, value);
    }
    if let Some(units) = authority.demanded_string_value(occurrence.source, occurrence.hash, value)
    {
        return Some(super::string_token_authority::token_at(
            occurrence.source,
            occurrence.hash,
            value,
            &units,
            "ProviderToken",
        ));
    }
    Some(declaration_entity(
        occurrence.reference(authority, value)?,
        "ProviderToken",
    ))
}

pub(super) fn canonical_token(
    authority: &TypeScriptAuthority,
    owner: &crate::ir::typescript_resolution::DeclarationId,
    token: &ExpressionFacet,
) -> Option<EntityRef> {
    if let ExpressionValue::Reference(Resolution::Resolved(id)) = &token.value {
        let token_span =
            crate::ir::invocation_evidence::SourceSpan::new(token.start, token.end).ok()?;
        if authority.value_reference_at(token_span) != Some(id) {
            return None;
        }
    }
    let token = if let Some(units) = authority.string_value(token) {
        super::string_token_authority::token(owner, token, &units, "ProviderToken")
    } else {
        let ExpressionValue::Reference(Resolution::Resolved(id)) = &token.value else {
            return None;
        };
        declaration_entity(id, "ProviderToken")
    };
    Some(token)
}

fn contains(container: &ExpressionFacet, value: &ExpressionFacet) -> bool {
    container.start <= value.start && value.start < value.end && value.end <= container.end
}

fn declaration_entity(
    id: &crate::ir::typescript_resolution::DeclarationId,
    kind: &'static str,
) -> EntityRef {
    let mut entity = EntityRef::new("angular", kind, &id.name)
        .with_file(id.source.to_string_lossy().into_owned());
    entity.canonical = Some(CanonicalEndpoint::Declaration {
        declaration: id.into(),
        site: None,
    });
    entity
}

/// Replay the exact retained provider positions, never an endpoint cross-product.
pub(crate) fn admits(authority: &TypeScriptAuthority, edge: &SemanticEdge) -> bool {
    use crate::layers::meta::semantic::SemanticRelation;
    let strategy = match edge.relation {
        SemanticRelation::ProviderUsesClass => Strategy::Class,
        SemanticRelation::ProviderUsesFactory => Strategy::Factory,
        SemanticRelation::ProviderUsesExisting => Strategy::Existing,
        _ => return false,
    };
    if edge.layer != "angular" {
        return false;
    }
    let same = |actual: &EntityRef, expected: &EntityRef| {
        actual.domain == expected.domain
            && actual.entity_type == expected.entity_type
            && actual.name == expected.name
            && actual.file == expected.file
            && actual.canonical == expected.canonical
    };
    super::provider_imported::admits(authority, edge)
        || authority
            .declaration_facets
            .iter()
            .filter(|facet| {
                facet.declaration.source == authority.source
                    && facet.declaration.source_hash == authority.source_hash
            })
            .any(|facet| {
                [
                    super::provider_position::Position::Ordinary,
                    super::provider_position::Position::View,
                ]
                .into_iter()
                .any(|position| {
                    let Some((_, array)) = super::string_token_authority::provider_array_at(
                        authority,
                        &facet.declaration.name,
                        position,
                    ) else {
                        return false;
                    };
                    super::provider_composition::entries(
                        authority,
                        array,
                        position.owner_type(authority, &facet.declaration).is_some(),
                    )
                    .iter()
                    .any(|entry| {
                        pair_at(
                            authority,
                            &facet.declaration.name,
                            entry.start,
                            entry.end,
                            strategy,
                            position,
                        )
                        .is_some_and(|(token, target)| {
                            same(&edge.subject, &token) && same(&edge.object, &target)
                        })
                    })
                })
            })
}
