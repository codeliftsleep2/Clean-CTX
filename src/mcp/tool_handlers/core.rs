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
use super::super::tool_helpers::{compress_text_body, compile_file_ir, resolve_file_path_checked, diff_code_context_handler, count_tokens_with_tokenizer, inject_baseline_breakpoint, inject_tail_breakpoint};

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
    let resolved_path = match resolve_file_path_checked(file_path_str, workspace_root) {
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
    let source_arc = state.read_source(&resolved_path).ok();
    let source_ref = source_arc.as_ref().map(|s| s.as_str());
    let source_text = source_ref.unwrap_or("");

    let tokenizer_kind = parse_tokenizer_arg(params, &state.config);
    let tokenizer_box = crate::tokenizer::create_tokenizer(tokenizer_kind).ok();
    let tokenizer_ref: Option<&dyn crate::tokenizer::Tokenizer> = tokenizer_box.as_deref();

    // Verbatim: full raw source, zero compression. Byte-exact entire document.
    // Short-circuit the IR path — the contract is "return the source as-is".
    if effective_fidelity == crate::compression::Fidelity::Verbatim {
        let raw_tokens = count_tokens_with_tokenizer(source_text, tokenizer_ref);
        let comp_tokens = raw_tokens;
        // H-9 (FAANG audit): `handle_compress_code_context` has no heuristics
        // decision, so `is_angular` is unknown here. `handle_provide_code_context`
        // passes the real `is_angular` from the decision. Keep `false` for
        // consistency with the IR path in this handler (which also passes `false`).
        state.record_compression(&resolved_path, raw_tokens, comp_tokens,
            "verbatim", false, "full", None, "verbatim");
        {
            if let Some(ref store) = *state.persistence_store_lock() {
                let mut hasher = Sha256::new();
                hasher.update(source_text.as_bytes());
                let source_hash = format!("{:x}", hasher.finalize());
                store.queue_save_context(
                    &resolved_path, effective_fidelity, source_text,
                    &[], &source_hash, raw_tokens as u64, comp_tokens as u64,
                );
            }
        }
        state.flush_persistence();
        let mut response = serde_json::json!({
            "jsonrpc": "2.0", "id": id,
            "result": {
                "content": [{ "type": "text", "text": source_text }],
                "content_kind": "verbatim", "byte_exact": ["entire_document"]
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

        let content_kind = match effective_fidelity {
            crate::compression::Fidelity::Edit => "structural_body",
            crate::compression::Fidelity::Verbatim => "verbatim",
            _ => "structural_skeleton",
        };
        let byte_exact = match effective_fidelity {
            crate::compression::Fidelity::Edit => vec!["method_bodies"],
            crate::compression::Fidelity::Verbatim => vec!["entire_document"],
            _ => vec![],
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

                // H-10 (FAANG audit): The legacy-fallback path at Edit fidelity
                // now carries byte-exact method bodies (C-4 fix), so the
                // metadata must reflect that instead of being absent.
                let content_kind = match effective_fidelity {
                    crate::compression::Fidelity::Edit => "structural_body",
                    crate::compression::Fidelity::Verbatim => "verbatim",
                    _ => "legacy_compressed",
                };
                let byte_exact = match effective_fidelity {
                    crate::compression::Fidelity::Edit => vec!["method_bodies"],
                    crate::compression::Fidelity::Verbatim => vec!["entire_document"],
                    _ => vec![],
                };
                serde_json::json!({
                    "jsonrpc": "2.0", "id": id,
                    "result": { "content": [{ "type": "text", "text": compressed_text }], "content_kind": content_kind, "byte_exact": byte_exact }
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

    // Inject baseline cache breakpoint into the response so the LLM
    // client can set cache_control on the stable compressed output.
    if let Some(text) = response["result"]["content"][0]["text"].as_str() {
        let text_owned = text.to_string();
        inject_baseline_breakpoint(&mut response, state, &text_owned);
    }

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
    let resolved_path = match resolve_file_path_checked(file_path_str, workspace_root) {
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
        Err(e) => { send_response(&serde_json::json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32603, "message": format!("Cannot read file: {}", e) } })); return; }
    };
    // Verbatim: full raw source, zero compression. Byte-exact entire document.
    // Short-circuit the diff path — the contract is "return the source as-is".
    if fidelity == crate::compression::Fidelity::Verbatim {
        let mut response = serde_json::json!({
            "jsonrpc": "2.0", "id": id,
            "result": {
                "content": [{ "type": "text", "text": source }],
                "content_kind": "verbatim", "byte_exact": ["entire_document"]
            }
        });
        inject_baseline_breakpoint(&mut response, state, &source);
        send_response(&response);
        return;
    }
    match diff_code_context_handler(PathBuf::from(&resolved_path), &source, &mut state.cache_write(), fidelity) {
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

pub(crate) fn handle_delta_code_context(
    id: &Value,
    params: &Value,
    state: &McpState,
) {
    let file_path_str = params["arguments"]["filePath"].as_str().unwrap_or("");
    let workspace_root = params["arguments"]["workspaceRoot"].as_str();
    let resolved_path = match resolve_file_path_checked(file_path_str, workspace_root) {
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

    // Verbatim: full raw source, zero compression. Byte-exact entire document.
    // Short-circuit the IR/delta path — the contract is "return the source as-is".
    if fidelity == crate::compression::Fidelity::Verbatim {
        let source_arc = match state.read_source(&resolved_path) {
            Ok(s) => s,
            Err(e) => { send_response(&serde_json::json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32603, "message": format!("Cannot read file: {}", e) } })); return; }
        };
        let source_text = source_arc.as_str();
        let mut response = serde_json::json!({
            "jsonrpc": "2.0", "id": id,
            "result": {
                "content": [{ "type": "text", "text": source_text }],
                "content_kind": "verbatim", "byte_exact": ["entire_document"]
            }
        });
        inject_baseline_breakpoint(&mut response, state, source_text);
        send_response(&response);
        return;
    }
    
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
                // C-9 (FAANG audit): At Edit fidelity the cached-IR notice
                // has NO method bodies — the LLM can't edit. Return the raw
                // source as-is (byte-exact entire document) instead.
                    // C-13 (FAANG audit): `byte_exact` must say `entire_document`
                    // because the FULL raw source is returned, not just method
                    // bodies — `method_bodies` would mislead the client into
                    // treating the whole document as a SEARCH block.
                    if fidelity == crate::compression::Fidelity::Edit {
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
use super::super::tool_helpers::{compress_text_body, compile_file_ir, resolve_file_path_checked, diff_code_context_handler, count_tokens_with_tokenizer, inject_baseline_breakpoint, inject_tail_breakpoint};

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
    let resolved_path = match resolve_file_path_checked(file_path_str, workspace_root) {
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
    let source_arc = state.read_source(&resolved_path).ok();
    let source_ref = source_arc.as_ref().map(|s| s.as_str());
    let source_text = source_ref.unwrap_or("");

    let tokenizer_kind = parse_tokenizer_arg(params, &state.config);
    let tokenizer_box = crate::tokenizer::create_tokenizer(tokenizer_kind).ok();
    let tokenizer_ref: Option<&dyn crate::tokenizer::Tokenizer> = tokenizer_box.as_deref();

    // Verbatim: full raw source, zero compression. Byte-exact entire document.
    // Short-circuit the IR path — the contract is "return the source as-is".
    if effective_fidelity == crate::compression::Fidelity::Verbatim {
        let raw_tokens = count_tokens_with_tokenizer(source_text, tokenizer_ref);
        let comp_tokens = raw_tokens;
        // H-9 (FAANG audit): `handle_compress_code_context` has no heuristics
        // decision, so `is_angular` is unknown here. `handle_provide_code_context`
        // passes the real `is_angular` from the decision. Keep `false` for
        // consistency with the IR path in this handler (which also passes `false`).
        state.record_compression(&resolved_path, raw_tokens, comp_tokens,
            "verbatim", false, "full", None, "verbatim");
        {
            if let Some(ref store) = *state.persistence_store_lock() {
                let mut hasher = Sha256::new();
                hasher.update(source_text.as_bytes());
                let source_hash = format!("{:x}", hasher.finalize());
                store.queue_save_context(
                    &resolved_path, effective_fidelity, source_text,
                    &[], &source_hash, raw_tokens as u64, comp_tokens as u64,
                );
            }
        }
        state.flush_persistence();
        let mut response = serde_json::json!({
            "jsonrpc": "2.0", "id": id,
            "result": {
                "content": [{ "type": "text", "text": source_text }],
                "content_kind": "verbatim", "byte_exact": ["entire_document"]
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

        let content_kind = match effective_fidelity {
            crate::compression::Fidelity::Edit => "structural_body",
            crate::compression::Fidelity::Verbatim => "verbatim",
            _ => "structural_skeleton",
        };
        let byte_exact = match effective_fidelity {
            crate::compression::Fidelity::Edit => vec!["method_bodies"],
            crate::compression::Fidelity::Verbatim => vec!["entire_document"],
            _ => vec![],
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

                // H-10 (FAANG audit): The legacy-fallback path at Edit fidelity
                // now carries byte-exact method bodies (C-4 fix), so the
                // metadata must reflect that instead of being absent.
                let content_kind = match effective_fidelity {
                    crate::compression::Fidelity::Edit => "structural_body",
                    crate::compression::Fidelity::Verbatim => "verbatim",
                    _ => "legacy_compressed",
                };
                let byte_exact = match effective_fidelity {
                    crate::compression::Fidelity::Edit => vec!["method_bodies"],
                    crate::compression::Fidelity::Verbatim => vec!["entire_document"],
                    _ => vec![],
                };
                serde_json::json!({
                    "jsonrpc": "2.0", "id": id,
                    "result": { "content": [{ "type": "text", "text": compressed_text }], "content_kind": content_kind, "byte_exact": byte_exact }
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

    // Inject baseline cache breakpoint into the response so the LLM
    // client can set cache_control on the stable compressed output.
    if let Some(text) = response["result"]["content"][0]["text"].as_str() {
        let text_owned = text.to_string();
        inject_baseline_breakpoint(&mut response, state, &text_owned);
    }

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
    let resolved_path = match resolve_file_path_checked(file_path_str, workspace_root) {
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
        Err(e) => { send_response(&serde_json::json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32603, "message": format!("Cannot read file: {}", e) } })); return; }
    };
    // Verbatim: full raw source, zero compression. Byte-exact entire document.
    // Short-circuit the diff path — the contract is "return the source as-is".
    if fidelity == crate::compression::Fidelity::Verbatim {
        let mut response = serde_json::json!({
            "jsonrpc": "2.0", "id": id,
            "result": {
                "content": [{ "type": "text", "text": source }],
                "content_kind": "verbatim", "byte_exact": ["entire_document"]
            }
        });
        inject_baseline_breakpoint(&mut response, state, &source);
        send_response(&response);
        return;
    }
    match diff_code_context_handler(PathBuf::from(&resolved_path), &source, &mut state.cache_write(), fidelity) {
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

pub(crate) fn handle_delta_code_context(
    id: &Value,
    params: &Value,
    state: &McpState,
) {
    let file_path_str = params["arguments"]["filePath"].as_str().unwrap_or("");
    let workspace_root = params["arguments"]["workspaceRoot"].as_str();
    let resolved_path = match resolve_file_path_checked(file_path_str, workspace_root) {
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

    // Verbatim: full raw source, zero compression. Byte-exact entire document.
    // Short-circuit the IR/delta path — the contract is "return the source as-is".
    if fidelity == crate::compression::Fidelity::Verbatim {
        let source_arc = match state.read_source(&resolved_path) {
            Ok(s) => s,
            Err(e) => { send_response(&serde_json::json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32603, "message": format!("Cannot read file: {}", e) } })); return; }
        };
        let source_text = source_arc.as_str();
        let mut response = serde_json::json!({
            "jsonrpc": "2.0", "id": id,
            "result": {
                "content": [{ "type": "text", "text": source_text }],
                "content_kind": "verbatim", "byte_exact": ["entire_document"]
            }
        });
        inject_baseline_breakpoint(&mut response, state, source_text);
        send_response(&response);
        return;
    }
    
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
                // C-9 (FAANG audit): At Edit fidelity the cached-IR notice
                // has NO method bodies — the LLM can't edit. Return the raw
                // source as-is (byte-exact entire document) instead.
                    // C-14 (FAANG audit): At Edit/Verbatim fidelity the cached-IR notice
                    // has NO method bodies — the LLM can't edit. Return the raw source
                    // byte-for-byte? No: the whole document includes the notice; we must
                    // say `byte_exact: ["entire_document"]` in the metadata.
                    match crate::compression::pipeline::compress_file_with_source(
                        &resolved_path, source_arc, fidelity,
                        &mut state.cache_write(), &mut state.dict_lock(), effective_fidelity,
                        Some(&state.config),
                    ) {
                        Ok(mut compressed_text) => {
                            compressed_text.push_str(&state.format_dict_footer());
                            // H-10 (FAANG audit): The legacy-fallback path at Edit
                            // fidelity now carries byte-exact method bodies (C-4 fix),
                            // so the metadata must reflect that instead of being absent.
                            let content_kind = match effective_fidelity {
                                crate::compression::Fidelity::Edit => "structural_body",
                                crate::compression::Fidelity::Verbatim => "verbatim",
                                _ => "legacy_compressed",
                            };
                            let byte_exact = match effective_fidelity {
                                crate::compression::Fidelity::Edit => vec!["method_bodies"],
                                crate::compression::Fidelity::Verbatim => vec!["entire_document"],
                                _ => vec![],
                            };
                            let mut response = serde_json::json!({
                                "jsonrpc": "2.0", "id": id, "result": {
                                    "content": [{ "type": "text", "text": compressed_text }],
                                    "strategy": "full", "fidelity": format!("{:?}", effective_fidelity).to_lowercase(),
                                    "decision_summary": decision.summary(),
                                    "content_kind": content_kind, "byte_exact": byte_exact,
                                    "degradation": { "ir_compiler": if ir_error_reason.is_empty() { "fallback_legacy" } else { ir_error_reason.as_str() }, "angular_meta": if cfg!(feature = "angular") { "enabled" } else { "feature_disabled" } }
                                }
                            });
                            // Fallback compression output is a stable full snapshot — inject baseline breakpoint.
                            inject_baseline_breakpoint(&mut response, state, &compressed_text);
                            send_response(&response);
                        }
                        Err(e) => send_response(&serde_json::json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32603, "message": e.to_string() } })),
                    }
                }
            }
        }
    }
}

// ── Handler: delta_text_context ───────────────────────────────────

(pub(crate) fn handle_delta_text_context(
    id: &Value,
    params: &Value,
    state: &McpState,
) {
    let file_path_str = params["arguments"]["filePath"].as_str().unwrap_or("");
    let workspace_root = params["arguments"]["workspaceRoot"].as_str();
    let resolved_path = match resolve_file_path_checked(file_path_str, workspace_root) {
        Ok(p) => p,
        Err(msg) => {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "error": { "code": -32602, "message": msg }
            }));
            return;
        }
    };
    let artifacts = match parse_fidelity_arg(id, params, &state.config) {
        Ok(f) => f,
        Err(()) => return,
    };

    // Verbatim: full raw source, zero compression. Byte-exact entire document.
    // This MUST be checked before the Angular template path — ".component.html"
    // files with explicit fidelity "verbatim" must return the raw source,
    // not a compressed template.
    // H-4 (FAANG audit): `verbatim_requested` is only used inside the
    // `#[cfg(feature = "angular")]` block, so when angular is disabled the
    // variable would be unused → clippy warning. Gate the declaration too.
    #[cfg(feature = "angular")]
    let verbatim_requested = params["arguments"]["fidelity"].as_str()
        == Some("verbatim");

    // ANGULAR_HTML_COMPRESSION_PLAN Phase 3: route `.component.html`
    // files through the Angular template compressor. These files have
    // no tree-sitter grammar for the IR compiler, so we handle them
    // specially before the heuristics decision.
    #[cfg(feature = "angular")]
    if resolved_path.to_lowercase().ends_with(".component.html")
        && !verbatim_requested
    {
        let explicit_fidelity = params["arguments"]["fidelity"].as_str();
        let explicit_intent = params["arguments"]["intent"].as_str();
        let artifact = match explicit_fidelity {
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
        let lines = crate::angular_meta::template_compress::compress_template_with_prime_ng(source, fidelity);
        let body = lines.join("\n");
        let tokenizer_kind = parse_tokenizer_arg(params, &state.config);
        let tokenizer_box = crate::tokenizer::create_tokenizer(tokenizer_kind).ok();
        let tokenizer_ref: Option<&dyn crate::tokenizer::Tokenizer> = tokenizer_box.as_deref();
        let raw_tokens = count_tokens_with_tokenizer(source, tokenizer_ref);
        let comp_tokens = count_tokens_with_tokenizer(&body, tokenizer_ref);
        state.record_compression(&resolved_path, raw_tokens, comp_tokens,
            &format!("{:?}", fidelity).to_lowercase(), true, "full", None, "angular_template");

        // Persist to DB so `context_stats` and cross-session dashboards
        // can report Angular template compression savings.
        {
            if let Some(ref store) = *state.persistence_store_lock() {
                let mut hasher = Sha256::new();
                hasher.update(source.as_bytes());
                let source_hash = format!("{:x}", hasher.finalize());
                store.queue_save_context(
                    &resolved_path, fidelity, &body,
                    &[], &source_hash, raw_tokens as u64, comp_tokens as u64,
                );
            }
        }
        state.flush_persistence();
        let mut response = serde_json::json!({
            "jsonrpc": "2.0", "id": id,
            "result": {
                "content": [{ "type": "text", "text": body }],
                "strategy": "full", "fidelity": format!("{:?}", fidelity).to_lowercase(),
                "is_angular": true, "template_compressed": true
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
        &resolved_path, explicit_fidelity, explicit_intent,
        &state.config, &td_guard,
        &ir_read,
        source, Some(&alias), None,
    ) {
        Ok(d) => d,
        Err(e) => {
            drop(td_guard);
            drop(ir_read);
            // Gap 2 fix: invalid explicit fidelity must surface as -32602,
            // not silently degrade to the default.
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
    let strategy = decision.strategy;
    let is_angular = decision.is_angular;
    let tokenizer_kind = parse_tokenizer_arg(params, &state.config);
    let tokenizer_box = crate::tokenizer::create_tokenizer(tokenizer_kind).ok();
    let tokenizer_ref: Option<&dyn crate::tokenizer::Tokenizer> = tokenizer_box.as_deref();

    // Verbatim: full raw source, zero compression. Byte-exact entire document.
    // Short-circuit the IR path — the contract is "return the source as-is".
    if effective_fidelity == crate::compression::Fidelity::Verbatim {
        let raw_tokens = count_tokens_with_tokenizer(source, tokenizer_ref);
        let comp_tokens = raw_tokens;
        state.record_compression(&resolved_path, raw_tokens, comp_tokens,
            "verbatim", is_angular, "full", None, "verbatim");
        {
            if let Some(ref store) = *state.persistence_store_lock() {
                let mut hasher = Sha256::new();
                hasher.update(source.as_bytes());
                let source_hash = format!("{:x}", hasher.finalize());
                store.queue_save_context(
                    &resolved_path, effective_fidelity, source,
                    &[], &source_hash, raw_tokens as u64, comp_tokens as u64,
                );
            }
        }
        state.flush_persistence();
        let mut response = serde_json::json!({
            "jsonrpc": "2.0", "id": id,
            "result": {
                "content": [{ "type": "text", "text": source_text }],
                "strategy": "full", "fidelity": format!("{:?}", effective_fidelity).to_lowercase(),
                "is_angular": true, "template_compressed": true
            }
        });
        inject_baseline_breakpoint(&mut response, state, source_text);
        send_response(&response);
        return;
    }

    // A-04: Create tracing span for this call
    let _span = tracing::info_span!(
        "provide_code_context",
        file_path = %resolved_path,
        fidelity = %format!("{:?}", effective_fidelity),
        strategy = %format!("{:?}", strategy),
        phm_status = %state.phm_status.summary(),
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
            let ir_ctx = { ir_ctx = state.ir_context_lock(); }
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
                Some(ref d) => {
                    let wire_delta = serde_json::to_value(d).unwrap_or_default();
                    // DASHBOARD FIX (R-02 FAANG): count the delta wire tokens
                    // (the actual payload sent to the LLM) so the dashboard
                    // can show delta efficiency. The previous full compression's
                    // compressed token count is passed as `full_compressed_tokens`
                    // so `record_compression` can compute CPU savings vs a full
                    let delta_text = serde_json::to_string(&wire_delta).unwrap_or_default();
                    raw_tokens = count_tokens_with_tokenizer(&delta_text, tokenizer_ref);
                    comp_tokens = raw_tokens; // delta is the payload itself
                    let prev_full_compressed = state.session_stats_lock()
                        .file_stats(&resolved_path)
                        .map(|f| f.compressed_tokens);
                    let content_kind = match effective_fidelity {
                        crate::compression::Fidelity::Edit => "structural_body_delta",
                        crate::compression::Fidelity::Verbatim => "verbatim_delta",
                        _ => "structural_delta",
                    };
                    let mut response = serde_json::json!({
                        "jsonrpc": "2.0", "id": id, "result": {
                            "content": [{ "type": "text", "text": format!("Δ delta for {} (v{} → v{}): +{} ~{} -{} ops", compiled.file_id, d.from, d.to, d.ops.adds.len(), d.ops.mods.len(), d.ops.dels.len()) }],
                            "delta": wire_delta, "from_version": d.from, "to_version": d.to,
                            "strategy": "delta", "fidelity": format!("{:?}", effective_fidelity).to_lowercase(),
                            "decision_summary": decision.summary(),
                            "content_kind": content_kind, "byte_exact": match effective_fidelity {
                                crate::compression::Fidelity::Edit => vec!["method_bodies"],
                                crate::compression::Fidelity::Verbatim => vec!["entire_document"],
                                _ => vec![],
                            },
                            "degradation": { "ir_compiler": "ok", "angular_meta": if cfg!(feature = "angular") { "enabled" } else { "feature_disabled" } }
                        }
                    });
                    // Delta output is rolling dynamic content — mark as tail (ephemeral).
                    inject_tail_breakpoint(&mut response, state);
                    send_response(&response);
                    // Record the delta with the previous full compressed token
                    // count for delta efficiency computation.
                    state.record_delta(&resolved_path, raw_tokens, comp_tokens,
                        &format!("{:?}", effective_fidelity).to_lowercase(), is_angular, "delta",
                        prev_full_compressed, "ir_compression");
                }
                None => {
                    let render_start = Instant::now();
                    let hir = crate::ir::hierarchical::ir_to_hierarchical(&compiled);
                    let llm_text = crate::ir::render_hierarchical_for_llm(&hir, effective_fidelity);
                    let full = format!("{}\n// ── {} ({}) ──\n{}", llm_text.trim(), ir.file_id, resolved_path, state.format_dict_footer().trim());
                    let render_ms = render_start.elapsed().as_millis() as u64;
                    raw_tokens = count_tokens_with_tokenizer(source, tokenizer_ref);
                    comp_tokens = count_tokens_with_tokenizer(&full, tokenizer_ref);
                    state.record_compression(&resolved_path, raw_tokens, comp_tokens, &format!("{:?}", effective_fidelity).to_lowercase(), is_angular, "full", None, "ir_compression");
                    let content_kind = match effective_fidelity {
                        crate::compression::Fidelity::Edit => "structural_body",
                        crate::compression::Fidelity::Verbatim => "verbatim",
                        _ => "structural_skeleton",
                    };
                    let byte_exact = match effective_fidelity {
                        crate::compression::Fidelity::Edit => vec!["method_bodies"],
                        crate::compression::Fidelity::Verbatim => vec!["entire_document"],
                        _ => vec![],
                    };
                    let mut response = serde_json::json!({
                        "jsonrpc": "2.0", "id": id, "result": {
                            "content": [{ "type": "text", "text": full }], "version": compiled.version,
                            "strategy": "full", "fidelity": format!("{:?}", effective_fidelity).to_lowercase(),
                            "decision_summary": decision.summary(),
                            "content_kind": content_kind, "byte_exact": byte_exact,
                            "degradation": { "ir_compiler": "ok", "angular_meta": if cfg!(feature = "angular") { "enabled" } else { "feature_disabled" } }
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
                        savings_pct = if raw_tokens > 0 { ((raw_tokens - comp_tokens) as f64 / raw_tokens as f64 * 100.0) as u64 } else { 0 },
                        "provide_code_context full complete"
                    );
                } else {
                // Log the structured IR error before falling back (4.4 audit fix)
                let ir_error_reason = match &ir_result {
                    Err(e) => {
                        tracing::warn!(error = %e, path = %resolved_path, "IR compilation failed in provide_code_context, falling back to legacy compression");
                        e.to_string()
                    }
                    Ok(_) => String::new(),
                };
                let fallback_start = Instant::now();
                match crate::compression::pipeline::compress_file_with_source(
                    PathBuf::from(&resolved_path), Some(source_text),
                    &mut state.dict_lock(), &mut state.cache_write(), effective_fidelity,
                    Some(&state.config),
                ) {
                    Ok(mut compressed_text) => {
                        compressed_text.push_str(&state.format_dict_footer());
                        let raw_tokens = count_tokens_with_tokenizer(source, tokenizer_ref);
                        let comp_tokens = count_tokens_with_tokenizer(&compressed_text, tokenizer_ref);
                        state.record_compression(&resolved_path, raw_tokens, comp_tokens,
                            &format!("{:?}", effective_fidelity).to_lowercase(), is_angular, "full", None, "ir_compression");
                        // C-14 (FAANG audit): This fallback calls
                        // `compress_file_with_source` which now returns byte-exact
                        // method bodies at Edit (C-4) and raw source at Verbatim
                        // (C-8). The metadata must reflect that instead of the
                        // generic "legacy_compressed" label.
                        let content_kind = match effective_fidelity {
                            crate::compression::Fidelity::Edit => "structural_body",
                            crate::compression::Fidelity::Verbatim => "verbatim",
                            _ => "legacy_compressed",
                        };
                        let byte_exact = match effective_fidelity {
                            crate::compression::Fidelity::Edit => vec!["method_bodies"],
                            crate::compression::Fidelity::Verbatim => vec!["entire_document"],
                            _ => vec![],
                        };
                        let mut response = serde_json::json!({ "jsonrpc": "2.0", "id": id, "result": { "content": [{ "type": "text", "text": compressed_text }], "strategy": "full", "fidelity": format!("{:?}", effective_fidelity).to_lowercase(), "decision_summary": decision.summary(), "content_kind": content_kind, "byte_exact": byte_exact } });
                        // Fallback compression output is a stable full snapshot — inject baseline breakpoint.
                        inject_baseline_breakpoint(&mut response, state, &compressed_text);
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

    // Verbatim: full raw source, zero compression. Byte-exact entire document.
    if fidelity == crate::compression::Fidelity::Verbatim {
        let mut response = serde_json::json!({ "jsonrpc": "2.0", "id": id, "result": { "content": [{ "type": "text", "text": source_text }], "restored": true, "content_kind": "verbatim", "byte_exact": ["entire_document"] } });
        inject_baseline_breakpoint(&mut response, state, source_text);
        send_response(&response);
        return;
    }

    match compile_file_ir(&resolved_path, fidelity, state) {
        Ok((ir, _source_hash)) => {
            let hir = crate::ir::hierarchical::ir_to_hierarchical(&ir);
            let llm_text = crate::ir::render_hierarchical_for_llm(&hir, fidelity);
            let full = format!("{}\n// ── {} ({}) ──\n{}", llm_text.trim(), ir.file_id, resolved_path, state.format_dict_footer().trim());
            state.llm_text_cache_lock().insert(ir.file_id.clone(), full.clone());
            let mut response = serde_json::json!({ "jsonrpc": "2.0", "id": id, "result": { "content": [{ "type": "text", "text": full }], "version": ir.version, "restored": true } });
            // Restored context is a stable full snapshot — inject baseline breakpoint.
            inject_baseline_breakpoint(&mut response, state, &full);
            send_response(&response);
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
                    // H-10 (FAANG audit): The legacy-fallback path at Edit
                    // fidelity now carries byte-exact method bodies (C-4 fix),
                    // so the metadata must reflect that instead of being absent.
                    let content_kind = match fidelity {
                        crate::compression::Fidelity::Edit => "structural_body",
                        crate::compression::Fidelity::Verbatim => "verbatim",
                        _ => "legacy_compressed",
                    };
                    let byte_exact = match fidelity {
                        crate::compression::Fidelity::Edit => vec!["method_bodies"],
                        crate::compression::Fidelity::Verbatim => vec!["entire_document"],
                        _ => vec![],
                    };
                    let mut response = serde_json::json!({ "jsonrpc": "2.0", "id": id, "result": { "content": [{ "type": "text", "text": compressed_text }], "restored": true, "content_kind": content_kind, "byte_exact": byte_exact } });
                    // Restored context is a stable full snapshot — inject baseline breakpoint.
                    inject_baseline_breakpoint(&mut response, state, &compressed_text);
                    send_response(&response);
                }
                Err(e) => send_response(&serde_json::json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32603, "message": e.to_string() } })),
            }
        }
    }
}
