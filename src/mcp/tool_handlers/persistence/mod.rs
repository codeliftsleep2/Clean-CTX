// src/mcp/tool_handlers/persistence/mod.rs
//
// Persistence tool handlers: save, list sessions, replay history,
// and purge old deltas.

use crate::mcp::McpState;
use crate::protocol::send_response;
use serde_json::Value;
mod replay;
pub(crate) use replay::handle_replay_history;
#[cfg(test)]
#[path = "../../../tests/mcp/save_context_race_support.rs"]
pub(super) mod save_snapshot_test_support;

/// Handle `save_context` — persists current in-memory context to the DB.
pub(crate) fn handle_save_context(id: &Value, params: &Value, state: &McpState) {
    let requested = crate::mcp::tool_helpers::arg_str_or_empty(params, "filePath");
    if requested.is_empty() {
        return send_persistence_error(id, "Missing required parameter: filePath");
    }
    let direct_alias = state.ir_context_lock().has_file(requested);
    let alias = if direct_alias {
        requested.to_string()
    } else if let Some(alias) = state.alias_for_path(requested) {
        alias
    } else {
        return send_persistence_error(id, "No session-owned context for requested file");
    };
    let durable_path = match state.persisted_path(&alias) {
        Some(path) => path,
        None => return send_persistence_error(id, "Missing durable identity for requested file"),
    };
    let requested_path = if direct_alias {
        match state.path_for_alias(&alias) {
            Some(path) => path,
            None => return send_persistence_error(id, "Missing path identity for requested alias"),
        }
    } else {
        requested.to_string()
    };
    if state.semantic_owner_path(&requested_path) != state.semantic_owner_path(&durable_path) {
        return send_persistence_error(id, "Requested file does not match its durable identity");
    }
    if let Err(error) = state.preflight_semantic_publication(&durable_path) {
        return send_persistence_error(id, &error);
    }
    let snapshot_owner = durable_path.clone();
    let persist = state.with_semantic_authority_snapshot(&snapshot_owner, || {
        save_context_snapshot(id, state, alias, requested_path, durable_path)
    });
    if let Some(persist) = persist {
        persist();
    }
}
fn save_context_snapshot<'a>(
    id: &'a Value,
    state: &'a McpState,
    alias: String,
    requested_path: String,
    durable_path: String,
) -> Option<impl FnOnce() + 'a> {
    let fidelity = match state.context_fidelity(&alias) {
        Some(fidelity) => fidelity,
        None => {
            send_persistence_error(id, "Missing fidelity for requested file");
            return None;
        }
    };
    let (tuples, version, source_hash) = {
        let context = state.ir_context_lock();
        let Some(tuples) = context.get_ir(&alias).cloned() else {
            send_persistence_error(id, "Missing canonical IR for requested file");
            return None;
        };
        let Some(version) = context.file_version(&alias) else {
            send_persistence_error(id, "Missing canonical IR version for requested file");
            return None;
        };
        let Some(source_hash) = context.get_source_hash(&alias).cloned() else {
            send_persistence_error(id, "Missing source hash for requested file");
            return None;
        };
        (tuples, version, source_hash)
    };
    #[cfg(test)]
    save_snapshot_test_support::pause_after_canonical_capture(&durable_path);
    let mut instructions = Vec::with_capacity(tuples.len());
    for tuple in &tuples {
        let Some(operation) = crate::ir::wire::tuple_to_op(tuple) else {
            send_persistence_error(id, "Session canonical IR contains an invalid tuple");
            return None;
        };
        instructions.push(operation);
    }
    let session_ir = crate::ir::compiler::CompiledIR {
        file_id: alias.clone(),
        version,
        instructions,
    };
    let hierarchy = match crate::ir::hierarchical::try_ir_to_hierarchical(&session_ir) {
        Ok(hierarchy) => hierarchy,
        Err(error) => {
            send_response(&crate::mcp::tool_handlers::core::projection_error_response(
                id, &error,
            ));
            return None;
        }
    };
    let semantic_edges = match state.semantic_edges(&alias) {
        Some(edges) => edges,
        None => {
            send_persistence_error(id, "Missing authoritative semantic-edge state");
            return None;
        }
    };
    let compact = crate::mcp::tool_handlers::core::content::presentation_document(
        &session_ir,
        &hierarchy,
        fidelity,
        state,
    );
    let durable_ir = crate::mcp::persistence_ir::baseline(&session_ir, &durable_path);
    let binary = crate::ir::binary_wire::encode(&durable_ir);
    let (raw_tokens, compressed_tokens) = state
        .session_stats_lock()
        .file_stats(&requested_path)
        .map(|stats| (stats.raw_tokens as u64, stats.compressed_tokens as u64))
        .unwrap_or((0, 0));
    let source = match state.read_source(&durable_path) {
        Ok(source) => source,
        Err(error) => {
            send_persistence_error(id, &error.to_string());
            return None;
        }
    };
    if state.cache_read().compute_hash(source.as_bytes()) != source_hash {
        send_persistence_error(
            id,
            "Checkpoint source changed outside the captured authority epoch",
        );
        return None;
    }
    let identities = match crate::mcp::compatibility::derive_identities(
        &source,
        std::path::Path::new(&durable_path),
        &state.config,
    ) {
        Ok(identities) => identities,
        Err(error) => {
            send_persistence_error(id, &error.to_string());
            return None;
        }
    };

    Some(move || {
        let store_guard = state.persistence_store_lock();
        let Some(store) = store_guard.as_ref() else {
            return send_persistence_error(id, "Persistence is not enabled");
        };
        let already_durable = store.sqlite().is_some_and(|sqlite| {
            sqlite
                .durable_state_matches(
                    &durable_path,
                    fidelity,
                    &compact,
                    &binary,
                    &source_hash,
                    version,
                    &semantic_edges,
                    &identities,
                )
                .unwrap_or(false)
        });
        let saved_count = if already_durable {
            0
        } else {
            let persisted = store.sqlite().is_some_and(|mut sqlite| {
                sqlite
                    .save_context_with_compatibility(
                        &durable_path,
                        fidelity,
                        &compact,
                        &binary,
                        &source_hash,
                        version,
                        &semantic_edges,
                        raw_tokens,
                        compressed_tokens,
                        &identities,
                    )
                    .is_ok()
            });
            if !persisted {
                return send_persistence_error(id, "Requested checkpoint was not persisted");
            }
            1
        };

        send_response(&serde_json::json!({
            "jsonrpc": "2.0", "id": id,
            "result": {
                "content": [{ "type": "text", "text": format!("Saved {} context(s) to persistence DB.", saved_count) }],
                "_meta": { "ok": true, "saved": saved_count, "already_durable": already_durable, "file": durable_path }
            }
        }));
    })
}

