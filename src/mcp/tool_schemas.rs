//! Reusable schema fragments for public tools with polymorphic requests.

use serde_json::{Value, json};

pub(super) fn apply_edit_operations() -> Value {
    json!({
        "oneOf": [
            {
                "type": "object",
                "properties": {
                    "type": { "const": "replace_body" },
                    "target": { "type": "string", "description": "Qualified structural unit to replace." },
                    "expectedOldText": { "type": "string", "description": "Byte-exact unit text previously delivered to this session." },
                    "newText": { "type": "string", "description": "Complete replacement body text." }
                },
                "required": ["type", "target", "expectedOldText", "newText"]
            },
            {
                "type": "object",
                "properties": {
                    "type": { "const": "delete" },
                    "target": { "type": "string", "description": "Qualified structural unit to delete." },
                    "expectedOldText": { "type": "string", "description": "Byte-exact unit text previously delivered to this session." }
                },
                "required": ["type", "target", "expectedOldText"]
            },
            {
                "type": "object",
                "properties": {
                    "type": { "const": "insert_after" },
                    "anchor": { "type": "string", "description": "Qualified structural unit after which to insert." },
                    "unitText": { "type": "string", "description": "Complete structural unit text to insert." }
                },
                "required": ["type", "anchor", "unitText"]
            },
            {
                "type": "object",
                "properties": {
                    "type": { "const": "insert_before" },
                    "anchor": { "type": "string", "description": "Qualified structural unit before which to insert." },
                    "unitText": { "type": "string", "description": "Complete structural unit text to insert." }
                },
                "required": ["type", "anchor", "unitText"]
            }
        ]
    })
}

pub(super) fn workspace_query_variants() -> Value {
    json!([
        query_variant("find_entities", &["name"]),
        query_variant("forward_edges", &["name"]),
        query_variant("reverse_edges", &["name"]),
        query_variant("entities_in_file", &["file_path"]),
        query_variant("transitive_dependencies", &["name"]),
        query_variant("has_cycle", &[]),
        query_variant(
            "calls_in_file",
            &["filePath", "workspaceRoot", "owner", "method"],
        )
    ])
}

fn query_variant(kind: &str, operation_fields: &[&str]) -> Value {
    let mut required = vec!["type"];
    required.extend_from_slice(operation_fields);
    json!({
        "properties": { "type": { "const": kind } },
        "required": required
    })
}
