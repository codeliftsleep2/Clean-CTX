//! Replay source-local provider membership; never evaluate runtime array values.
use crate::ir::typescript_resolution::declaration_facets::{ExpressionFacet, ExpressionValue};
use crate::ir::typescript_resolution::{DeclarationId, Resolution, TypeScriptAuthority};
use std::collections::BTreeSet;

fn contains(parent: &ExpressionFacet, child: &ExpressionFacet) -> bool {
    parent.start <= child.start && child.start < child.end && child.end <= parent.end
}

/// Original retained terminal occurrences, reached through their actual containers.
/// Repeated routes may return the same occurrence; no synthetic flattened facet.
pub(super) fn entries<'a>(
    authority: &'a TypeScriptAuthority,
    root: &'a ExpressionFacet,
    compose: bool,
) -> Vec<&'a ExpressionFacet> {
    let mut result = Vec::new();
    if !compose {
        if let ExpressionValue::Array(items) = &root.value {
            result.extend(items.iter().filter(|item| contains(root, item)));
        }
    } else {
        visit(authority, root, false, &mut BTreeSet::new(), &mut result);
    }
    result
}

fn visit<'a>(
    authority: &'a TypeScriptAuthority,
    value: &'a ExpressionFacet,
    inside_array: bool,
    active: &mut BTreeSet<DeclarationId>,
    result: &mut Vec<&'a ExpressionFacet>,
) {
    if value.start >= value.end {
        return;
    }
    match &value.value {
        ExpressionValue::Array(items) => {
            for item in items.iter().filter(|item| contains(value, item)) {
                visit(authority, item, true, active, result);
            }
        }
        ExpressionValue::Spread(operand) if inside_array && contains(value, operand) => {
            visit(authority, operand, false, active, result);
        }
        ExpressionValue::Reference(Resolution::Resolved(id))
            if id.kind == "variable_declarator" =>
        {
            let Ok(span) = crate::ir::invocation_evidence::SourceSpan::new(value.start, value.end)
            else {
                return;
            };
            if id.source != authority.source
                || id.source_hash != authority.source_hash
                || authority.value_reference_at(span) != Some(id)
                || !active.insert(id.clone())
            {
                return;
            }
            let mut facets = authority
                .declaration_facets
                .iter()
                .filter(|facet| facet.declaration == *id);
            if let Some(facet) = facets.next()
                && facets.next().is_none()
                && let Some(initializer) = &facet.initializer
                && id.start <= initializer.start
                && initializer.start < initializer.end
                && initializer.end <= id.end
                && matches!(
                    initializer.value,
                    ExpressionValue::Array(_) | ExpressionValue::Reference(_)
                )
            {
                // Reference hops establish association; initializer spans are not
                // required to lie geometrically inside the metadata expression.
                visit(authority, initializer, false, active, result);
            }
            active.remove(id);
        }
        ExpressionValue::Object(_) | ExpressionValue::Reference(_) if inside_array => {
            result.push(value)
        }
        _ => {}
    }
}

/// Select a uniquely retained terminal, allowing repeated paths to the same facet.
pub(super) fn entry_at<'a>(
    authority: &'a TypeScriptAuthority,
    root: &'a ExpressionFacet,
    compose: bool,
    start: usize,
    end: usize,
) -> Option<&'a ExpressionFacet> {
    let entries = entries(authority, root, compose);
    let mut found = entries
        .into_iter()
        .filter(|entry| entry.start == start && entry.end == end);
    let entry = found.next()?;
    found
        .all(|other| std::ptr::eq(other, entry))
        .then_some(entry)
}

pub(super) fn component_owner(authority: &TypeScriptAuthority, owner: &DeclarationId) -> bool {
    owner.source == authority.source
        && owner.source_hash == authority.source_hash
        && super::class_role_authority::declaration_role(authority, owner)
            == Some(super::decorators::ClassKind::Component)
}

/// Only used to select the stricter retained-registration contract for composed roots.
pub(crate) fn has_composition(authority: &TypeScriptAuthority, name: &str) -> bool {
    let Some((owner, root)) = super::string_token_authority::provider_array_at(
        authority,
        name,
        super::provider_position::Position::Ordinary,
    ) else {
        return false;
    };
    if !component_owner(authority, owner) {
        return false;
    }
    match &root.value {
        ExpressionValue::Reference(_) => true,
        ExpressionValue::Array(items) => items.iter().any(|item| match &item.value {
            ExpressionValue::Array(_) | ExpressionValue::Spread(_) => true,
            ExpressionValue::Reference(Resolution::Resolved(id)) => {
                id.kind == "variable_declarator"
            }
            _ => false,
        }),
        _ => false,
    }
}
