// src/mcp/tool_handlers/edit.rs
//
// `apply_edit` MCP handler — the write-path surface
// (docs/plans/APPLY_EDIT_PLAN.md Phase 3).
//
use std::io::Write;
use std::sync::{Mutex, OnceLock};

use serde_json::Value;

use crate::compression::Fidelity;
use crate::edit::apply::{self, EditError};
use crate::edit::locate::UnitTable;
use crate::edit::ops::{EditOperation, MAX_OPERATIONS_PER_CALL};
use crate::mcp::McpState;
use crate::protocol::send_response;

use super::super::tool_helpers::{compile_source_ir_candidate, resolve_file_path_checked};

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

    let Some(alias) = state.alias_for_path(&resolved_path) else {
        let e = EditError::NoTrackedState(resolved_path.clone());
        return err_response(id, -32602, e.to_string(), Some(e.structured()));
    };
    // v1 policy (Open Question 2): no prior tracked state → refuse.
    if !state.ir_context_read().has_file(&alias) {
        let e = EditError::NoTrackedState(resolved_path.clone());
        return err_response(id, -32602, e.to_string(), Some(e.structured()));
    }

    // ── Unit relocation against CURRENT bytes (plan step 2/3) ────────
    match recover_pending_edit(state, &resolved_path) {
        Ok(crate::mcp::sqlite_store::EditRecovery::TargetCommitted) => {
            return err_response(
                id,
                -32603,
                "An interrupted edit was recovered durably; restore_context is required"
                    .to_string(),
                None,
            );
        }
        Ok(_) => {}
        Err(error) => return err_response(id, -32603, error, None),
    }
    let prior_bytes = match std::fs::read(&resolved_path) {
        Ok(bytes) => bytes,
        Err(e) => return err_response(id, -32603, format!("Cannot read file: {e}"), None),
    };
    let source = match std::str::from_utf8(&prior_bytes) {
        Ok(source) => source,
        Err(error) => {
            return err_response(
                id,
                -32603,
                format!("Source is not valid UTF-8: {error}"),
                None,
            );
        }
    };
    let actual_hash = hash_bytes(state, &prior_bytes);
    let (live_hash, prior_version) = {
        let context = state.ir_context_read();
        let Some(hash) = context.get_source_hash(&alias).cloned() else {
            return stale_source_response(id, None, None, &actual_hash);
        };
        let Some(version) = context.file_version(&alias) else {
            return stale_source_response(id, Some(&hash), None, &actual_hash);
        };
        (hash, version)
    };
    let durable_hash = match durable_source_hash(state, &resolved_path, prior_version) {
        Ok(hash) => hash,
        Err(error) => return err_response(id, -32603, error, None),
    };
    if actual_hash != live_hash
        || durable_hash
            .as_deref()
            .is_some_and(|hash| hash != actual_hash)
    {
        return stale_source_response(id, Some(&live_hash), durable_hash.as_deref(), &actual_hash);
    }
    let extension = std::path::Path::new(&resolved_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let pre_compiled =
        match compile_source_ir_candidate(&resolved_path, source, Fidelity::Edit, state) {
            Ok((ir, _, _)) => ir,
            Err(e) => return err_response(id, -32603, e.to_string(), None),
        };
    let units = if operations
        .iter()
        .any(|operation| matches!(operation, EditOperation::Delete { .. }))
    {
        let Some((language, query)) =
            crate::compression::language::language_for_extension(extension)
        else {
            let e = EditError::UnsupportedExtension(extension.to_string());
            return err_response(id, -32602, e.to_string(), Some(e.structured()));
        };
        match UnitTable::from_instructions_with_declarations(
            &pre_compiled.instructions,
            source,
            language,
            query,
        ) {
            Ok(table) => table,
            Err(error) => {
                return err_response(
                    id,
                    -32603,
                    format!("Cannot resolve declaration spans: {error}"),
                    None,
                );
            }
        }
    } else {
        UnitTable::from_instructions(&pre_compiled.instructions)
    };
    if units.is_empty() {
        let e = EditError::Locate(crate::edit::locate::LocateError::NotFound(String::from(
            "no span-addressable units in current compile (file may have changed shape)",
        )));
        return err_response(id, -32602, e.to_string(), Some(e.structured()));
    }

    // ── Verify + splice + gate (all in memory) ───────────────────────
    let report = match apply::apply(source, &units, &operations) {
        Ok(r) => r,
        // All EditError variants are caller-state problems (bad params,
        // stale expectations, policy gates) → invalid-request code.
        Err(e) => return err_response(id, -32602, e.to_string(), Some(e.structured())),
    };
    if let Err(e) = apply::verify_syntax(&report.new_source, extension) {
        // Hard gate: nothing was written; report parse location.
        return err_response(id, -32602, e.to_string(), Some(e.structured()));
    }

    // Compile and checked-project the exact in-memory candidate before any
    // source, durable, or live owner changes.
    let (mut target_ir, semantic_edges, new_hash) = match compile_source_ir_candidate(
        &resolved_path,
        &report.new_source,
        Fidelity::Edit,
        state,
    ) {
        Ok(candidate) => candidate,
        Err(error) => return err_response(id, -32603, error.to_string(), None),
    };
    if let Err(error) = crate::ir::hierarchical::try_ir_to_hierarchical(&target_ir) {
        return err_response(
            id,
            -32603,
            format!("Edited candidate projection failed: {error}"),
            None,
        );
    }
    let target_bytes = report.new_source.as_bytes();
    if new_hash != hash_bytes(state, target_bytes) {
        return err_response(
            id,
            -32603,
            "Edited candidate hash mismatch".to_string(),
            None,
        );
    }

    // ── Crash-recoverable commit critical section ────────────────────
    let _commit_guard = commit_lock().lock().unwrap_or_else(|p| p.into_inner());
    let staged = match stage_exact_bytes(&resolved_path, target_bytes) {
        Ok(staged) => staged,
        Err(error) => return err_response(id, -32603, error, None),
    };
    let stage_path = staged.path().to_string_lossy().into_owned();
    let transition_id = edit_transition_id(
        state,
        &resolved_path,
        &actual_hash,
        &new_hash,
        prior_version,
        target_ir.version,
    );
    let mut durable_ir = target_ir.clone();
    durable_ir.file_id.clone_from(&resolved_path);
    let target_binary = crate::ir::binary_wire::encode(&durable_ir);
    let intent = crate::mcp::sqlite_store::EditIntent {
        transition_id: transition_id.clone(),
        file_path: resolved_path.clone(),
        prior_hash: actual_hash.clone(),
        target_hash: new_hash.clone(),
        prior_version,
        target_version: target_ir.version,
        prior_source: prior_bytes.clone(),
        target_source: target_bytes.to_vec(),
        target_ir: target_binary.clone(),
        target_edges: semantic_edges.clone(),
        fidelity: Fidelity::Edit,
        stage_path,
    };
    if let Err(error) = establish_edit_intent(state, &intent) {
        return err_response(id, -32603, error, None);
    }
    if let Err(error) = staged.persist(&resolved_path) {
        clear_edit_intent_best_effort(state, &resolved_path, &transition_id);
        return err_response(
            id,
            -32603,
            format!("Atomic source replacement failed: {}", error.error),
            None,
        );
    }
    state.invalidate_source_cache(&resolved_path);
    if let Err(error) = commit_edit_semantics(
        state,
        &resolved_path,
        &transition_id,
        &target_binary,
        &new_hash,
        target_ir.version,
        &semantic_edges,
    ) {
        if let Err(recovery_error) = atomic_replace_exact(&resolved_path, &prior_bytes) {
            return err_response(
                id,
                -32603,
                format!(
                    "Durable edit commit failed ({error}); exact source recovery failed ({recovery_error}); recovery intent retained"
                ),
                None,
            );
        }
        state.invalidate_source_cache(&resolved_path);
        clear_edit_intent_best_effort(state, &resolved_path, &transition_id);
        return err_response(
            id,
            -32603,
            format!("Durable edit commit failed; exact prior source restored: {error}"),
            None,
        );
    }

    let version = target_ir.version;
    target_ir.file_id.clone_from(&alias);
    let canonical_path = crate::dictionary::path::canonical_identity_key(&resolved_path);
    state
        .ir_context_lock()
        .load_ir(target_ir, Some(new_hash.clone()));
    state.remember_context_fidelity(&alias, Fidelity::Edit);
    state.remember_semantic_edges(&alias, semantic_edges.clone());
    state.forget_pending_transitions(&alias);
    if state.persistence_store_lock().is_some() {
        state.remember_persisted_path(&alias, &resolved_path);
    }
    {
        let mut idx = state.workspace_index_lock();
        idx.remove_file(&canonical_path);
        idx.add_edges(&canonical_path, semantic_edges);
    }
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

    // ── Invalidate hydration discovery for the edited root ───────
    // The edit may have introduced or removed a declaration/consumer that
    // only a fresh discovery pass can see, so any hydration discovery already
    // completed for this root in the current generation is no longer valid.
    // Cheap and unconditional — the next hydration rediscovers.
    super::hydration::invalidate_discovery_for_edited_path(state, &resolved_path);

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

fn hash_bytes(state: &McpState, bytes: &[u8]) -> String {
    state.cache_read().compute_hash(bytes)
}

fn stale_source_response(
    id: &Value,
    live_hash: Option<&str>,
    durable_hash: Option<&str>,
    actual_hash: &str,
) {
    err_response(
        id,
        -32603,
        "Source identity does not match live and durable edit authority".to_string(),
        Some(serde_json::json!({
            "code": "stale_edit_source",
            "expectedLiveHash": live_hash,
            "expectedDurableHash": durable_hash,
            "actualSourceHash": actual_hash,
        })),
    );
}

fn durable_source_hash(
    state: &McpState,
    file_path: &str,
    live_version: u64,
) -> Result<Option<String>, String> {
    let guard = state.persistence_store_lock();
    let Some(store) = guard.as_ref() else {
        return Ok(None);
    };
    store.flush();
    let sqlite = store
        .sqlite()
        .ok_or_else(|| "Persistence DB is unavailable".to_string())?;
    let durable = sqlite
        .load_durable_context(file_path, None)
        .map_err(|error| format!("Durable edit precondition failed: {error}"))?
        .ok_or_else(|| "No durable context exists for the requested edit".to_string())?;
    if durable.ir.version != live_version {
        return Err(format!(
            "Durable edit version mismatch: live v{live_version}, durable v{}",
            durable.ir.version
        ));
    }
    Ok(Some(durable.source_hash))
}

pub(crate) fn recover_pending_edit(
    state: &McpState,
    file_path: &str,
) -> Result<crate::mcp::sqlite_store::EditRecovery, String> {
    let guard = state.persistence_store_lock();
    let Some(store) = guard.as_ref() else {
        return Ok(crate::mcp::sqlite_store::EditRecovery::None);
    };
    store.flush();
    let mut sqlite = store
        .sqlite()
        .ok_or_else(|| "Persistence DB is unavailable".to_string())?;
    sqlite
        .recover_edit_intent(file_path)
        .map_err(|error| format!("Edit recovery failed: {error}"))
}

fn stage_exact_bytes(file_path: &str, bytes: &[u8]) -> Result<tempfile::NamedTempFile, String> {
    let parent = std::path::Path::new(file_path)
        .parent()
        .ok_or_else(|| "Edited file has no parent directory".to_string())?;
    let mut staged = tempfile::NamedTempFile::new_in(parent)
        .map_err(|error| format!("Cannot create edit staging file: {error}"))?;
    staged
        .write_all(bytes)
        .and_then(|()| staged.flush())
        .and_then(|()| staged.as_file().sync_all())
        .map_err(|error| format!("Cannot stage exact edited bytes: {error}"))?;
    Ok(staged)
}

fn atomic_replace_exact(file_path: &str, bytes: &[u8]) -> Result<(), String> {
    stage_exact_bytes(file_path, bytes)?
        .persist(file_path)
        .map(|_| ())
        .map_err(|error| format!("Atomic exact-byte replacement failed: {}", error.error))
}

fn edit_transition_id(
    state: &McpState,
    file_path: &str,
    prior_hash: &str,
    target_hash: &str,
    prior_version: u64,
    target_version: u64,
) -> String {
    hash_bytes(
        state,
        format!("edit:v1:{file_path}:{prior_hash}:{target_hash}:{prior_version}:{target_version}")
            .as_bytes(),
    )
}

fn establish_edit_intent(
    state: &McpState,
    intent: &crate::mcp::sqlite_store::EditIntent,
) -> Result<(), String> {
    let guard = state.persistence_store_lock();
    let Some(store) = guard.as_ref() else {
        return Ok(());
    };
    store.flush();
    store
        .sqlite()
        .ok_or_else(|| "Persistence DB is unavailable".to_string())?
        .establish_edit_intent(intent)
        .map_err(|error| format!("Cannot establish durable edit intent: {error}"))
}

#[allow(clippy::too_many_arguments)]
fn commit_edit_semantics(
    state: &McpState,
    file_path: &str,
    transition_id: &str,
    target_binary: &[u8],
    target_hash: &str,
    target_version: u64,
    semantic_edges: &[crate::layers::meta::semantic::SemanticEdge],
) -> Result<(), String> {
    let guard = state.persistence_store_lock();
    let Some(store) = guard.as_ref() else {
        return Ok(());
    };
    store.flush();
    let mut sqlite = store
        .sqlite()
        .ok_or_else(|| "Persistence DB is unavailable".to_string())?;
    sqlite
        .save_context_with_semantics(
            file_path,
            Fidelity::Edit,
            "",
            target_binary,
            target_hash,
            target_version,
            semantic_edges,
            0,
            0,
        )
        .map_err(|error| format!("Atomic durable semantic commit failed: {error}"))?;
    if let Err(error) = sqlite.clear_edit_intent(file_path, transition_id) {
        tracing::warn!(%error, %file_path, "committed edit intent retained for idempotent recovery");
    }
    Ok(())
}

fn clear_edit_intent_best_effort(state: &McpState, file_path: &str, transition_id: &str) {
    let guard = state.persistence_store_lock();
    if let Some(store) = guard.as_ref()
        && let Some(mut sqlite) = store.sqlite()
    {
        let _ = sqlite.clear_edit_intent(file_path, transition_id);
    }
}

#[cfg(test)]
#[path = "../../tests/mcp/apply_edit_transaction.rs"]
mod transaction_tests;
