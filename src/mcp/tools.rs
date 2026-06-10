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

use std::path::PathBuf;
use serde_json::Value;
use crate::compressor::{compress_file, Fidelity};
use crate::decompression::Decompressor;
use crate::diff::{build_snapshot, diff_snapshots, format_diff, diff_summary};
use crate::ir::wire::ir_to_wire;
use crate::ir::delta::{IRDelta, DeltaComputer};
use crate::ir::replay::DeltaError;
use crate::mcp::context_store::ContextStore;
use crate::mcp::McpState;
use crate::mcp::workspace;
use crate::protocol::send_response;

/// Return the list of tool definitions (for `tools/list`).
pub(crate) fn tool_list() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
            "name": "compress_code_context",
            "description": "High-speed local AST compilation, hash-caching, and variable mapping tool.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "filePath": { "type": "string", "description": "Absolute path to .ts or .cs file." },
                    "fidelity": { "type": "string", "description": "Compression fidelity: 'low' (max compression, ~85% reduction), 'medium' (balanced, preserves fields/async/markers, ~70-80%), 'high' (minimal compression, preserves most semantic depth, ~50-60%). Default: 'low'." },
                    "encoding": { "type": "string", "description": "IR encoding format: 'named' (standard tuple with opcode strings), 'positional' (stripped opcode ~30% savings), or 'tagged' (positional with opcode preserved). Default: 'named'." }
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
            "description": "Compresses all TypeScript/C# files in a directory tree. Outputs a manifest of compressed file signatures with shared opcode dictionary.",
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
                    "filePath": { "type": "string", "description": "Absolute path to .ts or .cs file." },
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
                    "filePath": { "type": "string", "description": "Absolute path to .ts or .cs file." },
                    "fidelity": { "type": "string", "description": "Compression fidelity: 'low', 'medium', 'high'. Default: 'low'." }
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
                    "filePath": { "type": "string", "description": "Absolute path to .ts or .cs file." },
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
                    "workspaceRoot": { "type": "string", "description": "Optional workspace root for relative paths." }
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
                    send_response(&serde_json::json!({
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
                    }));
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
        _ => {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32601, "message": format!("Tool not found: {}", tool_name) }
            }));
        }
    }
}

// ── Handler: compress_code_context (upgraded, backward compatible) ──

/// Handle `compress_code_context` — includes `ir` field alongside `pretty`.
///
/// Supports an optional `encoding` parameter:
///   - `"named"` (default): standard tuple format with opcode strings
///   - `"positional"`: stripped opcode format (30%+ savings per the spec)
///   - `"tagged"`: positional with opcode preserved for mixed streams
///
/// F-08 (FAANG audit): the positional encoding was previously unreachable
/// from the MCP surface. This change wires `ir_to_positional_wire` into
/// the production path.
fn handle_compress_code_context(
    id: &Value,
    params: &Value,
    state: &mut McpState,
    encoding: &str,
) {
    let file_path_str = params["arguments"]["filePath"].as_str().unwrap_or("");
    let fidelity = match parse_fidelity_arg(id, params) {
        Ok(f) => f,
        Err(()) => return,
    };

    // F-05: consult `is_excluded` *before* any file I/O.
    if state.config.is_excluded(file_path_str) {
        send_response(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": -32603,
                "message": format!("File excluded by config: {}", file_path_str)
            }
        }));
        return;
    }

    // F-05: extension-based fidelity override (only kicks in
    // when the caller didn't pass an explicit `fidelity`).
    let explicit = params["arguments"]["fidelity"].as_str();
    let path_buf = PathBuf::from(file_path_str);
    let ext = path_buf.extension().and_then(|e| e.to_str());
    let effective_fidelity = if explicit.is_some() {
        fidelity
    } else {
        resolve_fidelity(explicit, ext, &state.config)
    };

    match compress_file(
        PathBuf::from(file_path_str),
        &mut state.dict,
        &mut state.cache,
        effective_fidelity,
    ) {
        Ok(mut compressed_text) => {
            compressed_text.push_str(&state.dict.format_footer());

            // Also compile IR and store in context state
            let ir_result = compile_file_ir(file_path_str, effective_fidelity, state);

            let response = if let Ok(ir) = ir_result {
                // Store the full IR in context state for delta tracking
                state.ir_context.load_ir(ir.clone());

                // F-08: Determine the IR wire format based on the encoding parameter.
                // Positional encoding strips opcode strings for ~30% size reduction.
                let ir_value = match encoding {
                    "positional" => {
                        let config = crate::ir::positional::PositionalConfig::stripped();
                        crate::ir::positional::ir_to_positional_wire(
                            &ir.file_id, ir.version, &ir.instructions, config,
                        )
                    }
                    "tagged" => {
                        let config = crate::ir::positional::PositionalConfig::tagged();
                        crate::ir::positional::ir_to_positional_wire(
                            &ir.file_id, ir.version, &ir.instructions, config,
                        )
                    }
                    _ => ir_to_wire(&ir),
                };

                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "content": [{ "type": "text", "text": compressed_text }],
                        "ir": ir_value,
                        "pretty": compressed_text,
                        "v": ir.version,
                        "file": ir.file_id
                    }
                })
            } else {
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": { "content": [{ "type": "text", "text": compressed_text }] }
                })
            };

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

