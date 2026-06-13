// src/mcp/tool_handlers.rs
//
// All tool handler functions for the MCP server.
// Extracted from tools.rs during the Phase 1 module split.
//
// Each handler receives (id, params, state) and sends a JSON-RPC
// response via `send_response`. Handlers are called from
// `dispatch_tools_call` in `tools.rs`.

use std::path::PathBuf;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::SystemTime;
use serde_json::Value;

/// Write debug log to .clean-ctx/debug.log (anchored to project root).
fn debug_log(msg: impl AsRef<str>) {
    let log_path = crate::mcp::server::find_project_root()
        .join(".clean-ctx")
        .join("debug.log");
    if let Some(parent) = log_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(mut f) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        let _ = writeln!(f, "[{:?}] {}", SystemTime::now(), msg.as_ref());
    }
}
use crate::compressor::{compress_file, Fidelity};
use crate::compression::pipeline::compress_file_with_source;
use crate::ir::wire::ir_to_wire;
use crate::ir::delta::{IRDelta, DeltaComputer};
use crate::ir::replay::DeltaError;
use sha2::Digest;
use crate::mcp::McpState;
use crate::mcp::context_store::ContextStore;
use crate::protocol::send_response;

use super::tools::{parse_fidelity_arg, resolve_fidelity, parse_tokenizer_arg};
use super::tool_helpers::{compress_text_body, compile_file_ir, resolve_file_path, estimate_tokens, diff_code_context_handler, count_tokens_with_tokenizer};

