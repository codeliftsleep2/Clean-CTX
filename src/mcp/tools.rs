// src/mcp/tools.rs
//
// Tool definitions and dispatch for the MCP server.
//
// Phase 1 (FAANG audit F-03): the three `Fidelity::parse` call sites
// used to silently down-typo'd input to `Low`. They now use
// [`parse_fidelity_arg`], which returns a `-32602 Invalid params`
// JSON-RPC error on unrecognised input.
//
// Phase 2 (FAANG audit F-05): the dispatcher now takes `&mut McpState`
// (which bundles the path dict, cache, and project config) instead of
// separate dict/cache arguments. Tool handlers consult the user's
// `exclude_patterns` (via `is_excluded`) and `fidelity_overrides`
// (via `get_fidelity_for_extension`) before any file I/O.
//
// Phase G: Integration & MCP Tools — wires the IR system into the MCP
// tool interface. `compress_code_context` now includes IR output,
// `delta_code_context` computes instruction-level deltas, and
// `apply_delta` allows clients to update state incrementally.
//
// Phase 1 Module Split: handlers extracted to `tool_handlers.rs`,
// shared helpers extracted to `tool_helpers.rs`.
//
// Phase 1 (workspace cache): compress_workspace now injects a baseline
// cache breakpoint keyed on a SHA-256 hash of the manifest, so the
// entire workspace scan result is cacheable.

use serde_json::Value;
use crate::compressor::Fidelity;
use crate::decompression::Decompressor;
use crate::mcp::McpState;
use crate::mcp::workspace;
use crate::mcp::cache_hints::{inject_cache_breakpoints, compute_workspace_breaker};
use crate::protocol::send_response;
use crate::tokenizer::{TokenizerKind, resolve_tokenizer_kind};
use crate::cbm;

// Re-import handlers from sibling modules
use super::tool_handlers::*;
// Re-export helper for test access (tests use `use super::*`)
#[cfg(test)]
pub(crate) use super::tool_helpers::diff_code_context_handler;

