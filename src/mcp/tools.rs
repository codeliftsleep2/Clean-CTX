// src/mcp/tools.rs
//
// Tool definitions and dispatch for the MCP server.
// v0.3.0: Registry-based dispatch for modular handlers, fallback to legacy.

use serde_json::Value;
use crate::compressor::Fidelity;
use crate::decompression::Decompressor;
use crate::mcp::McpState;
use crate::mcp::workspace;
use crate::mcp::cache_hints::{inject_cache_breakpoints, compute_workspace_breaker};
use crate::protocol::send_response;
use crate::tokenizer::{TokenizerKind, resolve_tokenizer_kind};
use crate::cbm;

use super::tool_handlers;

#[cfg(test)]
pub(crate) use super::tool_helpers::diff_code_context_handler;

pub(crate) fn tool_list() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
            "name": "compress_code_context",
            "description": "High-speed local AST compilation, hash-caching, and variable mapping tool.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "filePath": { "type": "string", "description": "Absolute path to .ts, .cs, .rs, or .java file." },
                    "fidelity": { "type": "string", "description": "Compression fidelity: 'low' (max compression, ~85% reduction), 'medium' (balanced, preserves fields/async/markers, ~70-80%), 'high' (minimal compression, preserves most semantic depth, ~50-60%). Default: 'low'." },
                    "encoding": { "type": "string", "description": "IR encoding format: 'named' (standard tuple with opcode strings), 'positional' (stripped opcode ~30% savings), or 'tagged' (positional with opcode preserved). Default: 'named'." },
                    "tokenizer": { "type": "string", "description": "Tokenizer backend for token counting: 'o200k' (GPT-4o, default), 'cl100k' (GPT-4), 'claude' (Anthropic), 'llama3' (Meta). Overrides config default." }
                },
                "required": ["filePath"]
            }
        }),
        serde_json::json!({
            "name": "decompress_code_context",
            "description": "Expands a compressed structural skeleton back into human-readable format.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "compressedText": { "type": "string", "description": "The compressed output from compress_code_context to expand." }
                },
                "required": ["compressedText"]
            }
        }),
        serde_json::json!({
            "name": "compress_workspace",
            "description": "Compresses all TypeScript, C#, and Rust files in a directory tree.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "directoryPath": { "type": "string", "description": "Absolute path to the project directory to scan." },
                    "fidelity": { "type": "string", "description": "Compression fidelity: 'low', 'medium', 'high'. Default: 'low'." }
                },
                "required": ["directoryPath"]
            }
        }),
        serde_json::json!({
            "name": "diff_code_context",
            "description": "AST-level diff compression.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "filePath": { "type": "string", "description": "Absolute path to .ts, .cs, or .rs file." },
                    "fidelity": { "type": "string", "description": "Compression fidelity: 'low', 'medium', 'high'. Default: 'low'." }
                },
                "required": ["filePath"]
            }
        }),
        serde_json::json!({
            "name": "delta_code_context",
            "description": "IR-level delta compression.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "filePath": { "type": "string" },
                    "fidelity": { "type": "string" },
                    "workspaceRoot": { "type": "string" }
                },
                "required": ["filePath"]
            }
        }),
        serde_json::json!({
            "name": "delta_text_context",
            "description": "Text-level delta compression.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "filePath": { "type": "string" },
                    "fidelity": { "type": "string" }
                },
                "required": ["filePath"]
            }
        }),
        serde_json::json!({
            "name": "apply_delta",
            "description": "Applies an IR delta envelope to the in-session state machine.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "delta": { "type": "object" },
                    "currentVersion": { "type": "integer" }
                },
                "required": ["delta", "currentVersion"]
            }
        }),
        serde_json::json!({
            "name": "provide_code_context",
            "description": "Automatically provides the best possible compressed context for a file.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "filePath": { "type": "string" },
                    "intent": { "type": "string", "enum": ["edit", "refactor", "overview", "debug", "implement"] },
                    "fidelity": { "type": "string" },
                    "workspaceRoot": { "type": "string" },
                    "tokenizer": { "type": "string" }
                },
                "required": ["filePath"]
            }
        }),
        serde_json::json!({
            "name": "restore_context",
            "description": "Explicitly restores compressed context for a file.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "filePath": { "type": "string" },
                    "fidelity": { "type": "string" }
                },
                "required": ["filePath"]
            }
        }),
        serde_json::json!({
            "name": "context_history",
            "description": "View compression history and savings for tracked files.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "filePath": { "type": "string" }
                }
            }
        }),
        serde_json::json!({
            "name": "save_context",
            "description": "Explicitly save current in-memory context to the persistence DB.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "filePath": { "type": "string" }
                }
            }
        }),
        serde_json::json!({
            "name": "list_sessions",
            "description": "List all persistence sessions stored in the DB.",
            "inputSchema": { "type": "object", "properties": {} }
        }),
        serde_json::json!({
            "name": "replay_history",
            "description": "Replay deltas from the DB for a file up to a specific edit sequence.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "filePath": { "type": "string" },
                    "targetSequence": { "type": "integer" },
                    "fidelity": { "type": "string" }
                },
                "required": ["filePath"]
            }
        }),
        serde_json::json!({
            "name": "purge_old_deltas",
            "description": "Purge old delta history from the persistence DB.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "days": { "type": "integer" },
                    "filePath": { "type": "string" }
                }
            }
        }),
        serde_json::json!({
            "name": "context_stats",
            "description": "View the Clean-CTX dashboard: token savings, compression stats, and session metrics.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "filePath": { "type": "string" },
                    "format": { "type": "string", "enum": ["text", "json"] }
                }
            }
        }),
    ]
    .into_iter()
    .chain(cbm::cbm_tool_list())
    .collect()
}

