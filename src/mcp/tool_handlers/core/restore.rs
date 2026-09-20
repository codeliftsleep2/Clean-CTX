// Durable restore_context MCP handler.

use super::common::checked_hierarchy_or_respond;
use crate::mcp::McpState;
use crate::mcp::tool_helpers::inject_baseline_breakpoint;
use crate::protocol::send_response;
use serde_json::Value;

pub(crate) fn handle_restore_context(id: &Value, params: &Value, state: &McpState) {
    let requested = crate::mcp::tool_helpers::arg_str_or_empty(params, "filePath");
    if requested.is_empty() {
        return send_restore_error(id, "Missing required parameter: filePath");
    }

    let durable_path = state
        .alias_for_path(requested)
        .and_then(|alias| state.persisted_path(&alias))
        .unwrap_or_else(|| requested.to_string());
    if crate::dictionary::path::canonical_identity_key(requested)
        != crate::dictionary::path::canonical_identity_key(&durable_path)
    {
        return send_restore_error(id, "Requested file does not match its durable identity");
    }

    if let Err(error) = crate::mcp::tool_handlers::edit::recover_pending_edit(state, &durable_path)
    {
        return send_restore_error(id, &error);
    }

    // Validate every durable artifact before mutating live session state.
    let restored = {
        let guard = state.persistence_store_lock();
        let Some(store) = guard.as_ref() else {
            return send_restore_error(id, "Persistence is not enabled");
        };
        store.flush();
        let Some(sqlite) = store.sqlite() else {
            return send_restore_error(id, "Persistence DB is unavailable");
        };
        match sqlite.load_durable_context(&durable_path, None) {
            Ok(Some(restored)) => restored,
            Ok(None) => return send_restore_error(id, "No persisted context for requested file"),
            Err(error) => {
                return send_restore_error(id, &format!("Durable restore failed: {error}"));
            }
        }
    };

    if checked_hierarchy_or_respond(id, &restored.ir).is_none() {
        return;
    }
    let alias = state.get_or_create_alias(durable_path.clone());
    let mut session_ir = restored.ir;
    session_ir.file_id.clone_from(&alias);
    let hierarchy = match checked_hierarchy_or_respond(id, &session_ir) {
        Some(hierarchy) => hierarchy,
        None => return,
    };
    let full = restored.compact_output.unwrap_or_else(|| {
        let compact = crate::ir::render_hierarchical_for_llm(&hierarchy, restored.fidelity);
        format!(
            "{}\n// ── {} ({}) ──\n{}",
            compact.trim(),
            alias,
            durable_path,
            state.format_dict_footer_for_aliases(&[&alias]).trim()
        )
    });
    let canonical_path = crate::dictionary::path::canonical_identity_key(&durable_path);
    let edge_count = restored.semantic_edges.len();

    state
        .ir_context_lock()
        .load_ir(session_ir.clone(), Some(restored.source_hash.clone()));
    state.remember_persisted_path(&alias, &durable_path);
    state.remember_context_fidelity(&alias, restored.fidelity);
    state.remember_semantic_edges(&alias, restored.semantic_edges.clone());
    {
        let mut index = state.workspace_index_lock();
        index.remove_file(&canonical_path);
        index.add_edges(&canonical_path, restored.semantic_edges);
    }
    state
        .llm_text_cache_lock()
        .insert(alias.clone(), full.clone());

    let mut response = serde_json::json!({
        "jsonrpc": "2.0", "id": id,
        "result": {
            "content": [{ "type": "text", "text": full }],
            "ir": crate::ir::hierarchical::hierarchy_to_wire(&session_ir, &hierarchy),
            "_meta": {
                "version": session_ir.version, "restored": true,
                "file": durable_path,
                "instruction_count": session_ir.instructions.len(),
                "semantic_edge_count": edge_count
            }
        }
    });
    inject_baseline_breakpoint(&mut response, state, &full);
    send_response(&response);
}

fn send_restore_error(id: &Value, message: &str) {
    send_response(&crate::mcp::tool_helpers::jsonrpc_error(
        id.clone(),
        -32603,
        message,
        None,
    ));
}
