// src/mcp/tool_handlers/core.rs
//
// Core MCP tool handlers: compression, IR, delta, and unified entry point.
use crate::error::to_jsonrpc_error;
use crate::ir::compiler::CompiledIR;
use crate::ir::delta::{DeltaComputer, IRDelta};
use crate::ir::opcodes::CoreOp;
use crate::ir::wire::ir_to_wire;
use crate::ir::wire::tuple_to_op;
use crate::mcp::McpState;
use crate::mcp::context_store::ContextStore;
use crate::protocol::send_response;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::path::PathBuf;

use super::super::tool_helpers::{
    compile_file_ir, compile_file_ir_focused, compress_text_body, count_tokens_with_tokenizer,
    diff_code_context_handler, inject_baseline_breakpoint, inject_tail_breakpoint,
    resolve_file_path_checked,
};
use super::super::tools::{parse_fidelity_arg, parse_tokenizer_arg};

fn tuples_to_coreops(tuples: Vec<Vec<String>>) -> Vec<CoreOp> {
    tuples.into_iter().filter_map(|t| tuple_to_op(&t)).collect()
}

/// Self-reporting contract fields (Gap 5/3/6 fixes).
///
/// Returns `(content_kind, byte_exact_regions)` describing what the
/// response contains so the LLM can tell structural-only output from
/// body-inclusive output without re-parsing the text.
///
/// - `content_kind`: `"skeleton"` (structural-only), `"skeleton_with_verbatim_bodies"`
///   (Edit — method bodies are byte-exact), or `"verbatim_document"`
///   (Verbatim — entire document byte-exact).
/// - `byte_exact`: which regions are safe for `replace_in_file` SEARCH
///   blocks. Edit → `["method_bodies"]`; Verbatim → `["document"]`;
///   others → `[]`.
pub(crate) fn contract_fields(
    fidelity: crate::compression::Fidelity,
) -> (&'static str, Vec<&'static str>) {
    contract_fields_focused(fidelity, None)
}

/// Self-reporting contract fields for `provide_code_context`, accounting for
/// symbol targeting via `focusMethods`.
///
/// When `focus` is `None` (no `focusMethods` supplied), `Edit` fidelity reports
/// `"skeleton_with_verbatim_bodies"`/`["method_bodies"]` — every method's body
/// is byte-exact (legacy behavior).
///
/// When `focus` is `Some(_)` (silently ignored unless the effective fidelity is
/// `Edit`), only the focused method bodies are byte-exact. The contract reports
/// `"skeleton_with_focused_verbatim_bodies"`/`["focused_method_bodies"]` so the
/// LLM knows NOT to attempt `replace_in_file` SEARCH on unfocused method bodies.
pub(crate) fn contract_fields_focused(
    fidelity: crate::compression::Fidelity,
    focus: Option<&HashSet<String>>,
) -> (&'static str, Vec<&'static str>) {
    match fidelity {
        crate::compression::Fidelity::Verbatim => ("verbatim_document", vec!["document"]),
        // No focus set → every method body is byte-exact (legacy behavior).
        crate::compression::Fidelity::Edit if focus.is_none() => {
            ("skeleton_with_verbatim_bodies", vec!["method_bodies"])
        }
        // Focus set but EMPTY → ZERO method bodies are byte-exact. The
        // output is effectively all-signatures, so report `"skeleton"`
        // with no byte-exact regions (otherwise the LLM would attempt
        // replace_in_file SEARCH on bodies that don't exist).
        crate::compression::Fidelity::Edit if focus.is_some_and(HashSet::is_empty) => {
            ("skeleton", Vec::new())
        }
        // Focus set with names → only the focused method bodies are
        // byte-exact. The LLM must NOT attempt SEARCH on unfocused bodies.
        crate::compression::Fidelity::Edit => (
            "skeleton_with_focused_verbatim_bodies",
            vec!["focused_method_bodies"],
        ),
        _ => ("skeleton", Vec::new()),
    }
}

// ── Handler: compress_code_context ───────────────────────────────

