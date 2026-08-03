// src/mcp/tool_handlers/core.rs
//
// Core MCP tool handlers: compression, IR, delta, and unified entry point.
use std::path::PathBuf;
use serde_json::Value;
use sha2::{Sha256, Digest};
use crate::ir::wire::ir_to_wire;
use crate::ir::wire::tuple_to_op;
use crate::ir::delta::{IRDelta, DeltaComputer};
use crate::ir::compiler::CompiledIR;
use crate::ir::opcodes::CoreOp;
use crate::error::to_jsonrpc_error;
use crate::mcp::context_store::ContextStore;
use crate::mcp::McpState;
use crate::protocol::send_response;

use super::super::tools::{parse_fidelity_arg, parse_tokenizer_arg};
use super::super::tool_helpers::{compress_text_body, compile_file_ir, resolve_file_path, diff_code_context_handler, count_tokens_with_tokenizer};

fn tuples_to_coreops(tuples: Vec<Vec<String>>) -> Vec<CoreOp> {
    tuples.into_iter().filter_map(|t| tuple_to_op(&t)).collect()
}

// ── Handler: compress_code_context ───────────────────────────────

/// P3-2: Main handler for compress_code_context tool.
/// Orchestrates validation, compilation, and response building.
pub(crate) fn handle_compress_code_context(
    id: &Value,
    params: &Value,
    state: &McpState,
) {
    let file_path_str = params["arguments"]["filePath"].as_str().unwrap_or("");
    let encoding = params["arguments"]["encoding"].as_str().unwrap_or("named");
    let workspace_root = params["arguments"]["workspaceRoot"].as_str();
    let resolved_path = resolve_file_path(file_path_str, workspace_root);
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
    let source_arc = state.read_source(&resolved_path).ok();
    let source_ref = source_arc.as_ref().map(|s| s.as_str());
    let source_text = source_ref.unwrap_or("");

    let ir_result = compile_file_ir(&resolved_path, effective_fidelity, state);

    let tokenizer_kind = parse_tokenizer_arg(params, &state.config);
    let tokenizer_box = crate::tokenizer::create_tokenizer(tokenizer_kind).ok();
    let tokenizer_ref: Option<&dyn crate::tokenizer::Tokenizer> = tokenizer_box.as_deref();

    // P3-2: Build response using extracted helpers
    // If IR compilation fails, fall back to legacy compression but log
    // the structured error for diagnostics (4.4 audit fix).
    let response = if let Ok((ir, _source_hash)) = ir_result {
        state.ir_context_lock().load_ir(ir.clone(), None);
        let hir = crate::ir::hierarchical::ir_to_hierarchical(&ir);
        let llm_text = crate::ir::render_hierarchical_for_llm(&hir, effective_fidelity);
        let footer = state.format_dict_footer();
        let llm_text_with_footer = format!("{}\n// ── {} ({}) ──\n{}",
            llm_text.trim(), ir.file_id, resolved_path, footer.trim());
        state.llm_text_cache_lock().insert(ir.file_id.clone(), llm_text_with_footer.clone());

        let raw_tokens = count_tokens_with_tokenizer(source_text, tokenizer_ref);
        let compressed_tokens = count_tokens_with_tokenizer(&llm_text_with_footer, tokenizer_ref);
        state.record_compression(&resolved_path, raw_tokens, compressed_tokens,
            &format!("{:?}", effective_fidelity).to_lowercase(), false, "full", None, "ir_compression");

        // Persist to DB
        {
            if let Some(ref store) = *state.persistence_store_lock() {
                let mut hasher = Sha256::new();
                hasher.update(source_text.as_bytes());
                let source_hash = format!("{:x}", hasher.finalize());
                
                store.queue_save_context(
                    &resolved_path, effective_fidelity, &llm_text_with_footer,
                    &[], &source_hash, raw_tokens as u64, compressed_tokens as u64,
                );
            }
        }
        state.flush_persistence();

        let ir_value = match encoding {
            "positional" => {
                let config = crate::ir::positional::PositionalConfig::stripped();
                crate::ir::positional::ir_to_positional_wire(&ir.file_id, ir.version, &ir.instructions, config)
            }
            "tagged" => {
                let config = crate::ir::positional::PositionalConfig::tagged();
                crate::ir::positional::ir_to_positional_wire(&ir.file_id, ir.version, &ir.instructions, config)
            }
            _ => ir_to_wire(&ir),
        };

        serde_json::json!({
            "jsonrpc": "2.0", "id": id,
            "result": {
                "content": [{ "type": "text", "text": llm_text_with_footer }],
                "ir": crate::ir::hierarchical::ir_to_hierarchical_wire(&ir),
                "pretty": ir_value, "v": ir.version, "file": ir.file_id
            }
        })
    } else {
        // Log the structured IR error before falling back (4.4 audit fix)
        if let Err(ref e) = ir_result {
            tracing::warn!(error = %e, path = %resolved_path, "IR compilation failed, falling back to legacy compression");
        }
        match crate::compression::pipeline::compress_file_with_source(
            PathBuf::from(&resolved_path), source_ref,
            &mut state.dict_lock(), &mut state.cache_write(), effective_fidelity,
            Some(&state.config),
        ) {
            Ok(mut compressed_text) => {
                compressed_text.push_str(&state.format_dict_footer());
                let raw_tokens = count_tokens_with_tokenizer(source_text, tokenizer_ref);
                let comp_tokens = count_tokens_with_tokenizer(&compressed_text, tokenizer_ref);
                state.record_compression(&resolved_path, raw_tokens, comp_tokens,
                    &format!("{:?}", effective_fidelity).to_lowercase(), false, "full", None, "ir_compression");

                // Persist to DB
                {
                    if let Some(ref store) = *state.persistence_store_lock() {
                        let mut hasher = Sha256::new();
                        hasher.update(source_text.as_bytes());
                        let source_hash = format!("{:x}", hasher.finalize());
                        
                        store.queue_save_context(
                            &resolved_path, effective_fidelity, &compressed_text,
                            &[], &source_hash, raw_tokens as u64, comp_tokens as u64,
                        );
                    }
                }
                state.flush_persistence();

                serde_json::json!({
                    "jsonrpc": "2.0", "id": id,
                    "result": { "content": [{ "type": "text", "text": compressed_text }] }
                })
            }
            Err(e) => {
                serde_json::json!({
                    "jsonrpc": "2.0", "id": id,
                    "error": { "code": -32603, "message": e.to_string() }
                })
            }
        }
    };
    send_response(&response);
}

