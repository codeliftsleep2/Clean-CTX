// compress_code_context MCP handler.

use super::common::{checked_hierarchy_or_respond, contract_fields};
use crate::ir::wire::ir_to_wire;
use crate::mcp::McpState;
use crate::mcp::tool_helpers::{
    compile_file_ir_candidate, count_tokens_with_tokenizer, inject_baseline_breakpoint,
    resolve_file_path_checked,
};
use crate::mcp::tools::{parse_fidelity_arg, parse_tokenizer_arg};
use crate::protocol::send_response;
use serde_json::Value;
// ── Handler: compress_code_context ───────────────────────────────

/// P3-2: Main handler for compress_code_context tool.
/// Orchestrates validation, compilation, and response building.
pub(crate) fn handle_compress_code_context(id: &Value, params: &Value, state: &McpState) {
    let file_path_str = crate::mcp::tool_helpers::arg_str_or_empty(params, "filePath");
    let encoding = params["arguments"]["encoding"].as_str().unwrap_or("named");
    let workspace_root = crate::mcp::tool_helpers::arg_str(params, "workspaceRoot");
    let resolved_path = match resolve_file_path_checked(
        file_path_str,
        workspace_root,
        &state.config.additional_roots,
    ) {
        Ok(p) => p,
        Err(msg) => {
            send_response(&crate::mcp::tool_helpers::jsonrpc_error(
                id.clone(),
                -32602,
                msg,
                None,
            ));
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

    if fidelity != crate::compression::Fidelity::Verbatim {
        if let Err(error) = state.preflight_semantic_publication(&resolved_path) {
            send_response(&crate::mcp::tool_helpers::jsonrpc_error(
                id.clone(),
                -32603,
                error,
                None,
            ));
            return;
        }
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

    let ir_result = compile_file_ir_candidate(&resolved_path, effective_fidelity, state);

    // P3-2: Build response using extracted helpers
    // If IR compilation fails, fall back to legacy compression but log
    // the structured error for diagnostics (4.4 audit fix).
    let mut response = if let Ok((mut ir, semantic_edges, source_hash)) = ir_result {
        let hir = match checked_hierarchy_or_respond(id, &ir) {
            Some(hierarchy) => hierarchy,
            None => return,
        };
        let raw_tokens = count_tokens_with_tokenizer(source_text, tokenizer_ref);
        let candidate_compact = crate::ir::render_hierarchical_for_llm(&hir, effective_fidelity);
        let compressed_tokens = count_tokens_with_tokenizer(&candidate_compact, tokenizer_ref);

        // P9-14: durability is the publication boundary. Persist the checked
        // candidate and its complete edge snapshot before creating aliases or
        // changing any live owner. Compact output is rendered after the
        // committed candidate receives its session-local alias.
        {
            if let Some(ref store) = *state.persistence_store_lock() {
                let mut durable_ir = ir.clone();
                durable_ir.file_id.clone_from(&resolved_path);
                let ir_binary = crate::ir::binary_wire::encode(&durable_ir);
                let persisted = store.sqlite().is_some_and(|mut sqlite| {
                    sqlite
                        .save_context_with_semantics(
                            &resolved_path,
                            effective_fidelity,
                            "",
                            &ir_binary,
                            &source_hash,
                            ir.version,
                            &semantic_edges,
                            raw_tokens as u64,
                            compressed_tokens as u64,
                        )
                        .is_ok()
                });
                if !persisted {
                    send_response(&crate::mcp::tool_helpers::jsonrpc_error(
                        id.clone(),
                        -32603,
                        "Canonical IR and semantic edges could not be persisted atomically",
                        None,
                    ));
                    return;
                }
            }
        }

        let path_alias = state.get_or_create_alias(resolved_path.clone());
        ir.file_id.clone_from(&path_alias);
        let canonical_path = crate::dictionary::path::canonical_identity_key(&resolved_path);
        let llm_text = crate::ir::render_hierarchical_for_llm(&hir, effective_fidelity);
        let footer = state.format_dict_footer_for_aliases(&[&path_alias]);
        let llm_text_with_footer = format!(
            "{}\n// ── {} ({}) ──\n{}",
            llm_text.trim(),
            path_alias,
            resolved_path,
            footer.trim()
        );
        let compressed_tokens = count_tokens_with_tokenizer(&llm_text_with_footer, tokenizer_ref);

        state
            .ir_context_lock()
            .load_ir(ir.clone(), Some(source_hash.clone()));
        state.remember_context_fidelity(&path_alias, effective_fidelity);
        {
            let mut idx = state.workspace_index_lock();
            idx.remove_file(&canonical_path);
            idx.add_edges(&canonical_path, semantic_edges.clone());
        }
        state.remember_semantic_edges(&path_alias, semantic_edges.clone());
        if state.persistence_store_lock().is_some() {
            state.remember_persisted_path(&path_alias, &resolved_path);
        }
        state
            .llm_text_cache_lock()
            .insert(path_alias, llm_text_with_footer.clone());
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
                "ir": crate::ir::hierarchical::hierarchy_to_wire(&ir, &hir),
                "pretty": ir_value, "v": ir.version, "file": ir.file_id,
                "content_kind": content_kind, "byte_exact": byte_exact,
                "semantic_edges": serde_json::to_value(&semantic_edges).unwrap_or_default()
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
                    "IR compilation unavailable for {}: {}. SCHEMA v5 output \
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
