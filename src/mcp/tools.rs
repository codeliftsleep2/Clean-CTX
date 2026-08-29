// src/mcp/tools.rs
//
// Tool definitions and dispatch for the MCP server.
// v0.3.0: Registry-based dispatch for modular handlers, fallback to legacy.

use crate::cbm;
use crate::compression::Fidelity;
use crate::mcp::McpState;
use crate::protocol::send_response;
use crate::tokenizer::{TokenizerKind, resolve_tokenizer_kind};
use serde_json::Value;

use super::tool_handlers;

#[cfg(test)]
pub(crate) use super::tool_helpers::diff_code_context_handler;

/// Compute the list of supported languages based on enabled Cargo features.
/// This surfaces to clients which file extensions the binary can actually
/// process, avoiding "unsupported extension" errors for unbuilt grammars.
fn supported_languages() -> Vec<&'static str> {
    let mut langs = Vec::new();
    if cfg!(feature = "typescript") {
        langs.push("typescript");
    }
    if cfg!(feature = "csharp") {
        langs.push("csharp");
    }
    if cfg!(feature = "rust") {
        langs.push("rust");
    }
    if cfg!(feature = "java") {
        langs.push("java");
    }
    langs
}

/// Inject the `supportedLanguages` field into each tool's schema so clients
/// can discover which languages the current binary supports.
fn inject_supported_languages(mut tools: Vec<serde_json::Value>) -> Vec<serde_json::Value> {
    let supported = supported_languages();
    for tool in &mut tools {
        if let Some(obj) = tool.as_object_mut() {
            obj.insert(
                "supportedLanguages".to_string(),
                serde_json::json!(supported),
            );
        }
    }
    tools
}

