// restore_context MCP handler.

use super::common::checked_hierarchy_or_respond;
use crate::mcp::McpState;
use crate::mcp::context_store::ContextStore;
use crate::mcp::tool_helpers::{
    compile_file_ir, inject_baseline_breakpoint, resolve_file_path_checked,
};
use crate::mcp::tools::parse_fidelity_arg;
use crate::protocol::send_response;
use serde_json::Value;
// ── Handler: restore_context ───────────────────────────────────────

pub(crate) fn handle_restore_context(id: &Value, params: &Value, state: &McpState) {
    let file_path_str = crate::mcp::tool_helpers::arg_str_or_empty(params, "filePath");
    if file_path_str.is_empty() {
        send_response(
            &serde_json::json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32602, "message": "Missing required parameter: filePath" } }),
        );
        return;
    }
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

    // A-13: Check resource limits before processing
    let limits = &state.config.resource_limits;

    // Check file size if we can read it
    if let Ok(metadata) = std::fs::metadata(&resolved_path) {
        if let Err(e) = limits.check_file_size(metadata.len()) {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "error": { "code": -32603, "message": e }
            }));
            return;
        }
    }

    let path_alias = state.get_or_create_alias(resolved_path.clone());
    state.ir_context_lock().remove_file(&path_alias);
    state.llm_text_cache_lock().remove(&path_alias);
    // Clear workspace index for this file (canonical identity, not alias).
    let canonical_path = crate::dictionary::path::canonical_identity_key(&resolved_path);
    state.workspace_index_lock().remove_file(&canonical_path);

    // Clear persistence DB entry for this file (use resolved_path, not alias)
    if let Some(ref mut store) = *state.persistence_store_lock() {
        store.clear_file(&resolved_path);
    }

    let source_arc = match state.read_source(&resolved_path) {
        Ok(s) => s,
        Err(e) => {
            send_response(
                &serde_json::json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32603, "message": format!("Cannot read file: {}", e) } }),
            );
            return;
        }
    };
    // The read doubles as an existence check before IR compilation;
    // the content itself is only consumed by the compiler via the
    // shared source cache (Phase A removed the legacy text consumer).
    let _source_text = source_arc.as_str();

    match compile_file_ir(&resolved_path, fidelity, state) {
        Ok((ir, _semantic_edges, _source_hash)) => {
            let hir = match checked_hierarchy_or_respond(id, &ir) {
                Some(hierarchy) => hierarchy,
                None => return,
            };
            let llm_text = crate::ir::render_hierarchical_for_llm(&hir, fidelity);
            let full = format!(
                "{}\n// ── {} ({}) ──\n{}",
                llm_text.trim(),
                ir.file_id,
                resolved_path,
                state.format_dict_footer_for_aliases(&[&ir.file_id]).trim()
            );
            state
                .llm_text_cache_lock()
                .insert(ir.file_id.clone(), full.clone());
            let mut response = serde_json::json!({ "jsonrpc": "2.0", "id": id, "result": { "content": [{ "type": "text", "text": full }], "_meta": { "version": ir.version, "restored": true } } });
            // Restored context is a stable full snapshot — inject baseline breakpoint.
            inject_baseline_breakpoint(&mut response, state, &full);
            send_response(&response);
        }
        Err(e) => {
            // Phase A retirement (2026-08-25): legacy `$`/`⊕`/`§` fallback
            // removed (see compress_code_context site).
            let reason = e.to_string();
            tracing::warn!(
                error = %reason,
                path = %resolved_path,
                "IR compilation failed in restore_context; returning structured ir_unavailable error"
            );
            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "error": {
                    "code": -32603,
                    "message": format!(
                        "IR compilation unavailable for {}: {}. SCHEMA v3 output \
                         cannot be produced for this input; retry with fidelity \
                         \"verbatim\" or read the source directly.",
                        resolved_path, reason
                    ),
                    "data": {
                        "reason": "ir_unavailable",
                        "path": resolved_path,
                        "ir_compiler": reason,
                    }
                }
            }));
        }
    }
}
