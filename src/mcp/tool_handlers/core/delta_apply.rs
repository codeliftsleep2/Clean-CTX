//! Apply-delta handler, separated from delta production orchestration.

use super::common::{
    ContentKind, compiled_from_tuples, contract_fields, invalid_session_ir_response,
};
use super::delta::persistence::{ensure_apply_baseline, persisted_context_id};
use crate::ir::delta::{IRDelta, SequenceDelta};
use crate::mcp::McpState;
use crate::mcp::tool_helpers::inject_tail_breakpoint;
use crate::mcp::tools::parse_tokenizer_arg;
use crate::protocol::send_response;
use serde_json::Value;

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
    let fidelity = match state.context_fidelity(&file) {
        Some(fidelity) => fidelity,
        None => {
            send_response(&invalid_session_ir_response(
                id,
                "missing authoritative fidelity for delta application",
            ));
            return;
        }
    };
    if let Err(error) = state.preflight_semantic_publication(&durable_file) {
        send_response(&invalid_session_ir_response(id, &error));
        return;
    }
    let persistence_enabled = state.persistence_store_lock().is_some();
    let pending_transition = match &delta {
        IncomingDelta::Sequence(delta) if !delta.edits.is_empty() => {
            match state.pending_transition(&file, &durable_file, delta) {
                Ok(transition) => Some(transition),
                Err(error) => {
                    send_response(&invalid_session_ir_response(id, &error));
                    return;
                }
            }
        }
        IncomingDelta::Legacy(delta)
            if !delta.ops.adds.is_empty()
                || !delta.ops.mods.is_empty()
                || !delta.ops.dels.is_empty() =>
        {
            send_response(&invalid_session_ir_response(
                id,
                "mutating legacy delta application lacks authoritative semantic-edge state",
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
    if let Err(error) = ensure_apply_baseline(state, &file, &durable_file, fidelity) {
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
            let target_edges = pending_transition
                .as_ref()
                .map(|transition| transition.semantic_edges.clone())
                .or_else(|| state.semantic_edges(&file))
                .unwrap_or_default();
            let normalized = crate::ir::normalize_control_full(
                &target_ir.file_id,
                &durable_file,
                target_ir.version,
                fidelity,
                &hierarchy,
                &target_edges,
            );
            let compact = crate::ir::compact_a::render_file_context(&normalized);

            if let Some(transition) = &pending_transition {
                debug_assert_eq!(transition.from, from);
                debug_assert_eq!(transition.to, new_version);
                if persistence_enabled {
                    let snapshot =
                        crate::mcp::state::durable_semantics::DurableSemanticSnapshot::new(
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
                }
            } else if let Some(context_id) = &persisted_context {
                if let Some(ref store) = *state.persistence_store_lock() {
                    let persisted = store.sqlite().is_some_and(|mut sqlite| {
                        crate::mcp::context_store::ContextStore::append_delta(
                            &mut *sqlite,
                            context_id,
                            &persisted_payload,
                            Some(edit_type),
                        )
                        .is_ok()
                    });
                    if !persisted {
                        drop(ir_ctx);
                        send_response(&invalid_session_ir_response(
                            id,
                            "accepted delta was not persisted by its scoped transaction",
                        ));
                        return;
                    }
                }
            }

            *ir_ctx = candidate;
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
            let source = match state.read_source(&durable_file) {
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
            let economic = super::content::economical_presentation_document(
                &target_ir,
                &hierarchy,
                fidelity,
                &durable_file,
                &source,
                state,
                tokenizer_kind,
                tokenizer_box.as_deref(),
            );
            let raw_passthrough = matches!(
                economic.selected,
                crate::mcp::content_economics::SelectedRepresentation::RawPassthrough
            );
            let rendered = economic.text;
            let (_, candidate_byte_exact) = contract_fields(fidelity);
            let hierarchical_wire =
                crate::ir::hierarchical::hierarchy_to_wire_reduced(&target_ir, &hierarchy);
            let mut response = serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "result": {
                    "content": [{ "type": "text", "text": rendered }],
                    "structuredContent": {
                        "ir": hierarchical_wire
                    },
                    "_meta": {
                        "version": new_version,
                "content_kind": if raw_passthrough { ContentKind::RawPassthrough } else { ContentKind::Skeleton },
                        "byte_exact": if raw_passthrough {
                            serde_json::json!(["document"])
                        } else {
                            serde_json::to_value(candidate_byte_exact).unwrap_or_default()
                        }
                    }
                }
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