/// P3-2: Main handler for compress_code_context tool.
/// Orchestrates validation, compilation, and response building.
pub(crate) fn handle_compress_code_context(id: &Value, params: &Value, state: &McpState) {
    let file_path_str = params["arguments"]["filePath"].as_str().unwrap_or("");
    let encoding = params["arguments"]["encoding"].as_str().unwrap_or("named");
    let workspace_root = params["arguments"]["workspaceRoot"].as_str();
    let resolved_path = match resolve_file_path_checked(
        file_path_str,
        workspace_root,
        &state.config.additional_roots,
    ) {
        Ok(p) => p,
        Err(msg) => {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "error": { "code": -32602, "message": msg }
            }));
            return;
        }
    };
    let fidelity = match parse_fidelity_arg(id, params, &state.config) {
        Ok(f) => f,
        Err(()) => return,
    };

    if state.config.is_excluded(&resolved_path) {
        send_response(&serde_json::json!({
            "jsonrpc": "2.0", "id": id,
            "error": { "code": -32603, "message": format!("File excluded by config: {}", file_path_str) }
        }));
        return;
    }

    // A-13: Check resource limits before processing
    let limits = &state.config.resource_limits;
    if let Ok(metadata) = std::fs::metadata(&resolved_path) {
        if let Err(e) = limits.check_file_size(metadata.len()) {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "error": { "code": -32603, "message": e }
            }));
            return;
        }
    }

    let effective_fidelity = fidelity;
    // Gap 5/3/6 fixes: self-reporting contract fields so the LLM knows
    // whether the response contains byte-exact regions (Edit/Verbatim)
    // or is structural-only, without re-parsing the output text.
    let (content_kind, byte_exact) = contract_fields(effective_fidelity);
    let source_arc = state.read_source(&resolved_path).ok();
    let source_ref = source_arc.as_ref().map(|s| s.as_str());
    let source_text = source_ref.unwrap_or("");

    let tokenizer_kind = parse_tokenizer_arg(params, &state.config);
    let tokenizer_box = crate::tokenizer::create_tokenizer(tokenizer_kind).ok();
    let tokenizer_ref: Option<&dyn crate::tokenizer::Tokenizer> = tokenizer_box.as_deref();

    // Verbatim fidelity: return the full raw source byte-exact, as the
    // plan's fidelity table promises ("Full raw source, byte-exact entire
    // document"). The IR and legacy compressors both compress, so bypass
    // them — otherwise `contract_fields` would report `["document"]` while
    // the payload is a structural skeleton (self-reporting contract leak).
    if effective_fidelity == crate::compression::Fidelity::Verbatim {
        let raw_tokens = count_tokens_with_tokenizer(source_text, tokenizer_ref);
        state.record_compression(
            &resolved_path,
            raw_tokens,
            raw_tokens,
            "verbatim",
            false,
            "full",
            None,
            "verbatim",
        );
        let mut response = serde_json::json!({
            "jsonrpc": "2.0", "id": id,
            "result": {
                "content": [{ "type": "text", "text": source_text }],
                "content_kind": "verbatim_document", "byte_exact": ["document"],
                "verbatim": true
            }
        });
        inject_baseline_breakpoint(&mut response, state, source_text);
        send_response(&response);
        return;
    }

    let ir_result = compile_file_ir(&resolved_path, effective_fidelity, state);

    // P3-2: Build response using extracted helpers
    // If IR compilation fails, fall back to legacy compression but log
    // the structured error for diagnostics (4.4 audit fix).
    let mut response = if let Ok((ir, _source_hash)) = ir_result {
        state.ir_context_lock().load_ir(ir.clone(), None);
        let hir = crate::ir::hierarchical::ir_to_hierarchical(&ir);
        let llm_text = crate::ir::render_hierarchical_for_llm(&hir, effective_fidelity);
        let footer = state.format_dict_footer();
        let llm_text_with_footer = format!(
            "{}\n// ── {} ({}) ──\n{}",
            llm_text.trim(),
            ir.file_id,
            resolved_path,
            footer.trim()
        );
        state
            .llm_text_cache_lock()
            .insert(ir.file_id.clone(), llm_text_with_footer.clone());

        let raw_tokens = count_tokens_with_tokenizer(source_text, tokenizer_ref);
        let compressed_tokens = count_tokens_with_tokenizer(&llm_text_with_footer, tokenizer_ref);
        state.record_compression(
            &resolved_path,
            raw_tokens,
            compressed_tokens,
            &format!("{:?}", effective_fidelity).to_lowercase(),
            false,
            "full",
            None,
            "ir_compression",
        );

        // Persist to DB
        {
            if let Some(ref store) = *state.persistence_store_lock() {
                let mut hasher = Sha256::new();
                hasher.update(source_text.as_bytes());
                let source_hash = format!("{:x}", hasher.finalize());

                store.queue_save_context(
                    &resolved_path,
                    effective_fidelity,
                    &llm_text_with_footer,
                    &[],
                    &source_hash,
                    raw_tokens as u64,
                    compressed_tokens as u64,
                );
            }
        }
        state.flush_persistence();

        let ir_value = match encoding {
            "positional" => {
                let config = crate::ir::positional::PositionalConfig::stripped();
                crate::ir::positional::ir_to_positional_wire(
                    &ir.file_id,
                    ir.version,
                    &ir.instructions,
                    config,
                )
            }
            "tagged" => {
                let config = crate::ir::positional::PositionalConfig::tagged();
                crate::ir::positional::ir_to_positional_wire(
                    &ir.file_id,
                    ir.version,
                    &ir.instructions,
                    config,
                )
            }
            _ => ir_to_wire(&ir),
        };

        serde_json::json!({
            "jsonrpc": "2.0", "id": id,
            "result": {
                "content": [{ "type": "text", "text": llm_text_with_footer }],
                "ir": crate::ir::hierarchical::ir_to_hierarchical_wire(&ir),
                "pretty": ir_value, "v": ir.version, "file": ir.file_id,
                "content_kind": content_kind, "byte_exact": byte_exact
            }
        })
    } else {
        // Phase A retirement (2026-08-25): the legacy `$`/`⊕`/`§`
        // text-compression fallback has been REMOVED. When the primary IR
        // compiler fails we return a structured `ir_unavailable` error
        // instead of silently degrading the LLM-facing notation.
        //
        // (Removing this branch also eliminates a latent self-deadlock:
        // the old body called `format_dict_footer()` — which locks the
        // dictionary — while the `match` scrutinee temporaries still held
        // `state.dict_lock()` / `state.cache_write()` for the legacy
        // compressor call. The branch was unreachable before retirement,
        // so the deadlock had never been observed.)
        let reason = ir_result
            .err()
            .map(|e| e.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        tracing::warn!(
            error = %reason,
            path = %resolved_path,
            "IR compilation failed; returning structured ir_unavailable error"
        );
        serde_json::json!({
            "jsonrpc": "2.0", "id": id,
            "error": {
                "code": -32603,
                "message": format!(
                    "IR compilation unavailable for {}: {}. SCHEMA v2 output \
                     cannot be produced for this input; retry with fidelity \
                     \"verbatim\" or read the source directly.",
                    resolved_path, reason
                ),
                "data": {
                    "reason": "ir_unavailable",
                    "path": resolved_path,
                    "ir_compiler": reason,
                }
            }
        })
    };

    // Inject baseline cache breakpoint into the response so the LLM
    // client can set cache_control on the stable compressed output.
    if let Some(text) = response["result"]["content"][0]["text"].as_str() {
        let text_owned = text.to_string();
        inject_baseline_breakpoint(&mut response, state, &text_owned);
    }

    send_response(&response);
}