#[cfg(test)]
#[path = "../tests/mcp/tool_handlers.rs"]
mod tests;

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
pub(super) fn handle_compress_code_context(
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

    // Finding F: Pre-read source into source_cache so compile_file_ir
    // gets a cache hit instead of a redundant disk read.
    let source_arc = state.read_source(file_path_str).ok();
    let source_ref = source_arc.as_ref().map(|s| s.as_str());
    // Source text for stats recording (empty fallback if read failed)
    let source_text = source_ref.unwrap_or("");

    match compress_file_with_source(
        PathBuf::from(file_path_str),
        source_ref,
        &mut state.dict,
        &mut state.cache,
        effective_fidelity,
    ) {
        Ok(mut compressed_text) => {
            compressed_text.push_str(&state.dict.format_footer());

            // Also compile IR and store in context state
            let ir_result = compile_file_ir(file_path_str, effective_fidelity, state);

    // R-19: Resolve tokenizer from tool arg + config
    let tokenizer_kind = parse_tokenizer_arg(params, &state.config);
    let tokenizer_box = crate::tokenizer::create_tokenizer(tokenizer_kind)
        .ok();
    let tokenizer_ref: Option<&dyn crate::tokenizer::Tokenizer> = tokenizer_box.as_deref();

    // Record stats using pluggable tokenizer (R-19)
    let raw_tokens = count_tokens_with_tokenizer(source_text, tokenizer_ref);
    let compressed_tokens = count_tokens_with_tokenizer(&compressed_text, tokenizer_ref);
            state.session_stats.record_compression(
                file_path_str,
                raw_tokens,
                compressed_tokens,
                &format!("{:?}", effective_fidelity).to_lowercase(),
                false,
                "full",
            );

            // Persistence hook: save baseline context (with or without IR)
            debug_log(format!("handle_compress: persist_store={}", state.persistence_store.is_some()));
            if let Some(store) = &mut state.persistence_store {
                use sha2::Digest;
                let source_hash = sha2::Sha256::digest(source_text.as_bytes());
                let hash_hex = format!("{:x}", source_hash);
                let ir_binary = if let Ok(ref ir) = ir_result {
                    Some(crate::ir::binary_wire::encode(ir))
                } else {
                    None
                };
                debug_log(format!("handle_compress: calling save_context for {}", file_path_str));
                match store.save_context(
                    file_path_str,
                    effective_fidelity,
                    &compressed_text,
                    ir_binary.as_deref(),
                    &hash_hex,
                ) {
                    Ok(ctx_id) => debug_log(format!("handle_compress: save_context OK id={}", ctx_id)),
                    Err(e) => debug_log(format!("handle_compress: save_context FAILED: {e}")),
                }
            } else {
                debug_log("handle_compress: persist_store is None, skipping");
            }

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
pub(super) fn handle_diff_code_context(
    id: &Value,
    params: &Value,
    state: &mut McpState,
) {
    let file_path_str = params["arguments"]["filePath"].as_str().unwrap_or("");
    let fidelity = match parse_fidelity_arg(id, params) {
        Ok(f) => f,
        Err(()) => return,
    };

    // Resolve path for consistency with other handlers
    let resolved_path = resolve_file_path(file_path_str, None);

    // F-05: same exclusion check as `compress_code_context`.
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

    match diff_code_context_handler(
        PathBuf::from(&resolved_path),
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
pub(super) fn handle_delta_code_context(
    id: &Value,
    params: &Value,
    state: &mut McpState,
) {
    let file_path_str = params["arguments"]["filePath"].as_str().unwrap_or("");
    let workspace_root = params["arguments"]["workspaceRoot"].as_str();
    let fidelity = match parse_fidelity_arg(id, params) {
        Ok(f) => f,
        Err(()) => return,
    };

    if file_path_str.is_empty() {
        send_response(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": -32602, "message": "Missing required parameter: filePath" }
        }));
        return;
    }

    // Resolve absolute path (using workspaceRoot if provided)
    let resolved_path = resolve_file_path(file_path_str, workspace_root);

    // Check exclusion against resolved path
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

    // Compile current IR using resolved path for consistency
    let current_ir = match compile_file_ir(&resolved_path, fidelity, state) {
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

    // Estimate tokens from source for stats
    let delta_source = state.read_source(&resolved_path).ok();
    let delta_source_text = delta_source.as_ref().map(|s| s.as_str()).unwrap_or("");
    let delta_raw_tokens = estimate_tokens(delta_source_text);

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

    // R-19: Resolve tokenizer from tool arg + config
    let tokenizer_kind = parse_tokenizer_arg(params, &state.config);
    let tokenizer_box = crate::tokenizer::create_tokenizer(tokenizer_kind)
        .ok();
    let tokenizer_ref: Option<&dyn crate::tokenizer::Tokenizer> = tokenizer_box.as_deref();

    // Record stats for the IR delta operation using pluggable tokenizer
    let delta_compressed_tokens = count_tokens_with_tokenizer(
        &delta_source_text,
        tokenizer_ref,
    );
    state.session_stats.record_compression(
        &resolved_path,
        delta_raw_tokens,
        delta_compressed_tokens,
        &format!("{:?}", fidelity).to_lowercase(),
        false,
        "delta",
    );

    send_response(&result);
}

// ── Handler: delta_text_context (Phase IV — text-level delta) ──────

/// Handle `delta_text_context` — computes text-level delta between
/// the file's previous compressed body snapshot and its current state.
///
/// First call: stores the compressed body as baseline, returns full output.
/// Subsequent calls: computes line-level delta, returns compact §Δ format.
pub(super) fn handle_delta_text_context(
    id: &Value,
    params: &Value,
    state: &mut McpState,
) {
    let file_path_str = params["arguments"]["filePath"].as_str().unwrap_or("");
    let fidelity = match parse_fidelity_arg(id, params) {
        Ok(f) => f,
        Err(()) => return,
    };

    // Resolve path for consistency with other handlers
    let resolved_path = resolve_file_path(file_path_str, None);

    // Check exclusion against resolved path
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

    // Read source for token estimation
    let dt_source = state.read_source(&resolved_path).ok();
    let dt_source_text = dt_source.as_ref().map(|s| s.as_str()).unwrap_or("");
    let dt_raw_tokens = estimate_tokens(dt_source_text);

    // Compress the file to get current compressed body lines
    let path_alias = state.dict.get_or_create_alias(resolved_path.clone());

    // Run the full compression pipeline to get the body lines
    let result = compress_text_body(&resolved_path, fidelity, state);
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
            let dt_compressed_tokens = estimate_tokens(&wire);
            // Record stats for delta transport
            state.session_stats.record_compression(
                &resolved_path,
                dt_raw_tokens,
                dt_compressed_tokens,
                &format!("{:?}", fidelity).to_lowercase(),
                false,
                "delta",
            );
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
            let dt_compressed_tokens = estimate_tokens(&full_output);
            // Record stats for full compress
            state.session_stats.record_compression(
                &resolved_path,
                dt_raw_tokens,
                dt_compressed_tokens,
                &format!("{:?}", fidelity).to_lowercase(),
                false,
                "full",
            );
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

// ── Handler: apply_delta (new — client-side state update) ──────────

/// Handle `apply_delta` — applies a delta envelope to the in-session
/// state machine, returning the updated state and re-rendered pretty output.
pub(super) fn handle_apply_delta(
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
pub(super) fn handle_provide_code_context(
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

    // Read source for heuristics (uses source_cache)
    let source = match state.read_source(&resolved_path) {
        Ok(s) => s.as_ref().clone(),
        Err(e) => {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32603, "message": format!("Failed to read file: {}", e) }
            }));
            return;
        }
    };

    // Get path alias for delta tracking (must be before heuristics
    // so delta baselines stored under the alias key can be found)
    let path_alias = state.dict.get_or_create_alias(resolved_path.clone());

    // Run heuristics engine
    let decision = crate::mcp::heuristics::decide(
        &resolved_path,
        explicit_fidelity,
        explicit_intent,
        &state.config,
        &state.text_delta,
        &state.ir_context,
        &source,
        Some(&path_alias),
    );

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
                        state.ir_context.load_ir(ir.clone());

                        // Persistence hook: save baseline context + IR binary
                        if let Some(store) = &mut state.persistence_store {
                            let source_hash = sha2::Sha256::digest(source.as_bytes());
                            let hash_hex = format!("{:x}", source_hash);
                            let ir_binary = crate::ir::binary_wire::encode(&ir);

                            if let Err(e) = store.save_context(
                                &resolved_path,
                                decision.fidelity,
                                &compressed_text,
                                Some(&ir_binary),
                                &hash_hex,
                            ) {
                                eprintln!("[clean-ctx] WARNING: Failed to persist context: {e}");
                            }
                        }
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

            // Finding 4: Hash source *before* any mutable state borrows,
            // so persistence can use the hash without borrow conflicts.
            use sha2::Digest;
            let source_hash_hex = format!("{:x}", sha2::Sha256::digest(source.as_bytes()));

            // Finding 2: Compile IR once, reuse for both context tracking and
            // persistence — eliminates the redundant second compile_file_ir call
            // that previously existed in the persistence block.
            let compiled_ir = compile_file_ir(&resolved_path, decision.fidelity, state);
            if let Ok(ref ir) = compiled_ir {
                state.ir_context.load_ir(ir.clone());
            }

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

            let (output_text, is_delta) = match &delta {
                Some(d) => (d.to_wire_format(), true),
                None => (full_output, false),
            };

            // Persistence hook: save baseline + delta.
            // Uses compiled_ir from above (Single Compile pattern) and
            // source_hash_hex (pre-computed) to avoid redundant work.
            if let Some(store) = &mut state.persistence_store {
                if let Ok(ref ir) = compiled_ir {
                    let ir_binary = crate::ir::binary_wire::encode(ir);

                    if let Err(e) = store.save_context(
                        &resolved_path,
                        decision.fidelity,
                        &output_text,
                        Some(&ir_binary),
                        &source_hash_hex,
                    ) {
                        eprintln!("[clean-ctx] WARNING: Failed to persist context: {e}");
                    }
                }

                if let Some(d) = &delta {
                    let delta_bytes = serde_json::to_vec(d).unwrap_or_default();
                    if let Err(e) = store.append_delta(&source_hash_hex, &delta_bytes, Some("edit")) {
                        eprintln!("[clean-ctx] WARNING: Failed to persist delta: {e}");
                    }
                }
            }

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
pub(super) fn handle_restore_context(
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

    // Finding 3: Use resolved_path consistently for dict alias, compress, and IR
    let path_alias = state.dict.get_or_create_alias(resolved_path.clone());
    state.text_delta.store_snapshot(&path_alias, Vec::new());
    state.ir_context.remove_file(&path_alias);
    // LOW-02: Clear both the in-memory and persistence stores
    state.context_store.clear_file(&resolved_path);
    if let Some(store) = &mut state.persistence_store {
        store.clear_file(&resolved_path);
    }

    // Full re-compression
    match compress_file(
        PathBuf::from(&resolved_path),
        &mut state.dict,
        &mut state.cache,
        fidelity,
    ) {
        Ok(mut compressed_text) => {
            compressed_text.push_str(&state.dict.format_footer());

            // Re-store baseline
            let body_lines: Vec<String> = compressed_text.lines().map(String::from).collect();
            state.text_delta.compute_and_store(&path_alias, body_lines);

            // Re-compile IR (uses resolved_path for consistency — Finding 3)
            if let Ok(ir) = compile_file_ir(&resolved_path, fidelity, state) {
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
pub(super) fn handle_context_history(
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
/// Auto-flushes any pending persistence writes before generating stats.
/// Merges in-memory stats with DB-persisted stats for a cumulative view.
pub(super) fn handle_context_stats(
    id: &Value,
    params: &Value,
    state: &mut McpState,
) {
    // Flush any pending persistence writes before returning stats
    state.flush_persistence();

    // Query DB for persisted stats (after flush)
    let db_stats = state.persistence_store.as_ref().and_then(|store| {
        store.sqlite().and_then(|guard| guard.rebuild_stats().ok())
    });

    // Build a merged cumulative stats view: in-memory + DB
    let mut merged = state.session_stats.clone();
    if let Some(ref db) = db_stats {
        merged.merge(db);
    }

    let file_path = params["arguments"]["filePath"].as_str();
    let format = params["arguments"]["format"].as_str().unwrap_or("text");

    if let Some(fp) = file_path {
        // Stats for specific file (from merged view)
        let stats = merged.file_stats(fp);
        match stats {
            Some(fs) => {
                let mut text = format!(
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
                // Persistence info
                if db_stats.is_some() {
                    text.push_str("\n  Persistence: enabled");
                } else {
                    text.push_str("\n  Persistence: disabled");
                }
                if format == "json" {
                    let mut json = serde_json::json!(fs);
                    json["persistence"] = serde_json::json!({"enabled": db_stats.is_some()});
                    send_response(&serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "content": [{ "type": "text", "text": serde_json::to_string_pretty(&json).unwrap_or_default() }]
                        }
                    }));
                } else {
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
                let mut text = format!("No stats for file: {}", fp);
                if db_stats.is_some() {
                    text.push_str("\n  Persistence: enabled (no data for this file)");
                } else {
                    text.push_str("\n  Persistence: disabled");
                }
                send_response(&serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "content": [{ "type": "text", "text": text }]
                    }
                }));
            }
        }
    } else {
        // Full cumulative dashboard: in-memory + DB merged
        let mut text = crate::mcp::session_stats::render_dashboard_text(&merged);

        // R-19: Show active tokenizer
        let active_tokenizer = state.config.tokenizer.to_string();
        text.push_str(&format!("── Tokenizer ──\n  Active: {} (config: {})\n", active_tokenizer, active_tokenizer));

        // Persistence status line
        if db_stats.is_some() {
            let db_summary = db_stats.as_ref().map(|db| db.summary()).unwrap();
            text.push_str(&format!(
                "── Persistence (SQLite) ──\n  Status: enabled\n  DB Files: {}\n  DB Compressions: {}\n  DB Deltas: {}\n",
                db_summary.total_files,
                db_summary.full_compress_count,
                db_summary.delta_count,
            ));
        } else {
            text.push_str("── Persistence (SQLite) ──\n  Status: disabled\n");
        }

        if format == "json" {
            let mut json = crate::mcp::session_stats::render_dashboard_json(&merged);
            json["tokenizer"] = serde_json::json!(state.config.tokenizer.to_string());
            json["persistence"] = serde_json::json!({"enabled": db_stats.is_some()});
            send_response(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "content": [{ "type": "text", "text": serde_json::to_string_pretty(&json).unwrap_or_default() }]
                }
            }));
        } else {
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

// ── Persistence Tool Handlers ─────────────────────────────────────

/// Handle `save_context` — explicit save to DB.
pub(super) fn handle_save_context(
    id: &Value,
    params: &Value,
    state: &mut McpState,
) {
    if let Some(store) = &mut state.persistence_store {
        let file_path = params["arguments"]["filePath"].as_str();
        let mut saved_count = 0;

        if let Some(fp) = file_path {
            // Save specific file's IR from context state
            let path_alias = state.dict.get_or_create_alias(fp.to_string());
            if let Some(instructions) = state.ir_context.get_ir(&path_alias) {
                let version = state.ir_context.file_version(&path_alias).unwrap_or(1);
                let ops: Vec<crate::ir::opcodes::CoreOp> = instructions.iter()
                    .filter_map(|t| crate::ir::wire::tuple_to_op(t))
                    .collect();
                let ir = crate::ir::compiler::CompiledIR {
                    file_id: fp.to_string(),
                    instructions: ops,
                    version,
                };
                let ir_binary = crate::ir::binary_wire::encode(&ir);
                // CRIT-02 fix: use file-path-only hash for stable context ID
                // (version-dependent hash orphaned previous context rows)
                let hash = format!("{:x}", sha2::Sha256::digest(fp.as_bytes()));
                // MED-03: Include compressed_output when saving (previously empty string "")
                let compressed_text = state.ir_context.render_pretty(&path_alias, Fidelity::Low)
                    .unwrap_or_default();
                if let Err(e) = store.save_context(
                    fp, Fidelity::Low, &compressed_text, Some(&ir_binary), &hash
                ) {
                    eprintln!("[clean-ctx] WARNING: Failed to persist context for {}: {e}", fp);
                } else {
                    saved_count = 1;
                }
            }
        } else {
            // Save all tracked files
            let file_ids: Vec<String> = state.ir_context.file_ids();
            for fp in &file_ids {
                let path_alias = state.dict.get_or_create_alias(fp.clone());
                if let Some(instructions) = state.ir_context.get_ir(&path_alias) {
                    let version = state.ir_context.file_version(&path_alias).unwrap_or(1);
                    let ops: Vec<crate::ir::opcodes::CoreOp> = instructions.iter()
                        .filter_map(|t| crate::ir::wire::tuple_to_op(t))
                        .collect();
                    let ir = crate::ir::compiler::CompiledIR {
                        file_id: fp.clone(),
                        instructions: ops,
                        version,
                    };
                    let ir_binary = crate::ir::binary_wire::encode(&ir);
                    // CRIT-02 fix: use file-path-only hash for stable context ID
                    let hash = format!("{:x}", sha2::Sha256::digest(fp.as_bytes()));
                    // MED-03: Include compressed_output when saving (previously empty string "")
                    let compressed_text = state.ir_context.render_pretty(&path_alias, Fidelity::Low)
                        .unwrap_or_default();
                    if let Err(e) = store.save_context(
                        fp, Fidelity::Low, &compressed_text, Some(&ir_binary), &hash
                    ) {
                        eprintln!("[clean-ctx] WARNING: Failed to persist context for {}: {e}", fp);
                    } else {
                        saved_count += 1;
                    }
                }
            }
        }

        send_response(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "ok": true,
                "saved": saved_count,
                "message": format!("Saved {} file(s) to persistence DB.", saved_count)
            }
        }));
    } else {
        send_response(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "ok": true,
                "saved": 0,
                "message": "Persistence DB not enabled. No files saved."
            }
        }));
    }
}