pub(crate) fn parse_fidelity_arg(id: &Value, params: &Value) -> Result<Fidelity, ()> {
    let fidelity_str = params["arguments"]["fidelity"].as_str().unwrap_or("low");
    match Fidelity::parse(fidelity_str) {
        Ok(f) => Ok(f),
        Err(e) => {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32602, "message": e.to_string() }
            }));
            Err(())
        }
    }
}

pub(crate) fn parse_tokenizer_arg(params: &Value, config: &crate::config::CleanCtxConfig) -> TokenizerKind {
    let tool_arg = params["arguments"]["tokenizer"].as_str();
    resolve_tokenizer_kind(tool_arg, Some(&config.tokenizer.to_string()))
}

/// Resolve the effective fidelity for a (explicit_arg, file_extension) pair.
/// Currently unused in the new dispatch but kept for potential future use.
#[allow(dead_code)]
pub(crate) fn resolve_fidelity(
    explicit: Option<&str>,
    ext: Option<&str>,
    config: &crate::config::CleanCtxConfig,
) -> Fidelity {
    if let Some(s) = explicit
        && let Ok(f) = Fidelity::parse(s)
    {
        return f;
    }
    if let Some(e) = ext
        && let Some(s) = config.get_fidelity_for_extension(e)
    {
        return Fidelity::parse_or_default(s);
    }
    Fidelity::parse_or_default(&config.default_fidelity)
}

static HANDLER_REGISTRY: std::sync::OnceLock<tool_handlers::registry::HandlerRegistry> = std::sync::OnceLock::new();

fn get_registry() -> &'static tool_handlers::registry::HandlerRegistry {
    HANDLER_REGISTRY.get_or_init(|| {
        tool_handlers::registry::create_default_registry()
    })
}

