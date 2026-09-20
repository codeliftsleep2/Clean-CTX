// Diff, delta, and apply-delta MCP handlers.

use super::common::{compiled_from_tuples, contract_fields, invalid_session_ir_response};
use crate::error::to_jsonrpc_error;
use crate::ir::delta::{IRDelta, SequenceDelta, SequenceDeltaComputer};
use crate::mcp::McpState;
use crate::mcp::tool_helpers::{
    compile_file_ir, diff_code_context_handler, inject_baseline_breakpoint, inject_tail_breakpoint,
    resolve_file_path_checked,
};
use crate::mcp::tools::parse_fidelity_arg;
use crate::protocol::send_response;
use serde_json::Value;
use std::path::PathBuf;
mod persistence;
use persistence::{
    ensure_apply_baseline, ensure_persisted_baseline, persist_baseline, persisted_context_id,
};
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
    let (compiled, semantic_edges, source_hash) =
        match compile_file_ir(&resolved_path, fidelity, state) {
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
        ir_ctx.load_ir(compiled.clone(), Some(source_hash.clone()));
        let mut idx = state.workspace_index_lock();
        idx.remove_file(&canonical_path);
        idx.add_edges(&canonical_path, semantic_edges.clone());
        drop(idx);
        state.remember_semantic_edges(&path_alias, semantic_edges.clone());
        None
    };
    if let Some(delta) = &mut delta {
        delta.target_hash = Some(source_hash.clone());
    }
    state.remember_context_fidelity(&path_alias, fidelity);
    if let Some(delta) = &delta {
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
            let mut response = serde_json::json!({
                "jsonrpc": "2.0", "id": id, "result": {
                    "content": [{ "type": "text", "text": format!("Δ delta for {} (v{} → v{}): {} positional edits", compiled.file_id, d.from, d.to, d.edits.len()) }],
                    "delta": wire_delta, "from_version": d.from, "to_version": d.to,
                    "strategy": "delta", "fidelity": format!("{:?}", fidelity).to_lowercase(),
                    "content_kind": content_kind, "byte_exact": byte_exact,
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
            }
            let version = if prev_version == 0 {
                compiled.version
            } else {
                prev_version
            };
            let mut response = serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "result": {
                    "content": [{ "type": "text", "text": format!("Baseline stored for {} (v{})", compiled.file_id, version) }],
                    "version": version, "instruction_count": compiled.instructions.len(),
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

// ── Handler: apply_delta ──────────────────────────────────────────

pub(crate) fn handle_apply_delta(id: &Value, params: &Value, state: &McpState) {
    let delta_value = &params["arguments"]["delta"];
    let current_version = params["arguments"]["currentVersion"].as_i64();

    enum IncomingDelta {
        Sequence(SequenceDelta),
        Legacy(IRDelta),
    }
    let delta = if delta_value.get("dv").is_some() {
        serde_json::from_value(delta_value.clone()).map(IncomingDelta::Sequence)
    } else {
        serde_json::from_value(delta_value.clone()).map(IncomingDelta::Legacy)
    };
    let delta = match delta {
        Ok(delta) => delta,
        Err(e) => {
            send_response(&crate::mcp::tool_helpers::jsonrpc_error(
                id.clone(),
                -32602,
                format!("Invalid delta: {e}"),
                None,
            ));
            return;
        }
    };
    let (file, from) = match &delta {
        IncomingDelta::Sequence(delta) => (delta.file.clone(), delta.from),
        IncomingDelta::Legacy(delta) => (delta.file.clone(), delta.from),
    };

    if current_version != Some(from as i64) {
        send_response(&crate::mcp::tool_helpers::jsonrpc_error(
            id.clone(),
            -32602,
            format!(
                "Version mismatch: client has v{:?}, delta expects from v{}",
                current_version, from
            ),
            None,
        ));
        return;
    }

    let durable_file = state
        .persisted_path(&file)
        .or_else(|| state.path_for_alias(&file))
        .unwrap_or_else(|| file.clone());
    let persistence_enabled = state.persistence_store_lock().is_some();
    let pending_transition = match &delta {
        IncomingDelta::Sequence(delta) if persistence_enabled => {
            match state.pending_transition(&file, &durable_file, delta) {
                Ok(transition) => Some(transition),
                Err(error) => {
                    send_response(&invalid_session_ir_response(id, &error));
                    return;
                }
            }
        }
        IncomingDelta::Legacy(_) if persistence_enabled => {
            send_response(&invalid_session_ir_response(
                id,
                "durable legacy delta application lacks authoritative semantic-edge state",
            ));
            return;
        }
        _ => None,
    };
    let persisted_payload = match &delta {
        IncomingDelta::Sequence(delta) => {
            crate::mcp::persistence_ir::PersistedDelta::normalize_sequence(delta, &durable_file)
        }
        IncomingDelta::Legacy(delta) => {
            crate::mcp::persistence_ir::PersistedDelta::normalize_legacy(delta, &durable_file)
        }
    };
    let persisted_payload = match persisted_payload {
        Ok(payload) => payload,
        Err(error) => {
            send_response(&crate::mcp::tool_helpers::jsonrpc_error(
                id.clone(),
                -32603,
                format!("Delta persistence encoding failed: {error}"),
                None,
            ));
            return;
        }
    };
    if let Err(error) = ensure_apply_baseline(state, &file, &durable_file) {
        send_response(&invalid_session_ir_response(id, &error));
        return;
    }
    let persisted_context = match persisted_context_id(state, &durable_file) {
        Ok(context) => context,
        Err(error) => {
            send_response(&crate::mcp::tool_helpers::jsonrpc_error(
                id.clone(),
                -32603,
                error,
                None,
            ));
            return;
        }
    };

    let edit_type = match &delta {
        IncomingDelta::Sequence(_) => "sequence_v2",
        IncomingDelta::Legacy(_) => "legacy",
    };
    let mut ir_ctx = state.ir_context_lock();
    let mut candidate = ir_ctx.clone();
    let applied = match &delta {
        IncomingDelta::Sequence(delta) => candidate.apply_sequence(delta.clone()),
        IncomingDelta::Legacy(delta) => candidate.apply(delta.clone()),
    };
    match applied {
        Ok(new_version) => {
            let source_hash = pending_transition
                .as_ref()
                .map(|transition| transition.target_source_hash.clone())
                .or_else(|| candidate.get_source_hash(&file).cloned())
                .unwrap_or_else(|| format!("ir-{file}-{new_version}"));
            candidate.set_source_hash(&file, source_hash.clone());
            let target_tuples = candidate
                .get_ir(&file)
                .cloned()
                .ok_or_else(|| "applied delta lost canonical target state".to_string());
            let target_ir = target_tuples
                .and_then(|tuples| compiled_from_tuples(file.clone(), new_version, tuples));
            let target_ir = match target_ir {
                Ok(target) => target,
                Err(error) => {
                    drop(ir_ctx);
                    send_response(&invalid_session_ir_response(id, &error));
                    return;
                }
            };
            let hierarchy = match crate::ir::hierarchical::try_ir_to_hierarchical(&target_ir) {
                Ok(hierarchy) => hierarchy,
                Err(error) => {
                    drop(ir_ctx);
                    send_response(&crate::mcp::tool_handlers::core::projection_error_response(
                        id, &error,
                    ));
                    return;
                }
            };
            let fidelity = state
                .context_fidelity(&file)
                .unwrap_or(crate::compression::Fidelity::Low);
            let compact = crate::ir::render_hierarchical_for_llm(&hierarchy, fidelity);

            if let Some(transition) = &pending_transition {
                debug_assert_eq!(transition.from, from);
                debug_assert_eq!(transition.to, new_version);
                let snapshot = crate::mcp::state::durable_semantics::DurableSemanticSnapshot::new(
                    durable_file.clone(),
                    source_hash.clone(),
                    new_version,
                    &transition.semantic_edges,
                );
                let persisted = persisted_context.as_ref().is_some_and(|context_id| {
                    state
                        .persistence_store_lock()
                        .as_ref()
                        .is_some_and(|store| {
                            store.flush();
                            store.sqlite().is_some_and(|mut sqlite| {
                                sqlite
                                    .append_delta_with_semantics(
                                        context_id,
                                        &persisted_payload,
                                        edit_type,
                                        &compact,
                                        &snapshot,
                                    )
                                    .is_ok()
                            })
                        })
                });
                if !persisted {
                    drop(ir_ctx);
                    send_response(&invalid_session_ir_response(
                        id,
                        "canonical delta and semantic edges were not persisted atomically",
                    ));
                    return;
                }
            } else if let Some(context_id) = &persisted_context {
                if let Some(ref store) = *state.persistence_store_lock() {
                    store.queue_append_delta(context_id, &persisted_payload, Some(edit_type));
                    store.flush();
                }
            }

            *ir_ctx = candidate;
            let rendered = ir_ctx.render_pretty(&file, crate::compression::Fidelity::Low);
            drop(ir_ctx);
            if let Some(transition) = pending_transition {
                let canonical_path = crate::dictionary::path::canonical_identity_key(&durable_file);
                let mut index = state.workspace_index_lock();
                index.remove_file(&canonical_path);
                index.add_edges(&canonical_path, transition.semantic_edges.clone());
                drop(index);
                state.remember_semantic_edges(&file, transition.semantic_edges);
                state.consume_pending_transition(&file, from, new_version);
            }
            let mut response = serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "result": { "content": [{ "type": "text", "text": rendered.unwrap_or_default() }], "_meta": { "version": new_version } }
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

#[cfg(test)]
#[path = "../../../tests/mcp/delta_sequence.rs"]
mod sequence_tests;