fn send_persistence_error(id: &Value, message: &str) {
    send_response(&crate::mcp::tool_helpers::jsonrpc_error(
        id.clone(),
        -32603,
        message,
        None,
    ));
}

/// Handle `delete_context` — transactionally delete one file-scoped semantic
/// context without modifying the source file.
pub(crate) fn handle_delete_context(id: &Value, params: &Value, state: &McpState) {
    let requested = crate::mcp::tool_helpers::arg_str_or_empty(params, "filePath");
    if requested.is_empty() {
        return send_persistence_error(id, "Missing required parameter: filePath");
    }

    let direct_alias = state.ir_context_read().has_file(requested);
    let path_alias = state.alias_for_path(requested);
    let alias = match (direct_alias, path_alias) {
        (true, Some(path_alias)) if path_alias != requested => {
            return send_persistence_error(id, "Ambiguous session ownership for requested file");
        }
        (true, _) => requested.to_string(),
        (false, Some(alias)) => alias,
        (false, None) => {
            return send_persistence_error(id, "No session-owned context for requested file");
        }
    };
    if !state.ir_context_read().has_file(&alias) {
        return send_persistence_error(
            id,
            "Missing canonical session ownership for requested file",
        );
    }
    let owned_path = match state.path_for_alias(&alias) {
        Some(path) => path,
        None => return send_persistence_error(id, "Missing path identity for requested context"),
    };
    let durable_path = match state.persisted_path(&alias) {
        Some(path) => path,
        None => {
            return send_persistence_error(id, "Missing durable identity for requested context");
        }
    };
    let requested_path = if direct_alias {
        owned_path.as_str()
    } else {
        requested
    };
    let canonical_requested = state.semantic_owner_path(requested_path);
    if canonical_requested != state.semantic_owner_path(&owned_path)
        || canonical_requested != state.semantic_owner_path(&durable_path)
    {
        return send_persistence_error(id, "Requested file does not match its durable identity");
    }

    let deletion = {
        let store_guard = state.persistence_store_lock();
        let Some(store) = store_guard.as_ref() else {
            return send_persistence_error(id, "Persistence is not enabled");
        };
        let Some(mut sqlite) = store.sqlite() else {
            return send_persistence_error(id, "Persistence DB is unavailable");
        };
        match sqlite.delete_context_transactionally(&durable_path) {
            Ok(deletion) => deletion,
            Err(error) => {
                return send_persistence_error(id, &format!("Context deletion failed: {error}"));
            }
        }
    };
    match deletion.count {
        1 => {}
        0 => return send_persistence_error(id, "No matching persisted context was deleted"),
        _ => return send_persistence_error(id, "Ambiguous persisted ownership for requested file"),
    }

    if let Some(stage_path) = deletion.recovery_stage.as_deref() {
        crate::mcp::sqlite_store::remove_deleted_recovery_stage(&durable_path, stage_path);
    }

    state.ir_context_lock().remove_file(&alias);
    state
        .workspace_index_lock()
        .remove_file(&canonical_requested);
    state.llm_text_cache_lock().remove(&alias);
    state.forget_context_caches(&durable_path);
    state.forget_persisted_path(&alias);
    let alias_removed = state.forget_path_alias(&alias, &owned_path);
    debug_assert!(
        alias_removed,
        "validated alias ownership must remain stable"
    );

    send_response(&serde_json::json!({
        "jsonrpc": "2.0", "id": id,
        "result": {
            "content": [{ "type": "text", "text": format!("Deleted persisted context for {durable_path}.") }],
            "_meta": { "ok": true, "deleted": 1, "file": durable_path, "source_deleted": false }
        }
    }));
}