// ── Handler: diff_code_context ────────────────────────────────────

pub(crate) fn handle_diff_code_context(
    id: &Value,
    params: &Value,
    state: &McpState,
) {
    let file_path_str = params["arguments"]["filePath"].as_str().unwrap_or("");
    let workspace_root = params["arguments"]["workspaceRoot"].as_str();
    let resolved_path = resolve_file_path(file_path_str, workspace_root);
    let fidelity = match parse_fidelity_arg(id, params, &state.config) {
        Ok(f) => f,
        Err(()) => return,
    };
    // A-08: Use source_cache via state.read_source() instead of direct disk read
    let source = match state.read_source(&resolved_path) {
        Ok(s) => s.as_str().to_string(),
        Err(e) => { send_response(&serde_json::json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32603, "message": format!("Cannot read file: {}", e) } })); return; }
    };
    match diff_code_context_handler(PathBuf::from(&resolved_path), &source, &mut state.cache_write(), fidelity) {
        Ok(body) => send_response(&serde_json::json!({
            "jsonrpc": "2.0", "id": id,
            "result": { "content": [{ "type": "text", "text": body }] }
        })),
        Err(e) => send_response(&serde_json::json!({
            "jsonrpc": "2.0", "id": id,
            "error": { "code": -32603, "message": e.to_string() }
        })),
    }
}

// ── Handler: delta_code_context ───────────────────────────────────

