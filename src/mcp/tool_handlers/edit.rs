// src/mcp/tool_handlers/edit.rs
//
// `apply_edit` MCP handler — the write-path surface
// (docs/plans/APPLY_EDIT_PLAN.md Phase 3).
//
// Flow (plan Design steps 1–6):
//   1. Parse + validate params; resolve/exclude-check/size-check the file.
//   2. v1 policy: require prior tracked state (Open Question 2) so there is
//      always a "last known state" to verify against.
//   3. Build a fresh UnitTable by recompiling the CURRENT on-disk bytes at
//      Edit fidelity — this is plan step 2's unit-level relocation keyed on
//      qualified name + structural fingerprint. (The whole-file hash fast
//      path of step 1 is subsumed here: relocation against fresh spans is
//      strictly safer and costs one local parse, not client tokens.)
//   4. Verify expected-old-text per operation, splice, run the hard syntax
//      gate, then commit to disk under the module commit lock.
//   5. Refresh session state (hash registry, IR context baseline,
//      llm-text cache) so the next provide_code_context produces a delta.
//   6. Respond minimally: hash + per-op spans + byte deltas; echo new text
//      only when `verify: true`.
//
// Persistence note (Open Question 3): the SQLite baseline is deliberately
// NOT written here — it refreshes on the next provide_code_context call,
// matching the existing fire-and-forget pattern without a new design.

use std::sync::{Mutex, OnceLock};

use serde_json::Value;

use crate::compression::Fidelity;
use crate::edit::apply::{self, EditError};
use crate::edit::locate::UnitTable;
use crate::edit::ops::{EditOperation, MAX_OPERATIONS_PER_CALL};
use crate::mcp::McpState;
use crate::protocol::send_response;

use super::super::tool_helpers::{compile_file_ir_focused, resolve_file_path_checked};

/// Serializes apply_edit COMMIT critical sections (disk write + session
/// state refresh). The plan's "reuse the RwLock" idea deadlocks today:
/// `compile_file_ir_focused` internally takes an ir_context READ lock via
/// `state.file_version`, so a caller cannot hold that lock's WRITE guard
/// across compilation. This independent lock gives the same guarantee —
/// concurrent commits never interleave — without touching lock order.
fn commit_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn err_response(id: &Value, code: i64, message: String, data: Option<Value>) {
    send_response(&crate::mcp::tool_helpers::jsonrpc_error(
        id.clone(),
        code,
        message,
        data,
    ));
}