/// Handle `list_sessions` — enumerate persisted contexts stored in the DB.
///
/// Non-CBM audit 2026-08-25 #7: this tool previously returned a static
/// `"Persistence DB active."` string while its name/description promised
/// an enumeration. The persistence model has NO session concept (the
/// `sessions` table is dead schema that nothing ever writes), so rather
/// than inventing fake session data the tool lists what genuinely exists:
/// one entry per persisted FILE CONTEXT — path, fidelity, token counts,
/// delta count, last-update timestamp.
pub(crate) fn handle_list_sessions(id: &Value, params: &Value, state: &McpState) {
    let _ = params;
    let guard = state.persistence_store_lock();
    let store = match *guard {
        Some(ref store) => store,
        None => {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "result": { "content": [{ "type": "text", "text": "Persistence not enabled" }] }
            }));
            return;
        }
    };

    match store.list_contexts(200) {
        Ok(rows) => {
            let mut text = format!("Persisted contexts: {}\n", rows.len());
            if rows.is_empty() {
                text.push_str("(none yet — compressed contexts appear here once saved)\n");
            }
            for row in &rows {
                text.push_str(&format!(
                    "  - {} [{}] deltas={} tokens={}→{} updated={}\n",
                    row.file_path,
                    row.fidelity,
                    row.delta_count,
                    row.raw_tokens,
                    row.compressed_tokens,
                    row.updated_at,
                ));
            }
            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "result": { "content": [{ "type": "text", "text": text }] }
            }));
        }
        Err(e) => {
            send_response(&crate::mcp::tool_helpers::jsonrpc_error(
                id.clone(),
                -32603,
                format!("List failed: {}", e),
                None,
            ));
        }
    }
}

