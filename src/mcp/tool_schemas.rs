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

fn provide_code_context_item_properties() -> Value {
    json!({
        "filePath": { "type": "string" },
        "intent": { "type": "string", "enum": ["edit", "refactor", "overview", "debug", "implement"], "description": "edit: byte-exact method bodies for safe apply_edit operations. refactor: full structural detail. overview: max compression. debug: balanced. implement: moderate detail." },
        "fidelity": { "type": "string", "enum": ["low", "medium", "high", "edit", "verbatim"], "description": "Compression fidelity: low, medium, high, edit, or verbatim." },
        "focusMethods": { "type": "array", "items": { "type": "string" }, "description": "Select qualified Owner.method names, or an unambiguous bare method name. Focused requests use Edit fidelity unless an explicit compatible mode is supplied." }
    })
}

pub(super) fn provide_code_context_properties() -> Value {
    let mut properties = provide_code_context_item_properties();
    let object = properties
        .as_object_mut()
        .expect("provide context properties are an object");
    object.insert("workspaceRoot".into(), json!({ "type": "string", "description": "Strongly recommended. Shared explicit workspace root for reliable path resolution; defaults to CWD for backward compatibility." }));
    object.insert("tokenizer".into(), json!({ "type": "string" }));

    let mut item_properties = provide_code_context_item_properties();
    item_properties
        .as_object_mut()
        .expect("provide context item properties are an object")
        .insert("id".into(), json!({ "type": "string", "minLength": 1 }));
    object.insert(
        "files".into(),
        json!({
            "type": "array",
            "minItems": 1,
            "maxItems": 8,
            "items": {
                "type": "object",
                "properties": item_properties,
                "required": ["id", "filePath"]
            }
        }),
    );
    Value::Object(object.clone())
}

pub(super) fn provide_code_context_request_variants() -> Value {
    json!([
        {
            "required": ["filePath"],
            "not": { "required": ["files"] }
        },
        {
            "properties": {
                "files": provide_code_context_properties()["files"].clone()
            },
            "required": ["files"],
            "not": { "required": ["filePath"] }
        }
    ])
}

pub(super) fn provide_code_context_batch_results() -> Value {
    json!({
        "type": "array",
        "description": "Ordered outcomes, exactly one per accepted batch item.",
        "items": {
            "type": "object",
            "properties": {
                "id": { "type": "string" },
                "status": { "type": "string", "enum": ["ok", "error"] },
                "content_index": { "type": "integer", "minimum": 0, "description": "Index of the exact successful text block in result.content." },
                "meta": { "type": "object", "description": "The corresponding single-file semantic metadata." },
                "error": {
                    "type": "object",
                    "properties": {
                        "code": { "type": "integer" },
                        "message": { "type": "string" },
                        "data": {}
                    },
                    "required": ["code", "message"]
                }
            },
            "required": ["id", "status"],
            "oneOf": [
                {
                    "properties": { "status": { "const": "ok" } },
                    "required": ["content_index", "meta"],
                    "not": { "required": ["error"] }
                },
                {
                    "properties": { "status": { "const": "error" } },
                    "required": ["error"],
                    "not": {
                        "anyOf": [
                            { "required": ["content_index"] },
                            { "required": ["meta"] }
                        ]
                    }
                }
            ]
        }
    })
}

pub(super) fn workspace_query_properties() -> Value {
    let mut properties = workspace_query_operation_properties();
    let properties = properties
        .as_object_mut()
        .expect("workspace query properties are an object");
    properties.insert("workspaceRoot".into(), json!({ "type": "string", "description": "Shared primary trusted workspace root. Required when any item is calls_in_file; optional for cross-file operations." }));
    properties.insert("withinPath".into(), json!({ "type": "string", "description": "Shared optional narrowing inside workspaceRoot plus configured additional roots. Rejected when outside that authorized set or when workspaceRoot is absent." }));
    properties.insert("queries".into(), workspace_query_batch_queries());
    Value::Object(properties.clone())
}

