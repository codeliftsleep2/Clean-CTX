// src/mcp/tool_handlers/persistence/mod.rs
//
// Persistence tool handlers: save, list sessions, replay history,
// and purge old deltas.

use crate::mcp::McpState;
use crate::mcp::context_store::ContextStore;
use crate::protocol::send_response;
use serde_json::Value;

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
    if crate::dictionary::path::canonical_identity_key(&requested_path)
        != crate::dictionary::path::canonical_identity_key(&durable_path)
    {
        return send_persistence_error(id, "Requested file does not match its durable identity");
    }
    let fidelity = match state.context_fidelity(&alias) {
        Some(fidelity) => fidelity,
        None => return send_persistence_error(id, "Missing fidelity for requested file"),
    };
    let (tuples, version, source_hash) = {
        let context = state.ir_context_lock();
        let Some(tuples) = context.get_ir(&alias).cloned() else {
            return send_persistence_error(id, "Missing canonical IR for requested file");
        };
        let Some(version) = context.file_version(&alias) else {
            return send_persistence_error(id, "Missing canonical IR version for requested file");
        };
        let Some(source_hash) = context.get_source_hash(&alias).cloned() else {
            return send_persistence_error(id, "Missing source hash for requested file");
        };
        (tuples, version, source_hash)
    };
    let mut instructions = Vec::with_capacity(tuples.len());
    for tuple in &tuples {
        let Some(operation) = crate::ir::wire::tuple_to_op(tuple) else {
            return send_persistence_error(id, "Session canonical IR contains an invalid tuple");
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
            return;
        }
    };
    let compact = state
        .llm_text_cache_lock()
        .get(&alias)
        .cloned()
        .unwrap_or_else(|| crate::ir::render_hierarchical_for_llm(&hierarchy, fidelity));
    let durable_ir = crate::mcp::persistence_ir::baseline(&session_ir, &durable_path);
    let binary = crate::ir::binary_wire::encode(&durable_ir);

    let store_guard = state.persistence_store_lock();
    let Some(store) = store_guard.as_ref() else {
        return send_persistence_error(id, "Persistence is not enabled");
    };
    let already_durable = store.sqlite().is_some_and(|sqlite| {
        sqlite
            .checkpoint_matches(&durable_path, fidelity, &compact, &binary, &source_hash)
            .unwrap_or(false)
    });
    let saved_count = if already_durable {
        0
    } else {
        store.queue_save_context(
            &durable_path,
            fidelity,
            &compact,
            &binary,
            &source_hash,
            0,
            0,
        );
        store.flush();
        let persisted = store.sqlite().is_some_and(|sqlite| {
            sqlite
                .checkpoint_matches(&durable_path, fidelity, &compact, &binary, &source_hash)
                .unwrap_or(false)
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
}

fn send_persistence_error(id: &Value, message: &str) {
    send_response(&crate::mcp::tool_helpers::jsonrpc_error(
        id.clone(),
        -32603,
        message,
        None,
    ));
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

/// Handle `replay_history` — loads and replays delta history from DB.
pub(crate) fn handle_replay_history(id: &Value, params: &Value, state: &McpState) {
    let file_path = crate::mcp::tool_helpers::arg_str_or_empty(params, "filePath");
    let target_seq = params["arguments"]["targetSequence"]
        .as_i64()
        .map(|v| v as u32);

    if file_path.is_empty() {
        send_response(&crate::mcp::tool_helpers::jsonrpc_error(
            id.clone(),
            -32602,
            "Missing required parameter: filePath",
            None,
        ));
        return;
    }

    let guard = state.persistence_store_lock();
    if let Some(ref store) = *guard {
        let (fidelity, source_hash) = match store.load_latest(file_path) {
            Ok(Some(metadata)) => (metadata.fidelity, metadata.source_hash),
            Ok(None) => {
                drop(guard);
                send_response(&crate::mcp::tool_helpers::jsonrpc_error(
                    id.clone(),
                    -32603,
                    format!("No context metadata found for: {file_path}"),
                    None,
                ));
                return;
            }
            Err(error) => {
                drop(guard);
                send_response(&crate::mcp::tool_helpers::jsonrpc_error(
                    id.clone(),
                    -32603,
                    format!("Replay metadata failed: {error}"),
                    None,
                ));
                return;
            }
        };
        let persisted_compact = store
            .sqlite()
            .and_then(|sqlite| sqlite.latest_compact_output(file_path).ok().flatten());
        match store.load_context_with_deltas(file_path, target_seq) {
            Ok(Some((mut ir, version))) => {
                drop(guard);
                let path_alias = state.get_or_create_alias(file_path.to_string());
                state.forget_persisted_path(&path_alias);
                state.remember_persisted_path(&path_alias, file_path);
                ir.file_id = path_alias;
                state
                    .ir_context_lock()
                    .load_ir(ir.clone(), Some(source_hash));
                state.remember_context_fidelity(&ir.file_id, fidelity);
                let hierarchy = match crate::ir::hierarchical::try_ir_to_hierarchical(&ir) {
                    Ok(hierarchy) => hierarchy,
                    Err(error) => {
                        send_response(&crate::mcp::tool_handlers::core::projection_error_response(
                            id, &error,
                        ));
                        return;
                    }
                };
                let rendered = persisted_compact.unwrap_or_else(|| {
                    crate::ir::render_hierarchical_for_llm(&hierarchy, fidelity)
                });
                state
                    .llm_text_cache_lock()
                    .insert(ir.file_id.clone(), rendered.clone());
                send_response(&serde_json::json!({
                    "jsonrpc": "2.0", "id": id,
                    "result": {
                        "content": [{ "type": "text", "text": rendered }],
                        "ir": crate::ir::hierarchical::hierarchy_to_wire(&ir, &hierarchy),
                        "_meta": { "file": file_path, "version": version, "instruction_count": ir.instructions.len() }
                    }
                }));
            }
            Ok(None) => {
                drop(guard);
                send_response(&crate::mcp::tool_helpers::jsonrpc_error(
                    id.clone(),
                    -32603,
                    format!("No context found for: {}", file_path),
                    None,
                ));
            }
            Err(e) => {
                drop(guard);
                send_response(&crate::mcp::tool_helpers::jsonrpc_error(
                    id.clone(),
                    -32603,
                    format!("Replay failed: {}", e),
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

#[cfg(all(test, feature = "typescript"))]
#[path = "../../../tests/mcp/save_context_contract.rs"]
mod save_context_contract_tests;