// ── Handler: diff_code_context (existing, kept for backward compat) ──

/// Handle `diff_code_context` — AST-level text diff (existing).
fn handle_diff_code_context(
    id: &Value,
    params: &Value,
    state: &mut McpState,
) {
    let file_path_str = params["arguments"]["filePath"].as_str().unwrap_or("");
    let fidelity = match parse_fidelity_arg(id, params) {
        Ok(f) => f,
        Err(()) => return,
    };

    // F-05: same exclusion check as `compress_code_context`.
    if state.config.is_excluded(file_path_str) {
        send_response(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": -32603,
                "message": format!("File excluded by config: {}", file_path_str)
            }
        }));
        return;
    }

    match diff_code_context_handler(
        PathBuf::from(file_path_str),
        &mut state.cache,
        fidelity,
    ) {
        Ok(output) => {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": { "content": [{ "type": "text", "text": output }] }
            }));
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

// ── Handler: delta_code_context (new — IR-level delta) ───────────────

/// Handle `delta_code_context` — computes IR-level delta between
/// the file's previous in-session IR state and its current state.
fn handle_delta_code_context(
    id: &Value,
    params: &Value,
    state: &mut McpState,
) {
    let file_path_str = params["arguments"]["filePath"].as_str().unwrap_or("");
    let fidelity = match parse_fidelity_arg(id, params) {
        Ok(f) => f,
        Err(()) => return,
    };

    // Check exclusion
    if state.config.is_excluded(file_path_str) {
        send_response(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": -32603,
                "message": format!("File excluded by config: {}", file_path_str)
            }
        }));
        return;
    }

    // Compile current IR
    let current_ir = match compile_file_ir(file_path_str, fidelity, state) {
        Ok(ir) => ir,
        Err(e) => {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32603, "message": e.to_string() }
            }));
            return;
        }
    };

    // Check if we have a baseline IR in context state
    let file_alias = current_ir.file_id.clone();
    let result = if state.ir_context.has_file(&file_alias) {
        // Get baseline version before loading new IR
        let baseline_version = state.ir_context.file_version(&file_alias).unwrap_or(0);
        let baseline_ir = {
            // Reconstruct baseline CompiledIR from context state
            let instructions = state.ir_context.get_ir(&file_alias).cloned().unwrap_or_default();
            crate::ir::compiler::CompiledIR {
                file_id: file_alias.clone(),
                instructions: instructions.iter()
                    .filter_map(|t| crate::ir::wire::tuple_to_op(t))
                    .collect(),
                version: baseline_version,
            }
        };

        // Now store the new IR as baseline for next call
        state.ir_context.load_ir(current_ir.clone());

        // Compute delta
        let computer = DeltaComputer::new();
        match computer.compute(&baseline_ir, &current_ir) {
            Some(delta) => {
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "content": [{ "type": "text", "text": format!("IR delta: v{} → v{}", delta.from, delta.to) }],
                        "delta": delta,
                        "file": file_alias,
                        "from": delta.from,
                        "to": delta.to,
                        "ops": delta.ops
                    }
                })
            }
            None => {
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "content": [{ "type": "text", "text": format!("No changes (v{}).", current_ir.version) }],
                        "delta": null,
                        "file": file_alias,
                        "v": current_ir.version
                    }
                })
            }
        }
    } else {
        // No baseline — store current IR as baseline
        state.ir_context.load_ir(current_ir.clone());
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "content": [{ "type": "text", "text": format!("No baseline — stored IR (v{}). Call again after editing to see delta.", current_ir.version) }],
                "file": file_alias,
                "v": current_ir.version,
                "ir": ir_to_wire(&current_ir)
            }
        })
    };

    send_response(&result);
}

