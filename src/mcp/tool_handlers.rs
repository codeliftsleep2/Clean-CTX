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
use crate::mcp::cache_hints::{inject_cache_breakpoints, compute_baseline_breaker, mark_tail_ephemeral, render_cache_text, render_cache_json};
use crate::mcp::context_store::ContextStore;
use crate::protocol::send_response;

use super::tools::{parse_fidelity_arg, resolve_fidelity, parse_tokenizer_arg};
use super::tool_helpers::{compress_text_body, compile_file_ir, resolve_file_path, diff_code_context_handler, count_tokens_with_tokenizer};

#[cfg(test)]
#[path = "../tests/mcp/tool_handlers.rs"]
mod tests;

// ── Handler: compress_code_context (upgraded, backward compatible) ──

/// Handle `compress_code_context` — IR-first response.
///
/// Phase 6 (IR-first architecture): `content[0].text` now contains the
/// LLM-optimized hierarchical IR text (compact structural markers).
/// The old text pipeline output moves to `"pretty"`. The structured
/// hierarchical JSON is in `"ir"`.
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

    // Pre-read source into source_cache so compile_file_ir gets a cache hit
    let source_arc = state.read_source(file_path_str).ok();
    let source_ref = source_arc.as_ref().map(|s| s.as_str());
    let source_text = source_ref.unwrap_or("");

    // Phase 6: Compile IR first — this is now the primary output
    let ir_result = compile_file_ir(file_path_str, effective_fidelity, state);

    // R-19: Resolve tokenizer from tool arg + config
    let tokenizer_kind = parse_tokenizer_arg(params, &state.config);
    let tokenizer_box = crate::tokenizer::create_tokenizer(tokenizer_kind).ok();
    let tokenizer_ref: Option<&dyn crate::tokenizer::Tokenizer> = tokenizer_box.as_deref();

    // Phase 6: Build response from IR data
    let response = if let Ok(ir) = ir_result {
        // Store the full IR in context state for delta tracking
        state.ir_context.load_ir(ir.clone());

        // Convert flat IR → hierarchical via ir_to_hierarchical
        let hir = crate::ir::hierarchical::ir_to_hierarchical(&ir);

        // Render hierarchical → compact LLM text via render_hierarchical_for_llm
        use crate::ir::render_hierarchical_for_llm;
        let llm_text = render_hierarchical_for_llm(&hir, effective_fidelity);

        // Pathmap footer for LLM text
        let llm_text_with_footer = format!("{}\n// ── {} ({}) ──\n{}",
            llm_text.trim(),
            ir.file_id,
            file_path_str,
            state.dict.format_footer().trim(),
        );

        // Cache the rendered text for delta mode (Fix D)
        state.llm_text_cache.insert(ir.file_id.clone(), llm_text_with_footer.clone());

        // Record stats
        let raw_tokens = count_tokens_with_tokenizer(source_text, tokenizer_ref);
        let compressed_tokens = count_tokens_with_tokenizer(&llm_text_with_footer, tokenizer_ref);
        let ccc_canonical = resolve_file_path(file_path_str, None);
        state.session_stats.record_compression(
            &ccc_canonical,
            raw_tokens,
            compressed_tokens,
            &format!("{:?}", effective_fidelity).to_lowercase(),
            false,
            "full",
            None,
        );

        // Persistence hook
        debug_log(format!("handle_compress: persist_store={}", state.persistence_store.is_some()));
        if let Some(store) = &mut state.persistence_store {
            let source_hash = sha2::Sha256::digest(source_text.as_bytes());
            let hash_hex = format!("{:x}", source_hash);
            let ir_binary = Some(crate::ir::binary_wire::encode(&ir));
            debug_log(format!("handle_compress: calling save_context for {}", file_path_str));
            match store.save_context(
                file_path_str,
                effective_fidelity,
                &llm_text_with_footer,
                ir_binary.as_deref(),
                &hash_hex,
                raw_tokens as u64,
                compressed_tokens as u64,
            ) {
                Ok(ctx_id) => debug_log(format!("handle_compress: save_context OK id={}", ctx_id)),
                Err(e) => debug_log(format!("handle_compress: save_context FAILED: {e}")),
            }
        } else {
            debug_log("handle_compress: persist_store is None, skipping");
        }

        // F-08: Determine the IR wire format based on the encoding parameter.
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

        // Build hierarchical JSON for the "ir" field
        let hierarchical_json = crate::ir::hierarchical::ir_to_hierarchical_wire(&ir);

        serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "content": [{ "type": "text", "text": llm_text_with_footer }],
                "ir": hierarchical_json,
                "pretty": ir_value,
                "v": ir.version,
                "file": ir.file_id
            }
        })
    } else {
        // IR compilation failed — fall back to text pipeline only
        match compress_file_with_source(
            PathBuf::from(file_path_str),
            source_ref,
            &mut state.dict,
            &mut state.cache,
            effective_fidelity,
        ) {
            Ok(mut compressed_text) => {
                compressed_text.push_str(&state.dict.format_footer());
                let raw_tokens = count_tokens_with_tokenizer(source_text, tokenizer_ref);
                let compressed_tokens = count_tokens_with_tokenizer(&compressed_text, tokenizer_ref);
                let ccc_canonical = resolve_file_path(file_path_str, None);
                state.session_stats.record_compression(
                    &ccc_canonical,
                    raw_tokens,
                    compressed_tokens,
                    &format!("{:?}", effective_fidelity).to_lowercase(),
                    false,
                    "full",
                    None,
                );
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "content": [{ "type": "text", "text": compressed_text }]
                    }
                })
            }
            Err(e) => {
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": { "code": -32603, "message": e.to_string() }
                })
            }
        }
    };

    send_response(&response);
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
            // Record stats for this file (was previously missing — Phase 1 fix)
            let source_opt = state.read_source(&resolved_path).ok();
            let source_text = source_opt.as_ref().map(|s| s.as_str()).unwrap_or("");
            // HIGH-1 fix: Use pluggable tokenizer for accurate token counts
            let tok_kind_diff = parse_tokenizer_arg(params, &state.config);
            let tok_box_diff = crate::tokenizer::create_tokenizer(tok_kind_diff).ok();
            let tok_ref_diff: Option<&dyn crate::tokenizer::Tokenizer> = tok_box_diff.as_deref();
            let raw_tokens = count_tokens_with_tokenizer(source_text, tok_ref_diff);
            let compressed_tokens = count_tokens_with_tokenizer(&output, tok_ref_diff);
            state.session_stats.record_compression(
                &resolved_path,
                raw_tokens,
                compressed_tokens,
                &format!("{:?}", fidelity).to_lowercase(),
                false,
                "diff",
                None,
            );

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

    // Read source for stats
    let delta_source = state.read_source(&resolved_path).ok();
    let delta_source_text = delta_source.as_ref().map(|s| s.as_str()).unwrap_or("");
    // HIGH-1 fix: raw tokens also use pluggable tokenizer (was estimate_tokens before)
    let tok_kind_dc = parse_tokenizer_arg(params, &state.config);
    let tok_box_dc = crate::tokenizer::create_tokenizer(tok_kind_dc).ok();
    let tok_ref_dc: Option<&dyn crate::tokenizer::Tokenizer> = tok_box_dc.as_deref();
    let delta_raw_tokens = count_tokens_with_tokenizer(delta_source_text, tok_ref_dc);

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

    // CRIT-3 fix: Count tokens on the delta wire output (what the client actually
    // receives), NOT on the raw source text. This gives accurate savings %.
    let delta_output_text = serde_json::to_string(&result).unwrap_or_default();
    let delta_compressed_tokens = count_tokens_with_tokenizer(
        &delta_output_text,
        tok_ref_dc,
    );
    state.session_stats.record_compression(
        &resolved_path,
        delta_raw_tokens,
        delta_compressed_tokens,
        &format!("{:?}", fidelity).to_lowercase(),
        false,
        "delta",
        None,
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

    // Read source for token counting
    let dt_source = state.read_source(&resolved_path).ok();
    let dt_source_text = dt_source.as_ref().map(|s| s.as_str()).unwrap_or("");
    // HIGH-1 fix: Use pluggable tokenizer for accurate token counts
    let tok_kind_dt = parse_tokenizer_arg(params, &state.config);
    let tok_box_dt = crate::tokenizer::create_tokenizer(tok_kind_dt).ok();
    let tok_ref_dt: Option<&dyn crate::tokenizer::Tokenizer> = tok_box_dt.as_deref();
    let dt_raw_tokens = count_tokens_with_tokenizer(dt_source_text, tok_ref_dt);

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
            let dt_compressed_tokens = count_tokens_with_tokenizer(&wire, tok_ref_dt);
            // Full compressed tokens for delta efficiency computation
            let dt_full_compressed = count_tokens_with_tokenizer(&full_output, tok_ref_dt);
            // Record stats for delta transport
            state.session_stats.record_compression(
                &resolved_path,
                dt_raw_tokens,
                dt_compressed_tokens,
                &format!("{:?}", fidelity).to_lowercase(),
                false,
                "delta",
                Some(dt_full_compressed),
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
            let dt_compressed_tokens = count_tokens_with_tokenizer(&full_output, tok_ref_dt);
            // Record stats for full compress
            state.session_stats.record_compression(
                &resolved_path,
                dt_raw_tokens,
                dt_compressed_tokens,
                &format!("{:?}", fidelity).to_lowercase(),
                false,
                "full",
                None,
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
    // C-1: pass persisted fidelity from DB for session-aware re-use
    let stored_fidelity = state.context_store.load_latest(&resolved_path)
        .ok()
        .flatten()
        .map(|meta| meta.fidelity);
    let decision = crate::mcp::heuristics::decide(
        &resolved_path,
        explicit_fidelity,
        explicit_intent,
        &state.config,
        &state.text_delta,
        &state.ir_context,
        &source,
        Some(&path_alias),
        stored_fidelity,
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

                    // HIGH-1 fix: Use pluggable tokenizer for accurate token counts.
                    // Parse tokenizer EARLY so persistence uses it instead of
                    // the old estimate_tokens heuristic.
                    let tok_kind_pcc = parse_tokenizer_arg(params, &state.config);
                    let tok_box_pcc = crate::tokenizer::create_tokenizer(tok_kind_pcc).ok();
                    let tok_ref_pcc: Option<&dyn crate::tokenizer::Tokenizer> = tok_box_pcc.as_deref();
                    let raw_tokens = count_tokens_with_tokenizer(&source, tok_ref_pcc);
                    let compressed_tokens = count_tokens_with_tokenizer(&compressed_text, tok_ref_pcc);

                    // Compile IR and store baseline
                    let ir_result = compile_file_ir(&resolved_path, decision.fidelity, state);
                    if let Ok(ir) = ir_result {
                        state.ir_context.load_ir(ir.clone());

                        // Persistence hook: save baseline context + IR binary.
                        // Uses pluggable tokenizer counts instead of estimate_tokens.
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
                                raw_tokens as u64,
                                compressed_tokens as u64,
                            ) {
                                eprintln!("[clean-ctx] WARNING: Failed to persist context: {e}");
                            }
                        }
                    }

                    state.session_stats.record_compression(
                        &resolved_path,
                        raw_tokens,
                        compressed_tokens,
                        &format!("{:?}", decision.fidelity).to_lowercase(),
                        decision.is_angular,
                        "full",
                        None,
                    );

                    let mut response = serde_json::json!({
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
                    });

                    // Inject CBM enrichment metadata when available
                    enrich_with_cbm(&mut response, &resolved_path, state);

                    // Inject baseline cache breakpoint for stable content
                    let cache_enabled = state.config.cache.enabled;
                    if cache_enabled {
                        let ttl = state.config.cache.baseline_ttl.clone();
                        let breaker = compute_baseline_breaker(&compressed_text);
                        inject_cache_breakpoints(&mut response, state, "baseline", &ttl, &breaker, tok_ref_pcc);
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

            // Parse tokenizer EARLY so persistence and delta efficiency use it
            let tok_kind_dt2 = parse_tokenizer_arg(params, &state.config);
            let tok_box_dt2 = crate::tokenizer::create_tokenizer(tok_kind_dt2).ok();
            let tok_ref_dt2: Option<&dyn crate::tokenizer::Tokenizer> = tok_box_dt2.as_deref();
            // Tokenize full_output BEFORE it's consumed by the match below (borrow after move)
            let dt2_full_compressed = count_tokens_with_tokenizer(&full_output, tok_ref_dt2);

            let (output_text, is_delta) = match &delta {
                Some(d) => (d.to_wire_format(), true),
                None => (full_output, false),
            };

            // HIGH-1 fix: Use pluggable tokenizer for accurate token counts.
            let raw_tokens = count_tokens_with_tokenizer(&source, tok_ref_dt2);
            let compressed_tokens = count_tokens_with_tokenizer(&output_text, tok_ref_dt2);

            // Persistence hook: save baseline + delta.
            // Uses compiled_ir from above (Single Compile pattern) and
            // source_hash_hex (pre-computed) to avoid redundant work.
            // Uses pluggable tokenizer counts instead of estimate_tokens.
            if let Some(store) = &mut state.persistence_store {
                if let Ok(ref ir) = compiled_ir {
                    let ir_binary = crate::ir::binary_wire::encode(ir);

                    if let Err(e) = store.save_context(
                        &resolved_path,
                        decision.fidelity,
                        &output_text,
                        Some(&ir_binary),
                        &source_hash_hex,
                        raw_tokens as u64,
                        compressed_tokens as u64,
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

            let strategy_label = if is_delta { "delta" } else { "full" };
            state.session_stats.record_compression(
                &resolved_path,
                raw_tokens,
                compressed_tokens,
                &format!("{:?}", decision.fidelity).to_lowercase(),
                decision.is_angular,
                strategy_label,
                if is_delta { Some(dt2_full_compressed) } else { None },
            );

            let mut response = serde_json::json!({
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
            });

            // Inject CBM enrichment metadata when available
            enrich_with_cbm(&mut response, &resolved_path, state);

            // Inject tail cache breakpoint for dynamic content (5m TTL, never cached across turns)
            let cache_enabled = state.config.cache.enabled;
            if cache_enabled {
                let ttl = state.config.cache.tail_ttl.clone();
                inject_cache_breakpoints(&mut response, state, "tail", &ttl, "rolling", tok_ref_dt2);
                mark_tail_ephemeral(state);
            }

            send_response(&response);
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

            // Record stats (was previously missing — Phase 1 fix)
            let rc_source = state.read_source(&resolved_path).ok();
            let rc_source_text = rc_source.as_ref().map(|s| s.as_str()).unwrap_or("");
            // HIGH-1 fix: Use pluggable tokenizer for accurate token counts
            let tok_kind_rc = parse_tokenizer_arg(params, &state.config);
            let tok_box_rc = crate::tokenizer::create_tokenizer(tok_kind_rc).ok();
            let tok_ref_rc: Option<&dyn crate::tokenizer::Tokenizer> = tok_box_rc.as_deref();
            let rc_raw_tokens = count_tokens_with_tokenizer(rc_source_text, tok_ref_rc);
            let rc_compressed_tokens = count_tokens_with_tokenizer(&compressed_text, tok_ref_rc);
            state.session_stats.record_compression(
                &resolved_path,
                rc_raw_tokens,
                rc_compressed_tokens,
                &format!("{:?}", fidelity).to_lowercase(),
                false,
                "restore",
                None,
            );

            let mut response = serde_json::json!({
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
            });

            // restore_context returns stable persisted state — emit baseline cache hint
            let cache_enabled = state.config.cache.enabled;
            if cache_enabled {
                let ttl = state.config.cache.baseline_ttl.clone();
                let breaker = compute_baseline_breaker(&compressed_text);
                inject_cache_breakpoints(&mut response, state, "baseline", &ttl, &breaker, tok_ref_rc);
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

        // Cache metrics for this file (Phase 2: session-level cache status)
        // Note: cache_metrics.breakpoints is keyed by region ("baseline",
        // "tools", etc.), not by file path. We show session-level cache
        // hit/miss metrics which apply to all files uniformly.
        let total = state.cache_metrics.hits + state.cache_metrics.misses;
        lines.push(format!("  Cache Hit Rate: {}/{} ({}%)",
            state.cache_metrics.hits,
            total,
            if total > 0 {
                (state.cache_metrics.hits as f64 / total as f64 * 100.0) as usize
            } else {
                0
            },
        ));
        lines.push(format!("  Cache Tokens Saved: {}", state.cache_metrics.tokens_saved));

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

        // Cache metrics summary (Phase 2)
        let hit_rate = if state.cache_metrics.hits + state.cache_metrics.misses > 0 {
            state.cache_metrics.hits as f64 / (state.cache_metrics.hits + state.cache_metrics.misses) as f64
        } else {
            0.0
        };
        output.push_str(&format!(
            "── Cache ──\n  Hits: {} | Misses: {} | Hit Rate: {:.0}% | Tokens Saved: {}\n",
            state.cache_metrics.hits,
            state.cache_metrics.misses,
            hit_rate * 100.0,
            state.cache_metrics.tokens_saved,
        ));

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

        // Cache status section (only shown when active or enabled)
        if let Some(cache_text) = render_cache_text(&state.cache_metrics, state.config.cache.enabled) {
            text.push_str(&cache_text);
        }

        if format == "json" {
            let mut json = crate::mcp::session_stats::render_dashboard_json(&merged);
            json["tokenizer"] = serde_json::json!(state.config.tokenizer.to_string());
            json["persistence"] = serde_json::json!({"enabled": db_stats.is_some()});
            json["cache"] = render_cache_json(&state.cache_metrics, state.config.cache.enabled);
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
                // CRIT-2 fix: Look up token counts + fidelity from session_stats
                let (rt, ct, fidelity_str) = state.session_stats.file_stats(fp)
                    .map(|fs| (fs.raw_tokens as u64, fs.compressed_tokens as u64, fs.fidelity.clone()))
                    .unwrap_or((0, 0, "low".to_string()));
                // MED-03 fix: Use actual fidelity from session_stats instead of hardcoded Low
                let actual_fidelity = Fidelity::parse(&fidelity_str).unwrap_or(Fidelity::Low);
                if let Err(e) = store.save_context(
                    fp, actual_fidelity, &compressed_text, Some(&ir_binary), &hash, rt, ct
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
                    // CRIT-2 fix: Look up token counts + fidelity from session_stats
                    let (rt, ct, fidelity_str) = state.session_stats.file_stats(fp)
                        .map(|fs| (fs.raw_tokens as u64, fs.compressed_tokens as u64, fs.fidelity.clone()))
                        .unwrap_or((0, 0, "low".to_string()));
                    // MED-03 fix: Use actual fidelity from session_stats instead of hardcoded Low
                    let actual_fidelity = Fidelity::parse(&fidelity_str).unwrap_or(Fidelity::Low);
                    if let Err(e) = store.save_context(
                        fp, actual_fidelity, &compressed_text, Some(&ir_binary), &hash, rt, ct
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

// ── CBM Enrichment ────────────────────────────────────────────────

/// Inject CBM graph metadata into the `_meta` field of a `provide_code_context`
/// response when the graph bridge is available.
///
/// Adds:
///   - `cbm_status`: "available" | "degraded" | "unavailable"
///   - `cbm_symbol_importance`: top symbols for this file (if any)
///   - `cbm_architecture`: module count + dependency count (if cached)
///
/// When CBM is unavailable, only `cbm_status: "unavailable"` is added.
/// This is a no-op for non-`provide_code_context` handlers.
pub(super) fn enrich_with_cbm(
    response: &mut serde_json::Value,
    file_path: &str,
    state: &mut McpState,
) {
    let meta = match response.get_mut("result").and_then(|r| r.get_mut("_meta")) {
        Some(m) => m,
        None => return,
    };

    // Surface CBM status
    let status_str = state.cbm_status.summary().to_string();
    meta["cbm_status"] = serde_json::Value::String(status_str.clone());

    // If CBM is unavailable, stop here — no graph data to add
    if status_str != "available" {
        return;
    }

    // Query graph bridge for this file's metadata
    let bridge = match state.graph_bridge.as_mut() {
        Some(b) => b,
        None => return,
    };

    // Symbol importance for this file
    let importance = bridge.get_symbol_importance_mut();
    let file_importance: Vec<_> = importance
        .values()
        .filter(|s| s.file.contains(file_path) || file_path.contains(&s.file))
        .take(5)
        .collect();

    if !file_importance.is_empty() {
        let symbols: Vec<serde_json::Value> = file_importance
            .iter()
            .map(|s| {
                serde_json::json!({
                    "symbol": s.symbol,
                    "score": s.score,
                    "file": s.file,
                })
            })
            .collect();
        meta["cbm_symbol_importance"] = serde_json::Value::Array(symbols);
    }

    // Architecture overview (cached, cheap)
    if let Some(arch) = bridge.get_architecture() {
        meta["cbm_architecture"] = serde_json::json!({
            "modules": arch.modules.len(),
            "dependencies": arch.dependencies.len(),
        });
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