pub(crate) fn handle_apply_edit(id: &Value, params: &Value, state: &McpState) {
    let args = &params["arguments"];

    // ── Parameter parsing ────────────────────────────────────────────
    let Some(file_path_str) = args["filePath"].as_str().filter(|s| !s.is_empty()) else {
        return err_response(
            id,
            -32602,
            "Missing required parameter: filePath".to_string(),
            None,
        );
    };
    let Some(ops_json) = args["operations"].as_array() else {
        return err_response(
            id,
            -32602,
            "Missing required parameter: operations (array)".to_string(),
            None,
        );
    };
    if ops_json.is_empty() {
        return err_response(id, -32602, "operations must not be empty".to_string(), None);
    }
    if ops_json.len() > MAX_OPERATIONS_PER_CALL {
        return err_response(
            id,
            -32602,
            format!(
                "too many operations: {} (max {})",
                ops_json.len(),
                MAX_OPERATIONS_PER_CALL
            ),
            None,
        );
    }
    let mut operations: Vec<EditOperation> = Vec::with_capacity(ops_json.len());
    for (i, op_val) in ops_json.iter().enumerate() {
        match serde_json::from_value::<EditOperation>(op_val.clone()) {
            Ok(op) => operations.push(op),
            Err(e) => {
                return err_response(id, -32602, format!("operations[{}]: {}", i, e), None);
            }
        }
    }
    let verify = args["verify"].as_bool().unwrap_or(false);

    // ── Path resolution + policy gates ───────────────────────────────
    let workspace_root = args["workspaceRoot"].as_str();
    let resolved_path = match resolve_file_path_checked(
        file_path_str,
        workspace_root,
        &state.config.additional_roots,
    ) {
        Ok(p) => p,
        Err(msg) => return err_response(id, -32602, msg, None),
    };
    if state.config.is_excluded(&resolved_path) {
        return err_response(
            id,
            -32603,
            format!("File excluded by config: {}", file_path_str),
            None,
        );
    }
    if let Ok(metadata) = std::fs::metadata(&resolved_path)
        && let Err(e) = state.config.resource_limits.check_file_size(metadata.len())
    {
        return err_response(id, -32603, e, None);
    }

    let alias = state.get_or_create_alias(resolved_path.clone());
    // v1 policy (Open Question 2): no prior tracked state → refuse.
    if !state.ir_context_read().has_file(&alias) {
        let e = EditError::NoTrackedState(resolved_path.clone());
        return err_response(id, -32602, e.to_string(), Some(e.structured()));
    }

    // ── Unit relocation against CURRENT bytes (plan step 2/3) ────────
    let source_arc = match state.read_source(&resolved_path) {
        Ok(s) => s,
        Err(e) => return err_response(id, -32603, format!("Cannot read file: {}", e), None),
    };
    let extension = std::path::Path::new(&resolved_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let pre_compiled = match compile_file_ir_focused(&resolved_path, Fidelity::Edit, state, None) {
        Ok((ir, _)) => ir,
        Err(e) => return err_response(id, -32603, e.to_string(), None),
    };
    let units = UnitTable::from_instructions(&pre_compiled.instructions);
    if units.is_empty() {
        let e = EditError::Locate(crate::edit::locate::LocateError::NotFound(String::from(
            "no span-addressable units in current compile (file may have changed shape)",
        )));
        return err_response(id, -32602, e.to_string(), Some(e.structured()));
    }

    // ── Verify + splice + gate (all in memory) ───────────────────────
    let report = match apply::apply(&source_arc, &units, &operations) {
        Ok(r) => r,
        // All EditError variants are caller-state problems (bad params,
        // stale expectations, policy gates) → invalid-request code.
        Err(e) => return err_response(id, -32602, e.to_string(), Some(e.structured())),
    };
    if let Err(e) = apply::verify_syntax(&report.new_source, extension) {
        // Hard gate: nothing was written; report parse location.
        return err_response(id, -32602, e.to_string(), Some(e.structured()));
    }

    // ── Commit critical section ──────────────────────────────────────
    let new_hash = {
        let cache = state.cache_read();
        cache.compute_hash(report.new_source.as_bytes())
    };
    let _commit_guard = commit_lock().lock().unwrap_or_else(|p| p.into_inner());
    if let Err(e) = std::fs::write(&resolved_path, report.new_source.as_bytes()) {
        return err_response(id, -32603, format!("Write failed: {}", e), None);
    }
    state.invalidate_source_cache(&resolved_path);

    // Refresh session baseline so the next provide_code_context call on
    // this file yields an incremental delta instead of a full recompress
    // (plan step 5). Post-edit recompile also re-validates the final file.
    let version = match compile_file_ir_focused(&resolved_path, Fidelity::Edit, state, None) {
        // compile_file_ir_focused already assigns version = prev + 1.
        Ok((post, _)) => {
            let v = post.version;
            state
                .ir_context_lock()
                .load_ir(post, Some(new_hash.clone()));
            v
        }
        Err(e) => {
            tracing::warn!(error = %e, path = %resolved_path, "post-apply_edit recompile failed; session baseline left stale");
            state.file_version(&alias).unwrap_or(0)
        }
    };
    state
        .cache_write()
        .update_and_verify(&resolved_path, &new_hash);
    drop(_commit_guard);
    state.llm_text_cache_lock().remove(&alias);
    // ── Mark CBM project dirty (lazy reindex) ────────────────────
    if let Some(ref mut bridge) = *state.graph_bridge_lock() {
        if bridge.is_available() {
            bridge.mark_project_dirty(std::path::Path::new(&resolved_path));
            // No synchronous CBM reindex — the next graph query will
            // refresh automatically.
        }
    }

    // ── Minimal response (plan step 6) ───────────────────────────────
    let mut ops_report: Vec<Value> = report
        .operations
        .iter()
        .map(|o| {
            serde_json::json!({
                "kind": o.kind, "target": o.target,
                "startByte": o.start_byte, "endByte": o.end_byte,
                "byteDelta": o.byte_delta,
            })
        })
        .collect();
    if verify {
        for (entry, op) in ops_report.iter_mut().zip(operations.iter()) {
            if let EditOperation::ReplaceBody { new_text, .. }
            | EditOperation::InsertAfter {
                unit_text: new_text,
                ..
            }
            | EditOperation::InsertBefore {
                unit_text: new_text,
                ..
            } = op
            {
                entry["newText"] = Value::String(new_text.clone());
            }
        }
    }
    let summary = format!(
        "applied {} operation(s) to {} (v{})",
        report.operations.len(),
        resolved_path,
        version
    );
    send_response(&serde_json::json!({
        "jsonrpc": "2.0", "id": id, "result": {
            "content": [{ "type": "text", "text": summary }],
            "structuredContent": {
                "operations": ops_report,
            },
            "_meta": {
                "filePath": resolved_path,
                "fileHash": new_hash,
                "version": version,
                "applied": report.operations.len(),
                "syntaxGated": true,
            }
        }
    }));
}