// ── Handler: delta_text_context (Phase IV — text-level delta) ──────

/// Handle `delta_text_context` — computes text-level delta between
/// the file's previous compressed body snapshot and its current state.
///
/// First call: stores the compressed body as baseline, returns full output.
/// Subsequent calls: computes line-level delta, returns compact §Δ format.
fn handle_delta_text_context(
    id: &Value,
    params: &Value,
    state: &mut McpState,
) {
    let file_path_str = params["arguments"]["filePath"].as_str().unwrap_or("");
    let fidelity = match parse_fidelity_arg(id, params) {
        Ok(f) => f,
        Err(()) => return,
    };

    // Check exclusion
    if state.config.is_excluded(file_path_str) {
        send_response(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": -32603,
                "message": format!("File excluded by config: {}", file_path_str)
            }
        }));
        return;
    }

    // Compress the file to get current compressed body lines
    let path_alias = state.dict.get_or_create_alias(file_path_str.to_string());

    // Run the full compression pipeline to get the body lines
    let result = compress_text_body(file_path_str, fidelity, state);
    let (body_lines, full_output) = match result {
        Ok(r) => r,
        Err(e) => {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32603, "message": e.to_string() }
            }));
            return;
        }
    };

    // Use the text delta computer to compute delta or store baseline
    let delta = state.text_delta.compute_and_store(&path_alias, body_lines);

    let response = match delta {
        Some(d) => {
            // Delta computed — emit compact delta format
            let wire = d.to_wire_format();
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "content": [{ "type": "text", "text": wire }],
                    "delta": {
                        "file": d.file,
                        "from": d.from,
                        "to": d.to,
                        "adds": d.adds,
                        "dels": d.dels,
                        "mods": d.mods.into_iter().map(|(o, n)| {
                            serde_json::json!({"old": o, "new": n})
                        }).collect::<Vec<_>>(),
                    },
                    "format": "text_delta"
                }
            })
        }
        None => {
            // No baseline or no changes — emit full output
            let version = state.text_delta.file_version(&path_alias);
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "content": [{ "type": "text", "text": full_output }],
                    "delta": null,
                    "file": path_alias,
                    "v": version,
                    "format": "full"
                }
            })
        }
    };

    send_response(&response);
}

/// Compress a file and extract the body lines (without header) for
/// delta comparison. Returns `(body_lines, full_output)`.
fn compress_text_body(
    file_path: &str,
    fidelity: Fidelity,
    state: &mut McpState,
) -> Result<(Vec<String>, String), Box<dyn std::error::Error>> {
    use crate::compression::pipeline::{build_output_lines, assemble_body};
    use crate::compression::micro_opcodes::apply_micro_opcodes;
    use crate::compression::symbol_compression::apply_symbol_compression;
    use crate::compaction::{
        compact_expression, extract_class_name, extract_field,
        extract_method_sig,
    };
    use crate::compression::capture_pipeline::run_capture_pipeline;
    use crate::compression::language::language_for_extension;

    let source_code = std::fs::read_to_string(file_path)?;
    let path_buf = std::path::PathBuf::from(file_path);
    let extension = path_buf.extension().and_then(|e| e.to_str()).unwrap_or("");
    let (language, query_string) = language_for_extension(extension)
        .ok_or_else(|| format!("Unsupported file extension: .{}", extension))?;

    let all_captures = run_capture_pipeline(
        language,
        query_string,
        &source_code,
        fidelity,
        |capture_name, raw, f| {
            if capture_name == "class.root" {
                Some(extract_class_name(raw))
            } else if capture_name == "method.root" {
                Some(extract_method_sig(raw, f))
            } else if capture_name == "field.root" {
                Some(extract_field(raw, f))
            } else {
                Some(compact_expression(raw, f))
            }
        },
    )?;

    let built = build_output_lines(&all_captures, &source_code, fidelity);
    let mut body_content = assemble_body(&built.output_lines, fidelity);
    if let Some(block) = &built.meta_block {
        body_content.push_str(&block.render());
    }
    body_content = apply_micro_opcodes(&body_content, fidelity);
    let (display_body, sym_footer) = apply_symbol_compression(&body_content, fidelity);

    // Split the body into lines for delta comparison
    let body_lines: Vec<String> = display_body.lines().map(String::from).collect();

    // Build full output (header + body + symbol footer)
    let path_alias = state.dict.get_or_create_alias(file_path.to_string());
    let compacted_body = crate::compression::report::format_compacted_body(
        &display_body,
        &sym_footer,
        &path_alias,
        fidelity,
    );
    let full_output = crate::compression::report::format_final_output(
        &source_code,
        &compacted_body,
        fidelity,
        built.class_count,
        built.method_count,
        built.import_count,
    );

    Ok((body_lines, full_output))
}

