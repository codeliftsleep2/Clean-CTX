// src/mcp/tool_handlers/persistence/mod.rs
//
// Persistence tool handlers: save, list sessions, replay history,
// and purge old deltas.

use crate::mcp::McpState;
use crate::protocol::send_response;
use serde_json::Value;

/// Handle `save_context` — persists current in-memory context to the DB.
pub(crate) fn handle_save_context(id: &Value, params: &Value, state: &McpState) {
    let _file_path = params["arguments"]["filePath"].as_str();
    let mut saved_count = 0;

    let mut store_guard = state.persistence_store_lock();
    if let Some(ref mut store) = *store_guard {
        store.flush();
        saved_count = 1;
    }
    drop(store_guard);

    send_response(&serde_json::json!({
        "jsonrpc": "2.0", "id": id,
        "result": {
            "ok": true,
            "saved": saved_count,
            "message": format!("Saved {} context(s) to persistence DB.", saved_count)
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
            send_response(
                &serde_json::json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32603, "message": format!("List failed: {}", e) } }),
            );
        }
    }
}

/// Handle `replay_history` — loads and replays delta history from DB.
pub(crate) fn handle_replay_history(id: &Value, params: &Value, state: &McpState) {
    let file_path = params["arguments"]["filePath"].as_str().unwrap_or("");
    let target_seq = params["arguments"]["targetSequence"]
        .as_i64()
        .map(|v| v as u32);

    if file_path.is_empty() {
        send_response(
            &serde_json::json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32602, "message": "Missing required parameter: filePath" } }),
        );
        return;
    }

    let guard = state.persistence_store_lock();
    if let Some(ref store) = *guard {
        match store.load_context_with_deltas(file_path, target_seq) {
            Ok(Some((ir, version))) => {
                drop(guard);
                send_response(&serde_json::json!({
                    "jsonrpc": "2.0", "id": id,
                    "result": {
                        "content": [{ "type": "text", "text": format!("Replayed {} to v{} ({} instructions)", file_path, version, ir.instructions.len()) }],
                        "file": file_path, "version": version, "instruction_count": ir.instructions.len()
                    }
                }));
            }
            Ok(None) => {
                drop(guard);
                send_response(
                    &serde_json::json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32603, "message": format!("No context found for: {}", file_path) } }),
                );
            }
            Err(e) => {
                drop(guard);
                send_response(
                    &serde_json::json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32603, "message": format!("Replay failed: {}", e) } }),
                );
            }
        }
    } else {
        drop(guard);
        send_response(
            &serde_json::json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32603, "message": "Persistence DB not enabled." } }),
        );
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
                    &serde_json::json!({ "jsonrpc": "2.0", "id": id, "result": { "ok": true, "purged": n, "message": format!("Purged {} delta(s) older than {} days.", n, days) } }),
                );
            }
            Err(e) => {
                drop(guard);
                send_response(
                    &serde_json::json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32603, "message": format!("Purge failed: {}", e) } }),
                );
            }
        }
    } else {
        drop(guard);
        send_response(
            &serde_json::json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32603, "message": "Persistence DB not enabled." } }),
        );
    }
}