/// Handle `list_sessions` — show DB sessions.
pub(super) fn handle_list_sessions(
    id: &Value,
    _params: &Value,
    state: &mut McpState,
) {
    if let Some(_store) = &state.persistence_store {
        // We can't access the raw conn through ContextStore trait,
        // but we can report what we know from the in-memory state
        let file_ids: Vec<String> = state.ir_context.file_ids();
        let mut lines = Vec::new();
        lines.push("Persistence Sessions:".to_string());
        lines.push(format!("  DB: active ({} files tracked in memory)", file_ids.len()));
        for fp in &file_ids {
            lines.push(format!("    - {}", fp));
        }
        if file_ids.is_empty() {
            lines.push("  (no files tracked in this session)".to_string());
        }
        send_response(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "content": [{ "type": "text", "text": lines.join("\n") }]
            }
        }));
    } else {
        send_response(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "content": [{ "type": "text", "text": "Persistence DB not enabled." }]
            }
        }));
    }
}

/// Handle `replay_history` — load and replay from DB.
pub(super) fn handle_replay_history(
    id: &Value,
    params: &Value,
    state: &mut McpState,
) {
    let file_path = params["arguments"]["filePath"].as_str().unwrap_or("");
    let target_seq = params["arguments"]["targetSequence"].as_i64().map(|v| v as u32);

    if file_path.is_empty() {
        send_response(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": -32602, "message": "Missing required parameter: filePath" }
        }));
        return;
    }

    if let Some(store) = &state.persistence_store {
        match store.load_context_with_deltas(file_path, target_seq) {
            Ok(Some((ir, version))) => {
                // Load into in-memory state (CRIT-01 fix: assign directly, no clone)
                state.ir_context.load_ir(ir.clone());

                send_response(&serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "content": [{ "type": "text", "text": format!("Replayed {} to v{} ({} instructions)", file_path, version, ir.instructions.len()) }],
                        "file": file_path,
                        "version": version,
                        "instruction_count": ir.instructions.len()
                    }
                }));
            }
            Ok(None) => {
                send_response(&serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": { "code": -32603, "message": format!("No context found for: {}", file_path) }
                }));
            }
            Err(e) => {
                send_response(&serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": { "code": -32603, "message": format!("Replay failed: {}", e) }
                }));
            }
        }
    } else {
        send_response(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": -32603, "message": "Persistence DB not enabled." }
        }));
    }
}

/// Handle `purge_old_deltas` — clean up old deltas from DB.
pub(super) fn handle_purge_old_deltas(
    id: &Value,
    params: &Value,
    state: &mut McpState,
) {
    let days = params["arguments"]["days"].as_i64().unwrap_or(30);

    if let Some(store) = &mut state.persistence_store {
        // Delete deltas older than N days
        match store.purge_old_deltas(days as u32) {
            Ok(n) => {
                send_response(&serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "ok": true,
                        "purged": n,
                        "message": format!("Purged {} delta(s) older than {} days.", n, days)
                    }
                }));
            }
            Err(e) => {
                send_response(&serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": { "code": -32603, "message": format!("Purge failed: {}", e) }
                }));
            }
        }
    } else {
        send_response(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": -32603, "message": "Persistence DB not enabled." }
        }));
    }
}