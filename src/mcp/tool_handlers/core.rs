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
use crate::mcp::context_store::ContextStore;
use crate::mcp::McpState;
use crate::protocol::send_response;

use super::super::tools::{parse_fidelity_arg, parse_tokenizer_arg};
use super::super::tool_helpers::{compress_text_body, compile_file_ir, resolve_file_path, diff_code_context_handler, count_tokens_with_tokenizer};

fn tuples_to_coreops(tuples: Vec<Vec<String>>) -> Vec<CoreOp> {
    tuples.into_iter().filter_map(|t| tuple_to_op(&t)).collect()
}

// ── Handler: compress_code_context ───────────────────────────────

pub(crate) fn handle_compress_code_context(
    id: &Value,
    params: &Value,
    state: &McpState,
) {
    let file_path_str = params["arguments"]["filePath"].as_str().unwrap_or("");
    let encoding = params["arguments"]["encoding"].as_str().unwrap_or("named");
    let workspace_root = params["arguments"]["workspaceRoot"].as_str();
    let resolved_path = resolve_file_path(file_path_str, workspace_root);
    let fidelity = match parse_fidelity_arg(id, params) {
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

    let effective_fidelity = fidelity;
    let source_arc = state.read_source(&resolved_path).ok();
    let source_ref = source_arc.as_ref().map(|s| s.as_str());
    let source_text = source_ref.unwrap_or("");

    let ir_result = compile_file_ir(&resolved_path, effective_fidelity, state);

    let tokenizer_kind = parse_tokenizer_arg(params, &state.config);
    let tokenizer_box = crate::tokenizer::create_tokenizer(tokenizer_kind).ok();
    let tokenizer_ref: Option<&dyn crate::tokenizer::Tokenizer> = tokenizer_box.as_deref();

    let response = if let Ok(ir) = ir_result {
        state.ir_context_lock().load_ir(ir.clone());
        let hir = crate::ir::hierarchical::ir_to_hierarchical(&ir);
        let llm_text = crate::ir::render_hierarchical_for_llm(&hir, effective_fidelity);
        let footer = state.format_dict_footer();
        let llm_text_with_footer = format!("{}\n// ── {} ({}) ──\n{}",
            llm_text.trim(), ir.file_id, &resolved_path, footer.trim());
        state.llm_text_cache_lock().insert(ir.file_id.clone(), llm_text_with_footer.clone());

        let raw_tokens = count_tokens_with_tokenizer(source_text, tokenizer_ref);
        let compressed_tokens = count_tokens_with_tokenizer(&llm_text_with_footer, tokenizer_ref);
        state.record_compression(&resolved_path, raw_tokens, compressed_tokens,
            &format!("{:?}", effective_fidelity).to_lowercase(), false, "full", None, "ir_compression");

        // Persist to DB
        {
            if let Some(ref store) = *state.persistence_store_lock() {
                // Compute content hash from source text for deterministic ID
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
        match crate::compression::pipeline::compress_file_with_source(
            PathBuf::from(&resolved_path), source_ref,
            &mut state.dict_lock(), &mut state.cache_write(), effective_fidelity,
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
                        // Compute content hash from source text for deterministic ID
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
            Err(e) => serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "error": { "code": -32603, "message": e.to_string() }
            })
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
    let fidelity = match parse_fidelity_arg(id, params) {
        Ok(f) => f,
        Err(()) => return,
    };
    match diff_code_context_handler(PathBuf::from(&resolved_path), &mut state.cache_write(), fidelity) {
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
    let fidelity = match parse_fidelity_arg(id, params) {
        Ok(f) => f,
        Err(()) => return,
    };
    let compiled = match compile_file_ir(&resolved_path, fidelity, state) {
        Ok(c) => c,
        Err(e) => { send_response(&serde_json::json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32603, "message": e.to_string() } })); return; }
    };
    let path_alias = state.get_or_create_alias(resolved_path.clone());
    let prev_version = state.file_version(&path_alias).unwrap_or(0);

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
        ir_ctx.load_ir(compiled.clone());
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
    let fidelity = match parse_fidelity_arg(id, params) {
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

    let source_arc = match state.read_source(&resolved_path) {
        Ok(s) => s,
        Err(e) => { send_response(&serde_json::json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32603, "message": format!("Cannot read file: {}", e) } })); return; }
    };
    let source = source_arc.as_str();
    let alias = state.get_or_create_alias(resolved_path.clone());

    let explicit_fidelity = params["arguments"]["fidelity"].as_str();
    let explicit_intent = params["arguments"]["intent"].as_str();

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

    let effective_fidelity = decision.fidelity;
    let strategy = decision.strategy;
    let is_angular = decision.is_angular;
    let tokenizer_kind = parse_tokenizer_arg(params, &state.config);
    let tokenizer_box = crate::tokenizer::create_tokenizer(tokenizer_kind).ok();
    let tokenizer_ref: Option<&dyn crate::tokenizer::Tokenizer> = tokenizer_box.as_deref();

    match strategy {
        crate::mcp::heuristics::ContextStrategy::DeltaTransport => {
            let compiled = match compile_file_ir(&resolved_path, effective_fidelity, state) {
                Ok(c) => c,
                Err(e) => { send_response(&serde_json::json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32603, "message": e.to_string() } })); return; }
            };
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
                ir_ctx.load_ir(compiled.clone());
                None
            };
            drop(ir_ctx);

            match delta {
                Some(d) => {
                    let wire_delta = serde_json::to_value(&d).unwrap_or_default();
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
                    let hir = crate::ir::hierarchical::ir_to_hierarchical(&compiled);
                    let llm_text = crate::ir::render_hierarchical_for_llm(&hir, effective_fidelity);
                    let full = format!("{}\n// ── {} ({}) ──\n{}", llm_text.trim(), compiled.file_id, resolved_path, state.format_dict_footer().trim());
                    let raw_tokens = count_tokens_with_tokenizer(source, tokenizer_ref);
                    let comp_tokens = count_tokens_with_tokenizer(&full, tokenizer_ref);
                    state.record_compression(&resolved_path, raw_tokens, comp_tokens, &format!("{:?}", effective_fidelity).to_lowercase(), is_angular, "full", None, "ir_compression");
                    send_response(&serde_json::json!({
                        "jsonrpc": "2.0", "id": id, "result": {
                            "content": [{ "type": "text", "text": full }], "version": compiled.version,
                            "strategy": "full", "fidelity": format!("{:?}", effective_fidelity).to_lowercase(),
                            "decision_summary": decision.summary()
                        }
                    }));
                }
            }
        }
        crate::mcp::heuristics::ContextStrategy::FullCompress => {
            let ir_result = compile_file_ir(&resolved_path, effective_fidelity, state);
            if let Ok(ir) = ir_result {
                state.ir_context_lock().load_ir(ir.clone());
                let hir = crate::ir::hierarchical::ir_to_hierarchical(&ir);
                let llm_text = crate::ir::render_hierarchical_for_llm(&hir, effective_fidelity);
                let full = format!("{}\n// ── {} ({}) ──\n{}", llm_text.trim(), ir.file_id, resolved_path, state.format_dict_footer().trim());
                state.llm_text_cache_lock().insert(ir.file_id.clone(), full.clone());
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
            } else {
                match crate::compression::pipeline::compress_file_with_source(
                    PathBuf::from(&resolved_path), Some(source),
                    &mut state.dict_lock(), &mut state.cache_write(), effective_fidelity,
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
    let fidelity = match parse_fidelity_arg(id, params) {
        Ok(f) => f,
        Err(()) => return,
    };
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
        Ok(ir) => {
            let hir = crate::ir::hierarchical::ir_to_hierarchical(&ir);
            let llm_text = crate::ir::render_hierarchical_for_llm(&hir, fidelity);
            let full = format!("{}\n// ── {} ({}) ──\n{}", llm_text.trim(), ir.file_id, resolved_path, state.format_dict_footer().trim());
            state.llm_text_cache_lock().insert(ir.file_id.clone(), full.clone());
            send_response(&serde_json::json!({ "jsonrpc": "2.0", "id": id, "result": { "content": [{ "type": "text", "text": full }], "version": ir.version, "restored": true } }));
        }
        Err(_) => {
            match crate::compression::pipeline::compress_file_with_source(
                PathBuf::from(&resolved_path), Some(source_text),
                &mut state.dict_lock(), &mut state.cache_write(), fidelity,
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