// ── Handler: diff_code_context ────────────────────────────────────

pub(crate) fn handle_diff_code_context(id: &Value, params: &Value, state: &McpState) {
    let file_path_str = params["arguments"]["filePath"].as_str().unwrap_or("");
    let workspace_root = params["arguments"]["workspaceRoot"].as_str();
    let resolved_path = match resolve_file_path_checked(
        file_path_str,
        workspace_root,
        &state.config.additional_roots,
    ) {
        Ok(p) => p,
        Err(msg) => {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "error": { "code": -32602, "message": msg }
            }));
            return;
        }
    };
    let fidelity = match parse_fidelity_arg(id, params, &state.config) {
        Ok(f) => f,
        Err(()) => return,
    };
    // A-08: Use source_cache via state.read_source() instead of direct disk read
    let source = match state.read_source(&resolved_path) {
        Ok(s) => s.as_str().to_string(),
        Err(e) => {
            send_response(
                &serde_json::json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32603, "message": format!("Cannot read file: {}", e) } }),
            );
            return;
        }
    };
    match diff_code_context_handler(
        PathBuf::from(&resolved_path),
        &source,
        &mut state.cache_write(),
        fidelity,
    ) {
        Ok(body) => {
            let mut response = serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "result": { "content": [{ "type": "text", "text": body }] }
            });
            // Diff output is rolling dynamic content — mark as tail (ephemeral).
            inject_tail_breakpoint(&mut response, state);
            send_response(&response);
        }
        Err(e) => send_response(&serde_json::json!({
            "jsonrpc": "2.0", "id": id,
            "error": { "code": -32603, "message": e.to_string() }
        })),
    }
}

// ── Handler: delta_code_context ───────────────────────────────────

pub(crate) fn handle_delta_code_context(id: &Value, params: &Value, state: &McpState) {
    let file_path_str = params["arguments"]["filePath"].as_str().unwrap_or("");
    let workspace_root = params["arguments"]["workspaceRoot"].as_str();
    let resolved_path = match resolve_file_path_checked(
        file_path_str,
        workspace_root,
        &state.config.additional_roots,
    ) {
        Ok(p) => p,
        Err(msg) => {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "error": { "code": -32602, "message": msg }
            }));
            return;
        }
    };
    let fidelity = match parse_fidelity_arg(id, params, &state.config) {
        Ok(f) => f,
        Err(()) => return,
    };

    // A-08: Check if source has changed before compiling
    let path_alias = state.get_or_create_alias(resolved_path.clone());
    let prev_version = state.file_version(&path_alias).unwrap_or(0);

    // Try to skip compilation if source is unchanged
    // P0-4: Hold lock during entire check to prevent TOCTOU race
    let ir_ctx = state.ir_context_lock();
    if prev_version > 0 && ir_ctx.has_file(&path_alias) {
        if let Ok(source_arc) = state.read_source(&resolved_path) {
            let source_hash = {
                let cache = state.cache_read();
                cache.compute_hash(source_arc.as_bytes())
            };

            if ir_ctx.is_source_unchanged(&path_alias, &source_hash) {
                // Source unchanged - return cached IR without recompiling
                // P0-4: Lock still held, ensuring consistent state
                let cached_ir = ir_ctx.get_ir(&path_alias).unwrap().clone();
                let instruction_count = cached_ir.len();
                drop(ir_ctx);
                let mut response = serde_json::json!({
                    "jsonrpc": "2.0", "id": id,
                    "result": {
                        "content": [{ "type": "text", "text": format!("Cached IR for {} (v{})", path_alias, prev_version) }],
                        "version": prev_version,
                        "instruction_count": instruction_count,
                        "cached": true
                    }
                });
                // Cached IR is a stable snapshot — inject baseline breakpoint
                // so the client can cache the unchanged output.
                if let Some(text) = response["result"]["content"][0]["text"].as_str() {
                    let text_owned = text.to_string();
                    inject_baseline_breakpoint(&mut response, state, &text_owned);
                }
                send_response(&response);
                return;
            }
        }
    }
    drop(ir_ctx); // Release lock before expensive compile

    // Source changed or no baseline - compile
    let (compiled, source_hash) = match compile_file_ir(&resolved_path, fidelity, state) {
        Ok(c) => c,
        Err(e) => {
            send_response(&to_jsonrpc_error(id, &e));
            return;
        }
    };

    // P0-4: Re-acquire lock atomically for delta computation
    // This ensures no other worker modified ir_context between our check and delta computation
    let mut ir_ctx = state.ir_context_lock();
    let delta = if prev_version > 0 && ir_ctx.has_file(&path_alias) {
        ir_ctx
            .get_ir(&path_alias)
            .cloned()
            .and_then(|prev_instructions| {
                let prev_compiled = CompiledIR {
                    file_id: path_alias.clone(),
                    version: prev_version,
                    instructions: tuples_to_coreops(prev_instructions),
                };
                DeltaComputer::new().compute(&prev_compiled, &compiled)
            })
    } else {
        ir_ctx.load_ir(compiled.clone(), Some(source_hash));
        None
    };
    drop(ir_ctx);

    match delta {
        Some(d) => {
            let wire_delta = serde_json::to_value(&d).unwrap_or_default();
            let (content_kind, byte_exact) = contract_fields(fidelity);
            let mut response = serde_json::json!({
                "jsonrpc": "2.0", "id": id, "result": {
                    "content": [{ "type": "text", "text": format!("Δ delta for {} (v{} → v{}): +{} ~{} -{} ops", compiled.file_id, d.from, d.to, d.ops.adds.len(), d.ops.mods.len(), d.ops.dels.len()) }],
                    "delta": wire_delta, "from_version": d.from, "to_version": d.to,
                    "strategy": "delta", "fidelity": format!("{:?}", fidelity).to_lowercase(),
                    "content_kind": content_kind, "byte_exact": byte_exact,
                    "degradation": null
                }
            });
            // Delta output is rolling dynamic content — mark as tail (ephemeral).
            inject_tail_breakpoint(&mut response, state);
            send_response(&response);
        }
        None => {
            let mut response = serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "result": {
                    "content": [{ "type": "text", "text": format!("Baseline stored for {} (v{})", compiled.file_id, compiled.version) }],
                    "version": compiled.version, "instruction_count": compiled.instructions.len()
                }
            });
            // Baseline stored — this is a stable snapshot, inject baseline breakpoint.
            if let Some(text) = response["result"]["content"][0]["text"].as_str() {
                let text_owned = text.to_string();
                inject_baseline_breakpoint(&mut response, state, &text_owned);
            }
            send_response(&response);
        }
    }
}

