use super::common::{ContentKind, contract_fields_focused, resolve_focus_or_respond};
use crate::mcp::McpState;
use crate::mcp::tool_helpers::{
    compile_file_ir_focused, count_tokens_with_tokenizer, inject_baseline_breakpoint,
    resolve_file_path_checked,
};
use crate::mcp::tools::parse_tokenizer_arg;
use crate::protocol::send_response;
use serde_json::Value;
use std::collections::HashSet;
pub(crate) fn handle_provide_code_context(id: &Value, params: &Value, state: &McpState) {
    use std::time::Instant;
    let overall_start = Instant::now();
    let focus_methods: Option<HashSet<String>> =
        params["arguments"]["focusMethods"].as_array().map(|arr| {
            arr.iter()
                .filter_map(|s| s.as_str().map(String::from))
                .collect()
        });
    let file_path_str = crate::mcp::tool_helpers::arg_str_or_empty(params, "filePath");
    if file_path_str.is_empty() {
        send_response(
            &serde_json::json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32602, "message": "Missing required parameter: filePath" } }),
        );
        return;
    }
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
    if state.config.is_excluded(&resolved_path) {
        send_response(
            &serde_json::json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32603, "message": format!("File excluded by config: {}", file_path_str) } }),
        );
        return;
    }
    if let Err(error) = state.preflight_semantic_publication(&resolved_path) {
        send_response(&crate::mcp::tool_helpers::jsonrpc_error(
            id.clone(),
            -32603,
            error,
            None,
        ));
        return;
    }
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

    #[cfg(feature = "angular")]
    if super::provide_angular::try_handle_angular_template(
        id,
        params,
        state,
        &resolved_path,
        source,
    ) {
        return;
    }
    state.remember_persisted_path(&alias, &resolved_path);
    let explicit_fidelity = params["arguments"]["fidelity"].as_str();
    let explicit_intent = params["arguments"]["intent"].as_str();

    let heuristics_start = Instant::now();
    let stored_fidelity = state.context_fidelity(&alias);
    let ir_read = state.ir_context_read();
    let decision = match crate::mcp::heuristics::decide(
        &resolved_path,
        explicit_fidelity,
        explicit_intent,
        &state.config,
        &ir_read,
        source,
        Some(&alias),
        stored_fidelity,
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
    drop(ir_read);
    let heuristics_ms = heuristics_start.elapsed().as_millis() as u64;

    let effective_fidelity = decision.fidelity;
    let edit_focus = (effective_fidelity == crate::compression::Fidelity::Edit)
        .then_some(focus_methods.as_ref())
        .flatten();
    let (content_kind, byte_exact) =
        contract_fields_focused(effective_fidelity, focus_methods.as_ref());
    let is_angular = decision.is_angular;
    let tokenizer_kind = parse_tokenizer_arg(params, &state.config);
    let tokenizer_box = crate::tokenizer::create_tokenizer(tokenizer_kind).ok();
    let tokenizer_ref: Option<&dyn crate::tokenizer::Tokenizer> = tokenizer_box.as_deref();

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
                "_meta": {
                    "strategy": "full", "fidelity": "verbatim",
                    "decision_summary": decision.summary(),
                    "content_kind": ContentKind::VerbatimDocument, "byte_exact": ["document"],
                    "degradation": null, "verbatim": true
                }
            }
        });
        inject_baseline_breakpoint(&mut response, state, &full);
        send_response(&response);
        return;
    }

    let mut te_prediction: &str = "bypass";
    let mut te_threshold: usize = 0;
    if effective_fidelity == crate::compression::Fidelity::Edit {
        let extension = std::path::Path::new(&resolved_path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let raw_tokens = count_tokens_with_tokenizer(source, tokenizer_ref);
        let prediction = crate::mcp::token_economics::should_attempt_compression(
            raw_tokens,
            effective_fidelity,
            extension,
        );
        let threshold =
            crate::mcp::token_economics::compression_threshold(effective_fidelity, extension);
        te_prediction = if prediction {
            "favorable"
        } else {
            "unfavorable"
        };
        te_threshold = threshold;
    }

    let _span = tracing::info_span!(
        "provide_code_context",
        file_path = %resolved_path,
        fidelity = %format!("{:?}", effective_fidelity),
        strategy = "full",
        cbm_status = %state.cbm_status.summary(),
        is_angular = %is_angular,
        prediction = %te_prediction,
        threshold = %te_threshold,
    )
    .entered();

    let compile_start = Instant::now();
    let ir_result = compile_file_ir_focused(&resolved_path, effective_fidelity, state, None);
    let compile_ms = compile_start.elapsed().as_millis() as u64;

    if let Ok((mut ir, semantic_edges, source_hash)) = ir_result {
        let render_start = Instant::now();
        let hir = match resolve_focus_or_respond(id, &mut ir, edit_focus) {
            Some(hierarchy) => hierarchy,
            None => return,
        };

        let candidate = super::content::presentation_document(
            &ir,
            &hir,
            effective_fidelity,
            &resolved_path,
            state,
        );
        let economic = super::content::select_complete_content(
            source,
            candidate,
            edit_focus.is_some(),
            tokenizer_kind,
            tokenizer_ref,
        );
        let raw_tokens = economic.raw_tokens;
        let candidate_tokens = economic.candidate_tokens;
        let selected_tokens = economic.selected_tokens();
        let saved_tokens = economic.saved_tokens();
        let raw_passthrough = matches!(
            economic.selected,
            crate::mcp::content_economics::SelectedRepresentation::RawPassthrough
        );
        if let Err(error) = super::provide_persistence::persist_read_baseline(
            state,
            &resolved_path,
            effective_fidelity,
            &ir,
            &semantic_edges,
            &source_hash,
            raw_tokens,
            candidate_tokens,
        ) {
            send_response(&crate::mcp::tool_helpers::jsonrpc_error(
                id.clone(),
                -32603,
                error,
                None,
            ));
            return;
        }

        let canonical_path = crate::dictionary::path::canonical_identity_key(&resolved_path);
        state
            .ir_context_lock()
            .load_ir(ir.clone(), Some(source_hash));
        state.remember_context_fidelity(&ir.file_id, effective_fidelity);
        {
            let mut idx = state.workspace_index_lock();
            idx.remove_file(&canonical_path);
            idx.add_edges(&canonical_path, semantic_edges.clone());
        }
        state.remember_semantic_edges(&ir.file_id, semantic_edges.clone());
        state
            .llm_text_cache_lock()
            .insert(ir.file_id.clone(), economic.text.clone());
        let render_ms = render_start.elapsed().as_millis() as u64;
        state.record_compression(
            &resolved_path,
            raw_tokens,
            selected_tokens,
            &format!("{:?}", effective_fidelity).to_lowercase(),
            is_angular,
            "full",
            None,
            if raw_passthrough {
                "raw_passthrough"
            } else {
                "ir_compression"
            },
        );
        let visible_content_kind = if raw_passthrough {
            ContentKind::RawPassthrough
        } else {
            content_kind
        };
        let visible_byte_exact = if raw_passthrough {
            serde_json::json!(["document"])
        } else {
            serde_json::to_value(byte_exact).unwrap_or_default()
        };
        let visible_text = economic.text;
        let mut response = serde_json::json!({
            "jsonrpc": "2.0", "id": id, "result": {
                "content": [{ "type": "text", "text": visible_text.clone() }],
                "_meta": {
                    "version": ir.version,
                    "strategy": "full", "fidelity": format!("{:?}", effective_fidelity).to_lowercase(),
                    "is_angular": is_angular, "decision_summary": decision.summary(),
                    "content_kind": visible_content_kind, "byte_exact": visible_byte_exact,
                    "degradation": null
                }
            }
        });
        inject_baseline_breakpoint(&mut response, state, &visible_text);
        send_response(&response);
        let total_ms = overall_start.elapsed().as_millis() as u64;
        tracing::info!(
            heuristics_ms = heuristics_ms,
            compile_ms = compile_ms,
            render_ms = render_ms,
            total_ms = total_ms,
            raw_tokens = raw_tokens,
            comp_tokens = selected_tokens,
            savings_pct = if raw_tokens > 0 {
                (saved_tokens as f64 / raw_tokens as f64 * 100.0) as u64
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
                    "IR compilation unavailable for {}: {}. SCHEMA-v5 structural output \
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
