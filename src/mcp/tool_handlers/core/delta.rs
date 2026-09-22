// Diff, delta, and apply-delta MCP handlers.

use super::common::{
    checked_hierarchy_or_respond, compiled_from_tuples, contract_fields,
    invalid_session_ir_response,
};
use crate::error::to_jsonrpc_error;
use crate::ir::delta::SequenceDeltaComputer;
use crate::mcp::McpState;
use crate::mcp::tool_helpers::{
    compile_file_ir_candidate, diff_code_context_handler, inject_baseline_breakpoint,
    inject_tail_breakpoint, resolve_file_path_checked,
};
use crate::mcp::tools::{parse_fidelity_arg, parse_tokenizer_arg};
use crate::protocol::send_response;
use serde_json::Value;
use std::path::PathBuf;
pub(super) mod persistence;
use persistence::{ensure_persisted_baseline, persist_baseline};
// ── Handler: diff_code_context ────────────────────────────────────

pub(crate) fn handle_diff_code_context(id: &Value, params: &Value, state: &McpState) {
    let file_path_str = crate::mcp::tool_helpers::arg_str_or_empty(params, "filePath");
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
    let file_path_str = crate::mcp::tool_helpers::arg_str_or_empty(params, "filePath");
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
    if let Err(error) = state.preflight_semantic_publication(&resolved_path) {
        send_response(&invalid_session_ir_response(id, &error));
        return;
    }
    let fidelity = match parse_fidelity_arg(id, params, &state.config) {
        Ok(f) => f,
        Err(()) => return,
    };
    let source = match state.read_source(&resolved_path) {
        Ok(source) => source,
        Err(error) => {
            send_response(&invalid_session_ir_response(
                id,
                &format!("Cannot read source for economics gate: {error}"),
            ));
            return;
        }
    };
    let tokenizer_kind = parse_tokenizer_arg(params, &state.config);
    let tokenizer_box = crate::tokenizer::create_tokenizer(tokenizer_kind).ok();
    let tokenizer_ref = tokenizer_box.as_deref();

    // A-08: Check if source has changed before compiling
    let path_alias = state
        .alias_for_path(&resolved_path)
        .unwrap_or_else(|| resolved_path.clone());
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
                let compiled =
                    match compiled_from_tuples(path_alias.clone(), prev_version, cached_ir) {
                        Ok(compiled) => compiled,
                        Err(error) => {
                            send_response(&invalid_session_ir_response(id, &error));
                            return;
                        }
                    };
                let hierarchy = match checked_hierarchy_or_respond(id, &compiled) {
                    Some(hierarchy) => hierarchy,
                    None => return,
                };
                let edges = state.semantic_edges(&path_alias).unwrap_or_default();
                let economic = super::content::economical_compact_a_document(
                    &compiled,
                    &hierarchy,
                    &edges,
                    fidelity,
                    &resolved_path,
                    &source,
                    state,
                    tokenizer_kind,
                    tokenizer_ref,
                );
                let raw_passthrough = matches!(
                    economic.selected,
                    crate::mcp::content_economics::SelectedRepresentation::RawPassthrough
                );
                let content = economic.text;
                let mut response = serde_json::json!({
                    "jsonrpc": "2.0", "id": id,
                    "result": {
                        "content": [{ "type": "text", "text": content }],
                        "ir": crate::ir::hierarchical::hierarchy_to_wire(&compiled, &hierarchy),
                        "semantic_edges": serde_json::to_value(&edges).unwrap_or_default(),
                        "version": prev_version,
                        "instruction_count": instruction_count,
                "content_kind": if raw_passthrough { "raw_passthrough" } else { "compact_a2" },
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
    let (mut compiled, semantic_edges, source_hash) =
        match compile_file_ir_candidate(&resolved_path, fidelity, state) {
            Ok(c) => c,
            Err(e) => {
                send_response(&to_jsonrpc_error(id, &e));
                return;
            }
        };

    let canonical_path = crate::dictionary::path::canonical_identity_key(&resolved_path);

    // P0-4: Re-acquire lock atomically for delta computation
    // This ensures no other worker modified ir_context between our check and delta computation
    let mut ir_ctx = state.ir_context_lock();
    let mut delta = if prev_version > 0 && ir_ctx.has_file(&path_alias) {
        let Some(prev_instructions) = ir_ctx.get_ir(&path_alias).cloned() else {
            drop(ir_ctx);
            send_response(&invalid_session_ir_response(
                id,
                "missing prior canonical instruction stream",
            ));
            return;
        };
        let prev_compiled =
            match compiled_from_tuples(path_alias.clone(), prev_version, prev_instructions) {
                Ok(compiled) => compiled,
                Err(error) => {
                    drop(ir_ctx);
                    send_response(&invalid_session_ir_response(id, &error));
                    return;
                }
            };
        let previous_hash = ir_ctx
            .get_source_hash(&path_alias)
            .cloned()
            .unwrap_or_else(|| source_hash.clone());
        if let Err(error) = ensure_persisted_baseline(
            state,
            &resolved_path,
            fidelity,
            &prev_compiled,
            &previous_hash,
        ) {
            drop(ir_ctx);
            send_response(&invalid_session_ir_response(id, &error));
            return;
        }
        SequenceDeltaComputer::new().compute(&prev_compiled, &compiled)
    } else {
        None
    };
    if let Some(delta) = &mut delta {
        delta.target_hash = Some(source_hash.clone());
    }
    if let Some(delta) = &delta {
        state.remember_context_fidelity(&path_alias, fidelity);
        if let Err(error) = state.remember_pending_transition(
            &path_alias,
            &resolved_path,
            delta,
            source_hash.clone(),
            semantic_edges.clone(),
        ) {
            drop(ir_ctx);
            send_response(&invalid_session_ir_response(id, &error));
            return;
        }
    } else if prev_version > 0 {
        ir_ctx.set_source_hash(&path_alias, source_hash.clone());
    }
    drop(ir_ctx);

    match delta {
        Some(d) => {
            let wire_delta = serde_json::to_value(&d).unwrap_or_default();
            let (content_kind, byte_exact) = contract_fields(fidelity);
            let content = super::content::control_full_delta(
                &compiled.file_id,
                fidelity,
                &d,
                &semantic_edges,
            );
            let economic = crate::mcp::content_economics::select_with_local_tokenizer(
                &source,
                content,
                tokenizer_kind,
                tokenizer_ref,
            );
            let raw_passthrough = matches!(
                economic.selected,
                crate::mcp::content_economics::SelectedRepresentation::RawPassthrough
            );
            let content = economic.text;
            let mut response = serde_json::json!({
                "jsonrpc": "2.0", "id": id, "result": {
                    "content": [{ "type": "text", "text": content }],
                    "delta": wire_delta, "from_version": d.from, "to_version": d.to,
                    "strategy": "delta", "fidelity": format!("{:?}", fidelity).to_lowercase(),
                    "content_kind": if raw_passthrough { "raw_passthrough" } else { content_kind },
                    "byte_exact": if raw_passthrough { serde_json::json!(["document"]) } else { serde_json::to_value(byte_exact).unwrap_or_default() },
                    "degradation": null,
                    "semantic_edges": serde_json::to_value(&semantic_edges).unwrap_or_default()
                }
            });
            // Delta output is rolling dynamic content — mark as tail (ephemeral).
            inject_tail_breakpoint(&mut response, state);
            send_response(&response);
        }
        None => {
            if prev_version == 0 {
                if let Err(error) = persist_baseline(
                    state,
                    &resolved_path,
                    fidelity,
                    &compiled,
                    &source_hash,
                    &semantic_edges,
                ) {
                    send_response(&invalid_session_ir_response(id, &error));
                    return;
                }

                // P9-14: the durable baseline commit precedes publication of
                // every corresponding live owner.
                let committed_alias = state.get_or_create_alias(resolved_path.clone());
                compiled.file_id.clone_from(&committed_alias);
                state
                    .ir_context_lock()
                    .load_ir(compiled.clone(), Some(source_hash.clone()));
                state.remember_context_fidelity(&committed_alias, fidelity);
                {
                    let mut idx = state.workspace_index_lock();
                    idx.remove_file(&canonical_path);
                    idx.add_edges(&canonical_path, semantic_edges.clone());
                }
                state.remember_semantic_edges(&committed_alias, semantic_edges.clone());
                if state.persistence_store_lock().is_some() {
                    state.remember_persisted_path(&committed_alias, &resolved_path);
                }
            }
            let version = if prev_version == 0 {
                compiled.version
            } else {
                prev_version
            };
            let hierarchy = match checked_hierarchy_or_respond(id, &compiled) {
                Some(hierarchy) => hierarchy,
                None => return,
            };
            let economic = super::content::economical_compact_a_document(
                &compiled,
                &hierarchy,
                &semantic_edges,
                fidelity,
                &resolved_path,
                &source,
                state,
                tokenizer_kind,
                tokenizer_ref,
            );
            let raw_passthrough = matches!(
                economic.selected,
                crate::mcp::content_economics::SelectedRepresentation::RawPassthrough
            );
            let content = economic.text;
            let mut response = serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "result": {
                    "content": [{ "type": "text", "text": content }],
                    "ir": crate::ir::hierarchical::hierarchy_to_wire(&compiled, &hierarchy),
                    "version": version, "instruction_count": compiled.instructions.len(),
                    "content_kind": if raw_passthrough { "raw_passthrough" } else { "compact_a2" },
                    "semantic_edges": serde_json::to_value(&semantic_edges).unwrap_or_default()
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

#[cfg(test)]
#[path = "../../../tests/mcp/delta_sequence.rs"]
mod sequence_tests;

#[cfg(all(test, feature = "typescript"))]
#[path = "../../../tests/mcp/delta_edit_recovery.rs"]
mod edit_recovery_tests;