// ── Handler: delta_text_context ───────────────────────────────────

pub(crate) fn handle_delta_text_context(id: &Value, params: &Value, state: &McpState) {
    let file_path_str = params["arguments"]["filePath"].as_str().unwrap_or("");
    let workspace_root = params["arguments"]["workspaceRoot"].as_str();
    let resolved_path = match resolve_file_path_checked(
        file_path_str,
        workspace_root,
        &state.config.additional_roots,
    ) {
        Ok(p) => p,
        Err(msg) => {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "error": { "code": -32602, "message": msg }
            }));
            return;
        }
    };
    let fidelity = match parse_fidelity_arg(id, params, &state.config) {
        Ok(f) => f,
        Err(()) => return,
    };
    match compress_text_body(&resolved_path, fidelity, state) {
        Ok((body_lines, full_output)) => {
            let alias = state.get_or_create_alias(resolved_path.clone());
            let mut td = state.text_delta_lock();
            if td.has_baseline(&alias) {
                if let Some(delta) = td.compute_delta(&alias, &body_lines) {
                    // Store updated baseline for next call
                    td.store_snapshot(&alias, body_lines.clone());
                    drop(td);
                    let mut response = serde_json::json!({
                        "jsonrpc": "2.0", "id": id,
                        "result": {
                            "content": [{ "type": "text", "text": delta.to_wire_format() }],
                            "added": delta.adds.len(),
                            "removed": delta.dels.len(),
                            "modified": delta.mods.len(),
                        }
                    });
                    // Delta output is rolling dynamic content — mark as tail (ephemeral).
                    inject_tail_breakpoint(&mut response, state);
                    send_response(&response);
                } else {
                    drop(td);
                    let mut response = serde_json::json!({
                        "jsonrpc": "2.0", "id": id,
                        "result": {
                            "content": [{ "type": "text", "text": "No changes since last call." }]
                        }
                    });
                    // "No changes" is a stable response — inject baseline breakpoint
                    // so the client can cache the unchanged output.
                    if let Some(text) = response["result"]["content"][0]["text"].as_str() {
                        let text_owned = text.to_string();
                        inject_baseline_breakpoint(&mut response, state, &text_owned);
                    }
                    send_response(&response);
                }
            } else {
                // Actually store the baseline snapshot on first call
                td.store_snapshot(&alias, body_lines.clone());
                drop(td);
                let mut response = serde_json::json!({
                    "jsonrpc": "2.0", "id": id,
                    "result": {
                        "content": [{ "type": "text", "text": format!("Baseline stored for {}.\nCall again after edits.\n\nFull output:\n{}", alias, full_output) }],
                        "stored": true
                    }
                });
                // Baseline stored — this is a stable snapshot, inject baseline breakpoint.
                if let Some(text) = response["result"]["content"][0]["text"].as_str() {
                    let text_owned = text.to_string();
                    inject_baseline_breakpoint(&mut response, state, &text_owned);
                }
                send_response(&response);
            }
        }
        Err(e) => send_response(&serde_json::json!({
            "jsonrpc": "2.0", "id": id,
            "error": { "code": -32603, "message": e.to_string() }
        })),
    }
}

// ── Handler: apply_delta ──────────────────────────────────────────

pub(crate) fn handle_apply_delta(id: &Value, params: &Value, state: &McpState) {
    let delta_value = &params["arguments"]["delta"];
    let current_version = params["arguments"]["currentVersion"].as_i64();

    let delta: IRDelta = match serde_json::from_value(delta_value.clone()) {
        Ok(d) => d,
        Err(e) => {
            send_response(
                &serde_json::json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32602, "message": format!("Invalid delta: {}", e) } }),
            );
            return;
        }
    };

    if current_version != Some(delta.from as i64) {
        send_response(
            &serde_json::json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32602, "message": format!("Version mismatch: client has v{:?}, delta expects from v{}", current_version, delta.from) } }),
        );
        return;
    }

    let file = delta.file.clone();
    let mut ir_ctx = state.ir_context_lock();
    match ir_ctx.apply(delta) {
        Ok(new_version) => {
            let rendered = ir_ctx.render_pretty(&file, crate::compressor::Fidelity::Low);
            let mut response = serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "result": { "content": [{ "type": "text", "text": rendered.unwrap_or_default() }], "version": new_version }
            });
            // Applied delta output is rolling dynamic content — mark as tail (ephemeral).
            inject_tail_breakpoint(&mut response, state);
            send_response(&response);
        }
        Err(e) => send_response(&serde_json::json!({
            "jsonrpc": "2.0", "id": id,
            "error": { "code": -32603, "message": format!("Apply delta failed: {}", e) }
        })),
    }
}

