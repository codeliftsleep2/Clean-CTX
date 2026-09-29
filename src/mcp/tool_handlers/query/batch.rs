//! Request-local orchestration for heterogeneous workspace-query batches.

use super::outcome::{QueryFailure, QueryResult};
use super::prepare::{PreparationContext, PreparedQuery};
use super::request::WorkspaceQueryOperation;
use crate::mcp::McpState;
use serde_json::{Map, Value, json};
use std::collections::HashSet;

const MAX_BATCH_ITEMS: usize = 32;

struct PendingItem {
    id: String,
    query_type: String,
    args: Value,
    prepared: PreparedQuery,
}

struct CompletedItem {
    id: String,
    query_type: String,
    args: Value,
    result: QueryResult,
}

pub(super) fn handle(id: &Value, args: &Value, state: &McpState) {
    let result = try_execute(args, state);
    let response = match result {
        Ok(items) => success_response(id, args, items, &state.config.additional_roots),
        Err(error) => crate::mcp::tool_helpers::jsonrpc_error(
            id.clone(),
            error.code,
            error.message,
            error.data,
        ),
    };
    crate::protocol::send_response(&response);
}

fn try_execute(args: &Value, state: &McpState) -> Result<Vec<CompletedItem>, QueryFailure> {
    if args.get("type").is_some() {
        return Err(QueryFailure::invalid(
            "'type' and 'queries' are mutually exclusive request forms.",
        ));
    }
    let queries = args["queries"]
        .as_array()
        .ok_or_else(|| QueryFailure::invalid("'queries' must be an array."))?;
    if queries.is_empty() {
        return Err(QueryFailure::invalid(
            "'queries' must be a non-empty array.",
        ));
    }
    if queries.len() > MAX_BATCH_ITEMS {
        return Err(QueryFailure::invalid(
            "'queries' may contain at most 32 items.",
        ));
    }
    validate_items(queries)?;

    // The shared scope is validated once, before any item can perform IO,
    // compilation, hydration, or index work.
    let mut context = PreparationContext::new();
    let _ = context.scope(state, args)?;
    let mut pending = Vec::with_capacity(queries.len());
    for item in queries {
        let object = item.as_object().expect("validated batch item object");
        let item_id = object["id"].as_str().expect("validated batch item id");
        let query_type = object
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let item_args = item_arguments(object, args);
        let prepared = match WorkspaceQueryOperation::parse(query_type) {
            Some(operation) => operation.prepare(&item_args, state, &mut context),
            None if query_type.is_empty() => PreparedQuery::failure(QueryFailure::invalid(
                "Missing required argument: 'type' for batch query item.",
            )),
            None => PreparedQuery::failure(QueryFailure::invalid(format!(
                "Unknown query type: '{query_type}'."
            ))),
        };
        pending.push(PendingItem {
            id: item_id.to_string(),
            query_type: query_type.to_string(),
            args: item_args,
            prepared,
        });
    }

    // Every index-backed evaluator observes this one post-preparation guard.
    let index = state.workspace_index_read();
    Ok(pending
        .into_iter()
        .map(|item| CompletedItem {
            id: item.id,
            query_type: item.query_type,
            args: item.args,
            result: item.prepared.evaluate_with_index(&index),
        })
        .collect())
}

fn validate_items(queries: &[Value]) -> Result<(), QueryFailure> {
    let mut ids = HashSet::with_capacity(queries.len());
    for item in queries {
        let object = item
            .as_object()
            .ok_or_else(|| QueryFailure::invalid("Every query item must be an object."))?;
        let id = object
            .get("id")
            .and_then(Value::as_str)
            .filter(|id| !id.is_empty())
            .ok_or_else(|| QueryFailure::invalid("Every query item requires a non-empty 'id'."))?;
        if !ids.insert(id) {
            return Err(QueryFailure::invalid(format!(
                "Batch query item id '{id}' is duplicate."
            )));
        }
        for scope_field in ["workspaceRoot", "withinPath"] {
            if object.contains_key(scope_field) {
                return Err(QueryFailure::invalid(format!(
                    "'{scope_field}' is shared and may not appear in a query item."
                )));
            }
        }
    }
    Ok(())
}

fn item_arguments(item: &Map<String, Value>, batch: &Value) -> Value {
    let mut arguments = item.clone();
    arguments.remove("id");
    for field in ["workspaceRoot", "withinPath"] {
        if let Some(value) = batch.get(field) {
            arguments.insert(field.to_string(), value.clone());
        }
    }
    Value::Object(arguments)
}

fn success_response(
    id: &Value,
    args: &Value,
    items: Vec<CompletedItem>,
    additional_roots: &[String],
) -> Value {
    let mut results = Vec::with_capacity(items.len());
    let mut model_results = Vec::with_capacity(items.len());
    for item in items {
        match item.result {
            Ok(answer) => {
                model_results.push(super::content::batch_success_item(
                    item.id.clone(),
                    &item.query_type,
                    &item.args,
                    &answer.structured,
                    additional_roots,
                ));
                results.push(json!({
                    "id": item.id,
                    "type": item.query_type,
                    "status": "ok",
                    "result": answer.structured,
                }));
            }
            Err(error) => {
                let structured_error = error.structured();
                model_results.push(json!({
                    "id": item.id.clone(),
                    "type": item.query_type.clone(),
                    "status": "error",
                    "error": structured_error.clone(),
                }));
                results.push(json!({
                    "id": item.id,
                    "type": item.query_type,
                    "status": "error",
                    "error": structured_error,
                }));
            }
        }
    }
    let structured = json!({ "batch": true, "results": results });
    let text = super::content::render_batch(args, model_results, additional_roots);
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "content": [{ "type": "text", "text": text }],
            "structuredContent": structured,
        }
    })
}
