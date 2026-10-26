//! Replay source-qualified provider membership; never evaluate runtime array values.
use crate::ir::typescript_resolution::declaration_facets::{ExpressionFacet, ExpressionValue};
use crate::ir::typescript_resolution::{DeclarationId, Resolution, TypeScriptAuthority};
use std::collections::BTreeSet;

fn contains(parent: &ExpressionFacet, child: &ExpressionFacet) -> bool {
    parent.start <= child.start && child.start < child.end && child.end <= parent.end
}

/// A terminal facet always keeps the source in which its offsets have meaning.
#[derive(Clone, Copy)]
pub(super) struct Entry<'a> {
    pub source: &'a std::path::Path,
    pub hash: &'a str,
    pub facet: &'a ExpressionFacet,
    pub crossed: bool,
}

impl Entry<'_> {
    pub(super) fn literal<'a>(
        &self,
        authority: &'a TypeScriptAuthority,
        value: &ExpressionFacet,
    ) -> Option<&'a str> {
        use crate::ir::typescript_resolution::demanded_occurrences::DemandedValue;
        let DemandedValue::Literal { kind, text } =
            authority.demanded_at(self.source, self.hash, value.start, value.end)?
        else {
            return None;
        };
        let valid = match &value.value {
            ExpressionValue::Literal => matches!(kind.as_str(), "number" | "null"),
            ExpressionValue::Boolean(value) => kind == if *value { "true" } else { "false" },
            ExpressionValue::StringValue(_) => {
                kind == "string"
                    && authority
                        .demanded_string_value(self.source, self.hash, value)
                        .is_some()
            }
            _ => false,
        };
        valid.then_some(text.as_str())
    }

    pub(super) fn reference<'a>(
        &self,
        authority: &'a TypeScriptAuthority,
        value: &ExpressionFacet,
    ) -> Option<&'a DeclarationId> {
        let ExpressionValue::Reference(Resolution::Resolved(id)) = &value.value else {
            return None;
        };
        let found =
            if self.source == authority.source.as_path() && self.hash == authority.source_hash {
                authority.value_reference_at(
                    crate::ir::invocation_evidence::SourceSpan::new(value.start, value.end).ok()?,
                )
            } else {
                authority.demanded_reference_at(self.source, self.hash, value.start, value.end)
            }?;
        (found == id).then_some(found)
    }
}

/// Existing owner-file callers keep their source-local contract.
pub(super) fn entries<'a>(
    authority: &'a TypeScriptAuthority,
    root: &'a ExpressionFacet,
    compose: bool,
) -> Vec<&'a ExpressionFacet> {
    walk(authority, root, compose, false, false)
        .into_iter()
        .map(|entry| entry.facet)
        .collect()
}

pub(super) fn qualified_entries<'a>(
    authority: &'a TypeScriptAuthority,
    root: &'a ExpressionFacet,
) -> Vec<Entry<'a>> {
    walk(authority, root, true, true, false)
}

/// Route owners require independently captured container occurrences as well.
pub(super) fn proved_entries<'a>(
    authority: &'a TypeScriptAuthority,
    root: &'a ExpressionFacet,
) -> Vec<Entry<'a>> {
    walk(authority, root, true, true, true)
}

#[derive(Clone, Copy)]
struct ProofPolicy {
    imported: bool,
    structure: bool,
}

fn walk<'a>(
    authority: &'a TypeScriptAuthority,
    root: &'a ExpressionFacet,
    compose: bool,
    imported: bool,
    structure: bool,
) -> Vec<Entry<'a>> {
    let mut result = Vec::new();
    let root = Entry {
        source: &authority.source,
        hash: &authority.source_hash,
        facet: root,
        crossed: false,
    };
    if compose {
        visit(
            authority,
            root,
            false,
            ProofPolicy {
                imported,
                structure,
            },
            &mut BTreeSet::new(),
            &mut result,
        );
    } else if let ExpressionValue::Array(items) = &root.facet.value {
        result.extend(
            items
                .iter()
                .filter(|item| contains(root.facet, item))
                .map(|facet| Entry { facet, ..root }),
        );
    }
    result
}

fn visit<'a>(
    authority: &'a TypeScriptAuthority,
    entry: Entry<'a>,
    inside_array: bool,
    policy: ProofPolicy,
    active: &mut BTreeSet<DeclarationId>,
    result: &mut Vec<Entry<'a>>,
) {
    let value = entry.facet;
    if value.start >= value.end {
        return;
    }
    if policy.structure
        && matches!(
            value.value,
            ExpressionValue::Array(_) | ExpressionValue::Object(_) | ExpressionValue::Spread(_)
        )
        && !authority.admits_demanded_structure(entry.source, entry.hash, value)
    {
        return;
    }
    match &value.value {
        ExpressionValue::Array(items) => {
            for facet in items.iter().filter(|item| contains(value, item)) {
                visit(
                    authority,
                    Entry { facet, ..entry },
                    true,
                    policy,
                    active,
                    result,
                );
            }
        }
        ExpressionValue::Spread(operand) if inside_array && contains(value, operand) => {
            visit(
                authority,
                Entry {
                    facet: operand,
                    ..entry
                },
                false,
                policy,
                active,
                result,
            );
        }
        ExpressionValue::Reference(Resolution::Resolved(id))
            if id.kind == "variable_declarator" =>
        {
            let proved = if entry.source == authority.source.as_path()
                && entry.hash == authority.source_hash
            {
                crate::ir::invocation_evidence::SourceSpan::new(value.start, value.end)
                    .ok()
                    .and_then(|span| authority.value_reference_at(span))
                    == Some(id)
            } else {
                policy.imported
                    && authority.demanded_reference_at(
                        entry.source,
                        entry.hash,
                        value.start,
                        value.end,
                    ) == Some(id)
            };
            if !proved
                || (!policy.imported
                    && (id.source != authority.source || id.source_hash != authority.source_hash))
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
                visit(
                    authority,
                    Entry {
                        source: &id.source,
                        hash: &id.source_hash,
                        facet: initializer,
                        crossed: entry.crossed || id.source.as_path() != entry.source,
                    },
                    false,
                    policy,
                    active,
                    result,
                );
            }
            active.remove(id);
        }
        ExpressionValue::Object(_) | ExpressionValue::Reference(_) if inside_array => {
            result.push(entry)
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

/// Only used to select the stricter retained-registration contract for composed roots.
pub(crate) fn has_composition(authority: &TypeScriptAuthority, name: &str) -> bool {
    let Some((owner, root)) = super::string_token_authority::provider_array_at(
        authority,
        name,
        super::provider_position::Position::Ordinary,
    ) else {
        return false;
    };
    if super::provider_position::Position::Ordinary
        .owner_type(authority, owner)
        .is_none()
    {
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