// ── Handler: provide_code_context ─────────────────────────────────

pub(crate) fn handle_provide_code_context(id: &Value, params: &Value, state: &McpState) {
    use std::time::Instant;
    let overall_start = Instant::now();

    // Symbol targeting: optional set of method names that should receive
    // full verbatim bodies at Edit fidelity. All other methods are rendered
    // signature-only. When omitted (None), every method's body is rendered
    // (current default behavior).
    let focus_methods: Option<HashSet<String>> =
        params["arguments"]["focusMethods"].as_array().map(|arr| {
            arr.iter()
                .filter_map(|s| s.as_str().map(String::from))
                .collect()
        });

    let file_path_str = params["arguments"]["filePath"].as_str().unwrap_or("");
    if file_path_str.is_empty() {
        send_response(
            &serde_json::json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32602, "message": "Missing required parameter: filePath" } }),
        );
        return;
    }

    let workspace_root = params["arguments"]["workspaceRoot"].as_str();
    let resolved_path = match resolve_file_path_checked(
        file_path_str,
        workspace_root,
        &state.config.additional_roots,
    ) {
        Ok(p) => p,
        Err(msg) => {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "error": { "code": -32602, "message": msg }
            }));
            return;
        }
    };

    if state.config.is_excluded(&resolved_path) {
        send_response(
            &serde_json::json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32603, "message": format!("File excluded by config: {}", file_path_str) } }),
        );
        return;
    }

    // A-13: Check resource limits before processing
    let limits = &state.config.resource_limits;

    // Check file size if we can read it
    if let Ok(metadata) = std::fs::metadata(&resolved_path) {
        if let Err(e) = limits.check_file_size(metadata.len()) {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "error": { "code": -32603, "message": e }
            }));
            return;
        }
    }

    let source_arc = match state.read_source(&resolved_path) {
        Ok(s) => s,
        Err(e) => {
            send_response(
                &serde_json::json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32603, "message": format!("Cannot read file: {}", e) } }),
            );
            return;
        }
    };
    let source = source_arc.as_str();
    let alias = state.get_or_create_alias(resolved_path.clone());

    // ANGULAR_HTML_COMPRESSION_PLAN Phase 3: route `.component.html`
    // files through the Angular template compressor. These files have
    // no tree-sitter grammar for the IR compiler, so we handle them
    // specially before the heuristics decision.
    #[cfg(feature = "angular")]
    if resolved_path.to_lowercase().ends_with(".component.html") {
        let explicit_fidelity = params["arguments"]["fidelity"].as_str();
        let explicit_intent = params["arguments"]["intent"].as_str();
        let fidelity = match explicit_fidelity {
            Some(s) => match crate::compression::Fidelity::parse(s) {
                Ok(f) => f,
                Err(_) => {
                    // Template editing intent → High fidelity.
                    if explicit_intent == Some("edit") {
                        crate::compression::Fidelity::High
                    } else {
                        crate::compression::Fidelity::Medium
                    }
                }
            },
            None => {
                // Template editing intent → High fidelity.
                if explicit_intent == Some("edit") {
                    crate::compression::Fidelity::High
                } else {
                    crate::compression::Fidelity::Medium
                }
            }
        };
        // Verbatim fidelity: return the raw template source byte-exact.
        // The plan's fidelity table promises "Full raw source, byte-exact
        // entire document" — the template compressor would otherwise
        // produce a compressed skeleton while `contract_fields` reports
        // `verbatim_document`/`["document"]` (self-reporting contract leak).
        if fidelity == crate::compression::Fidelity::Verbatim {
            let mut response = serde_json::json!({
                "jsonrpc": "2.0", "id": id, "result": {
                    "content": [{ "type": "text", "text": source }],
                    "strategy": "full", "fidelity": "verbatim",
                    "is_angular": true, "template_compressed": false,
                    "content_kind": "verbatim_document", "byte_exact": ["document"],
                    "degradation": null
                }
            });
            inject_baseline_breakpoint(&mut response, state, source);
            send_response(&response);
            return;
        }
        let lines = crate::angular_meta::template_compress::compress_template_with_prime_ng(
            source, fidelity,
        );
        let body = lines.join("\n");
        let tokenizer_kind = parse_tokenizer_arg(params, &state.config);
        let tokenizer_box = crate::tokenizer::create_tokenizer(tokenizer_kind).ok();
        let tokenizer_ref: Option<&dyn crate::tokenizer::Tokenizer> = tokenizer_box.as_deref();
        let raw_tokens = count_tokens_with_tokenizer(source, tokenizer_ref);
        let comp_tokens = count_tokens_with_tokenizer(&body, tokenizer_ref);
        state.record_compression(
            &resolved_path,
            raw_tokens,
            comp_tokens,
            &format!("{:?}", fidelity).to_lowercase(),
            true,
            "full",
            None,
            "angular_template",
        );

        // Persist to DB so `context_stats` and cross-session dashboards
        // can report Angular template compression savings.
        {
            if let Some(ref store) = *state.persistence_store_lock() {
                let mut hasher = Sha256::new();
                hasher.update(source.as_bytes());
                let source_hash = format!("{:x}", hasher.finalize());
                store.queue_save_context(
                    &resolved_path,
                    fidelity,
                    &body,
                    &[],
                    &source_hash,
                    raw_tokens as u64,
                    comp_tokens as u64,
                );
            }
        }
        state.flush_persistence();

        // The Angular template compressor emits structural markers, never
        // verbatim method bodies — so the self-reporting contract must not
        // claim `["method_bodies"]` even at Edit fidelity (it would be a
        // Gap 5/3/6 contract leak: the LLM would attempt replace_in_file
        // SEARCH against bodies that don't exist in template output).
        let (content_kind, byte_exact) = ("skeleton", Vec::<&'static str>::new());
        let mut response = serde_json::json!({
            "jsonrpc": "2.0", "id": id, "result": {
                "content": [{ "type": "text", "text": body }],
                "strategy": "full", "fidelity": format!("{:?}", fidelity).to_lowercase(),
                "is_angular": true, "template_compressed": true,
                "content_kind": content_kind, "byte_exact": byte_exact,
                "degradation": null
            }
        });
        inject_baseline_breakpoint(&mut response, state, &body);
        send_response(&response);
        return;
    }

    let explicit_fidelity = params["arguments"]["fidelity"].as_str();
    let explicit_intent = params["arguments"]["intent"].as_str();

    // Phase 1: Heuristics decision
    let heuristics_start = Instant::now();
    let td_guard = state.text_delta_lock();
    let ir_read = state.ir_context_read();
    let decision = match crate::mcp::heuristics::decide(
        &resolved_path,
        explicit_fidelity,
        explicit_intent,
        &state.config,
        &td_guard,
        &ir_read,
        source,
        Some(&alias),
        None,
    ) {
        Ok(d) => d,
        Err(e) => {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "error": { "code": -32602, "message": e }
            }));
            return;
        }
    };
    drop(td_guard);
    drop(ir_read);
    let heuristics_ms = heuristics_start.elapsed().as_millis() as u64;

    let effective_fidelity = decision.fidelity;
    // Gap 5/3/6 fixes: self-reporting contract fields (content_kind,
    // byte_exact) plus a degradation signal for the legacy fallback.
    // When `focusMethods` is supplied, only the focused method bodies are
    // byte-exact — the contract must reflect that (not claim every body).
    let (content_kind, byte_exact) =
        contract_fields_focused(effective_fidelity, focus_methods.as_ref());
    let strategy = decision.strategy;
    let is_angular = decision.is_angular;
    let tokenizer_kind = parse_tokenizer_arg(params, &state.config);
    let tokenizer_box = crate::tokenizer::create_tokenizer(tokenizer_kind).ok();
    let tokenizer_ref: Option<&dyn crate::tokenizer::Tokenizer> = tokenizer_box.as_deref();

    // Verbatim fidelity: return the full raw source byte-exact, exactly as
    // the plan's fidelity table promises ("Full raw source, byte-exact
    // entire document"). Bypasses IR/legacy compression entirely so the
    // `verbatim_document`/`["document"]` contract fields match the payload.
    if effective_fidelity == crate::compression::Fidelity::Verbatim {
        let full = source.to_string();
        let raw_tokens = count_tokens_with_tokenizer(source, tokenizer_ref);
        state.record_compression(
            &resolved_path,
            raw_tokens,
            raw_tokens,
            "verbatim",
            is_angular,
            "full",
            None,
            "verbatim",
        );
        let mut response = serde_json::json!({
            "jsonrpc": "2.0", "id": id, "result": {
                "content": [{ "type": "text", "text": full }],
                "strategy": "full", "fidelity": "verbatim",
                "decision_summary": decision.summary(),
                "content_kind": "verbatim_document", "byte_exact": ["document"],
                "degradation": null, "verbatim": true
            }
        });
        inject_baseline_breakpoint(&mut response, state, &full);
        send_response(&response);
        return;
    }

    // A-04: Create tracing span for this call
    let _span = tracing::info_span!(
        "provide_code_context",
        file_path = %resolved_path,
        fidelity = %format!("{:?}", effective_fidelity),
        strategy = %format!("{:?}", strategy),
        cbm_status = %state.cbm_status.summary(),
        is_angular = %is_angular,
    )
    .entered();

    match strategy {
        crate::mcp::heuristics::ContextStrategy::DeltaTransport => {
            let compile_start = Instant::now();
            let (compiled, _source_hash) = match compile_file_ir_focused(
                &resolved_path,
                effective_fidelity,
                state,
                focus_methods.as_ref(),
            ) {
                Ok(c) => c,
                Err(e) => {
                    send_response(&to_jsonrpc_error(id, &e));
                    return;
                }
            };
            let compile_ms = compile_start.elapsed().as_millis() as u64;

            let delta_start = Instant::now();
            let prev_version = state.file_version(&alias).unwrap_or(0);
            let mut ir_ctx = state.ir_context_lock();
            let delta = if prev_version > 0 && ir_ctx.has_file(&alias) {
                ir_ctx
                    .get_ir(&alias)
                    .cloned()
                    .and_then(|prev_instructions| {
                        let prev_compiled = CompiledIR {
                            file_id: alias.clone(),
                            version: prev_version,
                            instructions: tuples_to_coreops(prev_instructions),
                        };
                        DeltaComputer::new().compute(&prev_compiled, &compiled)
                    })
            } else {
                ir_ctx.load_ir(compiled.clone(), None);
                None
            };
            drop(ir_ctx);
            let _delta_ms = delta_start.elapsed().as_millis() as u64;

            let raw_tokens;
            let comp_tokens;

            match delta {
                Some(ref d) => {
                    let wire_delta = serde_json::to_value(d).unwrap_or_default();
                    // DASHBOARD FIX (R-02 FAANG): count the delta wire tokens
                    // (the actual payload sent to the LLM) so the dashboard
                    // can show delta efficiency. The previous full compression's
                    // compressed token count is passed as `full_compressed_tokens`
                    // so `record_compression` can compute CPU savings vs a full
                    // re-compress.
                    let delta_text = serde_json::to_string(&wire_delta).unwrap_or_default();
                    raw_tokens = count_tokens_with_tokenizer(&delta_text, tokenizer_ref);
                    comp_tokens = raw_tokens; // delta is the payload itself
                    let prev_full_compressed = state
                        .session_stats_lock()
                        .file_stats(&resolved_path)
                        .map(|f| f.compressed_tokens);
                    let mut response = serde_json::json!({
                        "jsonrpc": "2.0", "id": id, "result": {
                            "content": [{ "type": "text", "text": format!("Δ delta for {} (v{} → v{}): +{} ~{} -{} ops", compiled.file_id, d.from, d.to, d.ops.adds.len(), d.ops.mods.len(), d.ops.dels.len()) }],
                            "delta": wire_delta, "from_version": d.from, "to_version": d.to,
                            "strategy": "delta", "fidelity": format!("{:?}", effective_fidelity).to_lowercase(),
                            "decision_summary": decision.summary(),
                            "content_kind": content_kind, "byte_exact": byte_exact,
                            "degradation": null
                        }
                    });
                    // Delta output is rolling dynamic content — mark as tail (ephemeral).
                    inject_tail_breakpoint(&mut response, state);
                    send_response(&response);
                    // Record the delta with the previous full compressed token
                    // count for delta efficiency computation.
                    state.record_compression(
                        &resolved_path,
                        raw_tokens,
                        comp_tokens,
                        &format!("{:?}", effective_fidelity).to_lowercase(),
                        is_angular,
                        "delta",
                        prev_full_compressed,
                        "ir_compression",
                    );
                }
                None => {
                    let render_start = Instant::now();
                    let hir = crate::ir::hierarchical::ir_to_hierarchical(&compiled);
                    let llm_text = crate::ir::render_hierarchical_for_llm_focused(
                        &hir,
                        effective_fidelity,
                        focus_methods.as_ref(),
                    );
                    let full = format!(
                        "{}\n// ── {} ({}) ──\n{}",
                        llm_text.trim(),
                        compiled.file_id,
                        resolved_path,
                        state.format_dict_footer().trim()
                    );
                    let render_ms = render_start.elapsed().as_millis() as u64;
                    raw_tokens = count_tokens_with_tokenizer(source, tokenizer_ref);
                    comp_tokens = count_tokens_with_tokenizer(&full, tokenizer_ref);
                    state.record_compression(
                        &resolved_path,
                        raw_tokens,
                        comp_tokens,
                        &format!("{:?}", effective_fidelity).to_lowercase(),
                        is_angular,
                        "full",
                        None,
                        "ir_compression",
                    );
                    let mut response = serde_json::json!({
                        "jsonrpc": "2.0", "id": id, "result": {
                            "content": [{ "type": "text", "text": full }], "version": compiled.version,
                            "strategy": "full", "fidelity": format!("{:?}", effective_fidelity).to_lowercase(),
                            "decision_summary": decision.summary(),
                            "content_kind": content_kind, "byte_exact": byte_exact,
                            "degradation": null
                        }
                    });
                    // Inject baseline cache breakpoint for the stable full-compression output.
                    inject_baseline_breakpoint(&mut response, state, &full);
                    send_response(&response);
                    tracing::info!(
                        heuristics_ms = heuristics_ms,
                        compile_ms = compile_ms,
                        delta_ms = _delta_ms,
                        render_ms = render_ms,
                        raw_tokens = raw_tokens,
                        comp_tokens = comp_tokens,
                        savings_pct = if raw_tokens > 0 {
                            ((raw_tokens - comp_tokens) as f64 / raw_tokens as f64 * 100.0) as u64
                        } else {
                            0
                        },
                        "provide_code_context delta full complete"
                    );
                }
            }
            let _total_ms = overall_start.elapsed().as_millis() as u64;
            // The delta branch already recorded stats inside `Some(d)`.
            // The `None` branch (baseline stored) records a full compression
            // below. This trailing call is now a no-op for the delta case
            // (it would double-record), so we only record for the None branch.
            if delta.is_none() {
                state.record_compression(
                    &resolved_path,
                    raw_tokens,
                    comp_tokens,
                    &format!("{:?}", effective_fidelity).to_lowercase(),
                    is_angular,
                    "delta",
                    None,
                    "ir_compression",
                );
            }
        }
        crate::mcp::heuristics::ContextStrategy::FullCompress => {
            let compile_start = Instant::now();
            let ir_result = compile_file_ir_focused(
                &resolved_path,
                effective_fidelity,
                state,
                focus_methods.as_ref(),
            );
            let compile_ms = compile_start.elapsed().as_millis() as u64;

            if let Ok((ir, _source_hash)) = ir_result {
                let render_start = Instant::now();
                // Note: IR error is logged below in the else branch (4.4 audit fix)
                state.ir_context_lock().load_ir(ir.clone(), None);
                let hir = crate::ir::hierarchical::ir_to_hierarchical(&ir);
                let llm_text = crate::ir::render_hierarchical_for_llm_focused(
                    &hir,
                    effective_fidelity,
                    focus_methods.as_ref(),
                );
                let full = format!(
                    "{}\n// ── {} ({}) ──\n{}",
                    llm_text.trim(),
                    ir.file_id,
                    resolved_path,
                    state.format_dict_footer().trim()
                );
                state
                    .llm_text_cache_lock()
                    .insert(ir.file_id.clone(), full.clone());
                let render_ms = render_start.elapsed().as_millis() as u64;
                let raw_tokens = count_tokens_with_tokenizer(source, tokenizer_ref);
                let comp_tokens = count_tokens_with_tokenizer(&full, tokenizer_ref);
                state.record_compression(
                    &resolved_path,
                    raw_tokens,
                    comp_tokens,
                    &format!("{:?}", effective_fidelity).to_lowercase(),
                    is_angular,
                    "full",
                    None,
                    "ir_compression",
                );
                let mut response = serde_json::json!({
                    "jsonrpc": "2.0", "id": id, "result": {
                        "content": [{ "type": "text", "text": full }], "version": ir.version,
                        "strategy": "full", "fidelity": format!("{:?}", effective_fidelity).to_lowercase(),
                        "is_angular": is_angular, "decision_summary": decision.summary(),
                        "content_kind": content_kind, "byte_exact": byte_exact,
                        "degradation": null
                    }
                });
                // Inject baseline cache breakpoint for the stable full-compression output.
                inject_baseline_breakpoint(&mut response, state, &full);
                send_response(&response);
                let total_ms = overall_start.elapsed().as_millis() as u64;
                tracing::info!(
                    heuristics_ms = heuristics_ms,
                    compile_ms = compile_ms,
                    render_ms = render_ms,
                    total_ms = total_ms,
                    raw_tokens = raw_tokens,
                    comp_tokens = comp_tokens,
                    savings_pct = if raw_tokens > 0 {
                        ((raw_tokens - comp_tokens) as f64 / raw_tokens as f64 * 100.0) as u64
                    } else {
                        0
                    },
                    "provide_code_context full complete"
                );
            } else {
                // Phase A retirement (2026-08-25): legacy `$`/`⊕`/`§`
                // fallback removed (see compress_code_context site — its
                // removal also eliminates a latent dict-lock self-deadlock).
                let reason = ir_result
                    .err()
                    .map(|e| e.to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                tracing::warn!(
                    error = %reason,
                    path = %resolved_path,
                    "IR compilation failed in provide_code_context; returning structured ir_unavailable error"
                );
                send_response(&serde_json::json!({
                    "jsonrpc": "2.0", "id": id,
                    "error": {
                        "code": -32603,
                        "message": format!(
                            "IR compilation unavailable for {}: {}. SCHEMA v2 output \
                             cannot be produced for this input; retry with fidelity \
                             \"verbatim\" or read the source directly.",
                            resolved_path, reason
                        ),
                        "data": {
                            "reason": "ir_unavailable",
                            "path": resolved_path,
                            "ir_compiler": reason,
                        }
                    }
                }));
            }
        }
    }
}

