// Durable restore_context MCP handler.

use super::common::{ContentKind, checked_hierarchy_or_respond, contract_fields_for_hierarchy};
use crate::mcp::McpState;
use crate::mcp::compatibility::validator::{CompatibilityFailure, validate_current_context};
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
        .unwrap_or_else(|| state.durable_owner_path(requested));
    if state.semantic_owner_path(requested) != state.semantic_owner_path(&durable_path) {
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
            Err(error) => return send_compatibility_error(id, &error),
        }
    };

    let source = match state.read_source(&durable_path) {
        Ok(source) => source,
        Err(error) => return send_restore_error(id, &format!("Cannot read current source: {error}")),
    };
    let required_fidelity = restored.fidelity;
    let compatible = match validate_current_context(
        restored,
        &source,
        std::path::Path::new(&durable_path),
        &state.config,
        required_fidelity,
    ) {
        Ok(compatible) => compatible,
        Err(error) => return send_compatibility_error(id, &error),
    };
    let canonical = compatible.canonical;
    let semantic = compatible.semantic;
    if checked_hierarchy_or_respond(id, &canonical.ir).is_none() {
        return;
    }
    let alias = state.get_or_create_alias(durable_path.clone());
    let mut session_ir = canonical.ir;
    session_ir.file_id.clone_from(&alias);
    let hierarchy = match checked_hierarchy_or_respond(id, &session_ir) {
        Some(hierarchy) => hierarchy,
        None => return,
    };
    let compact =
        || super::content::presentation_document(&session_ir, &hierarchy, canonical.fidelity, state);
    let source_matches = state.cache_read().compute_hash(source.as_bytes()) == canonical.source_hash;
    let tokenizer_kind = crate::mcp::tools::parse_tokenizer_arg(params, &state.config);
    let tokenizer_box = crate::tokenizer::create_tokenizer(tokenizer_kind).ok();
    let economic = super::content::economical_presentation_document(
        &session_ir,
        &hierarchy,
        canonical.fidelity,
        &source,
        state,
        tokenizer_kind,
        tokenizer_box.as_deref(),
    );
    let selected_raw = matches!(
        economic.selected,
        crate::mcp::content_economics::SelectedRepresentation::RawPassthrough
    );
    let (full, raw_passthrough) = if selected_raw && source_matches {
        (economic.text, true)
    } else {
        (compact(), false)
    };
    let edge_count = semantic.semantic_edges.len();

    state
        .ir_context_lock()
        .load_ir(session_ir.clone(), Some(canonical.source_hash.clone()));
    state.remember_persisted_path(&alias, &durable_path);
    state.remember_context_fidelity(&alias, canonical.fidelity);
    state.publish_compatible_semantic_projection(&alias, &durable_path, semantic);
    state
        .llm_text_cache_lock()
        .insert(alias.clone(), full.clone());
    let (content_kind, byte_exact) = contract_fields_for_hierarchy(canonical.fidelity, &hierarchy);

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

fn send_compatibility_error(id: &Value, error: &CompatibilityFailure) {
    send_response(&crate::mcp::tool_helpers::jsonrpc_error(
        id.clone(),
        -32603,
        &format!("Durable restore rejected: {error}"),
        Some(serde_json::json!({
            "reason": error.reason(),
            "component": error.component()
        })),
    ));
}

#[cfg(all(test, feature = "typescript"))]
#[path = "../../../tests/mcp/durable_restore_compatibility.rs"]
mod compatibility_tests;
