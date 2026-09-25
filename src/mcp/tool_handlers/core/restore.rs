// Durable restore_context MCP handler.

use super::common::{ContentKind, checked_hierarchy_or_respond, contract_fields_for_hierarchy};
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

    if let Err(error) = state.recover_pending_edit(&durable_path) {
        return send_restore_error(id, &error);
    }

    // Validate every durable artifact before mutating live session state.
    let restored = {
        let guard = state.persistence_store_lock();
        let Some(store) = guard.as_ref() else {
            return send_restore_error(id, "Persistence is not enabled");
        };
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
    let compact = || {
        super::content::presentation_document(
            &session_ir,
            &hierarchy,
            restored.fidelity,
            &durable_path,
            state,
        )
    };
    let (full, raw_passthrough) = match state.read_source(&durable_path) {
        Ok(source) => {
            let source_matches =
                state.cache_read().compute_hash(source.as_bytes()) == restored.source_hash;
            let tokenizer_kind = crate::mcp::tools::parse_tokenizer_arg(params, &state.config);
            let tokenizer_box = crate::tokenizer::create_tokenizer(tokenizer_kind).ok();
            let economic = super::content::economical_presentation_document(
                &session_ir,
                &hierarchy,
                restored.fidelity,
                &durable_path,
                &source,
                state,
                tokenizer_kind,
                tokenizer_box.as_deref(),
            );
            let selected_raw = matches!(
                economic.selected,
                crate::mcp::content_economics::SelectedRepresentation::RawPassthrough
            );
            if selected_raw && source_matches {
                (economic.text, true)
            } else {
                (compact(), false)
            }
        }
        Err(_) => (compact(), false),
    };
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
    let (content_kind, byte_exact) = contract_fields_for_hierarchy(restored.fidelity, &hierarchy);

    let mut response = serde_json::json!({
        "jsonrpc": "2.0", "id": id,
        "result": {
            "content": [{ "type": "text", "text": full }],
            "ir": crate::ir::hierarchical::hierarchy_to_wire_reduced(&session_ir, &hierarchy),
            "_meta": {
                "version": session_ir.version, "restored": true,
                "file": durable_path,
                "instruction_count": session_ir.instructions.len(),
                "semantic_edge_count": edge_count,
                "content_kind": if raw_passthrough { ContentKind::RawPassthrough } else { content_kind },
                "byte_exact": if raw_passthrough {
                    serde_json::json!(["document"])
                } else {
                    serde_json::to_value(byte_exact).unwrap_or_default()
                }
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
