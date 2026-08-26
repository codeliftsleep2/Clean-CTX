// src/mcp/handlers.rs
//
// JSON-RPC method handlers for the MCP server.
// Each function corresponds to a single method and sends its own response.

use crate::mcp::McpState;
use crate::mcp::cache_hints::{generate_vocabulary_text, inject_cache_breakpoints};
use crate::mcp::prompts;
use crate::mcp::tools;
use crate::protocol::send_response;
use crate::tokenizer::{create_tokenizer, resolve_tokenizer_kind};
use serde_json::Value;

/// Handle `initialize` — returns server capabilities and protocol version.
pub(crate) fn handle_initialize(id: &Value) {
    send_response(&serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "protocolVersion": "2025-11-25",
            "capabilities": { "tools": {}, "prompts": {} },
            "serverInfo": { "name": "clean-ctx", "version": "1.0.0" }
        }
    }));
}

/// Handle `tools/list` — returns the list of available tools with cache hints.
pub(crate) fn handle_tools_list(id: &Value, state: &McpState) {
    let mut response = serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "tools": tools::tool_list()
        }
    });

    // Inject tools cache breakpoint into result._meta, NOT the response root
    let cache_enabled = state.config.cache.enabled;
    let tok_box = create_tokenizer(resolve_tokenizer_kind(
        None,
        Some(&state.config.tokenizer.to_string()),
    ))
    .ok();
    let tok_ref: Option<&dyn crate::tokenizer::Tokenizer> = tok_box.as_deref();
    if cache_enabled {
        let ttl = state.config.cache.tools_ttl.clone();
        let breaker = format!("tools-{}", state.config.cache.tool_defs_version);
        if let Some(result_obj) = response.get_mut("result") {
            inject_cache_breakpoints(result_obj, state, "tools", &ttl, &breaker, tok_ref);
        }
    }

    send_response(&response);
}

/// Handle `prompts/list` — returns the list of available prompts with cache hints.
pub(crate) fn handle_prompts_list(id: &Value, state: &McpState) {
    let mut response = serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "prompts": prompts::prompt_list()
        }
    });

    // Inject system prompt cache breakpoint into result._meta, NOT the response root
    let cache_enabled = state.config.cache.enabled;
    let tok_box = create_tokenizer(resolve_tokenizer_kind(
        None,
        Some(&state.config.tokenizer.to_string()),
    ))
    .ok();
    let tok_ref: Option<&dyn crate::tokenizer::Tokenizer> = tok_box.as_deref();
    if cache_enabled {
        let ttl = state.config.cache.system_prompt_ttl.clone();
        let breaker = format!("vocab-{}", state.config.cache.vocab_version);
        if let Some(result_obj) = response.get_mut("result") {
            inject_cache_breakpoints(result_obj, state, "system_prompt", &ttl, &breaker, tok_ref);
        }
    }

    send_response(&response);
}

/// Handle `prompts/get` — returns the content of a specific prompt.
pub(crate) fn handle_prompts_get(id: &Value, prompt_name: &str, state: &McpState) {
    if prompt_name == "cleanctx-notation" {
        let mut response = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "description": "System instructions for reading and writing Clean-CTX compressed notation",
                "messages": [
                    {
                        "role": "user",
                        "content": {
                            "type": "text",
                            "text": prompts::SYSTEM_PROMPT
                        }
                    }
                ]
            }
        });

        // Inject system prompt cache breakpoint into result._meta, NOT the response root
        let cache_enabled = state.config.cache.enabled;
        let tok_box = create_tokenizer(resolve_tokenizer_kind(
            None,
            Some(&state.config.tokenizer.to_string()),
        ))
        .ok();
        let tok_ref: Option<&dyn crate::tokenizer::Tokenizer> = tok_box.as_deref();
        if cache_enabled {
            let ttl = state.config.cache.system_prompt_ttl.clone();
            let breaker = format!("vocab-{}", state.config.cache.vocab_version);
            if let Some(result_obj) = response.get_mut("result") {
                inject_cache_breakpoints(
                    result_obj,
                    state,
                    "system_prompt",
                    &ttl,
                    &breaker,
                    tok_ref,
                );
            }
        }

        send_response(&response);
    } else if prompt_name == "dashboard" {
        send_response(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "description": "View the Clean-CTX token savings dashboard",
                "messages": [
                    {
                        "role": "user",
                        "content": {
                            "type": "text",
                            "text": concat!(
                                "You have access to the Clean-CTX Dashboard. To view token savings and compression stats:\n\n",
                                "- Call `context_stats` with no arguments to see the full session dashboard.\n",
                                "- Call `context_stats` with a `filePath` to see stats for a specific file.\n",
                                "- Use `format: \"json\"` for structured data, or `format: \"text\"` for human-readable output.\n\n",
                                "The dashboard shows:\n",
                                "- Session duration and file count\n",
                                "- Total raw vs compressed tokens and savings percentage\n",
                                "- Full compression vs delta operation counts\n",
                                "- Per-file breakdown with version history and delta counts\n",
                                "- Delta hit rate (how often deltas were used instead of full re-compression)\n",
                            )
                        }
                    }
                ]
            }
        }));
    } else if prompt_name == "clean-ctx-vocabulary" {
        let mut response = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "description": "Clean-CTX SCHEMA v2 response vocabulary: structure letters, fl:/cl: flags, High-fidelity cf:/df:/se:/ec: metadata, α path aliases and current Φ framework-meta markers.",
                "messages": [
                    {
                        "role": "user",
                        "content": {
                            "type": "text",
                            "text": generate_vocabulary_text()
                        }
                    }
                ]
            }
        });

        // Inject system prompt cache breakpoint into result._meta, NOT the response root
        let cache_enabled = state.config.cache.enabled;
        let tok_box = create_tokenizer(resolve_tokenizer_kind(
            None,
            Some(&state.config.tokenizer.to_string()),
        ))
        .ok();
        let tok_ref: Option<&dyn crate::tokenizer::Tokenizer> = tok_box.as_deref();
        if cache_enabled {
            let ttl = state.config.cache.system_prompt_ttl.clone();
            let breaker = format!("vocab-{}", state.config.cache.vocab_version);
            if let Some(result_obj) = response.get_mut("result") {
                inject_cache_breakpoints(
                    result_obj,
                    state,
                    "system_prompt",
                    &ttl,
                    &breaker,
                    tok_ref,
                );
            }
        }

        send_response(&response);
    } else {
        send_response(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": -32601, "message": format!("Prompt not found: {}", prompt_name) }
        }));
    }
}