/// Return the list of tool definitions (for `tools/list`).
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
            "description": "Expands a compressed structural skeleton back into human-readable format. Reverses opcodes ($c→class), path aliases, and behavior markers (⊕guard→'// conditional branch').",
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
            "description": "Compresses all TypeScript, C#, and Rust files in a directory tree. Outputs a manifest of compressed file signatures with shared opcode dictionary.",
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
            "description": "AST-level diff compression. Returns only the structural deltas (added/removed/modified classes, methods, fields, imports) between the file's previous in-session snapshot and its current state. First call with no baseline stores the snapshot; subsequent calls emit a compact change-set using + / - / ~ / = markers.",
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
            "description": "IR-level delta compression. Returns only the structural deltas (added/removed/modified instructions) between the file's previous in-session IR state and its current state, using the structured delta envelope format. First call with no baseline stores the IR; subsequent calls emit a compact delta with + / ~ / - operations.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "filePath": { "type": "string", "description": "Path to the source file." },
                    "fidelity": { "type": "string", "description": "Compression fidelity: 'low', 'medium', 'high'. Default: 'low'." },
                    "workspaceRoot": { "type": "string", "description": "Optional workspace root for relative paths." }
                },
                "required": ["filePath"]
            }
        }),
        serde_json::json!({
            "name": "delta_text_context",
            "description": "Text-level delta compression. Returns line-level deltas (+added/-removed/~modified) between the file's previous compressed body snapshot and its current state. First call stores the snapshot; subsequent calls emit a compact delta instead of full re-compression. Saves 70-90% on edit sessions.",
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
            "name": "apply_delta",
            "description": "Applies an IR delta envelope to the in-session state machine, incrementally updating the tracked IR state without re-compressing. Returns the updated state and re-rendered pretty output.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "delta": {
                        "type": "object",
                        "description": "The IR delta envelope from delta_code_context",
                        "properties": {
                            "file": { "type": "string" },
                            "from": { "type": "integer" },
                            "to": { "type": "integer" },
                            "ops": {
                                "type": "object",
                                "properties": {
                                    "+": { "type": "array", "items": { "type": "array" } },
                                    "~": { "type": "array", "items": { "type": "object" } },
                                    "-": { "type": "array", "items": { "type": "array" } }
                                }
                            }
                        }
                    },
                    "currentVersion": { "type": "integer", "description": "The current version of the file state (must match delta.from)." }
                },
                "required": ["delta", "currentVersion"]
            }
        }),
        // ── Zero-Touch Workflow: provide_code_context ─────────────────
        serde_json::json!({
            "name": "provide_code_context",
            "description": "Automatically provides the best possible compressed context for a file. This is the RECOMMENDED single entry point for any file-related coding task. First call performs full compression; subsequent calls automatically use delta transport for minimal token usage. Auto-detects Angular files and enables Meta-Layer with Φ markers. Chooses optimal fidelity based on file characteristics and intent. Use this tool for ANY coding task involving code context.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "filePath": { "type": "string", "description": "Path to the source file." },
                    "intent": { "type": "string", "description": "Optional intent: 'edit', 'refactor', 'overview', 'debug', 'implement'. Controls fidelity selection.", "enum": ["edit", "refactor", "overview", "debug", "implement"] },
                    "fidelity": { "type": "string", "description": "Optional explicit fidelity override: 'low', 'medium', 'high'. Overrides intent-based selection." },
                    "workspaceRoot": { "type": "string", "description": "Optional workspace root for relative paths." },
                    "tokenizer": { "type": "string", "description": "Tokenizer backend for token counting: 'o200k' (GPT-4o, default), 'cl100k' (GPT-4), 'claude' (Anthropic), 'llama3' (Meta). Overrides config default." }
                },
                "required": ["filePath"]
            }
        }),
        // ── Zero-Touch Workflow: restore_context ─────────────────────
        serde_json::json!({
            "name": "restore_context",
            "description": "Explicitly restores compressed context for a file. Forces full re-compression from on-disk source, clearing any in-memory delta baselines and context store entries. Use when you need a guaranteed fresh context state.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "filePath": { "type": "string", "description": "Path to the source file." },
                    "fidelity": { "type": "string", "description": "Compression fidelity: 'low', 'medium', 'high'." }
                },
                "required": ["filePath"]
            }
        }),
        // ── Zero-Touch Workflow: context_history ────────────────────
        serde_json::json!({
            "name": "context_history",
            "description": "View compression history and savings for tracked files. Shows per-file version count, delta hit rate, and estimated token savings.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "filePath": { "type": "string", "description": "Optional: specific file. If omitted, shows all tracked files." }
                }
            }
        }),
        // ── Persistence: save_context ─────────────────────────────
        serde_json::json!({
            "name": "save_context",
            "description": "Explicitly save current in-memory context to the persistence DB. Useful for manual checkpointing before risky edits.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "filePath": { "type": "string", "description": "Optional: specific file to save. If omitted, saves all tracked files." }
                }
            }
        }),
        // ── Persistence: list_sessions ────────────────────────────
        serde_json::json!({
            "name": "list_sessions",
            "description": "List all persistence sessions stored in the DB. Shows workspace roots, active contexts, and last active timestamps.",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        }),
        // ── Persistence: replay_history ───────────────────────────
        serde_json::json!({
            "name": "replay_history",
            "description": "Replay deltas from the DB for a file up to a specific edit sequence. Useful for recovering state after a crash.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "filePath": { "type": "string", "description": "Path to the source file." },
                    "targetSequence": { "type": "integer", "description": "Optional: replay up to this edit sequence. If omitted, replays all." },
                    "fidelity": { "type": "string", "description": "Optional: output fidelity. Default: 'low'." }
                },
                "required": ["filePath"]
            }
        }),
        // ── Persistence: purge_old_deltas ─────────────────────────
        serde_json::json!({
            "name": "purge_old_deltas",
            "description": "Purge old delta history from the persistence DB. Use to free space or trim history.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "days": { "type": "integer", "description": "Delete deltas older than this many days. Default: 30." },
                    "filePath": { "type": "string", "description": "Optional: specific file to purge. If omitted, purges all files." }
                }
            }
        }),
        // ── Zero-Touch Workflow: context_stats (dashboard) ──────────
        serde_json::json!({
            "name": "context_stats",
            "description": "View the Clean-CTX dashboard: token savings, compression stats, and session metrics. Shows per-file breakdown and session summary. Use this to monitor compression efficiency at any time.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "filePath": { "type": "string", "description": "Optional: specific file to show stats for. If omitted, shows full session dashboard." },
                    "format": { "type": "string", "description": "Output format: 'text' (human-readable, default) or 'json' (structured).", "enum": ["text", "json"] }
                }
            }
        }),
    ]
    .into_iter()
    .chain(cbm::cbm_tool_list())
    .collect()
}

/// Parse the `fidelity` arg from a `tools/call` params object. On
/// success, returns the resolved `Fidelity`. On a parse error, sends
/// a `-32602 Invalid params` JSON-RPC response with the parse error
/// message and returns `Err(())` — the caller MUST then `return` from
/// the dispatch.
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

/// Parse the `tokenizer` arg from a `tools/call` params object.
///
/// R-19: resolves the tokenizer kind from the tool argument and
/// config default. Returns the resolved `TokenizerKind`.
pub(crate) fn parse_tokenizer_arg(params: &Value, config: &crate::config::CleanCtxConfig) -> TokenizerKind {
    let tool_arg = params["arguments"]["tokenizer"].as_str();
    resolve_tokenizer_kind(tool_arg, Some(&config.tokenizer.to_string()))
}

/// Resolve the effective fidelity for a `(explicit_arg, file_extension)`
/// pair, consulting the project config for an extension override.
///
/// F-05: the resolution order is:
///   1. The explicit `fidelity` arg (if present and valid).
///   2. `config.fidelity_overrides[ext]` (if present and valid).
///   3. The config's `default_fidelity` (if present and valid).
///   4. `Fidelity::Low` (hard fallback).
///
/// Steps 2 and 3 use `parse_or_default` (not `parse`) because a
/// typo in `.clean-ctx.json` should not be a hard error — it just
/// falls back to Low with a stderr warning.
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