// ── Handler: apply_delta (new — client-side state update) ──────────

/// Handle `apply_delta` — applies a delta envelope to the in-session
/// state machine, returning the updated state.
fn handle_apply_delta(
    id: &Value,
    params: &Value,
    state: &mut McpState,
) {
    let delta_val = &params["arguments"]["delta"];

    // Parse the delta from the JSON params
    let delta: IRDelta = match serde_json::from_value(delta_val.clone()) {
        Ok(d) => d,
        Err(e) => {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32602, "message": format!("Invalid delta format: {}", e) }
            }));
            return;
        }
    };

    // NF-03 + NF-10: Extract the file ID before the delta is consumed by apply().
    let delta_file_id = delta.file.clone();

    // Apply the delta.
    // NF-04: Removed redundant version check — ContextState::apply is the
    // single source of truth for version validation. The previous manual
    // check produced -32602 while apply produces -32603 for the same
    // condition. The 'currentVersion' parameter is still accepted for
    // backward compatibility but is no longer validated here.
    match state.ir_context.apply(delta) {
        Ok(new_version) => {
            // NF-03 + NF-10: Use delta's file ID instead of file_ids().last()
            // file_ids() returns HashMap keys — non-deterministic in multi-file sessions.
            let pretty = state.ir_context.render_pretty(&delta_file_id, Fidelity::Low);

            send_response(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "ok": true,
                    "newVersion": new_version,
                    "pretty": pretty.unwrap_or_default()
                }
            }));
        }
        Err(DeltaError::UnknownFile(file)) => {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32603,
                    "message": format!("Unknown file: {}. Use compress_code_context first to load the IR baseline.", file)
                }
            }));
        }
        Err(DeltaError::VersionMismatch { expected, got }) => {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32603,
                    "message": format!("Version mismatch: state is v{}, delta expects v{}", expected, got)
                }
            }));
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

// ── Zero-Touch Workflow Handlers ─────────────────────────────────

