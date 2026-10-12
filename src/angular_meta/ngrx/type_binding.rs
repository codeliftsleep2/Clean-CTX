//! Conservative same-file binding for already admitted top-level class Actions.
//! No module resolution, value interpretation, durable or public products.
use super::legacy::ClassActionEvidence;
use crate::compression::capture_pipeline::CapturedNode;
use crate::ir::invocation_evidence::SourceSpan;
use std::collections::{BTreeMap, BTreeSet};

// Compilation-scoped source identity retained for the legacy transition consumer.
// The record itself is never persisted or made a public entity; the consumer
// may project an existing relation only after its source-identity join succeeds.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ActionTypeBinding {
    pub file: String,
    pub source_hash: String,
    pub action_name: String,
    pub action_span: SourceSpan,
    pub field_span: SourceSpan,
    pub reference_span: SourceSpan,
    pub declaration_name: String,
    pub declaration_span: SourceSpan,
    pub declaration_name_span: SourceSpan,
}

fn span(capture: &CapturedNode) -> SourceSpan {
    SourceSpan::new(capture.start_byte, capture.end_byte).expect("ordered parser span")
}

pub(crate) fn collect(
    source: &str,
    canonical_path: Option<&str>,
    captures: &[CapturedNode],
    actions: &[ClassActionEvidence],
) -> Vec<ActionTypeBinding> {
    if actions.is_empty() {
        return Vec::new();
    }
    let Some(file) = canonical_path.filter(|path| !path.is_empty()) else {
        return Vec::new();
    };
    let mut matches: BTreeMap<usize, BTreeMap<&str, &CapturedNode>> = BTreeMap::new();
    let mut bindings: BTreeMap<&str, BTreeSet<SourceSpan>> = BTreeMap::new();
    for capture in captures.iter().filter(|c| c.name.starts_with("ngrx.type.")) {
        matches
            .entry(capture.match_index)
            .or_default()
            .insert(&capture.name, capture);
        if capture.name == "ngrx.type.binding" {
            bindings
                .entry(&capture.raw_text)
                .or_default()
                .insert(span(capture));
        }
    }
    // Unsupported destructuring/namespace/catch patterns fail closed for the
    // whole file. Direct identifier declarations are proved by exact AST spans.
    // Even an unrelated nested same-name binding refuses association: we do not
    // claim a general lexical resolver or pick a first/nearest declaration.
    let plain_bindings: BTreeSet<_> = bindings.values().flatten().copied().collect();
    if captures.iter().any(|c| {
        c.name == "ngrx.type.invalid"
            || (c.name == "ngrx.type.binding_pattern" && !plain_bindings.contains(&span(c)))
    }) {
        return Vec::new();
    }
    // Escaped identifier spellings require normalization before lexical
    // comparison; this bounded resolver deliberately does not normalize them.
    if bindings.keys().any(|name| name.contains('\\')) {
        return Vec::new();
    }
    let constants: BTreeMap<_, _> = matches
        .values()
        .filter_map(|matched| {
            let name = matched.get("ngrx.type.const_name")?;
            let declaration = matched.get("ngrx.type.const")?;
            Some((span(name), (*name, *declaration)))
        })
        .collect();
    let owners: BTreeMap<_, _> = actions
        .iter()
        .map(|action| (action.class_span, action))
        .collect();
    let mut members: BTreeMap<SourceSpan, BTreeSet<SourceSpan>> = BTreeMap::new();
    for matched in matches.values() {
        if let (Some(owner), Some(member)) = (
            matched.get("ngrx.type.any_owner"),
            matched.get("ngrx.type.any_member"),
        ) && member.raw_text == "type"
        {
            members.entry(span(owner)).or_default().insert(span(member));
        }
    }
    let static_fields: BTreeSet<_> = captures
        .iter()
        .filter(|c| c.name == "ngrx.type.static_field")
        .map(span)
        .collect();
    let unsupported_owners: BTreeSet<_> = captures
        .iter()
        .filter(|c| c.name == "ngrx.type.unsupported_owner")
        .map(span)
        .collect();
    let source_hash = crate::cache::content_hash(source.as_bytes());
    let mut result = BTreeMap::new();
    for matched in matches.values() {
        let (Some(owner), Some(field), Some(member), Some(reference)) = (
            matched.get("ngrx.type.owner"),
            matched.get("ngrx.type.field"),
            matched.get("ngrx.type.member"),
            matched.get("ngrx.type.reference"),
        ) else {
            continue;
        };
        let Some(action) = owners.get(&span(owner)) else {
            continue;
        };
        if member.raw_text != "type"
            || static_fields.contains(&span(field))
            || unsupported_owners.contains(&span(owner))
            || members
                .get(&span(owner))
                .is_none_or(|names| names.len() != 1)
        {
            continue;
        }
        let Some(declarations) = bindings.get(reference.raw_text.as_str()) else {
            continue;
        };
        if declarations.len() != 1 {
            continue;
        }
        let name_span = declarations.first().expect("one declaration");
        let Some((name, declaration)) = constants.get(name_span) else {
            continue;
        };
        result.insert(
            span(field),
            ActionTypeBinding {
                file: file.into(),
                source_hash: source_hash.clone(),
                action_name: action.name.clone(),
                action_span: action.class_span,
                field_span: span(field),
                reference_span: span(reference),
                declaration_name: name.raw_text.clone(),
                declaration_span: span(declaration),
                declaration_name_span: span(name),
            },
        );
    }
    result.into_values().collect()
}

#[cfg(all(test, feature = "typescript"))]
#[path = "../../tests/angular_meta/ngrx_action_type_binding.rs"]
mod tests;
