// Diff, delta, and apply-delta MCP handlers.

use super::common::{
    ContentKind, checked_hierarchy_or_respond, compiled_from_tuples, contract_fields_for_hierarchy,
    invalid_session_ir_response,
};
use crate::error::to_jsonrpc_error;
use crate::ir::delta::SequenceDeltaComputer;
use crate::mcp::McpState;
use crate::mcp::tool_helpers::{
    compile_file_ir_candidate, count_tokens_with_tokenizer, diff_code_context_handler,
    inject_baseline_breakpoint, inject_tail_breakpoint, resolve_file_path_checked,
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
    let semantic_owner = state.semantic_owner_path(&resolved_path);
    let durable_owner = state.durable_owner_path(&resolved_path);
    let publication = state.begin_semantic_publication(&semantic_owner);
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
    let fidelity_name = format!("{:?}", fidelity).to_lowercase();
    let is_angular =
        state.config.auto_angular && crate::angular_meta::detect::is_angular_file(source.as_str());

    // A-08: Check if source has changed before compiling
    let path_alias = state
        .alias_for_path(&resolved_path)
        .unwrap_or_else(|| semantic_owner.clone());
    // Try to skip compilation if source is unchanged
    // P0-4: Hold lock during entire check to prevent TOCTOU race
    let ir_ctx = state.ir_context_lock();
    let prev_version = ir_ctx.file_version(&path_alias).unwrap_or(0);
    let previous_authority = (prev_version > 0 && ir_ctx.has_file(&path_alias)).then(|| {
        (
            ir_ctx.get_ir(&path_alias).cloned().unwrap_or_default(),
            ir_ctx.get_source_hash(&path_alias).cloned(),
        )
    });
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
                let economic = super::content::economical_presentation_document(
                    &compiled,
                    &hierarchy,
                    fidelity,
                    &source,
                    state,
                    tokenizer_kind,
                    tokenizer_ref,
                );
                let raw_passthrough = matches!(
                    economic.selected,
                    crate::mcp::content_economics::SelectedRepresentation::RawPassthrough
                );
                let raw_tokens = economic.raw_tokens;
                let compressed_tokens = economic.selected_tokens();
                let content = economic.text;
                let (content_kind, byte_exact) =
                    contract_fields_for_hierarchy(fidelity, &hierarchy);
                let mut response = serde_json::json!({
                    "jsonrpc": "2.0", "id": id,
                    "result": {
                        "content": [{ "type": "text", "text": content }],
                        "ir": crate::ir::hierarchical::hierarchy_to_wire_reduced(&compiled, &hierarchy),
                        "version": prev_version,
                        "instruction_count": instruction_count,
                        "content_kind": if raw_passthrough { ContentKind::RawPassthrough } else { content_kind },
                        "byte_exact": if raw_passthrough {
                            serde_json::json!(["document"])
                        } else {
                            serde_json::to_value(byte_exact).unwrap_or_default()
                        },
                        "cached": true
                    }
                });
                // Cached IR is a stable snapshot — inject baseline breakpoint
                // so the client can cache the unchanged output.
                if let Some(text) = response["result"]["content"][0]["text"].as_str() {
                    let text_owned = text.to_string();
                    inject_baseline_breakpoint(&mut response, state, &text_owned);
                }
                state.record_compression(
                    &resolved_path,
                    raw_tokens,
                    compressed_tokens,
                    &fidelity_name,
                    is_angular,
                    "full",
                    None,
                    "ir_compression",
                );
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
    compiled.version = prev_version + 1;

    #[cfg(test)]
    super::provide::publication_race_test_support::pause_after_compile(&resolved_path);

    let canonical_path = semantic_owner;
    let expected_source_hash = source_hash.clone();
    let committed = publication.commit(
        || {
            Ok::<_, String>(state.read_source(&resolved_path).is_ok_and(|current| {
                state.cache_read().compute_hash(current.as_bytes()) == expected_source_hash
            }))
        },
        || {
            let mut ir_ctx = state.ir_context_lock();
            let authority_is_current = match &previous_authority {
                Some((instructions, previous_hash)) => {
                    ir_ctx.file_version(&path_alias) == Some(prev_version)
                        && ir_ctx.get_ir(&path_alias) == Some(instructions)
                        && ir_ctx.get_source_hash(&path_alias) == previous_hash.as_ref()
                }
                None => state
                    .alias_for_path(&resolved_path)
                    .is_none_or(|alias| !ir_ctx.has_file(&alias)),
            };
            if !authority_is_current {
                return Ok((false, None));
            }

            let mut delta = if let Some((prev_instructions, previous_hash)) = &previous_authority {
                let prev_compiled = compiled_from_tuples(
                    path_alias.clone(),
                    prev_version,
                    prev_instructions.clone(),
                )?;
                let previous_hash = previous_hash
                    .clone()
                    .unwrap_or_else(|| source_hash.clone());
                if super::provide_persistence::read_checkpoint_required(state, fidelity) {
                    ensure_persisted_baseline(
                        state,
                        &durable_owner,
                        fidelity,
                        &prev_compiled,
                        &previous_hash,
                    )?;
                }
                SequenceDeltaComputer::new().compute(&prev_compiled, &compiled)
            } else {
                None
            };
            if let Some(delta) = &mut delta {
                delta.target_hash = Some(source_hash.clone());
                state.remember_context_fidelity(&path_alias, fidelity);
                let compatibility = crate::mcp::compatibility::derive_identities(
                    &source,
                    std::path::Path::new(&durable_owner),
                    &state.config,
                )
                .map_err(|error| format!("cannot derive delta compatibility: {error}"))?;
                state.remember_pending_transition(
                    &path_alias,
                    &durable_owner,
                    delta,
                    source_hash.clone(),
                    crate::mcp::compatibility::validator::CompatibleSemanticProjection::from_current_compilation(
                        semantic_edges.clone(),
                        source_hash.clone(),
                        fidelity,
                    ),
                    compatibility,
                )?;
            } else if previous_authority.is_some() {
                ir_ctx.set_source_hash(&path_alias, source_hash.clone());
            } else {
                let checkpoint_required =
                    super::provide_persistence::read_checkpoint_required(state, fidelity);
                if checkpoint_required {
                    persist_baseline(
                        state,
                        &durable_owner,
                        fidelity,
                        &compiled,
                        &source_hash,
                        &semantic_edges,
                        &source,
                    )?;
                }
                let committed_alias = state.get_or_create_alias(canonical_path.clone());
                compiled.file_id.clone_from(&committed_alias);
                ir_ctx.load_ir(compiled.clone(), Some(source_hash.clone()));
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
            Ok((true, delta))
        },
    );
    let delta = match committed {
        Ok(Some((true, delta))) => delta,
        Ok(None) | Ok(Some((false, _))) => {
            send_response(&invalid_session_ir_response(
                id,
                "Delta authority was superseded by a newer same-file source snapshot; retry the request",
            ));
            return;
        }
        Err(error) => {
            send_response(&invalid_session_ir_response(id, &error));
            return;
        }
    };

    match delta {
        Some(d) => {
            let wire_delta = serde_json::to_value(&d).unwrap_or_default();
            // The delta is code-side only: the LLM is stateless and must never
            // receive a delta (envelope OR presentation). Emit the same minimal
            // acknowledgement main produces; the op list rides only in
            // `result.delta` for code-side consumption.
            let (adds, mods, dels) = d.summary_counts();
            let content = format!(
                "Δ delta for {} (v{} → v{}): +{} ~{} -{} ops",
                compiled.file_id, d.from, d.to, adds, mods, dels
            );
            let raw_tokens = count_tokens_with_tokenizer(source.as_str(), tokenizer_ref);
            let compressed_tokens = count_tokens_with_tokenizer(&content, tokenizer_ref);
            let previous_full_tokens = state
                .session_stats_lock()
                .file_stats(&canonical_path)
                .map(|stats| stats.compressed_tokens);
            let mut response = serde_json::json!({
                "jsonrpc": "2.0", "id": id, "result": {
                    "content": [{ "type": "text", "text": content }],
                    "delta": wire_delta, "from_version": d.from, "to_version": d.to,
                    "strategy": "delta", "fidelity": fidelity_name,
                    "content_kind": ContentKind::DeltaSummary,
                    "byte_exact": [],
                    "degradation": null
                }
            });
            // Delta output is rolling dynamic content — mark as tail (ephemeral).
            inject_tail_breakpoint(&mut response, state);
            state.record_compression(
                &canonical_path,
                raw_tokens,
                compressed_tokens,
                &fidelity_name,
                is_angular,
                "delta",
                previous_full_tokens,
                "ir_compression",
            );
            send_response(&response);
        }
        None => {
            let version = if prev_version == 0 {
                compiled.version
            } else {
                prev_version
            };
            let hierarchy = match checked_hierarchy_or_respond(id, &compiled) {
                Some(hierarchy) => hierarchy,
                None => return,
            };
            let economic = super::content::economical_presentation_document(
                &compiled,
                &hierarchy,
                fidelity,
                &source,
                state,
                tokenizer_kind,
                tokenizer_ref,
            );
            let raw_passthrough = matches!(
                economic.selected,
                crate::mcp::content_economics::SelectedRepresentation::RawPassthrough
            );
            let raw_tokens = economic.raw_tokens;
            let compressed_tokens = economic.selected_tokens();
            let content = economic.text;
            let (content_kind, byte_exact) = contract_fields_for_hierarchy(fidelity, &hierarchy);
            let mut response = serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "result": {
                    "content": [{ "type": "text", "text": content }],
                    "ir": crate::ir::hierarchical::hierarchy_to_wire_reduced(&compiled, &hierarchy),
                    "version": version, "instruction_count": compiled.instructions.len(),
                    "content_kind": if raw_passthrough { ContentKind::RawPassthrough } else { content_kind },
                    "byte_exact": if raw_passthrough {
                        serde_json::json!(["document"])
                    } else {
                        serde_json::to_value(byte_exact).unwrap_or_default()
                    }
                }
            });
            // Baseline stored — this is a stable snapshot, inject baseline breakpoint.
            if let Some(text) = response["result"]["content"][0]["text"].as_str() {
                let text_owned = text.to_string();
                inject_baseline_breakpoint(&mut response, state, &text_owned);
            }
            state.record_compression(
                &resolved_path,
                raw_tokens,
                compressed_tokens,
                &fidelity_name,
                is_angular,
                "full",
                None,
                "ir_compression",
            );
            send_response(&response);
        }
    }
}