/// Dispatch a tools/call request.
/// v0.3.0: Uses registry-based dispatch for modular handlers, fallback to legacy.
pub(crate) fn dispatch_tools_call(
    id: &Value,
    tool_name: &str,
    params: &Value,
    state: &McpState,
) {
    // PRIORITY 1: Direct inline dispatch for tools that are not in the registry
    // (decompress_code_context, compress_workspace, and all CBM tools).
    // These have no registry entry so they MUST be handled here.
    // Each arm returns to prevent double-fire if a tool is also registered.
    match tool_name {
        "decompress_code_context" => {
            let compressed_text = params["arguments"]["compressedText"].as_str().unwrap_or("");
            const MAX_DECOMPRESS_BYTES: usize = 4 * 1024 * 1024;
            if compressed_text.len() > MAX_DECOMPRESS_BYTES {
                send_response(&serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": { "code": -32603, "message": format!("compressedText too large: {} bytes (max {}).", compressed_text.len(), MAX_DECOMPRESS_BYTES) }
                }));
                return;
            }
            let mut decompressor = Decompressor::new();
            let decompressed = decompressor.quick_decompress(compressed_text);
            send_response(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": { "content": [{ "type": "text", "text": decompressed }] }
            }));
            return;
        }
        "compress_workspace" => {
            let dir_path = params["arguments"]["directoryPath"].as_str().unwrap_or(".");
            let fidelity = match parse_fidelity_arg(id, params) {
                Ok(f) => f,
                Err(()) => return,
            };
            match workspace::compress_workspace_dir(dir_path, fidelity, state) {
                Ok(result) => {
                    let mut response = serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "content": [{ "type": "text", "text": result.manifest }],
                            "_meta": {
                                "errors": result.errors.into_iter().map(|(p, e)| serde_json::json!({ "path": p, "error": e })).collect::<Vec<_>>(),
                                "excluded": result.excluded.into_iter().map(|(p, patterns)| serde_json::json!({ "path": p, "matched_patterns": patterns })).collect::<Vec<_>>(),
                                "warnings": result.warnings,
                            }
                        }
                    });
                    if state.config.cache.enabled {
                        let ttl = state.config.cache.baseline_ttl.clone();
                        let breaker = compute_workspace_breaker(std::slice::from_ref(&result.manifest));
                        let tok_box = crate::tokenizer::create_tokenizer(
                            crate::tokenizer::resolve_tokenizer_kind(None, Some(&state.config.tokenizer.to_string()))
                        ).ok();
                        let tok_ref: Option<&dyn crate::tokenizer::Tokenizer> = tok_box.as_deref();
                        if let Some(result_obj) = response.get_mut("result") {
                            inject_cache_breakpoints(result_obj, state, "baseline", &ttl, &breaker, tok_ref);
                        }
                    }
                    send_response(&response);
                }
                Err(e) => {
                    send_response(&serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": { "code": -32603, "message": e.to_string() }
                    }));
                }
            }
            return;
        }
        // CBM tools
        "graph_search" => { crate::cbm::handlers::handle_graph_search(id, params, state); return; }
        "graph_query" => { crate::cbm::handlers::handle_graph_query(id, params, state); return; }
        "graph_trace" => { crate::cbm::handlers::handle_graph_trace(id, params, state); return; }
        "get_architecture" => { crate::cbm::handlers::handle_get_architecture(id, params, state); return; }
        "get_cbm_status" => { crate::cbm::handlers::handle_get_cbm_status(id, params, state); return; }
        "cbm_proxy" => { crate::cbm::proxy::handle_cbm_proxy(id, params, state); return; }
        // Unknown — fall through to registry
        _ => {}
    }

    // PRIORITY 2: Registry fallback for tools not handled inline above.
    // The registry contains all handlers from tool_handlers/core.rs,
    // tool_handlers/context/, tool_handlers/persistence/, tool_handlers/stats/.
    if let Some(entry) = get_registry().get(tool_name) {
        (entry.handler)(id, params, state);
        return;
    }

    // PRIORITY 3: Unknown tool
    send_response(&serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": -32601, "message": format!("Tool not found: {}", tool_name) }
    }));
}

#[cfg(test)]
#[path = "../tests/mcp/tools.rs"]
mod tests;
