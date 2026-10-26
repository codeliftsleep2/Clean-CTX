//! Angular admission of proved string values at individually supported API sites.
use crate::ir::typescript_resolution::declaration_facets::{ExpressionFacet, ExpressionValue};
use crate::ir::typescript_resolution::{DeclarationId, TypeScriptAuthority};
use crate::layers::meta::canonical_endpoint::{CanonicalEndpoint, LiteralProof, string_label};
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};

pub(super) fn token(
    owner: &DeclarationId,
    expression: &ExpressionFacet,
    units: &[u16],
    entity_type: &'static str,
) -> EntityRef {
    let mut token = EntityRef::new("angular", entity_type, string_label(units))
        .with_file(owner.source.to_string_lossy().into_owned());
    token.canonical = Some(CanonicalEndpoint::StringValue {
        string_value: units.to_vec(),
        proof: LiteralProof {
            source: owner.source.clone(),
            source_hash: owner.source_hash.clone(),
            start: expression.start,
            end: expression.end,
        },
    });
    token
}

pub(super) fn owner(id: &DeclarationId, owner_type: &'static str) -> EntityRef {
    let mut owner = EntityRef::new("angular", owner_type, &id.name)
        .with_file(id.source.to_string_lossy().into_owned());
    owner.canonical = Some(CanonicalEndpoint::Declaration {
        declaration: id.into(),
        site: Some((id.start, id.end)),
    });
    owner
}

pub(super) fn injection_edge(
    id: &DeclarationId,
    owner_type: &'static str,
    expression: &ExpressionFacet,
    units: &[u16],
) -> SemanticEdge {
    SemanticEdge {
        relation: SemanticRelation::Injects,
        subject: owner(id, owner_type),
        object: token(id, expression, units, "Service"),
        layer: "angular",
        call_evidence: None,
    }
}

/// Retained provider-array proof for one exact, uniquely admitted Angular owner.
pub(super) fn provider_array<'a>(
    authority: &'a TypeScriptAuthority,
    name: &str,
) -> Option<(&'a DeclarationId, &'a ExpressionFacet)> {
    use super::{class_role_authority as roles, decorators::ClassKind};
    let mut owners = authority.declaration_facets.iter().filter(|facet| {
        facet.declaration.source == authority.source && facet.declaration.name == name
    });
    let facet = owners.next()?;
    if owners.next().is_some()
        || !matches!(
            roles::declaration_role(authority, &facet.declaration),
            Some(ClassKind::Component | ClassKind::Module | ClassKind::Directive)
        )
    {
        return None;
    }
    let mut arrays = Vec::new();
    for decorator in &facet.decorators {
        let ExpressionValue::Reference(callee) = &decorator.callee.value else {
            continue;
        };
        if !matches!(
            roles::decorator_role(authority, callee),
            Some(ClassKind::Component | ClassKind::Module | ClassKind::Directive)
        ) {
            continue;
        }
        for argument in &decorator.arguments {
            if let ExpressionValue::Object(fields) = &argument.value {
                if fields.iter().any(|(key, _)| key.is_empty()) {
                    return None;
                }
                arrays.extend(
                    fields
                        .iter()
                        .filter(|(key, _)| key == "providers")
                        .map(|(_, value)| value),
                );
            }
        }
    }
    let [array] = arrays.as_slice() else {
        return None;
    };
    Some((&facet.declaration, array))
}

pub(super) fn provider_literal<'a>(
    authority: &TypeScriptAuthority,
    array: &'a ExpressionFacet,
    start: usize,
    end: usize,
) -> Option<(&'a ExpressionFacet, Vec<u16>)> {
    let value = provider_value(array, start, end)?;
    authority.string_value(value).map(|units| (value, units))
}

/// Exact provider-position evidence is shared by string and declaration tokens.
pub(super) fn provider_value(
    array: &ExpressionFacet,
    start: usize,
    end: usize,
) -> Option<&ExpressionFacet> {
    provider_field(array, "provide", start, end)
}

/// One exact retained provider field; duplicate fields never supply proof.
pub(super) fn provider_field<'a>(
    array: &'a ExpressionFacet,
    field: &str,
    start: usize,
    end: usize,
) -> Option<&'a ExpressionFacet> {
    let ExpressionValue::Array(entries) = &array.value else {
        return None;
    };
    let mut candidates = Vec::new();
    for entry in entries {
        if field == "provide"
            && entry.start == start
            && entry.end == end
            && matches!(entry.value, ExpressionValue::Reference(_))
        {
            candidates.push(entry);
            continue;
        }
        let ExpressionValue::Object(fields) = &entry.value else {
            continue;
        };
        if fields.iter().any(|(key, _)| key.is_empty()) {
            continue;
        }
        let mut tokens = fields.iter().filter(|(key, _)| key == field);
        let Some((_, value)) = tokens.next() else {
            continue;
        };
        if tokens.next().is_none() && value.start == start && value.end == end {
            candidates.push(value);
        }
    }
    if candidates.len() == 1 {
        candidates.pop()
    } else {
        None
    }
}
