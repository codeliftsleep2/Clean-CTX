//! Source-bound legacy transitions: private proof -> existing semantic relation.
use super::type_binding::ActionTypeBinding;
use crate::compression::capture_pipeline::CapturedNode;
use crate::ir::invocation_evidence::SourceSpan;
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const QUERY: &str = r#"
(program (function_declaration parameters: (formal_parameters
        . (required_parameter !value pattern: (identifier) @ngrx.transition.state_parameter)
        . (required_parameter !value pattern: (identifier) @ngrx.transition.parameter
            type: (type_annotation (type_identifier) @ngrx.transition.parameter_type)) .)
    body: (statement_block
        (switch_statement value: (parenthesized_expression
            (member_expression !optional_chain
                object: (identifier) @ngrx.transition.receiver
                property: (property_identifier) @ngrx.transition.property))
            body: (switch_body
                (switch_case value: (identifier) @ngrx.transition.reference) @ngrx.transition.case)) @ngrx.transition.switch)) @ngrx.transition.owner)
(program (export_statement declaration: (function_declaration parameters: (formal_parameters
        . (required_parameter !value pattern: (identifier) @ngrx.transition.state_parameter)
        . (required_parameter !value pattern: (identifier) @ngrx.transition.parameter
            type: (type_annotation (type_identifier) @ngrx.transition.parameter_type)) .)
    body: (statement_block
        (switch_statement value: (parenthesized_expression
            (member_expression !optional_chain
                object: (identifier) @ngrx.transition.receiver
                property: (property_identifier) @ngrx.transition.property))
            body: (switch_body
                (switch_case value: (identifier) @ngrx.transition.reference) @ngrx.transition.case)) @ngrx.transition.switch)) @ngrx.transition.owner))
(for_in_statement kind: ["var" "let" "const"] left: (_) @ngrx.transition.unsupported_loop)

"#;

#[derive(Debug, Clone)]
pub(crate) struct LegacyTransition {
    pub action: EntityRef,
    pub reducer: EntityRef,
    // Exact case and declaration proof stays private. Public occurrence identity
    // follows existing asserting-file/triple semantics, not one occurrence/case.
    #[allow(dead_code)]
    pub case_span: SourceSpan,
    #[allow(dead_code)]
    pub declaration_span: SourceSpan,
}

impl LegacyTransition {
    pub(crate) fn edge(&self) -> SemanticEdge {
        SemanticEdge {
            relation: SemanticRelation::TriggersReducer,
            subject: self.action.clone(),
            object: self.reducer.clone(),
            layer: "ngrx",
            call_evidence: None,
        }
    }
}

fn span(capture: &CapturedNode) -> SourceSpan {
    SourceSpan::new(capture.start_byte, capture.end_byte).expect("ordered parser span")
}