#[cfg(test)]
#[path = "../../../tests/mcp/delta_sequence.rs"]
mod sequence_tests;

#[cfg(all(test, feature = "typescript"))]
#[path = "../../../tests/mcp/delta_publication_race.rs"]
mod publication_race_tests;

#[cfg(all(test, feature = "typescript"))]
#[path = "../../../tests/mcp/delta_edit_recovery.rs"]
mod edit_recovery_tests;

#[cfg(all(test, feature = "typescript"))]
#[path = "../../../tests/mcp/delta_fidelity_persistence.rs"]
mod fidelity_persistence_tests;

#[cfg(all(test, feature = "typescript"))]
#[path = "../../../tests/mcp/delta_compatibility_epoch.rs"]
mod compatibility_epoch_tests;

// Presentation boundary: the delta is code-side only. Its model-visible
// `content` is a minimal summary (adds/mods/dels counts), never a full
// presentation and never a `// FILE-CONTEXT-DELTA v1` envelope. The structured
// op list stays in `result.delta` for `apply_delta`.
#[cfg(all(test, feature = "typescript"))]
#[path = "../../../tests/mcp/delta_presentation_boundary.rs"]
mod delta_presentation_boundary_tests;

#[cfg(all(test, feature = "typescript"))]
#[path = "../../../tests/mcp/delta_stats_lifecycle.rs"]
mod delta_stats_lifecycle_tests;