/// Dispatch a `tools/call` request for the given tool name.
///
/// F-05: takes `&mut McpState` instead of separate dict/cache args.
pub(crate) fn dispatch_tools_call(
    id: &Value,
    tool_name: &str,
    params: &Value,
    state: &mut McpState,
) {
    match tool_name {
        "compress_code_context" => {
            let encoding = params["arguments"]["encoding"].as_str().unwrap_or("named");
            handle_compress_code_context(id, params, state, encoding);
        }
        "decompress_code_context" => {
            let compressed_text = params["arguments"]["compressedText"].as_str().unwrap_or("");

            // F-FULL-11: Validate compressedText length before processing.
            // The MCP server's line limit caps the request to ~16 MB, but
            // a string within that bound can still be large enough to cause
            // memory pressure. Return a clean error for oversized input.
            const MAX_DECOMPRESS_BYTES: usize = 4 * 1024 * 1024; // 4 MB
            if compressed_text.len() > MAX_DECOMPRESS_BYTES {
                send_response(&serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32603,
                        "message": format!(
                            "compressedText too large: {} bytes (max {}).",
                            compressed_text.len(), MAX_DECOMPRESS_BYTES
                        )
                    }
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
        }
        "compress_workspace" => {
            let dir_path = params["arguments"]["directoryPath"].as_str().unwrap_or(".");
            let fidelity = match parse_fidelity_arg(id, params) {
                Ok(f) => f,
                Err(()) => return,
            };

            match workspace::compress_workspace_dir(dir_path, fidelity, state) {
                Ok(result) => {
                    // F-13: the WorkspaceResult carries the manifest
                    // plus structured errors/excluded lists. We send
                    // the manifest as the primary text content; the
                    // errors are surfaced as a separate JSON field so
                    // MCP clients can inspect them programmatically.
                    //
                    // F-FINAL-04: `excluded` is now `Vec<(String, Vec<String>)>`
                    // — `(path, matching_patterns)` — so MCP clients can
                    // debug a misconfigured exclude list.
                    //
                    // F-FINAL-06: `warnings` is the per-session warning
                    // buffer (duplicate class names, etc.) so MCP
                    // clients can surface non-fatal anomalies.
                    //
                    // Phase 1: Inject a workspace-level baseline cache
                    // breakpoint keyed on a SHA-256 hash of the manifest.
                    let mut response = serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "content": [{ "type": "text", "text": result.manifest }],
                            "_meta": {
                                "errors": result.errors.into_iter().map(|(p, e)| {
                                    serde_json::json!({ "path": p, "error": e })
                                }).collect::<Vec<_>>(),
                                "excluded": result.excluded.into_iter().map(|(p, patterns)| {
                                    serde_json::json!({
                                        "path": p,
                                        "matched_patterns": patterns,
                                    })
                                }).collect::<Vec<_>>(),
                                "warnings": result.warnings,
                            }
                        }
                    });

                    // Inject workspace baseline cache breakpoint into result._meta, NOT the response root
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
        }
        "diff_code_context" => {
            handle_diff_code_context(id, params, state);
        }
        "delta_code_context" => {
            handle_delta_code_context(id, params, state);
        }
        "delta_text_context" => {
            handle_delta_text_context(id, params, state);
        }
        "apply_delta" => {
            handle_apply_delta(id, params, state);
        }
        // ── Zero-Touch Workflow dispatch ─────────────────────────
        "provide_code_context" => {
            handle_provide_code_context(id, params, state);
        }
        "restore_context" => {
            handle_restore_context(id, params, state);
        }
        "context_history" => {
            handle_context_history(id, params, state);
        }
        "context_stats" => {
            handle_context_stats(id, params, state);
        }
        // ── Persistence tool dispatch ─────────────────────────
        "save_context" => {
            handle_save_context(id, params, state);
        }
        "list_sessions" => {
            handle_list_sessions(id, params, state);
        }
        "replay_history" => {
            handle_replay_history(id, params, state);
        }
        "purge_old_deltas" => {
            handle_purge_old_deltas(id, params, state);
        }
        // ── CBM Integration tool dispatch (Phase 1) ──────────────
        // Handlers live in crate::cbm::handlers — self-contained module.
        "graph_search" => {
            crate::cbm::handlers::handle_graph_search(id, params, state);
        }
        "graph_query" => {
            crate::cbm::handlers::handle_graph_query(id, params, state);
        }
        "graph_trace" => {
            crate::cbm::handlers::handle_graph_trace(id, params, state);
        }
        "get_architecture" => {
            crate::cbm::handlers::handle_get_architecture(id, params, state);
        }
        "get_cbm_status" => {
            crate::cbm::handlers::handle_get_cbm_status(id, params, state);
        }
        // ── Phase 2: Pipe-level interception proxy ──────────
        "cbm_proxy" => {
            crate::cbm::proxy::handle_cbm_proxy(id, params, state);
        }
        _ => {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32601, "message": format!("Tool not found: {}", tool_name) }
            }));
        }
    }
}

#[cfg(test)]
#[path = "../tests/mcp/tools.rs"]
mod tests;