pub(crate) fn handle_delta_code_context(
    id: &Value,
    params: &Value,
    state: &McpState,
) {
    let file_path_str = params["arguments"]["filePath"].as_str().unwrap_or("");
    let workspace_root = params["arguments"]["workspaceRoot"].as_str();
    let resolved_path = resolve_file_path(file_path_str, workspace_root);
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
                send_response(&serde_json::json!({
                    "jsonrpc": "2.0", "id": id,
                    "result": {
                        "content": [{ "type": "text", "text": format!("Cached IR for {} (v{})", path_alias, prev_version) }],
                        "version": prev_version,
                        "instruction_count": instruction_count,
                        "cached": true
                    }
                }));
                return;
            }
        }
    }
    drop(ir_ctx);  // Release lock before expensive compile
    
    // Source changed or no baseline - compile
    let (compiled, source_hash) = match compile_file_ir(&resolved_path, fidelity, state) {
        Ok(c) => c,
        Err(e) => { send_response(&to_jsonrpc_error(id, &e)); return; }
    };
    
    // P0-4: Re-acquire lock atomically for delta computation
    // This ensures no other worker modified ir_context between our check and delta computation
    let mut ir_ctx = state.ir_context_lock();
    let delta = if prev_version > 0 && ir_ctx.has_file(&path_alias) {
        ir_ctx.get_ir(&path_alias).cloned().and_then(|prev_instructions| {
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
            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "result": {
                    "content": [{ "type": "text", "text": serde_json::to_string_pretty(&wire_delta).unwrap_or_default() }],
                    "delta": wire_delta, "from_version": prev_version, "to_version": compiled.version
                }
            }));
        }
        None => send_response(&serde_json::json!({
            "jsonrpc": "2.0", "id": id,
            "result": {
                "content": [{ "type": "text", "text": format!("Baseline stored for {} (v{})", compiled.file_id, compiled.version) }],
                "version": compiled.version, "instruction_count": compiled.instructions.len()
            }
        })),
    }
}

// ── Handler: delta_text_context ───────────────────────────────────

pub(crate) fn handle_delta_text_context(
    id: &Value,
    params: &Value,
    state: &McpState,
) {
    let file_path_str = params["arguments"]["filePath"].as_str().unwrap_or("");
    let workspace_root = params["arguments"]["workspaceRoot"].as_str();
    let resolved_path = resolve_file_path(file_path_str, workspace_root);
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
                    send_response(&serde_json::json!({
                        "jsonrpc": "2.0", "id": id,
                        "result": {
                            "content": [{ "type": "text", "text": delta.to_wire_format() }],
                            "added": delta.adds.len(),
                            "removed": delta.dels.len(),
                            "modified": delta.mods.len(),
                        }
                    }));
                } else {
                    drop(td);
                    send_response(&serde_json::json!({
                        "jsonrpc": "2.0", "id": id,
                        "result": {
                            "content": [{ "type": "text", "text": "No changes since last call." }]
                        }
                    }));
                }
            } else {
                // Actually store the baseline snapshot on first call
                td.store_snapshot(&alias, body_lines.clone());
                drop(td);
                send_response(&serde_json::json!({
                    "jsonrpc": "2.0", "id": id,
                    "result": {
                        "content": [{ "type": "text", "text": format!("Baseline stored for {}.\nCall again after edits.\n\nFull output:\n{}", alias, full_output) }],
                        "stored": true
                    }
                }));
            }
        }
        Err(e) => send_response(&serde_json::json!({
            "jsonrpc": "2.0", "id": id,
            "error": { "code": -32603, "message": e.to_string() }
        })),
    }
}

// ── Handler: apply_delta ──────────────────────────────────────────

pub(crate) fn handle_apply_delta(
    id: &Value,
    params: &Value,
    state: &McpState,
) {
    let delta_value = &params["arguments"]["delta"];
    let current_version = params["arguments"]["currentVersion"].as_i64();

    let delta: IRDelta = match serde_json::from_value(delta_value.clone()) {
        Ok(d) => d,
        Err(e) => { send_response(&serde_json::json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32602, "message": format!("Invalid delta: {}", e) } })); return; }
    };

    if current_version != Some(delta.from as i64) {
        send_response(&serde_json::json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32602, "message": format!("Version mismatch: client has v{:?}, delta expects from v{}", current_version, delta.from) } }));
        return;
    }

    let file = delta.file.clone();
    let mut ir_ctx = state.ir_context_lock();
    match ir_ctx.apply(delta) {
        Ok(new_version) => {
            let rendered = ir_ctx.render_pretty(&file, crate::compressor::Fidelity::Low);
            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "result": { "content": [{ "type": "text", "text": rendered.unwrap_or_default() }], "version": new_version }
            }));
        }
        Err(e) => send_response(&serde_json::json!({
            "jsonrpc": "2.0", "id": id,
            "error": { "code": -32603, "message": format!("Apply delta failed: {}", e) }
        })),
    }
}

