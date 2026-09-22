//! Versioned correctness baseline for model-visible semantic context.
//!
//! This is deliberately explicit. Future codecs are compared against this
//! normalized projection; they do not define its semantics.

use super::hierarchical::{FieldNode, HierarchicalIR, MethodNode, PatternEntry};
use crate::compression::Fidelity;
use crate::layers::meta::semantic::SemanticEdge;
use serde_json::{Value, json};

pub const CONTROL_FULL_SCHEMA: &str = "clean-ctx/control-full";
pub const CONTROL_FULL_VERSION: u64 = 2;
pub const CONTROL_FULL_NAVIGATION_SCHEMA: &str = "clean-ctx/control-full-navigation";
pub const CONTROL_FULL_NAVIGATION_VERSION: u64 = 1;

const OCCURRENCE_FAMILIES: [&str; 4] = [
    "modifier_occurrences",
    "control_summary_occurrences",
    "pattern_fact_occurrences",
    "legacy_flag_occurrences",
];

fn method_occurrence_navigation(
    owner_kind: &str,
    owner_id: &str,
    method: &MethodNode,
) -> Vec<Value> {
    OCCURRENCE_FAMILIES
        .iter()
        .map(|family| {
            json!({
                "locator": {
                    "owner": { "kind": owner_kind, "id": owner_id },
                    "member": { "kind": "method", "id": method.id },
                    "field": { "kind": "occurrence_group_array", "name": family },
                },
                "semantics": {
                    "outer_array": "ordered_occurrences",
                    "inner_array": "one_occurrence_group",
                    "duplicates_significant": true,
                    "empty_groups_significant": true,
                }
            })
        })
        .collect()
}

/// Stable typed descriptors for the existing semantic-edge records.
/// Locators use semantic identity and occurrence rather than serialization offsets.
pub fn semantic_edge_navigation(semantic_edges: &[SemanticEdge], collection: &str) -> Vec<Value> {
    semantic_edges
        .iter()
        .enumerate()
        .map(|(occurrence, edge)| {
            json!({
                "locator": {
                    "collection": collection,
                    "occurrence": occurrence,
                    "relation": edge.relation,
                    "layer": edge.layer,
                    "subject": {
                        "domain": edge.subject.domain,
                        "entity_type": edge.subject.entity_type,
                        "name": edge.subject.name,
                    },
                    "object": {
                        "domain": edge.object.domain,
                        "entity_type": edge.object.entity_type,
                        "name": edge.object.name,
                    },
                },
                "endpoint_fields": {
                    "subject_file": { "endpoint": "subject", "field": "file" },
                    "object_file": { "endpoint": "object", "field": "file" },
                }
            })
        })
        .collect()
}

fn navigation(hierarchy: &HierarchicalIR, semantic_edges: &[SemanticEdge]) -> Value {
    let occurrence_groups = hierarchy
        .classes
        .iter()
        .flat_map(|owner| {
            owner
                .methods
                .iter()
                .flat_map(|method| method_occurrence_navigation("class", &owner.id, method))
        })
        .chain(hierarchy.interfaces.iter().flat_map(|owner| {
            owner
                .methods
                .iter()
                .flat_map(|method| method_occurrence_navigation("interface", &owner.id, method))
        }))
        .collect::<Vec<_>>();
    json!({
        "schema": CONTROL_FULL_NAVIGATION_SCHEMA,
        "schema_version": CONTROL_FULL_NAVIGATION_VERSION,
        "occurrence_groups": occurrence_groups,
        "semantic_edge_provenance": semantic_edge_navigation(semantic_edges, "semantic_edges"),
    })
}

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
        "navigation": navigation(hierarchy, semantic_edges),
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

#[cfg(test)]
#[path = "../tests/ir/compact_a.rs"]
mod compact_a_tests;

#[cfg(test)]
#[path = "../tests/ir/compact_a_graph.rs"]
mod compact_a_graph_tests;

#[cfg(test)]
#[path = "../tests/ir/compact_a_envelope.rs"]
pub(crate) mod compact_a_envelope_tests;

#[cfg(test)]
#[path = "../tests/ir/compact_a_production_edges.rs"]
mod compact_a_production_edge_tests;