/// Handle `provide_code_context` — the unified entry point.
///
/// Orchestrates heuristics, compression/delta, Angular detection, and
/// stats recording. Delegates to existing handlers internally.
fn handle_provide_code_context(
    id: &Value,
    params: &Value,
    state: &mut McpState,
) {
    let file_path_str = params["arguments"]["filePath"].as_str().unwrap_or("");
    let explicit_intent = params["arguments"]["intent"].as_str();
    let explicit_fidelity = params["arguments"]["fidelity"].as_str();
    let workspace_root = params["arguments"]["workspaceRoot"].as_str();

    if file_path_str.is_empty() {
        send_response(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": -32602, "message": "Missing required parameter: filePath" }
        }));
        return;
    }

    // Resolve absolute path
    let resolved_path = resolve_file_path(file_path_str, workspace_root);

    // Check exclusion
    if state.config.is_excluded(&resolved_path) {
        send_response(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": -32603,
                "message": format!("File excluded by config: {}", file_path_str)
            }
        }));
        return;
    }

    // Read source for heuristics
    let source = match std::fs::read_to_string(&resolved_path) {
        Ok(s) => s,
        Err(e) => {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32603, "message": format!("Failed to read file: {}", e) }
            }));
            return;
        }
    };

    // Run heuristics engine
    let decision = crate::mcp::heuristics::decide(
        &resolved_path,
        explicit_fidelity,
        explicit_intent,
        &state.config,
        &state.text_delta,
        &state.ir_context,
        &source,
    );

    // Get path alias for delta tracking
    let path_alias = state.dict.get_or_create_alias(resolved_path.clone());

    // Execute based on strategy
    match decision.strategy {
        crate::mcp::heuristics::ContextStrategy::FullCompress => {
            // Full compression via existing pipeline
            match compress_file(
                PathBuf::from(&resolved_path),
                &mut state.dict,
                &mut state.cache,
                decision.fidelity,
            ) {
                Ok(mut compressed_text) => {
                    compressed_text.push_str(&state.dict.format_footer());

                    // Store text delta baseline
                    let body_lines: Vec<String> = compressed_text.lines().map(String::from).collect();
                    state.text_delta.compute_and_store(&path_alias, body_lines);

                    // Compile IR and store baseline
                    let ir_result = compile_file_ir(&resolved_path, decision.fidelity, state);
                    if let Ok(ir) = ir_result {
                        state.ir_context.load_ir(ir);
                    }

                    // Record stats
                    let raw_tokens = estimate_tokens(&source);
                    let compressed_tokens = estimate_tokens(&compressed_text);
                    state.session_stats.record_compression(
                        &resolved_path,
                        raw_tokens,
                        compressed_tokens,
                        &format!("{:?}", decision.fidelity).to_lowercase(),
                        decision.is_angular,
                        "full",
                    );

                    send_response(&serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "content": [{ "type": "text", "text": compressed_text }],
                            "_meta": {
                                "fidelity": format!("{:?}", decision.fidelity).to_lowercase(),
                                "strategy": "full_compress",
                                "angular_detected": decision.is_angular,
                                "line_count": decision.source_line_count,
                                "version": state.text_delta.file_version(&path_alias),
                                "decision_summary": decision.summary(),
                            }
                        }
                    }));
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
        crate::mcp::heuristics::ContextStrategy::DeltaTransport => {
            // Delta transport: delegate to delta_text_context logic
            let body_lines_result = compress_text_body(&resolved_path, decision.fidelity, state);
            let (body_lines, full_output) = match body_lines_result {
                Ok(r) => r,
                Err(e) => {
                    send_response(&serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": { "code": -32603, "message": e.to_string() }
                    }));
                    return;
                }
            };

            let delta = state.text_delta.compute_and_store(&path_alias, body_lines);

            // Also compile IR for delta tracking
            if let Ok(ir) = compile_file_ir(&resolved_path, decision.fidelity, state) {
                state.ir_context.load_ir(ir);
            }

            let (output_text, is_delta) = match &delta {
                Some(d) => (d.to_wire_format(), true),
                None => (full_output, false),
            };

            // Record stats
            let raw_tokens = estimate_tokens(&source);
            let compressed_tokens = estimate_tokens(&output_text);
            let strategy_label = if is_delta { "delta" } else { "full" };
            state.session_stats.record_compression(
                &resolved_path,
                raw_tokens,
                compressed_tokens,
                &format!("{:?}", decision.fidelity).to_lowercase(),
                decision.is_angular,
                strategy_label,
            );

            send_response(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "content": [{ "type": "text", "text": output_text }],
                    "_meta": {
                        "fidelity": format!("{:?}", decision.fidelity).to_lowercase(),
                        "strategy": if is_delta { "delta" } else { "full" },
                        "angular_detected": decision.is_angular,
                        "line_count": decision.source_line_count,
                        "version": state.text_delta.file_version(&path_alias),
                        "is_delta": is_delta,
                        "decision_summary": decision.summary(),
                    }
                }
            }));
        }
    }
}

