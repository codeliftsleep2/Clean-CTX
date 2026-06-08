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
                    "fidelity": { "type": "string", "description": "Compression fidelity: 'low' (max compression, ~85% reduction), 'medium' (balanced, preserves fields/async/markers, ~70-80%), 'high' (minimal compression, preserves most semantic depth, ~50-60%). Default: 'low'." }
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
                    "currentVersion": { "type": "integer", "description": "The current version of the file state (must match delta.from_version)." }
                },
                "required": ["delta", "currentVersion"]
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
            handle_compress_code_context(id, params, state);
        }
        "decompress_code_context" => {
            let compressed_text = params["arguments"]["compressedText"].as_str().unwrap_or("");

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
                    send_response(&serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "content": [{ "type": "text", "text": result.manifest }],
                            "_meta": {
                                "errors": result.errors.into_iter().map(|(p, e)| {
                                    serde_json::json!({ "path": p, "error": e })
                                }).collect::<Vec<_>>(),
                                "excluded": result.excluded,
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
        "apply_delta" => {
            handle_apply_delta(id, params, state);
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
fn handle_compress_code_context(
    id: &Value,
    params: &Value,
    state: &mut McpState,
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

                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "content": [{ "type": "text", "text": compressed_text }],
                        "ir": ir_to_wire(&ir),
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
                        "content": [{ "type": "text", "text": format!("IR delta: v{} → v{}", delta.from_version, delta.to_version) }],
                        "delta": delta,
                        "file": file_alias,
                        "from": delta.from_version,
                        "to": delta.to_version,
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

// ── Handler: apply_delta (new — client-side state update) ──────────

/// Handle `apply_delta` — applies a delta envelope to the in-session
/// state machine, returning the updated state.
fn handle_apply_delta(
    id: &Value,
    params: &Value,
    state: &mut McpState,
) {
    let delta_val = &params["arguments"]["delta"];
    let current_version = params["arguments"]["currentVersion"].as_u64().unwrap_or(0);

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

    // Validate version chain
    if delta.from_version != current_version {
        send_response(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": -32602,
                "message": format!(
                    "Version mismatch: state is v{}, delta expects v{}",
                    current_version, delta.from_version
                )
            }
        }));
        return;
    }

    // Apply the delta
    match state.ir_context.apply(delta) {
        Ok(new_version) => {
            let file_id = state.ir_context.file_ids().last().cloned().unwrap_or_default();
            let pretty = state.ir_context.render_pretty(&file_id, Fidelity::Low);

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

// ── Helpers ────────────────────────────────────────────────────────

/// Compile a file to IR, detecting language and running the full
/// 4-layer compilation pipeline.
///
/// Phase A (FAANG remediation): The compiler now instantiates the
/// appropriate language layers (TypeScriptLayer, CSharpLayer) and
/// meta layers (AngularMetaLayer) based on the detected language.
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

    let absolute_path = std::fs::canonicalize(&path_buf)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| file_path.to_string());
    let path_alias = state.dict.get_or_create_alias(absolute_path);

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

    // F-03: Pattern recognizers can be added here when wired in
    // For now, pattern recognizers are opt-in (no default ones added)

    let compiled = compiler.compile(
        &source,
        &path_alias,
        language,
        query_string,
        fidelity,
    )?;

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