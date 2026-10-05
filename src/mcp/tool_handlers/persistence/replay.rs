use super::send_persistence_error;
use crate::mcp::McpState;
use crate::mcp::compatibility::validator::{CompatibilityFailure, validate_historical_context};
use crate::mcp::tool_handlers::core::{ContentKind, contract_fields_for_hierarchy};
use crate::protocol::send_response;
use serde_json::Value;

fn send_replay_compatibility_error(id: &Value, error: &CompatibilityFailure) {
    send_response(&crate::mcp::tool_helpers::jsonrpc_error(
        id.clone(),
        -32603,
        format!("Historical replay rejected: {error}"),
        Some(serde_json::json!({
            "reason": error.reason(),
            "component": error.component()
        })),
    ));
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
    let publication = state.begin_semantic_publication(file_path);

    if let Err(error) = state.recover_pending_edit(file_path) {
        return send_persistence_error(id, &error);
    }

    let restored = {
        let guard = state.persistence_store_lock();
        let Some(store) = guard.as_ref() else {
            return send_persistence_error(id, "Persistence DB not enabled");
        };
        let Some(sqlite) = store.sqlite() else {
            return send_persistence_error(id, "Persistence DB is unavailable");
        };
        match sqlite.load_durable_context(file_path, target_seq) {
            Ok(Some(restored)) => restored,
            Ok(None) => return send_persistence_error(id, "No persisted context found"),
            Err(error) => return send_replay_compatibility_error(id, &error),
        }
    };

    let source = match state.read_source(file_path) {
        Ok(source) => source,
        Err(error) => {
            return send_persistence_error(id, &format!("Cannot read current source: {error}"));
        }
    };
    let required_fidelity = restored.fidelity;
    let compatible = match validate_historical_context(
        restored,
        &source,
        std::path::Path::new(file_path),
        &state.config,
        required_fidelity,
    ) {
        Ok(compatible) => compatible,
        Err(error) => return send_replay_compatibility_error(id, &error),
    };
    let canonical = compatible.canonical;
    let semantic = compatible.semantic;
    let path_alias = state.get_or_create_alias(file_path.to_string());
    let mut ir = canonical.ir;
    ir.file_id.clone_from(&path_alias);
    let hierarchy = match crate::ir::hierarchical::try_ir_to_hierarchical(&ir) {
        Ok(hierarchy) => hierarchy,
        Err(error) => {
            send_response(&crate::mcp::tool_handlers::core::projection_error_response(
                id, &error,
            ));
            return;
        }
    };
    let compact = || {
        crate::mcp::tool_handlers::core::content::presentation_document(
            &ir,
            &hierarchy,
            canonical.fidelity,
            state,
        )
    };
    let source_matches =
        state.cache_read().compute_hash(source.as_bytes()) == canonical.source_hash;
    let tokenizer_kind = crate::mcp::tools::parse_tokenizer_arg(params, &state.config);
    let tokenizer_box = crate::tokenizer::create_tokenizer(tokenizer_kind).ok();
    let economic = crate::mcp::tool_handlers::core::content::economical_presentation_document(
        &ir,
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
    let (rendered, raw_passthrough) = if selected_raw && source_matches {
        (economic.text, true)
    } else {
        (compact(), false)
    };
    let published = publication.commit(
        // Compatibility validation above owns historical eligibility. The
        // ticket orders this replay operation, not the age of its snapshot.
        || Ok::<_, String>(true),
        || {
            state
                .ir_context_lock()
                .load_ir(ir.clone(), Some(canonical.source_hash));
            state.remember_persisted_path(&path_alias, file_path);
            state.remember_context_fidelity(&path_alias, canonical.fidelity);
            state.publish_compatible_semantic_projection(&path_alias, file_path, semantic);
            state
                .llm_text_cache_lock()
                .insert(path_alias, rendered.clone());
            Ok(())
        },
    );
    match published {
        Ok(Some(())) => {}
        Ok(None) => {
            return send_persistence_error(
                id,
                "Historical replay was superseded by a newer same-owner authority operation; retry the request",
            );
        }
        Err(error) => return send_persistence_error(id, &error),
    }
    let (content_kind, byte_exact) = contract_fields_for_hierarchy(canonical.fidelity, &hierarchy);
    send_response(&serde_json::json!({
        "jsonrpc": "2.0", "id": id,
        "result": {
            "content": [{ "type": "text", "text": rendered }],
            "ir": crate::ir::hierarchical::hierarchy_to_wire_reduced(&ir, &hierarchy),
            "_meta": {
                "file": file_path, "version": ir.version,
                "instruction_count": ir.instructions.len(),
                "content_kind": if raw_passthrough { ContentKind::RawPassthrough } else { content_kind },
                "byte_exact": if raw_passthrough {
                    serde_json::json!(["document"])
                } else {
                    serde_json::to_value(byte_exact).unwrap_or_default()
                }
            }
        }
    }));
}
