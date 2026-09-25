//! Portable model-facing workspace-query answer envelope.

use serde_json::{Map, Value, json};

pub(super) const WORKSPACE_QUERY_CONTENT_VERSION: u64 = 1;

fn occurrences(values: &Value) -> Value {
    Value::Array(
        values
            .as_array()
            .into_iter()
            .flatten()
            .enumerate()
            .map(|(occurrence, value)| {
                let mut record = match value {
                    Value::Object(record) => record.clone(),
                    _ => {
                        let mut record = Map::new();
                        record.insert("value".into(), value.clone());
                        record
                    }
                };
                record.insert("occurrence".into(), occurrence.into());
                Value::Object(record)
            })
            .collect(),
    )
}

fn model_result(query_type: &str, structured: &Value) -> Value {
    let mut result = structured.clone();
    let Some(object) = result.as_object_mut() else {
        return result;
    };
    let family = match query_type {
        "find_entities" | "entities_in_file" => Some("entities"),
        "forward_edges" | "reverse_edges" => Some("edges"),
        "transitive_dependencies" => Some("dependencies"),
        _ => None,
    };
    if let Some(family) = family
        && let Some(values) = structured.get(family)
    {
        object.insert(family.into(), occurrences(values));
    }
    result
}

pub(super) fn render(
    query_type: &str,
    args: &Value,
    structured: &Value,
    additional_roots: &[String],
) -> String {
    let count = structured.get("count").and_then(Value::as_u64);
    let file_local_calls = query_type == "calls_in_file";
    let direction = match query_type {
        "forward_edges" => Some("outgoing"),
        "reverse_edges" => Some("incoming"),
        _ => None,
    };
    let mut query = json!({
        "type": query_type,
        "direction": direction,
        "domain": args.get("domain"),
        "entity_type": args.get("entity_type"),
        "name": args.get("name"),
        "file_path": args.get("file_path"),
        "depth": args.get("depth"),
    });
    if file_local_calls {
        let query = query
            .as_object_mut()
            .expect("workspace query descriptor is an object");
        query.insert(
            "file".into(),
            args.get("filePath").cloned().unwrap_or(Value::Null),
        );
        query.insert(
            "owner".into(),
            args.get("owner").cloned().unwrap_or(Value::Null),
        );
        query.insert(
            "method".into(),
            args.get("method").cloned().unwrap_or(Value::Null),
        );
    }
    let envelope = json!({
        "schema": "clean-ctx/workspace-query-answer",
        "schema_version": WORKSPACE_QUERY_CONTENT_VERSION,
        "query": query,
        "scope": {
            "mode": if args.get("workspaceRoot").and_then(Value::as_str).is_some() { "workspace" } else { "session" },
            "workspace_root": args.get("workspaceRoot"),
            "additional_roots": additional_roots,
            "within_path": args.get("withinPath"),
        },
        "completeness": {
            "authority": if file_local_calls { "fresh_canonical_file_ir" } else { "workspace_index_after_registered_hydration" },
            "status": if file_local_calls { "authoritative_file_snapshot" } else { "authoritative_index_snapshot_for_effective_scope" },
            "zero_result": count == Some(0),
            "discovery": structured.get("discovery"),
        },
        "result": model_result(query_type, structured),
    });
    format!(
        "// WORKSPACE-QUERY v{WORKSPACE_QUERY_CONTENT_VERSION}; structuredContent remains authoritative\n{}",
        serde_json::to_string_pretty(&envelope).expect("workspace-query answer is serializable")
    )
}

#[cfg(test)]
#[path = "../../../tests/mcp/workspace_query_content.rs"]
mod tests;