/// Handle `restore_context` — forces full re-compression, clears baselines.
fn handle_restore_context(
    id: &Value,
    params: &Value,
    state: &mut McpState,
) {
    let file_path_str = params["arguments"]["filePath"].as_str().unwrap_or("");
    let fidelity_str = params["arguments"]["fidelity"].as_str();

    if file_path_str.is_empty() {
        send_response(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": -32602, "message": "Missing required parameter: filePath" }
        }));
        return;
    }

    let fidelity = match fidelity_str {
        Some(s) => match Fidelity::parse(s) {
            Ok(f) => f,
            Err(e) => {
                send_response(&serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": { "code": -32602, "message": e.to_string() }
                }));
                return;
            }
        },
        None => Fidelity::parse_or_default(&state.config.default_fidelity),
    };

    // Resolve path (no workspaceRoot arg, so just CWD-join for relative)
    let resolved_path = resolve_file_path(file_path_str, None);

    // Check exclusion against resolved path (consistent with provide_code_context)
    if state.config.is_excluded(&resolved_path) {
        send_response(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": -32603,
                "message": format!("File excluded by config: {}", file_path_str)
            }
        }));
        return;
    }

    // Clear baselines — overwrite text delta with empty baseline and clear IR/file store
    let path_alias = state.dict.get_or_create_alias(file_path_str.to_string());
    // TextDeltaComputer doesn't have clear_file, so we store an empty snapshot
    // to overwrite any existing baseline.
    state.text_delta.store_snapshot(&path_alias, Vec::new());
    state.ir_context.remove_file(&path_alias);
    state.context_store.clear_file(file_path_str);

    // Full re-compression
    match compress_file(
        PathBuf::from(file_path_str),
        &mut state.dict,
        &mut state.cache,
        fidelity,
    ) {
        Ok(mut compressed_text) => {
            compressed_text.push_str(&state.dict.format_footer());

            // Re-store baseline
            let body_lines: Vec<String> = compressed_text.lines().map(String::from).collect();
            state.text_delta.compute_and_store(&path_alias, body_lines);

            // Re-compile IR
            if let Ok(ir) = compile_file_ir(file_path_str, fidelity, state) {
                state.ir_context.load_ir(ir);
            }

            send_response(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "content": [{ "type": "text", "text": compressed_text }],
                    "_meta": {
                        "fidelity": format!("{:?}", fidelity).to_lowercase(),
                        "strategy": "restore",
                        "baselines_cleared": true,
                    }
                }
            }));
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

/// Handle `context_history` — shows version/delta history for tracked files.
fn handle_context_history(
    id: &Value,
    params: &Value,
    state: &mut McpState,
) {
    let file_path = params["arguments"]["filePath"].as_str();

    if let Some(fp) = file_path {
        // Specific file
        let path_alias = state.dict.get_or_create_alias(fp.to_string());
        let version = state.text_delta.file_version(&path_alias);
        let has_ir = state.ir_context.has_file(&path_alias);
        let store_meta = state.context_store.load_latest(fp).ok().flatten();

        let mut lines = Vec::new();
        lines.push(format!("File: {}", fp));
        lines.push(format!("  Text Delta Versions: {}", version));
        lines.push(format!("  IR Baseline: {}", if has_ir { "yes" } else { "no" }));
        lines.push(format!("  Context Store: {}", if store_meta.is_some() { "yes" } else { "no" }));
        if let Some(meta) = store_meta {
            lines.push(format!("  Context Version: {}", meta.version));
        }

        send_response(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "content": [{ "type": "text", "text": lines.join("\n") }]
            }
        }));
    } else {
        // All tracked files — show from session_stats
        let stats = &state.session_stats;
        let file_stats = stats.all_file_stats();
        if file_stats.is_empty() {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "content": [{ "type": "text", "text": "No tracked files yet. Call `provide_code_context` first." }]
                }
            }));
            return;
        }

        let mut output = String::from("Tracked Files:\n");
        for (path, fstats) in file_stats {
            output.push_str(&format!(
                "  {} — v{}, {} deltas, {:.1}% savings\n",
                path, fstats.version, fstats.delta_count, fstats.savings_pct
            ));
        }

        send_response(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "content": [{ "type": "text", "text": output }]
            }
        }));
    }
}

/// Handle `context_stats` — dashboard (text or JSON).
fn handle_context_stats(
    id: &Value,
    params: &Value,
    state: &mut McpState,
) {
    let file_path = params["arguments"]["filePath"].as_str();
    let format = params["arguments"]["format"].as_str().unwrap_or("text");

    if let Some(fp) = file_path {
        // Stats for specific file
        let stats = state.session_stats.file_stats(fp);
        match stats {
            Some(fs) => {
                if format == "json" {
                    send_response(&serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "content": [{ "type": "text", "text": serde_json::to_string_pretty(fs).unwrap_or_default() }]
                        }
                    }));
                } else {
                    let text = format!(
                        "File: {}\n  Raw: {} → Compressed: {} ({:.1}% savings)\n  Version: {}, Deltas: {}, Fidelity: {}\n  Angular: {}, Strategy: {}",
                        fs.file_path,
                        fs.raw_tokens,
                        fs.compressed_tokens,
                        fs.savings_pct,
                        fs.version,
                        fs.delta_count,
                        fs.fidelity,
                        fs.is_angular,
                        fs.strategy,
                    );
                    send_response(&serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "content": [{ "type": "text", "text": text }]
                        }
                    }));
                }
            }
            None => {
                send_response(&serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "content": [{ "type": "text", "text": format!("No stats for file: {}", fp) }]
                    }
                }));
            }
        }
    } else {
        // Full dashboard
        if format == "json" {
            let json = crate::mcp::session_stats::render_dashboard_json(&state.session_stats);
            send_response(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "content": [{ "type": "text", "text": serde_json::to_string_pretty(&json).unwrap_or_default() }]
                }
            }));
        } else {
            let text = crate::mcp::session_stats::render_dashboard_text(&state.session_stats);
            send_response(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "content": [{ "type": "text", "text": text }]
                }
            }));
        }
    }
}