/// Inspect quarantined legacy fallback evidence without recovering or mutating it.
pub(crate) fn handle_inspect_legacy_fallbacks(id: &Value, params: &Value, state: &McpState) {
    let _ = params;
    let guard = state.persistence_store_lock();
    let Some(store) = guard.as_ref() else {
        return send_persistence_error(id, "Persistence is not enabled");
    };
    let result = legacy_fallback_inspection_result(store.inspect_legacy_fallbacks());
    send_response(&serde_json::json!({
        "jsonrpc": "2.0", "id": id, "result": result
    }));
}

fn legacy_fallback_inspection_result(
    artifacts: Vec<crate::mcp::buffered_store::LegacyFallbackArtifact>,
) -> Value {
    let rows = artifacts
        .iter()
        .map(|artifact| {
            serde_json::json!({
                "path": artifact.path.to_string_lossy(),
                "operation": artifact.operation,
                "identity": artifact.identity,
                "availableMetadata": artifact.available_metadata,
                "recoverable": false,
                "reason": artifact.reason,
            })
        })
        .collect::<Vec<_>>();
    serde_json::json!({
        "content": [{
            "type": "text",
            "text": format!("Quarantined legacy fallback artifacts: {}. None are recoverable under the current semantic contract.", rows.len())
        }],
        "structuredContent": { "count": rows.len(), "artifacts": rows }
    })
}

/// Handle `purge_old_deltas` — clean up old deltas from DB.
pub(crate) fn handle_purge_old_deltas(id: &Value, params: &Value, state: &McpState) {
    let days = params["arguments"]["days"].as_i64().unwrap_or(30).max(1);

    let mut guard = state.persistence_store_lock();
    if let Some(ref mut store) = *guard {
        match store.purge_old_deltas(days as u32) {
            Ok(n) => {
                drop(guard);
                send_response(
                    &serde_json::json!({ "jsonrpc": "2.0", "id": id, "result": { "content": [{ "type": "text", "text": format!("Purged {} delta(s) older than {} days.", n, days) }], "_meta": { "ok": true, "purged": n } } }),
                );
            }
            Err(e) => {
                drop(guard);
                send_response(&crate::mcp::tool_helpers::jsonrpc_error(
                    id.clone(),
                    -32603,
                    format!("Purge failed: {}", e),
                    None,
                ));
            }
        }
    } else {
        drop(guard);
        send_response(&crate::mcp::tool_helpers::jsonrpc_error(
            id.clone(),
            -32603,
            "Persistence DB not enabled.",
            None,
        ));
    }
}

#[cfg(all(test, feature = "typescript"))]
#[path = "../../../tests/mcp/persistence_lifecycle.rs"]
mod lifecycle_tests;

#[cfg(test)]
#[path = "../../../tests/mcp/baseline_publication.rs"]
mod baseline_publication_tests;

#[cfg(all(test, feature = "typescript"))]
#[path = "../../../tests/mcp/replay_edit_recovery.rs"]
mod replay_edit_recovery_tests;

#[cfg(all(test, feature = "typescript"))]
#[path = "../../../tests/mcp/save_context_contract.rs"]
mod save_context_contract_tests;

#[cfg(all(test, feature = "typescript"))]
#[path = "../../../tests/mcp/durable_semantic_restore.rs"]
mod durable_semantic_restore_tests;

#[cfg(all(test, feature = "typescript"))]
#[path = "../../../tests/mcp/historical_replay_compatibility.rs"]
mod historical_replay_compatibility_tests;

#[cfg(all(test, feature = "typescript"))]
#[path = "../../../tests/mcp/delete_context_contract.rs"]
mod delete_context_contract_tests;

#[cfg(test)]
#[path = "../../../tests/mcp/list_sessions_read_only.rs"]
mod list_sessions_read_only_tests;

#[cfg(test)]
#[path = "../../../tests/mcp/legacy_fallback_inspection.rs"]
mod legacy_fallback_inspection_tests;