pub(crate) fn collect(
    source: &str,
    file: Option<&str>,
    captures: &[CapturedNode],
    action_bindings: &[ActionTypeBinding],
) -> Vec<LegacyTransition> {
    // Declared for-in/of bindings are outside the supported lexical contract.
    // In particular `var` can hoist a case-token shadow into its function.
    // Refuse this source form rather than extending the frozen type binder.
    if captures
        .iter()
        .any(|c| c.name == "ngrx.transition.unsupported_loop")
    {
        return Vec::new();
    }
    let Some(file) = file else {
        return Vec::new();
    };
    if action_bindings.is_empty() {
        return Vec::new();
    }
    let source_hash = crate::cache::content_hash(source.as_bytes());
    let mut matches: BTreeMap<usize, BTreeMap<&str, &CapturedNode>> = BTreeMap::new();
    let mut declarations: BTreeMap<&str, BTreeSet<SourceSpan>> = BTreeMap::new();
    for c in captures {
        matches.entry(c.match_index).or_default().insert(&c.name, c);
        if c.name == "ngrx.type.binding" {
            declarations.entry(&c.raw_text).or_default().insert(span(c));
        }
    }
    // Canonical function names come from the existing invocation declaration
    // query, joined to the exact function declaration span, never nearby text.
    let owners: BTreeMap<_, _> = matches
        .values()
        .filter_map(|m| {
            let declaration = m.get("invoke.declaration.callable")?;
            let name = m.get("invoke.declaration_name")?;
            Some((span(declaration), name.raw_text.as_str()))
        })
        .collect();
    let constants: BTreeMap<_, _> = matches
        .values()
        .filter_map(|m| {
            let name = m.get("ngrx.type.const_name")?;
            let declaration = m.get("ngrx.type.const")?;
            Some((span(name), span(declaration)))
        })
        .collect();
    let mut binding_index: BTreeMap<_, Vec<_>> = BTreeMap::new();
    for binding in action_bindings
        .iter()
        .filter(|b| b.file == file && b.source_hash == source_hash)
    {
        binding_index
            .entry((binding.declaration_span, binding.declaration_name_span))
            .or_default()
            .push(binding);
    }
    let mut result = BTreeMap::new();
    for matched in matches.values() {
        let (
            Some(owner),
            Some(parameter),
            Some(parameter_type),
            Some(receiver),
            Some(property),
            Some(reference),
            Some(case),
        ) = (
            matched.get("ngrx.transition.owner"),
            matched.get("ngrx.transition.parameter"),
            matched.get("ngrx.transition.parameter_type"),
            matched.get("ngrx.transition.receiver"),
            matched.get("ngrx.transition.property"),
            matched.get("ngrx.transition.reference"),
            matched.get("ngrx.transition.case"),
        )
        else {
            continue;
        };
        if parameter_type.raw_text != "Action"
            || property.raw_text != "type"
            || receiver.raw_text != parameter.raw_text
        {
            continue;
        }
        let owner_span = span(owner);
        let Some(reducer_name) = owners.get(&owner_span) else {
            continue;
        };
        // Refuse a shadowed/duplicate parameter, or a local Action type parameter.
        let owner_window = SourceSpan::new(owner.start_byte, owner.start_byte)
            .expect("ordered span")
            ..SourceSpan::new(owner.end_byte, owner.end_byte).expect("ordered span");
        let parameter_bindings: Vec<_> = declarations
            .get(parameter.raw_text.as_str())
            .into_iter()
            .flat_map(|spans| spans.range(owner_window.clone()))
            .filter(|s| s.end_byte() <= owner.end_byte)
            .copied()
            .collect();
        if parameter_bindings != [span(parameter)] {
            continue;
        }
        if declarations.get("Action").is_some_and(|spans| {
            spans
                .range(owner_window)
                .any(|s| s.end_byte() <= owner.end_byte)
        }) {
            continue;
        }
        // Prereq2 admits only a globally unambiguous program const. Independently
        // resolve the case in that same bounded scope before joining identities.
        let Some(names) = declarations.get(reference.raw_text.as_str()) else {
            continue;
        };
        if names.len() != 1 {
            continue;
        }
        let name_span = names.first().expect("one supported declaration");
        let Some(declaration_span) = constants.get(name_span) else {
            continue;
        };
        let Some(matching) = binding_index.get(&(*declaration_span, *name_span)) else {
            continue;
        };
        // Multiple Actions sharing this declaration do not provide a unique
        // Action endpoint. Preserve their private bindings but refuse a guess.
        let [binding] = matching.as_slice() else {
            continue;
        };
        let mut action = EntityRef::new("ngrx", "Action", &binding.action_name);
        action.file = Some(file.into());
        let mut reducer = EntityRef::new("ngrx", "Reducer", *reducer_name);
        reducer.file = Some(file.into());
        result.insert(
            span(case),
            LegacyTransition {
                action,
                reducer,
                case_span: span(case),
                declaration_span: *declaration_span,
            },
        );
    }
    result.into_values().collect()
}

#[cfg(all(test, feature = "typescript"))]
#[path = "../../tests/angular_meta/ngrx_legacy_transition.rs"]
mod tests;