fn workspace_query_operation_properties() -> Value {
    json!({
        "type": { "type": "string", "enum": ["find_entities", "forward_edges", "reverse_edges", "entities_in_file", "transitive_dependencies", "has_cycle", "calls_in_file"], "description": "Type of workspace query." },
        "domain": { "type": "string", "description": "Optional exact domain for forward_edges, reverse_edges, and transitive_dependencies. With entity_type, uses the exact-identity fast path; alone, filters name resolution." },
        "entity_type": { "type": "string", "description": "Optional exact entity type for forward_edges, reverse_edges, and transitive_dependencies. With domain, uses the exact-identity fast path; alone, filters name resolution." },
        "name": { "type": "string", "minLength": 1, "description": "Exactly one entity name. Required for find_entities, forward_edges, reverse_edges, and transitive_dependencies. For multiple names, use top-level queries with one independently identified item per name; do not pass an array or a names field." },
        "file_path": { "type": "string", "description": "File path for entities_in_file query." },
        "fidelity": { "type": "string", "enum": ["low", "medium", "high", "edit", "verbatim"], "description": "Optional semantic compilation fidelity for entities_in_file. Edit and verbatim normalize to High because this query publishes no source bodies. Defaults to the configured fidelity." },
        "kind": { "type": "string", "enum": ["dependency"], "description": "Optional cycle policy for has_cycle. Defaults to dependency." },
        "filePath": { "type": "string", "description": "Trusted source file for calls_in_file." },
        "owner": { "type": "object", "description": "Typed caller owner for calls_in_file.", "properties": { "kind": { "type": "string", "enum": ["class", "interface"] }, "name": { "type": "string" } }, "required": ["kind", "name"] },
        "method": { "type": "object", "description": "Caller method for calls_in_file. Omit signature fields for the complete overload family.", "properties": { "name": { "type": "string" }, "parameters": { "type": "array", "items": { "type": "string" }, "description": "Optional exact visible parameter-signature selector." }, "return_type": { "type": "string" } }, "required": ["name"] },
        "depth": { "type": "integer", "description": "Traversal depth for transitive_dependencies: 0 = unlimited, 1 = direct, 2 = transitive. Default: 1." },
    })
}

pub(super) fn workspace_query_request_variants() -> Value {
    let mut variants = workspace_query_variants(true)
        .as_array()
        .expect("workspace query variants are an array")
        .clone();
    for variant in &mut variants {
        variant["not"] = json!({ "required": ["queries"] });
    }
    variants.push(json!({
        "properties": { "queries": workspace_query_batch_queries() },
        "required": ["queries"],
        "not": { "required": ["type"] },
    }));
    Value::Array(variants)
}

fn workspace_query_variants(require_calls_root: bool) -> Value {
    let calls_required: &[&str] = if require_calls_root {
        &["filePath", "workspaceRoot", "owner", "method"]
    } else {
        &["filePath", "owner", "method"]
    };
    json!([
        query_variant("find_entities", &["name"]),
        query_variant("forward_edges", &["name"]),
        query_variant("reverse_edges", &["name"]),
        query_variant("entities_in_file", &["file_path"]),
        query_variant("transitive_dependencies", &["name"]),
        query_variant("has_cycle", &[]),
        query_variant("calls_in_file", calls_required)
    ])
}

fn workspace_query_batch_queries() -> Value {
    let mut properties = workspace_query_operation_properties();
    let properties = properties
        .as_object_mut()
        .expect("workspace query properties are an object");
    properties.insert("id".into(), json!({ "type": "string", "minLength": 1 }));
    json!({
        "type": "array",
        "minItems": 1,
        "maxItems": 32,
        "items": {
            "type": "object",
            "properties": properties,
            "oneOf": workspace_query_variants(false),
            "required": ["id"],
            "not": {
                "anyOf": [
                    { "required": ["workspaceRoot"] },
                    { "required": ["withinPath"] }
                ]
            }
        }
    })
}

pub(super) fn workspace_query_batch_results() -> Value {
    json!({
        "type": "array",
        "description": "Ordered outcomes, exactly one per accepted batch item.",
        "items": {
            "type": "object",
            "properties": {
                "id": { "type": "string" },
                "type": { "type": "string" },
                "status": { "type": "string", "enum": ["ok", "error"] },
                "result": { "type": "object", "description": "The corresponding legacy structured result for a successful item." },
                "error": {
                    "type": "object",
                    "properties": {
                        "code": { "type": "integer" },
                        "message": { "type": "string" },
                        "data": {}
                    },
                    "required": ["code", "message"]
                }
            },
            "required": ["id", "type", "status"],
            "oneOf": [
                {
                    "properties": { "status": { "const": "ok" } },
                    "required": ["result"],
                    "not": { "required": ["error"] }
                },
                {
                    "properties": { "status": { "const": "error" } },
                    "required": ["error"],
                    "not": { "required": ["result"] }
                }
            ]
        }
    })
}

fn query_variant(kind: &str, operation_fields: &[&str]) -> Value {
    let mut required = vec!["type"];
    required.extend_from_slice(operation_fields);
    json!({
        "properties": { "type": { "const": kind } },
        "required": required
    })
}