/// Resolve a file path, handling relative paths with optional workspace root.
fn resolve_file_path(path: &str, workspace_root: Option<&str>) -> String {
    let path_obj = std::path::Path::new(path);
    if path_obj.is_absolute() {
        path.to_string()
    } else if let Some(root) = workspace_root {
        let root_path = std::path::Path::new(root);
        if root_path.is_absolute() {
            root_path.join(path).to_string_lossy().into_owned()
        } else {
            // workspace root is also relative — join with CWD
            let cwd = std::env::current_dir().unwrap_or_default();
            cwd.join(root).join(path).to_string_lossy().into_owned()
        }
    } else {
        // Relative path with no workspace root — join with CWD
        let cwd = std::env::current_dir().unwrap_or_default();
        cwd.join(path).to_string_lossy().into_owned()
    }
}

/// Rough token estimation (chars / 4).
/// In production, use tiktoken-rs; this is a lightweight approximation.
fn estimate_tokens(text: &str) -> usize {
    text.len() / 4
}

// ── Helpers ────────────────────────────────────────────────────────

/// Compile a file to IR, detecting language and running the full
/// 4-layer compilation pipeline.
///
/// Phase A (FAANG remediation): The compiler now instantiates the
/// appropriate language layers (TypeScriptLayer, CSharpLayer) and
/// meta layers (AngularMetaLayer) based on the detected language.
///
/// NF-02 fix: The version is set based on the previous version in the
/// context state, ensuring a monotonic version chain across successive
/// `delta_code_context` calls. If the file was previously tracked at
/// version N, the new compiled IR gets version N+1. If untracked,
/// version starts at 1.
///
/// NF-01 fix: The consumptive `CompressingPatternRecognizer` is wired
/// into the compile path *after* the additive `CodePatternRecognizer`,
/// so flags are emitted first, then patterns are consumed for wire-size
/// reduction. This enables the Phase H 30% compression on edits.
fn compile_file_ir(
    file_path: &str,
    fidelity: Fidelity,
    state: &mut McpState,
) -> Result<crate::ir::compiler::CompiledIR, Box<dyn std::error::Error>> {
    use crate::ir::compiler::IRCompiler;
    use crate::ir::layers::typescript::TypeScriptLayer;
    use crate::ir::layers::csharp::CSharpLayer;
    use crate::ir::layers::angular::AngularMetaLayer;
    use crate::compression::language::language_for_extension;

    let source = std::fs::read_to_string(file_path)?;
    let path_buf = PathBuf::from(file_path);
    let extension = path_buf.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    let (language, query_string) = language_for_extension(extension)
        .ok_or_else(|| format!("Unsupported file extension: .{}", extension))?;

    // F-FULL-10: Use raw path for alias key for deterministic results.
    // Canonicalize is still performed for the `α alias: <path>` footer
    // display, but the alias key itself uses the raw path.
    let path_alias = state.dict.get_or_create_alias(file_path.to_string());

    // NF-02: Determine the next version based on the previous context state
    let prev_version = state.ir_context.file_version(&path_alias).unwrap_or(0);

    let mut compiler = IRCompiler::new();

    // Add language-specific layers (Layer 2)
    match extension {
        "ts" | "js" => {
            compiler.add_language_layer(Box::new(TypeScriptLayer::new()));
        }
        "cs" => {
            compiler.add_language_layer(Box::new(CSharpLayer::new()));
        }
        _ => {}
    }

    // Add Angular meta layer (Layer 3) for TypeScript files
    if extension == "ts" || extension == "js" {
        compiler.add_meta_layer(Box::new(AngularMetaLayer::new()));
    }

    // F-07 (FAANG audit): Wire the additive CodePatternRecognizer into
    // the compile path. This is the Layer 4 additive recognizer that
    // emits CTOR/OBSERVABLE/GETTER/SETTER flags alongside the original
    // instructions. The recognizer is always-on because it adds context
    // without removing any instructions (zero regression).
    compiler.add_pattern_recognizer(Box::new(
        crate::ir::layers::patterns::CodePatternRecognizer::new(),
    ));

    // NF-01: Wire the consumptive CompressingPatternRecognizer *after*
    // the additive recognizer. This enables the Phase H 30% compression
    // on edits by consuming recognised patterns into single PAT ops.
    // The additive recognizer's flags (CTOR/OBSERVABLE/GETTER/SETTER)
    // are emitted first, then the consumptive recognizer collapses them
    // where possible. This ordering ensures maximum compression.
    compiler.add_pattern_recognizer(Box::new(
        crate::ir::patterns::CompressingPatternRecognizer::new(),
    ));

    let mut compiled = compiler.compile(
        &source,
        &path_alias,
        language,
        query_string,
        fidelity,
    )?;

    // NF-02: Override the version with the next monotonic value.
    // The compiler always sets version=1; we fix it here.
    compiled.version = prev_version.saturating_add(1);

    Ok(compiled)
}