// ── Handler: provide_code_context ─────────────────────────────────

pub(crate) fn handle_provide_code_context(
    id: &Value,
    params: &Value,
    state: &McpState,
) {
    use std::time::Instant;
    let overall_start = Instant::now();

    let file_path_str = params["arguments"]["filePath"].as_str().unwrap_or("");
    if file_path_str.is_empty() {
        send_response(&serde_json::json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32602, "message": "Missing required parameter: filePath" } }));
        return;
    }

    let workspace_root = params["arguments"]["workspaceRoot"].as_str();
    let resolved_path = resolve_file_path(file_path_str, workspace_root);

    if state.config.is_excluded(&resolved_path) {
        send_response(&serde_json::json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32603, "message": format!("File excluded by config: {}", file_path_str) } }));
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
        Err(e) => { send_response(&serde_json::json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32603, "message": format!("Cannot read file: {}", e) } })); return; }
    };
    let source = source_arc.as_str();
    let alias = state.get_or_create_alias(resolved_path.clone());

    let explicit_fidelity = params["arguments"]["fidelity"].as_str();
    let explicit_intent = params["arguments"]["intent"].as_str();

    // Phase 1: Heuristics decision
    let heuristics_start = Instant::now();
    let td_guard = state.text_delta_lock();
    let ir_read = state.ir_context_read();
    let decision = crate::mcp::heuristics::decide(
        &resolved_path, explicit_fidelity, explicit_intent,
        &state.config, &td_guard,
        &ir_read,
        source, Some(&alias), None,
    );
    drop(td_guard);
    drop(ir_read);
    let heuristics_ms = heuristics_start.elapsed().as_millis() as u64;

    let effective_fidelity = decision.fidelity;
    let strategy = decision.strategy;
    let is_angular = decision.is_angular;
    let tokenizer_kind = parse_tokenizer_arg(params, &state.config);
    let tokenizer_box = crate::tokenizer::create_tokenizer(tokenizer_kind).ok();
    let tokenizer_ref: Option<&dyn crate::tokenizer::Tokenizer> = tokenizer_box.as_deref();

    // A-04: Create tracing span for this call
    let _span = tracing::info_span!(
        "provide_code_context",
        file_path = %resolved_path,
        fidelity = %format!("{:?}", effective_fidelity),
        strategy = %format!("{:?}", strategy),
        cbm_status = %state.cbm_status.summary(),
        is_angular = %is_angular,
    ).entered();

    match strategy {
        crate::mcp::heuristics::ContextStrategy::DeltaTransport => {
            let compile_start = Instant::now();
            let (compiled, _source_hash) = match compile_file_ir(&resolved_path, effective_fidelity, state) {
                Ok(c) => c,
                Err(e) => { send_response(&to_jsonrpc_error(id, &e)); return; }
            };
            let compile_ms = compile_start.elapsed().as_millis() as u64;

            let delta_start = Instant::now();
            let prev_version = state.file_version(&alias).unwrap_or(0);
            let mut ir_ctx = state.ir_context_lock();
            let delta = if prev_version > 0 && ir_ctx.has_file(&alias) {
                ir_ctx.get_ir(&alias).cloned().and_then(|prev_instructions| {
                    let prev_compiled = CompiledIR {
                        file_id: alias.clone(), version: prev_version,
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
                Some(d) => {
                    let wire_delta = serde_json::to_value(&d).unwrap_or_default();
                    raw_tokens = 0;
                    comp_tokens = 0;
                    send_response(&serde_json::json!({
                        "jsonrpc": "2.0", "id": id, "result": {
                            "content": [{ "type": "text", "text": format!("Δ delta for {} (v{} → v{}): +{} ~{} -{} ops", compiled.file_id, d.from, d.to, d.ops.adds.len(), d.ops.mods.len(), d.ops.dels.len()) }],
                            "delta": wire_delta, "from_version": d.from, "to_version": d.to,
                            "strategy": "delta", "fidelity": format!("{:?}", effective_fidelity).to_lowercase(),
                            "decision_summary": decision.summary()
                        }
                    }));
                }
                None => {
                    let render_start = Instant::now();
                    let hir = crate::ir::hierarchical::ir_to_hierarchical(&compiled);
                    let llm_text = crate::ir::render_hierarchical_for_llm(&hir, effective_fidelity);
                    let full = format!("{}\n// ── {} ({}) ──\n{}", llm_text.trim(), compiled.file_id, resolved_path, state.format_dict_footer().trim());
                    let render_ms = render_start.elapsed().as_millis() as u64;
                    raw_tokens = count_tokens_with_tokenizer(source, tokenizer_ref);
                    comp_tokens = count_tokens_with_tokenizer(&full, tokenizer_ref);
                    state.record_compression(&resolved_path, raw_tokens, comp_tokens, &format!("{:?}", effective_fidelity).to_lowercase(), is_angular, "full", None, "ir_compression");
                    send_response(&serde_json::json!({
                        "jsonrpc": "2.0", "id": id, "result": {
                            "content": [{ "type": "text", "text": full }], "version": compiled.version,
                            "strategy": "full", "fidelity": format!("{:?}", effective_fidelity).to_lowercase(),
                            "decision_summary": decision.summary()
                        }
                    }));
                    tracing::info!(
                        heuristics_ms = heuristics_ms,
                        compile_ms = compile_ms,
                        delta_ms = _delta_ms,
                        render_ms = render_ms,
                        raw_tokens = raw_tokens,
                        comp_tokens = comp_tokens,
                        savings_pct = if raw_tokens > 0 { ((raw_tokens - comp_tokens) as f64 / raw_tokens as f64 * 100.0) as u64 } else { 0 },
                        "provide_code_context delta full complete"
                    );
                }
            }
            let _total_ms = overall_start.elapsed().as_millis() as u64;
            state.record_compression(&resolved_path, raw_tokens, comp_tokens, &format!("{:?}", effective_fidelity).to_lowercase(), is_angular, "delta", None, "ir_compression");
        }
        crate::mcp::heuristics::ContextStrategy::FullCompress => {
            let compile_start = Instant::now();
            let ir_result = compile_file_ir(&resolved_path, effective_fidelity, state);
            let compile_ms = compile_start.elapsed().as_millis() as u64;

            if let Ok((ir, _source_hash)) = ir_result {
                let render_start = Instant::now();
                // Note: IR error is logged below in the else branch (4.4 audit fix)
                state.ir_context_lock().load_ir(ir.clone(), None);
                let hir = crate::ir::hierarchical::ir_to_hierarchical(&ir);
                let llm_text = crate::ir::render_hierarchical_for_llm(&hir, effective_fidelity);
                let full = format!("{}\n// ── {} ({}) ──\n{}", llm_text.trim(), ir.file_id, resolved_path, state.format_dict_footer().trim());
                state.llm_text_cache_lock().insert(ir.file_id.clone(), full.clone());
                let render_ms = render_start.elapsed().as_millis() as u64;
                let raw_tokens = count_tokens_with_tokenizer(source, tokenizer_ref);
                let comp_tokens = count_tokens_with_tokenizer(&full, tokenizer_ref);
                state.record_compression(&resolved_path, raw_tokens, comp_tokens, &format!("{:?}", effective_fidelity).to_lowercase(), is_angular, "full", None, "ir_compression");
                send_response(&serde_json::json!({
                    "jsonrpc": "2.0", "id": id, "result": {
                        "content": [{ "type": "text", "text": full }], "version": ir.version,
                        "strategy": "full", "fidelity": format!("{:?}", effective_fidelity).to_lowercase(),
                        "is_angular": is_angular, "decision_summary": decision.summary()
                    }
                }));
                let total_ms = overall_start.elapsed().as_millis() as u64;
                tracing::info!(
                    heuristics_ms = heuristics_ms,
                    compile_ms = compile_ms,
                    render_ms = render_ms,
                    total_ms = total_ms,
                    raw_tokens = raw_tokens,
                    comp_tokens = comp_tokens,
                    savings_pct = if raw_tokens > 0 { ((raw_tokens - comp_tokens) as f64 / raw_tokens as f64 * 100.0) as u64 } else { 0 },
                    "provide_code_context full complete"
                );
            } else {
                // Log the structured IR error before falling back (4.4 audit fix)
                if let Err(ref e) = ir_result {
                    tracing::warn!(error = %e, path = %resolved_path, "IR compilation failed in provide_code_context, falling back to legacy compression");
                }
                let fallback_start = Instant::now();
                match crate::compression::pipeline::compress_file_with_source(
                    PathBuf::from(&resolved_path), Some(source),
                    &mut state.dict_lock(), &mut state.cache_write(), effective_fidelity,
                    Some(&state.config),
                ) {
                    Ok(mut compressed_text) => {
                        compressed_text.push_str(&state.format_dict_footer());
                        let raw_tokens = count_tokens_with_tokenizer(source, tokenizer_ref);
                        let comp_tokens = count_tokens_with_tokenizer(&compressed_text, tokenizer_ref);
                        state.record_compression(&resolved_path, raw_tokens, comp_tokens, &format!("{:?}", effective_fidelity).to_lowercase(), is_angular, "full", None, "ir_compression");
                        send_response(&serde_json::json!({
                            "jsonrpc": "2.0", "id": id, "result": {
                                "content": [{ "type": "text", "text": compressed_text }],
                                "strategy": "full", "fidelity": format!("{:?}", effective_fidelity).to_lowercase(),
                                "decision_summary": decision.summary()
                            }
                        }));
                        let fallback_ms = fallback_start.elapsed().as_millis() as u64;
                        tracing::info!(
                            heuristics_ms = heuristics_ms,
                            compile_ms = compile_ms,
                            fallback_ms = fallback_ms,
                            raw_tokens = raw_tokens,
                            comp_tokens = comp_tokens,
                            savings_pct = if raw_tokens > 0 { ((raw_tokens - comp_tokens) as f64 / raw_tokens as f64 * 100.0) as u64 } else { 0 },
                            "provide_code_context fallback complete"
                        );
                    }
                    Err(e) => send_response(&serde_json::json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32603, "message": e.to_string() } })),
                }
            }
        }
    }
}

