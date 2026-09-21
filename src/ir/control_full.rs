//! Versioned correctness baseline for model-visible semantic context.
//!
//! This is deliberately explicit. Future codecs are compared against this
//! normalized projection; they do not define its semantics.

use super::hierarchical::{FieldNode, HierarchicalIR, MethodNode, PatternEntry};
use crate::compression::Fidelity;
use crate::layers::meta::semantic::SemanticEdge;
use serde_json::{Value, json};

pub const CONTROL_FULL_SCHEMA: &str = "clean-ctx/control-full";
pub const CONTROL_FULL_VERSION: u64 = 1;

fn patterns(values: &[PatternEntry]) -> Value {
    Value::Array(
        values
            .iter()
            .map(|value| json!({ "name": value.name, "args": value.args }))
            .collect(),
    )
}

fn fields(values: &[FieldNode]) -> Value {
    Value::Array(
        values
            .iter()
            .map(|value| {
                json!({
                    "id": value.id,
                    "name": value.name,
                    "type": value.field_type,
                })
            })
            .collect(),
    )
}

fn methods(values: &[MethodNode]) -> Value {
    Value::Array(
        values
            .iter()
            .map(|value| {
                json!({
                    "id": value.id,
                    "name": value.name,
                    "parameters": value.params.iter().map(|parameter| json!({
                        "id": parameter.first(),
                        "type": parameter.get(1),
                        "name": parameter.get(2),
                    })).collect::<Vec<_>>(),
                    "return_type": value.return_type,
                    "modifier_occurrences": value.modifiers,
                    "control_summary_occurrences": value.control_summaries,
                    "pattern_fact_occurrences": value.pattern_facts,
                    "legacy_flag_occurrences": value.flags,
                    "patterns": patterns(&value.patterns),
                    "body": value.body,
                    "body_start": value.body_start,
                    "body_end": value.body_end,
                    "control_flow": value.control_flow,
                    "data_flow": value.data_flow,
                    "side_effects": value.side_effect,
                    "execution_contexts": value.execution_context,
                })
            })
            .collect(),
    )
}

/// Build the exact comparison oracle from checked hierarchy plus the complete
/// semantic-edge stream. Array order and nested occurrence grouping are kept.
pub fn normalize_control_full(
    file_id: &str,
    source_path: &str,
    version: u64,
    fidelity: Fidelity,
    hierarchy: &HierarchicalIR,
    semantic_edges: &[SemanticEdge],
) -> Value {
    let body_method_ids = hierarchy
        .classes
        .iter()
        .flat_map(|owner| owner.methods.iter())
        .chain(
            hierarchy
                .interfaces
                .iter()
                .flat_map(|owner| owner.methods.iter()),
        )
        .filter(|method| method.body.is_some())
        .map(|method| method.id.clone())
        .collect::<Vec<_>>();
    let classes = hierarchy
        .classes
        .iter()
        .map(|value| {
            json!({
                "kind": "class",
                "id": value.id,
                "name": value.name,
                "synthetic": value.synthetic,
                "methods": methods(&value.methods),
                "fields": fields(&value.fields),
                "modifier_occurrences": value.modifiers,
                "class_flag_occurrences": value.class_flags,
                "extends": value.extends,
                "implements": value.implements,
                "injection_occurrences": value.injects,
                "patterns": patterns(&value.patterns),
            })
        })
        .collect::<Vec<_>>();
    let interfaces = hierarchy
        .interfaces
        .iter()
        .map(|value| {
            json!({
                "kind": "interface",
                "id": value.id,
                "name": value.name,
                "methods": methods(&value.methods),
                "fields": fields(&value.fields),
                "modifier_occurrences": value.modifiers,
                "extends": value.extends,
            })
        })
        .collect::<Vec<_>>();
    let calls = hierarchy
        .calls
        .iter()
        .enumerate()
        .map(|(occurrence, value)| {
            json!({
                "occurrence": occurrence,
                "caller_method_id": value.caller,
                "callee_written_name": value.callee,
                "explicit_argument_count": value.explicit_arg_count,
                "has_spread": value.has_spread,
                "callee_resolution": "unresolved",
            })
        })
        .collect::<Vec<_>>();
    let imports = hierarchy
        .imports
        .iter()
        .enumerate()
        .map(|(occurrence, value)| {
            json!({
                "occurrence": occurrence,
                "alias": value.first(),
                "module": value.get(1),
                "named_export": value.get(2),
            })
        })
        .collect::<Vec<_>>();
    let type_aliases = hierarchy
        .type_aliases
        .iter()
        .enumerate()
        .map(|(occurrence, value)| {
            json!({
                "occurrence": occurrence,
                "alias": value.first(),
                "original_type": value.get(1),
            })
        })
        .collect::<Vec<_>>();
    let edges = semantic_edges
        .iter()
        .enumerate()
        .map(|(occurrence, value)| {
            json!({
                "occurrence": occurrence,
                "relation": value.relation,
                "subject": value.subject,
                "object": value.object,
                "layer": value.layer,
                "call_evidence": value.call_evidence,
            })
        })
        .collect::<Vec<_>>();

    json!({
        "schema": CONTROL_FULL_SCHEMA,
        "schema_version": CONTROL_FULL_VERSION,
        "file": { "id": file_id, "source_path": source_path, "ir_version": version },
        "mode": {
            "fidelity": fidelity,
            "exact_body_method_ids": body_method_ids,
            "source_requirement": "request edit or verbatim when exact source is required",
        },
        "classes": classes,
        "interfaces": interfaces,
        "imports": imports,
        "type_aliases": type_aliases,
        "calls": calls,
        "semantic_edges": edges,
    })
}

/// Render the portable, authoritative MCP `content` payload.
pub fn render_control_full(
    file_id: &str,
    source_path: &str,
    version: u64,
    fidelity: Fidelity,
    hierarchy: &HierarchicalIR,
    semantic_edges: &[SemanticEdge],
) -> String {
    let normalized = normalize_control_full(
        file_id,
        source_path,
        version,
        fidelity,
        hierarchy,
        semantic_edges,
    );
    format!(
        "// CONTROL-FULL v{CONTROL_FULL_VERSION}; canonical IDs are authoritative\n{}",
        serde_json::to_string_pretty(&normalized).expect("CONTROL-FULL values are serializable")
    )
}

#[cfg(test)]
#[path = "../tests/ir/control_full.rs"]
mod tests;
