// provide_code_context MCP handler.
use super::common::{
    checked_hierarchy_or_respond, compiled_from_tuples, contract_fields_focused,
    invalid_session_ir_response, maybe_economics_fallback,
};
use crate::error::to_jsonrpc_error;
use crate::ir::delta::SequenceDeltaComputer;
use crate::mcp::McpState;
use crate::mcp::tool_helpers::{
    compile_file_ir_focused, count_tokens_with_tokenizer, inject_baseline_breakpoint,
    inject_tail_breakpoint, resolve_file_path_checked,
};
use crate::mcp::tools::parse_tokenizer_arg;
use crate::protocol::send_response;
use serde_json::Value;
use std::collections::HashSet;
pub(crate) fn handle_provide_code_context(id: &Value, params: &Value, state: &McpState) {
    use std::time::Instant;
    let overall_start = Instant::now();

    // Optional method targeting retains verbatim bodies only for named units.
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

    // Angular templates have a separate producer and rendering lifecycle.
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

    // Phase 1: Heuristics decision
    let heuristics_start = Instant::now();
    let ir_read = state.ir_context_read();
    let decision = match crate::mcp::heuristics::decide(
        &resolved_path,
        explicit_fidelity,
        explicit_intent,
        &state.config,
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
                "_meta": {
                    "strategy": "full", "fidelity": "verbatim",
                    "decision_summary": decision.summary(),
                    "content_kind": "verbatim_document", "byte_exact": ["document"],
                    "degradation": null, "verbatim": true
                }
            }
        });
        inject_baseline_breakpoint(&mut response, state, &full);
        send_response(&response);
        return;
    }

    // Predict Edit-fidelity economics before entering the render pipeline.
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
        if !prediction {
            // Edit fidelity still requires tracked IR state for byte ranges.
            match compile_file_ir_focused(
                &resolved_path,
                crate::compression::Fidelity::Edit,
                state,
                focus_methods.as_ref(),
            ) {
                Ok((compiled, semantic_edges, source_hash)) => {
                    if checked_hierarchy_or_respond(id, &compiled).is_none() {
                        return;
                    }
                    let compiled_file = compiled.file_id.clone();
                    state
                        .ir_context_lock()
                        .load_ir(compiled, Some(source_hash));
                    state.remember_context_fidelity(&compiled_file, effective_fidelity);

                    // Rendering economics never changes semantic ownership.
                    let canonical_path =
                        crate::dictionary::path::canonical_identity_key(&resolved_path);
                    let mut idx = state.workspace_index_lock();
                    idx.remove_file(&canonical_path);
                    idx.add_edges(&canonical_path, semantic_edges.clone());
                    drop(idx);
                    state.remember_semantic_edges(&compiled_file, semantic_edges);
                }
                Err(e) => {
                    send_response(&to_jsonrpc_error(id, &e));
                    return;
                }
            }

            maybe_economics_fallback(
                id,
                source,
                raw_tokens,
                raw_tokens + 1, // comp_tokens > raw_tokens always (raw passthrough)
                state,
                &resolved_path,
                is_angular,
                crate::compression::Fidelity::Edit,
                &decision.summary(),
            );
            return;
        }
    }

    // A-04: Create tracing span for this call
    let _span = tracing::info_span!(
        "provide_code_context",
        file_path = %resolved_path,
        fidelity = %format!("{:?}", effective_fidelity),
        strategy = %format!("{:?}", strategy),
        cbm_status = %state.cbm_status.summary(),
        is_angular = %is_angular,
        prediction = %te_prediction,
        threshold = %te_threshold,
    )
    .entered();

    match strategy {
        crate::mcp::heuristics::ContextStrategy::DeltaTransport => {
            let compile_start = Instant::now();
            let (compiled, semantic_edges, source_hash) = match compile_file_ir_focused(
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
            let checked_hir = match checked_hierarchy_or_respond(id, &compiled) {
                Some(hierarchy) => hierarchy,
                None => return,
            };

            let canonical_path = crate::dictionary::path::canonical_identity_key(&resolved_path);

            let delta_start = Instant::now();
            let prev_version = state.file_version(&alias).unwrap_or(0);
            let mut ir_ctx = state.ir_context_lock();
            let mut delta = if prev_version > 0 && ir_ctx.has_file(&alias) {
                let Some(prev_instructions) = ir_ctx.get_ir(&alias).cloned() else {
                    drop(ir_ctx);
                    send_response(&invalid_session_ir_response(
                        id,
                        "missing prior instruction stream",
                    ));
                    return;
                };
                let prev_compiled =
                    match compiled_from_tuples(alias.clone(), prev_version, prev_instructions) {
                        Ok(compiled) => compiled,
                        Err(error) => {
                            drop(ir_ctx);
                            send_response(&invalid_session_ir_response(id, &error));
                            return;
                        }
                    };
                SequenceDeltaComputer::new().compute(&prev_compiled, &compiled)
            } else {
                ir_ctx.load_ir(compiled.clone(), Some(source_hash.clone()));
                let mut idx = state.workspace_index_lock();
                idx.remove_file(&canonical_path);
                idx.add_edges(&canonical_path, semantic_edges.clone());
                drop(idx);
                state.remember_semantic_edges(&alias, semantic_edges.clone());
                None
            };
            if let Some(delta) = &mut delta {
                delta.target_hash = Some(source_hash.clone());
            }
            state.remember_context_fidelity(&alias, effective_fidelity);
            if let Some(delta) = &delta {
                if let Err(error) = state.remember_pending_transition(
                    &alias,
                    &resolved_path,
                    delta,
                    source_hash,
                    semantic_edges.clone(),
                ) {
                    drop(ir_ctx);
                    send_response(&invalid_session_ir_response(id, &error));
                    return;
                }
            }
            drop(ir_ctx);
            let _delta_ms = delta_start.elapsed().as_millis() as u64;

            let raw_tokens;
            let comp_tokens;

            match delta {
                Some(ref d) => {
                    let wire_delta = serde_json::to_value(d).unwrap_or_default();
                    // Count the actual delta payload.
                    let delta_text = serde_json::to_string(&wire_delta).unwrap_or_default();
                    raw_tokens = count_tokens_with_tokenizer(&delta_text, tokenizer_ref);
                    comp_tokens = raw_tokens; // delta is the payload itself
                    let prev_full_compressed = state
                        .session_stats_lock()
                        .file_stats(&resolved_path)
                        .map(|f| f.compressed_tokens);
                    let mut response = serde_json::json!({
                        "jsonrpc": "2.0", "id": id, "result": {
                            "content": [{ "type": "text", "text": format!("Δ delta for {} (v{} → v{}): {} positional edits", compiled.file_id, d.from, d.to, d.edits.len()) }],
                            "_meta": {
                                "delta": wire_delta, "from_version": d.from, "to_version": d.to,
                                "strategy": "delta", "fidelity": format!("{:?}", effective_fidelity).to_lowercase(),
                                "decision_summary": decision.summary(),
                                "content_kind": content_kind, "byte_exact": byte_exact,
                                "degradation": null,
                                "semantic_edges": serde_json::to_value(&semantic_edges).unwrap_or_default()
                            }
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
                    let llm_text = crate::ir::render_hierarchical_for_llm_focused(
                        &checked_hir,
                        effective_fidelity,
                        focus_methods.as_ref(),
                    );
                    let full = format!(
                        "{}\n// ── {} ({}) ──\n{}",
                        llm_text.trim(),
                        compiled.file_id,
                        resolved_path,
                        state
                            .format_dict_footer_for_aliases(&[&compiled.file_id])
                            .trim()
                    );
                    let render_ms = render_start.elapsed().as_millis() as u64;
                    raw_tokens = count_tokens_with_tokenizer(source, tokenizer_ref);
                    comp_tokens = count_tokens_with_tokenizer(&full, tokenizer_ref);
                    // Fall back when the compact representation costs more
                    // tokens than the raw source, at every fidelity level.
                    if maybe_economics_fallback(
                        id,
                        source,
                        raw_tokens,
                        comp_tokens,
                        state,
                        &resolved_path,
                        is_angular,
                        effective_fidelity,
                        &decision.summary(),
                    ) {
                        return;
                    }
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
                            "content": [{ "type": "text", "text": full }],
                            "_meta": {
                                "version": compiled.version,
                                "strategy": "full", "fidelity": format!("{:?}", effective_fidelity).to_lowercase(),
                                "decision_summary": decision.summary(),
                                "content_kind": content_kind, "byte_exact": byte_exact,
                                "degradation": null,
                                "semantic_edges": serde_json::to_value(&semantic_edges).unwrap_or_default()
                            }
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

            if let Ok((ir, semantic_edges, source_hash)) = ir_result {
                let render_start = Instant::now();
                // Note: IR error is logged below in the else branch (4.4 audit fix)
                let hir = match checked_hierarchy_or_respond(id, &ir) {
                    Some(hierarchy) => hierarchy,
                    None => return,
                };

                // Compute canonical file identity for WorkspaceIndex provenance
                let canonical_path =
                    crate::dictionary::path::canonical_identity_key(&resolved_path);
                state
                    .ir_context_lock()
                    .load_ir(ir.clone(), Some(source_hash));
                state.remember_context_fidelity(&ir.file_id, effective_fidelity);
                // Update workspace index: remove stale edges, insert fresh ones.
                {
                    let mut idx = state.workspace_index_lock();
                    idx.remove_file(&canonical_path);
                    idx.add_edges(&canonical_path, semantic_edges.clone());
                }
                state.remember_semantic_edges(&ir.file_id, semantic_edges.clone());
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
                    state.format_dict_footer_for_aliases(&[&ir.file_id]).trim()
                );
                state
                    .llm_text_cache_lock()
                    .insert(ir.file_id.clone(), full.clone());
                let render_ms = render_start.elapsed().as_millis() as u64;
                let raw_tokens = count_tokens_with_tokenizer(source, tokenizer_ref);
                let comp_tokens = count_tokens_with_tokenizer(&full, tokenizer_ref);
                // Post-compression token-economics check: if the
                // compressed/hybrid representation costs more tokens than
                // the raw source, fall back to raw passthrough. This
                // applies to all fidelity levels (Edit, Low, Medium, High).
                if maybe_economics_fallback(
                    id,
                    source,
                    raw_tokens,
                    comp_tokens,
                    state,
                    &resolved_path,
                    is_angular,
                    effective_fidelity,
                    &decision.summary(),
                ) {
                    return;
                }
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
                        "content": [{ "type": "text", "text": full }],
                        "_meta": {
                            "version": ir.version,
                            "strategy": "full", "fidelity": format!("{:?}", effective_fidelity).to_lowercase(),
                            "is_angular": is_angular, "decision_summary": decision.summary(),
                            "content_kind": content_kind, "byte_exact": byte_exact,
                            "degradation": null,
                            "semantic_edges": serde_json::to_value(&semantic_edges).unwrap_or_default()
                        }
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
                }));
            }
        }
    }
}