/// Compute an AST-level diff between the file's in-session baseline and
/// its current on-disk state.
///
/// F-21 (FAANG audit): before calling the expensive `build_snapshot`,
/// the handler hashes the source and checks if a baseline exists *and*
/// the hash matches. On match, it returns a "no changes" message
/// without re-parsing the file with tree-sitter.
fn diff_code_context_handler(
    file: PathBuf,
    cache: &mut crate::cache::LocalStateCache,
    fidelity: Fidelity,
) -> Result<String, Box<dyn std::error::Error>> {
    let absolute_path = match std::fs::canonicalize(&file) {
        Ok(p) => p.to_string_lossy().into_owned(),
        Err(_) => file.to_string_lossy().into_owned(),
    };
    let cache_key = format!("{}::{}", absolute_path, fidelity as u8);

    let source = std::fs::read_to_string(&file)?;

    // F-21: hash the source content and check if the baseline is
    // still valid before paying for the expensive tree-sitter parse.
    let source_hash = cache.compute_hash(source.as_bytes());
    if let Some(stored_hash) = cache.get_baseline_hash(&cache_key)
        && stored_hash == source_hash
        && let Some(baseline_snap) = cache.get_baseline(&cache_key).cloned()
    {
        // Content is byte-identical to the stored baseline — no
        // structural changes possible.
        let class_count = baseline_snap.classes.len();
        return Ok(format!(
            "// --- AST Diff ---\n// No changes since last snapshot ({} classes).\n// Hash: {}",
            class_count, &source_hash[..12],
        ));
    }

    let current = build_snapshot(&source, fidelity)?;

    let baseline = cache.get_baseline(&cache_key).cloned();
    let body = match baseline {
        None => {
            let class_count = current.classes.len();
            // F-21: store the hash BEFORE `store_baseline` takes
            // ownership of `cache_key`.
            cache.store_baseline_hash(&cache_key, &source_hash);
            cache.store_baseline(cache_key, current);
            format!(
                "// --- AST Diff ---\n// No baseline snapshot for this file yet.\n// Current state stored as baseline ({} classes).\n// Call diff_code_context again after the file changes to see the delta.",
                class_count
            )
        }
        Some(baseline_snap) => {
            let actions = diff_snapshots(&baseline_snap, &current);
            let (added, removed, modified, unchanged) = diff_summary(&actions);
            let header = format!(
                "// --- AST Diff: {} ---\n// +{} -{} ~{} ={} (classes/methods/fields/imports)\n",
                absolute_path, added, removed, modified, unchanged
            );
            let body = format_diff(&actions, fidelity);
            cache.store_baseline_hash(&cache_key, &source_hash);
            cache.store_baseline(cache_key, current);
            format!("{}{}", header, body)
        }
    };
    Ok(body)
}

#[cfg(test)]
#[path = "../tests/mcp/tools.rs"]
mod tests;