// ── Handler: restore_context ───────────────────────────────────────

pub(crate) fn handle_restore_context(id: &Value, params: &Value, state: &McpState) {
    let file_path_str = params["arguments"]["filePath"].as_str().unwrap_or("");
    if file_path_str.is_empty() {
        send_response(
            &serde_json::json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32602, "message": "Missing required parameter: filePath" } }),
        );
        return;
    }
    let workspace_root = params["arguments"]["workspaceRoot"].as_str();
    let resolved_path = match resolve_file_path_checked(
        file_path_str,
        workspace_root,
        &state.config.additional_roots,
    ) {
        Ok(p) => p,
        Err(msg) => {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "error": { "code": -32602, "message": msg }
            }));
            return;
        }
    };
    let fidelity = match parse_fidelity_arg(id, params, &state.config) {
        Ok(f) => f,
        Err(()) => return,
    };

    // A-13: Check resource limits before processing
    let limits = &state.config.resource_limits;

    // Check file size if we can read it
    if let Ok(metadata) = std::fs::metadata(&resolved_path) {
        if let Err(e) = limits.check_file_size(metadata.len()) {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "error": { "code": -32603, "message": e }
            }));
            return;
        }
    }

    let path_alias = state.get_or_create_alias(resolved_path.clone());
    state.ir_context_lock().remove_file(&path_alias);
    state.llm_text_cache_lock().remove(&path_alias);

    // Clear persistence DB entry for this file (use resolved_path, not alias)
    if let Some(ref mut store) = *state.persistence_store_lock() {
        store.clear_file(&resolved_path);
    }

    let source_arc = match state.read_source(&resolved_path) {
        Ok(s) => s,
        Err(e) => {
            send_response(
                &serde_json::json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32603, "message": format!("Cannot read file: {}", e) } }),
            );
            return;
        }
    };
    // The read doubles as an existence check before IR compilation;
    // the content itself is only consumed by the compiler via the
    // shared source cache (Phase A removed the legacy text consumer).
    let _source_text = source_arc.as_str();

    match compile_file_ir(&resolved_path, fidelity, state) {
        Ok((ir, _source_hash)) => {
            let hir = crate::ir::hierarchical::ir_to_hierarchical(&ir);
            let llm_text = crate::ir::render_hierarchical_for_llm(&hir, fidelity);
            let full = format!(
                "{}\n// ── {} ({}) ──\n{}",
                llm_text.trim(),
                ir.file_id,
                resolved_path,
                state.format_dict_footer().trim()
            );
            state
                .llm_text_cache_lock()
                .insert(ir.file_id.clone(), full.clone());
            let mut response = serde_json::json!({ "jsonrpc": "2.0", "id": id, "result": { "content": [{ "type": "text", "text": full }], "version": ir.version, "restored": true } });
            // Restored context is a stable full snapshot — inject baseline breakpoint.
            inject_baseline_breakpoint(&mut response, state, &full);
            send_response(&response);
        }
        Err(e) => {
            // Phase A retirement (2026-08-25): legacy `$`/`⊕`/`§` fallback
            // removed (see compress_code_context site).
            let reason = e.to_string();
            tracing::warn!(
                error = %reason,
                path = %resolved_path,
                "IR compilation failed in restore_context; returning structured ir_unavailable error"
            );
            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "error": {
                    "code": -32603,
                    "message": format!(
                        "IR compilation unavailable for {}: {}. SCHEMA v2 output \
                         cannot be produced for this input; retry with fidelity \
                         \"verbatim\" or read the source directly.",
                        resolved_path, reason
                    ),
                    "data": {
                        "reason": "ir_unavailable",
                        "path": resolved_path,
                        "ir_compiler": reason,
                    }
                }
            }));
        }
    }
}
