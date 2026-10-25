//! Exact source-owned provider occurrences authorize class strategy pairs.
use crate::ir::typescript_resolution::declaration_facets::{ExpressionFacet, ExpressionValue};
use crate::ir::typescript_resolution::{Resolution, TypeScriptAuthority};
use crate::layers::meta::canonical_endpoint::CanonicalEndpoint;
use crate::layers::meta::semantic::{EntityRef, SemanticEdge};

pub(super) fn pair(
    authority: &TypeScriptAuthority,
    owner_name: &str,
    start: usize,
    end: usize,
) -> Option<(EntityRef, EntityRef)> {
    let (owner, array) = super::string_token_authority::provider_array(authority, owner_name)?;
    if owner.source != authority.source
        || owner.source_hash != authority.source_hash
        || array.start >= array.end
        || array.end > owner.end
    {
        return None;
    }
    let ExpressionValue::Array(entries) = &array.value else {
        return None;
    };
    let mut matches = entries
        .iter()
        .filter(|entry| entry.start == start && entry.end == end);
    let entry = matches.next()?;
    if matches.next().is_some() || !contains(array, entry) {
        return None;
    }
    let (token, target) = match &entry.value {
        ExpressionValue::Reference(_) => (entry, entry),
        ExpressionValue::Object(fields) => {
            if fields.iter().any(|(key, _)| {
                matches!(key.as_str(), "" | "useFactory" | "useValue" | "useExisting")
            }) {
                return None;
            }
            let unique = |key: &str| -> Option<&ExpressionFacet> {
                let mut found = fields.iter().filter(|(name, _)| name == key);
                let (_, value) = found.next()?;
                found.next().is_none().then_some(value)
            };
            (unique("provide")?, unique("useClass")?)
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
    let ExpressionValue::Reference(Resolution::Resolved(target_id)) = &target.value else {
        return None;
    };
    if target_id.kind != "class_declaration" {
        return None;
    }
    let token = if let Some(units) = authority.string_value(token) {
        super::string_token_authority::token(owner, token, &units, "ProviderToken")
    } else {
        let ExpressionValue::Reference(Resolution::Resolved(id)) = &token.value else {
            return None;
        };
        declaration_entity(id, "ProviderToken")
    };
    Some((token, declaration_entity(target_id, "ProviderClass")))
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
    authority
        .declaration_facets
        .iter()
        .filter(|facet| {
            facet.declaration.source == authority.source
                && facet.declaration.source_hash == authority.source_hash
        })
        .any(|facet| {
            let Some((_, array)) =
                super::string_token_authority::provider_array(authority, &facet.declaration.name)
            else {
                return false;
            };
            let ExpressionValue::Array(entries) = &array.value else {
                return false;
            };
            entries.iter().any(|entry| {
                pair(authority, &facet.declaration.name, entry.start, entry.end).is_some_and(
                    |(token, target)| same(&edge.subject, &token) && same(&edge.object, &target),
                )
            })
        })
}
