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
    let envelope = answer_envelope(query_type, args, structured, additional_roots);
    format!(
        "// WORKSPACE-QUERY v{WORKSPACE_QUERY_CONTENT_VERSION}; structuredContent remains authoritative\n{}",
        serde_json::to_string_pretty(&envelope).expect("workspace-query answer is serializable")
    )
}

fn answer_envelope(
    query_type: &str,
    args: &Value,
    structured: &Value,
    additional_roots: &[String],
) -> Value {
    let count = structured.get("count").and_then(Value::as_u64);
    let file_local_calls = query_type == "calls_in_file";
    let zero_result = if query_type == "has_cycle" {
        structured.get("has_cycle").and_then(Value::as_bool) == Some(false)
    } else {
        count == Some(0)
    };
    let direction = match query_type {
        "forward_edges" => Some("outgoing"),
        "reverse_edges" => Some("incoming"),
        _ => None,
    };
    let coverage = structured.get("coverage");
    let coverage_status = coverage
        .and_then(|value| value.get("status"))
        .and_then(Value::as_str);
    let source_complete = coverage
        .and_then(|value| value.get("source_complete"))
        .and_then(Value::as_bool);
    let mut query = json!({
        "type": query_type,
        "direction": direction,
        "domain": args.get("domain"),
        "entity_type": args.get("entity_type"),
        "name": args.get("name"),
        "file_path": args.get("file_path"),
        "depth": args.get("depth"),
        "kind": args.get("kind"),
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
    let mut completeness = json!({
        "authority": if file_local_calls { "fresh_canonical_file_ir" } else if query_type == "has_cycle" { "workspace_index" } else { "workspace_index_after_registered_hydration" },
        "status": coverage_status.unwrap_or(if file_local_calls { "authoritative_file_snapshot" } else { "authoritative_index_snapshot_for_effective_scope" }),
        "zero_result": zero_result,
        "discovery": structured.get("discovery"),
    });
    if let Some(source_complete) = source_complete {
        completeness["source_complete"] = source_complete.into();
    }
    if let Some(result_semantics) = coverage
        .and_then(|value| value.get("result_semantics"))
        .and_then(Value::as_str)
    {
        completeness["result_semantics"] = result_semantics.into();
    }
    if let Some(omitted_possible) = coverage
        .and_then(|value| value.get("omitted_possible"))
        .and_then(Value::as_bool)
    {
        completeness["omitted_possible"] = omitted_possible.into();
    }
    json!({
        "schema": "clean-ctx/workspace-query-answer",
        "schema_version": WORKSPACE_QUERY_CONTENT_VERSION,
        "query": query,
        "scope": {
            "mode": if args.get("workspaceRoot").and_then(Value::as_str).is_some() { "workspace" } else { "session" },
            "workspace_root": args.get("workspaceRoot"),
            "additional_roots": additional_roots,
            "within_path": args.get("withinPath"),
        },
        "completeness": completeness,
        "result": model_result(query_type, structured),
    })
}

pub(super) fn batch_success_item(
    id: String,
    query_type: &str,
    args: &Value,
    structured: &Value,
    additional_roots: &[String],
) -> Value {
    let answer = answer_envelope(query_type, args, structured, additional_roots);
    json!({
        "id": id,
        "type": query_type,
        "status": "ok",
        "query": answer["query"],
        "completeness": answer["completeness"],
        "result": answer["result"],
    })
}

pub(super) fn render_batch(
    args: &Value,
    results: Vec<Value>,
    additional_roots: &[String],
) -> String {
    let envelope = json!({
        "schema": "clean-ctx/workspace-query-batch-answer",
        "schema_version": 1,
        "scope": {
            "mode": if args.get("workspaceRoot").and_then(Value::as_str).is_some() { "workspace" } else { "session" },
            "workspace_root": args.get("workspaceRoot"),
            "additional_roots": additional_roots,
            "within_path": args.get("withinPath"),
        },
        "results": results,
    });
    format!(
        "// WORKSPACE-QUERY-BATCH v1; structuredContent remains authoritative\n{}",
        serde_json::to_string_pretty(&envelope)
            .expect("workspace-query batch answer is serializable")
    )
}

#[cfg(test)]
#[path = "../../../tests/mcp/workspace_query_content.rs"]
mod tests;
