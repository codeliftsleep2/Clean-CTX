//! Ordered, failure-isolated orchestration for batched code-context requests.

use super::evaluate;
use super::outcome::{ProvideFailure, ProvideResult};
use crate::mcp::McpState;
use crate::mcp::tool_helpers::{inject_baseline_breakpoint, resolve_file_path_checked};
use serde_json::{Map, Value, json};
use std::collections::HashSet;
use std::fmt::Write;

const MAX_BATCH_ITEMS: usize = 8;
const NO_SUCCESS_CONTENT: &str =
    "No context items succeeded; inspect structuredContent.results for item errors.";

struct CompletedItem {
    id: String,
    outcome: ProvideResult,
}

pub(super) fn response(id: &Value, args: &Value, state: &McpState) -> Value {
    match execute(args, state) {
        Ok(items) => success_response(id, items, state),
        Err(error) => error.response(id),
    }
}

fn execute(args: &Value, state: &McpState) -> Result<Vec<CompletedItem>, ProvideFailure> {
    if args.get("filePath").is_some() {
        return Err(ProvideFailure::invalid(
            "'filePath' and 'files' are mutually exclusive request forms.",
        ));
    }
    let files = args["files"]
        .as_array()
        .ok_or_else(|| ProvideFailure::invalid("'files' must be an array."))?;
    if files.is_empty() {
        return Err(ProvideFailure::invalid(
            "'files' must be a non-empty array.",
        ));
    }
    if files.len() > MAX_BATCH_ITEMS {
        return Err(ProvideFailure::invalid(
            "'files' may contain at most 8 items.",
        ));
    }
    validate_items(files)?;

    let mut canonical_files = HashSet::with_capacity(files.len());
    let mut completed = Vec::with_capacity(files.len());
    for item in files {
        let object = item.as_object().expect("validated context item object");
        let item_id = object["id"]
            .as_str()
            .expect("validated context item id")
            .to_string();
        let params = item_params(object, args);
        let outcome = if let Some(key) = canonical_key(&params, state) {
            if canonical_files.insert(key) {
                evaluate::evaluate(&params, state)
            } else {
                Err(ProvideFailure::invalid(
                    "A later batch item resolves to a duplicate canonical file; combine selectors in one focusMethods array.",
                ))
            }
        } else {
            evaluate::evaluate(&params, state)
        };
        completed.push(CompletedItem {
            id: item_id,
            outcome,
        });
    }
    Ok(completed)
}

fn validate_items(files: &[Value]) -> Result<(), ProvideFailure> {
    let mut ids = HashSet::with_capacity(files.len());
    for item in files {
        let object = item
            .as_object()
            .ok_or_else(|| ProvideFailure::invalid("Every file item must be an object."))?;
        let id = object
            .get("id")
            .and_then(Value::as_str)
            .filter(|id| !id.is_empty())
            .ok_or_else(|| ProvideFailure::invalid("Every file item requires a non-empty 'id'."))?;
        if !ids.insert(id) {
            return Err(ProvideFailure::invalid(format!(
                "Batch file item id '{id}' is duplicate."
            )));
        }
        for shared in ["workspaceRoot", "tokenizer"] {
            if object.contains_key(shared) {
                return Err(ProvideFailure::invalid(format!(
                    "'{shared}' is shared and may not appear in a file item."
                )));
            }
        }
    }
    Ok(())
}

fn item_params(item: &Map<String, Value>, batch: &Value) -> Value {
    let mut arguments = item.clone();
    arguments.remove("id");
    for shared in ["workspaceRoot", "tokenizer"] {
        if let Some(value) = batch.get(shared) {
            arguments.insert(shared.to_string(), value.clone());
        }
    }
    json!({ "arguments": arguments })
}

fn canonical_key(params: &Value, state: &McpState) -> Option<String> {
    let file_path = params["arguments"]["filePath"].as_str()?;
    let workspace_root = params["arguments"]["workspaceRoot"].as_str();
    let resolved =
        resolve_file_path_checked(file_path, workspace_root, &state.config.additional_roots)
            .ok()?;
    Some(crate::dictionary::path::canonical_identity_key(&resolved))
}

fn success_response(id: &Value, items: Vec<CompletedItem>, state: &McpState) -> Value {
    let mut content = Vec::new();
    let mut results = Vec::with_capacity(items.len());
    let mut cache_material = String::new();
    for item in items {
        match item.outcome {
            Ok(context) => {
                let content_index = content.len();
                let _ = write!(cache_material, "{}:", context.text.len());
                cache_material.push_str(&context.text);
                let content_block = json!({ "type": "text", "text": context.text });
                content.push(content_block.clone());
                results.push(json!({
                    "id": item.id,
                    "status": "ok",
                    "content_index": content_index,
                    "content": [content_block],
                    "meta": context.meta,
                }));
            }
            Err(error) => results.push(json!({
                "id": item.id,
                "status": "error",
                "error": error.structured(),
            })),
        }
    }
    if content.is_empty() {
        cache_material.push_str(NO_SUCCESS_CONTENT);
        content.push(json!({ "type": "text", "text": NO_SUCCESS_CONTENT }));
    }
    let mut response = json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "content": content,
            "structuredContent": {
                "batch": true,
                "results": results,
            }
        }
    });
    inject_baseline_breakpoint(&mut response, state, &cache_material);
    response
}
