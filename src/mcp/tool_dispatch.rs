//! Tool argument parsing, registry ownership, and `tools/call` dispatch.

use crate::compression::Fidelity;
use crate::mcp::McpState;
use crate::protocol::send_response;
use crate::tokenizer::{TokenizerKind, resolve_tokenizer_kind};
use serde_json::Value;

use super::tool_handlers;

pub(crate) fn parse_fidelity_arg(
    id: &Value,
    params: &Value,
    config: &crate::config::CleanCtxConfig,
) -> Result<Fidelity, ()> {
    let fidelity_str =
        params["arguments"]["fidelity"]
            .as_str()
            .unwrap_or(match config.default_fidelity {
                Fidelity::Low => "low",
                Fidelity::Medium => "medium",
                Fidelity::High => "high",
                Fidelity::Edit => "edit",
                Fidelity::Verbatim => "verbatim",
            });

    if params["arguments"]["fidelity"].is_null() {
        eprintln!(
            "[clean-ctx] fidelity not specified, using default: {} (from config)",
            fidelity_str
        );
    }

    match Fidelity::parse(fidelity_str) {
        Ok(fidelity) => Ok(fidelity),
        Err(error) => {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32602, "message": error.to_string() }
            }));
            Err(())
        }
    }
}

pub(crate) fn parse_tokenizer_arg(
    params: &Value,
    config: &crate::config::CleanCtxConfig,
) -> TokenizerKind {
    let tool_arg = params["arguments"]["tokenizer"].as_str();
    resolve_tokenizer_kind(tool_arg, Some(&config.tokenizer.to_string()))
}

#[allow(dead_code)]
pub(crate) fn resolve_fidelity(
    explicit: Option<&str>,
    ext: Option<&str>,
    config: &crate::config::CleanCtxConfig,
) -> Fidelity {
    if let Some(value) = explicit
        && let Ok(fidelity) = Fidelity::parse(value)
    {
        return fidelity;
    }
    if let Some(extension) = ext
        && let Some(fidelity) = config.get_fidelity_for_extension(extension)
    {
        return fidelity;
    }
    config.default_fidelity
}

static HANDLER_REGISTRY: std::sync::OnceLock<tool_handlers::registry::HandlerRegistry> =
    std::sync::OnceLock::new();

fn get_registry() -> &'static tool_handlers::registry::HandlerRegistry {
    HANDLER_REGISTRY.get_or_init(tool_handlers::registry::create_default_registry)
}

#[cfg(test)]
pub fn setup_handler_registry_for_tests() {
    let _ = get_registry();
}

#[allow(dead_code)]
pub(crate) fn inline_tool_names() -> std::collections::HashSet<&'static str> {
    use std::collections::HashSet;
    let mut names = HashSet::new();
    names.insert("cargo_check");
    names.insert("graph_search");
    names.insert("graph_query");
    names.insert("graph_trace");
    names.insert("get_architecture");
    names.insert("get_cbm_status");
    names.insert("cbm_proxy");
    names.insert("list_projects");
    names
}

pub(crate) fn dispatch_tools_call(id: &Value, tool_name: &str, params: &Value, state: &McpState) {
    match tool_name {
        "cargo_check" => {
            super::cargo_check::handle(id, params, state);
            return;
        }
        "graph_search" => {
            crate::cbm::handlers::handle_graph_search(id, params, state);
            return;
        }
        "graph_query" => {
            crate::cbm::handlers::handle_graph_query(id, params, state);
            return;
        }
        "graph_trace" => {
            crate::cbm::handlers::handle_graph_trace(id, params, state);
            return;
        }
        "get_architecture" => {
            crate::cbm::handlers::handle_get_architecture(id, params, state);
            return;
        }
        "get_cbm_status" => {
            crate::cbm::handlers::handle_get_cbm_status(id, params, state);
            return;
        }
        "cbm_proxy" => {
            crate::cbm::proxy::handle_cbm_proxy(id, params, state);
            return;
        }
        "list_projects" => {
            crate::cbm::proxy::handle_cbm_proxy(
                id,
                &serde_json::json!({"arguments": {
                    "cbm_tool": "list_projects",
                    "parameters": {}
                }}),
                state,
            );
            return;
        }
        _ => {}
    }

    if let Some(entry) = get_registry().get(tool_name) {
        (entry.handler)(id, params, state);
        return;
    }

    send_response(&serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": -32601, "message": format!("Tool not found: {}", tool_name) }
    }));
}