// ── Handler: restore_context ───────────────────────────────────────

pub(crate) fn handle_restore_context(
    id: &Value,
    params: &Value,
    state: &McpState,
) {
    let file_path_str = params["arguments"]["filePath"].as_str().unwrap_or("");
    if file_path_str.is_empty() {
        send_response(&serde_json::json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32602, "message": "Missing required parameter: filePath" } }));
        return;
    }
    let workspace_root = params["arguments"]["workspaceRoot"].as_str();
    let resolved_path = resolve_file_path(file_path_str, workspace_root);
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
        Err(e) => { send_response(&serde_json::json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32603, "message": format!("Cannot read file: {}", e) } })); return; }
    };
    let source_text = source_arc.as_str();

    match compile_file_ir(&resolved_path, fidelity, state) {
        Ok((ir, _source_hash)) => {
            let hir = crate::ir::hierarchical::ir_to_hierarchical(&ir);
            let llm_text = crate::ir::render_hierarchical_for_llm(&hir, fidelity);
            let full = format!("{}\n// ── {} ({}) ──\n{}", llm_text.trim(), ir.file_id, resolved_path, state.format_dict_footer().trim());
            state.llm_text_cache_lock().insert(ir.file_id.clone(), full.clone());
            send_response(&serde_json::json!({ "jsonrpc": "2.0", "id": id, "result": { "content": [{ "type": "text", "text": full }], "version": ir.version, "restored": true } }));
        }
        Err(e) => {
            // Log the structured IR error before falling back (4.4 audit fix)
            tracing::warn!(error = %e, path = %resolved_path, "IR compilation failed in restore_context, falling back to legacy compression");
            match crate::compression::pipeline::compress_file_with_source(
                PathBuf::from(&resolved_path), Some(source_text),
                &mut state.dict_lock(), &mut state.cache_write(), fidelity,
                Some(&state.config),
            ) {
                Ok(mut compressed_text) => {
                    compressed_text.push_str(&state.format_dict_footer());
                    send_response(&serde_json::json!({ "jsonrpc": "2.0", "id": id, "result": { "content": [{ "type": "text", "text": compressed_text }], "restored": true } }));
                }
                Err(e) => send_response(&serde_json::json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32603, "message": e.to_string() } })),
            }
        }
    }
}