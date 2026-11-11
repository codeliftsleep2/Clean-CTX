//! Explicit PCG read tool: closed request adapter, existing M3 projector, exact-byte sender.
use crate::{
    mcp::McpState,
    pcg::{Error, projection, query},
    protocol,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::PathBuf;

pub(crate) fn descriptor() -> Value {
    json!({"name":"pcg_query_v1",
        "description":"Explicit read-only evidence context from the owner-enrolled local PCG corpus. Requires exact generation, scope, effective time, tokenizer and work/token limits. Approval remains source-attributed. No automatic injection or source hydration.",
        "inputSchema":serde_json::from_str::<Value>(include_str!("pcg/schema.json")).expect("tracked PCG v1 schema")})
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Arguments {
    workspace_root: PathBuf,
    corpus: String,
    generation: String,
    mode: query::Mode,
    subject: query::Subject,
    records: Option<Vec<String>>,
    tokenizer: String,
    include_history: bool,
    limits: Limits,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Limits {
    records: u64,
    edges: u64,
    evidence_bytes: u64,
    token_budget: u64,
}

fn parse(params: &Value) -> Result<(query::Request, String, u64), Error> {
    let raw = params.get("arguments").ok_or(Error("invalid_request"))?;
    let a: Arguments = serde_json::from_value(raw.clone()).map_err(|_| Error("invalid_request"))?;
    for field in ["file", "entity", "operation", "task"] {
        if raw["subject"].get(field).is_some_and(Value::is_null) {
            return Err(Error("invalid_request"));
        }
    }
    if a.mode == query::Mode::Constraints {
        if raw.get("records").is_some() {
            return Err(Error("invalid_request"));
        }
    } else if a.records.as_ref().is_none_or(Vec::is_empty) {
        return Err(Error("invalid_request"));
    }
    if !["cl100k", "o200k"].contains(&a.tokenizer.as_str()) {
        return Err(Error("unsupported_tokenizer"));
    }
    if [
        a.limits.records,
        a.limits.edges,
        a.limits.evidence_bytes,
        a.limits.token_budget,
    ]
    .contains(&0)
    {
        return Err(Error("invalid_request"));
    }
    Ok((
        query::Request {
            workspace_root: a.workspace_root,
            corpus: a.corpus,
            generation: a.generation,
            mode: a.mode,
            subject: a.subject,
            records: a.records.unwrap_or_default(),
            include_history: a.include_history,
            limits: query::Limits {
                records: a.limits.records,
                edges: a.limits.edges,
                evidence_bytes: a.limits.evidence_bytes,
            },
        },
        a.tokenizer,
        a.limits.token_budget,
    ))
}

fn invoke(params: &Value, state: &McpState) -> Result<projection::Response, Error> {
    let (request, tokenizer, budget) = parse(params)?;
    let guard = state
        .pcg
        .documents
        .as_ref()
        .map(|documents| documents.guard(&request.workspace_root))
        .transpose()?
        .flatten();
    let response = state
        .pcg
        .session
        .as_ref()
        .ok_or(Error("pcg_disabled"))?
        .project(&request, &tokenizer, budget)?;
    if let Some(guard) = guard {
        guard.complete()?;
    }
    Ok(response)
}

pub(crate) fn handle(id: &Value, params: &Value, state: &McpState) {
    match invoke(params, state) {
        Ok(response) => {
            protocol::send_pcg_result(id, &response);
        }
        Err(error) => {
            let (code, message) = match error.0 {
                "token_budget_too_small" => (-32602, "token_budget_too_small"),
                "unsupported_tokenizer" => (-32602, "unsupported_tokenizer"),
                "invalid_request" | "scope_refused" | "request_limits" | "history_denied"
                | "invalid_id" | "invalid_hash" | "invalid_time" | "invalid_entity"
                | "invalid_roots" | "duplicate_id" | "unsafe_path" | "invalid_path" => {
                    (-32602, "pcg_invalid_request")
                }
                _ => (-32603, "pcg_control_unavailable"),
            };
            protocol::send_response(
                &json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":message}}),
            );
        }
    }
}

#[cfg(test)]
#[path = "../../tests/mcp/pcg_query_v1.rs"]
mod tests;