pub(crate) fn tool_list() -> Vec<serde_json::Value> {
    let tools = vec![
        serde_json::json!({
            "name": "compress_code_context",
            "description": "High-speed local AST compilation, hash-caching, and variable mapping tool.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "filePath": { "type": "string", "description": "Absolute path to .ts, .cs, .rs, or .java file." },
                    "fidelity": { "type": "string", "enum": ["low", "medium", "high", "edit", "verbatim"], "description": "Compression fidelity: 'low' (max compression, ~85% reduction), 'medium' (balanced, preserves fields/async/markers, ~70-80%), 'high' (minimal compression, preserves most semantic depth, ~50-60%), 'edit' (structural skeleton + verbatim method bodies for safe replace_in_file), 'verbatim' (full raw source, zero compression). Default: 'low'." },
                    "encoding": { "type": "string", "description": "IR encoding format: 'named' (standard tuple with opcode strings), 'positional' (stripped opcode ~30% savings), or 'tagged' (positional with opcode preserved). Default: 'named'." },
                    "tokenizer": { "type": "string", "description": "Tokenizer backend for token counting: 'o200k' (GPT-4o, default), 'cl100k' (GPT-4), 'claude' (Anthropic), 'llama3' (Meta). Overrides config default." },
                    "workspaceRoot": { "type": "string", "description": "Optional. Workspace root for path resolution. Defaults to CWD." }
                },
                "required": ["filePath"]
            }
        }),
        serde_json::json!({
            "name": "diff_code_context",
            "description": "AST-level diff compression.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "filePath": { "type": "string", "description": "Absolute path to .ts, .cs, or .rs file." },
                    "fidelity": { "type": "string", "enum": ["low", "medium", "high", "edit", "verbatim"], "description": "Compression fidelity: 'low', 'medium', 'high', 'edit', 'verbatim'. Default: 'low'." },
                    "workspaceRoot": { "type": "string", "description": "Optional. Workspace root for path resolution. Defaults to CWD." }
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
                    "fidelity": { "type": "string", "enum": ["low", "medium", "high", "edit", "verbatim"], "description": "Compression fidelity: 'low', 'medium', 'high', 'edit', 'verbatim'. Default: config default." },
                    "workspaceRoot": { "type": "string", "description": "Optional. Workspace root for path resolution. Defaults to CWD." }
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
                    "intent": { "type": "string", "enum": ["edit", "refactor", "overview", "debug", "implement"], "description": "edit: byte-exact method bodies for safe replace_in_file. refactor: full structural detail. overview: max compression. debug: balanced. implement: moderate detail." },
                    "fidelity": { "type": "string", "enum": ["low", "medium", "high", "edit", "verbatim"], "description": "Compression fidelity: 'low', 'medium', 'high', 'edit' (structural skeleton + verbatim method bodies), 'verbatim' (full raw source). Default: config default." },
                    "focusMethods": { "type": "array", "items": { "type": "string" }, "description": "Optional. When set alongside fidelity: \"edit\", only these method/function names get full verbatim bodies; all other methods in the file are rendered signature-only. Omit to render every method's body (current default behavior)." },
                    "workspaceRoot": { "type": "string", "description": "Optional. Workspace root for path resolution. Defaults to CWD." },
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
                    "fidelity": { "type": "string", "enum": ["low", "medium", "high", "edit", "verbatim"], "description": "Compression fidelity: 'low', 'medium', 'high', 'edit', 'verbatim'. Default: config default." },
                    "workspaceRoot": { "type": "string", "description": "Optional. Workspace root for path resolution. Defaults to CWD." }
                },
                "required": ["filePath"]
            }
        }),
        serde_json::json!({
            "name": "apply_edit",
            "description": "Editor for controlled filesystem edits on the text file at the provided path. Provide `insert_line` to insert `new_text` at a specific line number. Otherwise, the tool replaces `old_text` with `new_text`, or creates the file with `new_text` if file does not exist. Preferred write path for SINGLE-UNIT edits (one method body / insertion anchored to one unit) once this session has seen byte-exact content via provide_code_context(fidelity=\"edit\"|\"verbatim\"): verified against Clean-CTX's tracked unit spans, gated by an in-memory tree-sitter parse before any byte hits disk. Multi-unit batches targeting different units are supported. Cross-file renames/signature changes still belong in the host's native edit tool.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "filePath": { "type": "string", "description": "Absolute path to a previously-seen .ts, .cs, .rs, or .java file." },
                    "operations": {
                        "type": "array",
                        "description": "Structural operations applied atomically (all-or-nothing). Each: {\"type\":\"replace_body\",\"target\":\"Class.method\",\"expectedOldText\":\"{...}\",\"newText\":\"{...}\"} | {\"type\":\"delete\",\"target\":..., \"expectedOldText\":...} | {\"type\":\"insert_after\",\"anchor\":\"Class.method\",\"unitText\":...} | {\"type\":\"insert_before\",...}. expectedOldText must byte-match the text this session last delivered for that unit.",
                        "items": { "type": "object" }
                    },
                    "verify": { "type": "boolean", "description": "Optional. When true, echoes each replacement's new verbatim text back as a receipt. Default false." },
                    "workspaceRoot": { "type": "string", "description": "Optional. Workspace root for path resolution. Defaults to CWD." }
                },
                "required": ["filePath", "operations"]
            },
            "outputSchema": {
                "type": "object",
                "properties": {
                    "operations": {
                        "type": "array",
                        "description": "Per-operation outcomes in request order, measured against the NEW file. All operations apply atomically; this list equals the requested batch on success.",
                        "items": {
                            "type": "object",
                            "properties": {
                                "kind": {
                                    "type": "string",
                                    "enum": ["replace_body", "insert_after", "insert_before", "delete"],
                                    "description": "Applied operation type."
                                },
                                "target": {
                                    "type": "string",
                                    "description": "Targeted unit name (replace_body/delete) or anchor unit (insert_after/insert_before)."
                                },
                                "startByte": {
                                    "type": "integer",
                                    "minimum": 0,
                                    "description": "Absolute start byte affected in the NEW file (insertions: insertion point)."
                                },
                                "endByte": {
                                    "type": "integer",
                                    "minimum": 0,
                                    "description": "Absolute end byte affected in the NEW file (insertions: same as startByte)."
                                },
                                "byteDelta": {
                                    "type": "integer",
                                    "description": "Signed size change contributed by this operation (new minus old; negative for deletes)."
                                },
                                "newText": {
                                    "type": "string",
                                    "description": "Verbatim new text receipt. Present only when verify=true and the operation carries new text (replace_body, insert_after, insert_before); never for delete."
                                }
                            },
                            "required": ["kind", "target", "startByte", "endByte", "byteDelta"]
                        }
                    }
                },
                "required": ["operations"]
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
            "description": "List all persisted contexts stored in the DB — per-file rows with fidelity, token counts, delta count and last-update time.",
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
        serde_json::json!({
            "name": "diff_commits",
            "description": "Diff an entire workspace between two git refs; emits per-file AST-level change-sets in one call.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "workspaceRoot": { "type": "string", "description": "Optional. Defaults to CWD. Resolved against trusted root." },
                    "fromRef": { "type": "string", "description": "Required. e.g. HEAD~1, main, abc123, v1.0. Strictly validated." },
                    "toRef": { "type": "string", "description": "Optional. Defaults to working tree (uncommitted changes)." },
                    "fidelity": { "type": "string", "enum": ["low", "medium", "high", "edit", "verbatim"], "description": "Compression fidelity: 'low', 'medium', 'high', 'edit', 'verbatim'. Default: config default." }
                },
                "required": ["fromRef"]
            }
        }),
    ]
    .into_iter()
    .chain(cbm::cbm_tool_list())
    .collect();
    inject_supported_languages(tools)
}

/// P1-4: Parse fidelity argument from request, falling back to config default.
///
/// Uses the user's configured `default_fidelity` instead of hardcoded "low",
/// ensuring consistency across all tool invocations.
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

    // Log when using default
    if params["arguments"]["fidelity"].is_null() {
        eprintln!(
            "[clean-ctx] fidelity not specified, using default: {} (from config)",
            fidelity_str
        );
    }

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

pub(crate) fn parse_tokenizer_arg(
    params: &Value,
    config: &crate::config::CleanCtxConfig,
) -> TokenizerKind {
    let tool_arg = params["arguments"]["tokenizer"].as_str();
    resolve_tokenizer_kind(tool_arg, Some(&config.tokenizer.to_string()))
}

/// Resolve the effective fidelity for a (explicit_arg, file_extension) pair.
/// Used by tests (`src/tests/mcp/tools.rs`, `src/tests/mcp/tool_handlers.rs`)
/// and kept for potential future dispatch use. `#[allow(dead_code)]` is
/// required because this is only consumed by external test modules.
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
        && let Some(f) = config.get_fidelity_for_extension(e)
    {
        return f;
    }
    config.default_fidelity
}

static HANDLER_REGISTRY: std::sync::OnceLock<tool_handlers::registry::HandlerRegistry> =
    std::sync::OnceLock::new();

fn get_registry() -> &'static tool_handlers::registry::HandlerRegistry {
    HANDLER_REGISTRY.get_or_init(tool_handlers::registry::create_default_registry)
}

// P3-3: Handler registry initialization.
//
// The registry uses OnceLock for lazy initialization - it's created on first
// access rather than at load time. This avoids issues with sanitizers,
// test harnesses, and dynamic linking that #[ctor] can cause.
//
// For tests that need eager initialization (e.g., parallel tests on Windows),
// call `setup_handler_registry_for_tests()` in the test module.

/// P3-3: Force initialization of the handler registry for test setup.
/// Call this in test modules to avoid OnceLock contention during parallel tests.
#[cfg(test)]
pub fn setup_handler_registry_for_tests() {
    let _ = get_registry();
}

/// P1-6: Collect all inline-only tool names for verification.
/// Returns the set of tool names handled by the inline dispatch match arms.
/// Used by tests (`src/tests/mcp/tools.rs`) to verify no tool is registered
/// in both inline and registry. `#[allow(dead_code)]` is required because
/// this is only consumed by the external `src/tests/mcp/tools.rs` module,
/// which the lib build (non-test) never references.
#[allow(dead_code)]
pub(crate) fn inline_tool_names() -> std::collections::HashSet<&'static str> {
    use std::collections::HashSet;
    let mut names = HashSet::new();
    names.insert("graph_search");
    names.insert("graph_query");
    names.insert("graph_trace");
    names.insert("get_architecture");
    names.insert("get_cbm_status");
    names.insert("cbm_proxy");
    names
}

/// Dispatch a tools/call request.
/// v0.3.0: Uses registry-based dispatch for modular handlers, fallback to legacy.
///
/// P1-6: All inline-handled tools have early returns. The remaining tools
/// fall through to the registry. The `inline_tool_names()` function above
/// enables test-time verification that no tool name appears in both paths.
pub(crate) fn dispatch_tools_call(id: &Value, tool_name: &str, params: &Value, state: &McpState) {
    // Inline dispatch for tools that have special handling requirements
    // (decompress, compress_workspace, and all CBM tools).
    // Each arm returns to prevent double-fire if a tool is also registered.
    match tool_name {
        // compress_workspace and decompress_code_context removed in Phase C1.
        // CBM tools
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
            // Route through cbm_proxy with no parameters
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
        // Unknown — fall through to registry
        _ => {}
    }

    // P1-6: Registry fallback for tools not handled inline above.
    if let Some(entry) = get_registry().get(tool_name) {
        (entry.handler)(id, params, state);
        return;
    }

    // Unknown tool
    send_response(&serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": -32601, "message": format!("Tool not found: {}", tool_name) }
    }));
}

#[cfg(test)]
#[path = "../tests/mcp/tools.rs"]
mod tests;

#[cfg(test)]
#[path = "../tests/mcp/tool_contracts.rs"]
mod tool_contracts_tests;
