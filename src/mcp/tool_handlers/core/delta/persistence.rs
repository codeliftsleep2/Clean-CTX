//! Durable baseline ownership used by delta generation and application.

use super::compiled_from_tuples;
use crate::ir::compiler::CompiledIR;
use crate::layers::meta::semantic::SemanticEdge;
use crate::mcp::McpState;

pub(super) fn persist_baseline(
    state: &McpState,
    file_path: &str,
    fidelity: crate::compression::Fidelity,
    compiled: &CompiledIR,
    source_hash: &str,
    semantic_edges: &[SemanticEdge],
) -> Result<(), String> {
    let durable = crate::mcp::persistence_ir::baseline(compiled, file_path);
    let binary = crate::ir::binary_wire::encode(&durable);
    if let Some(ref store) = *state.persistence_store_lock() {
        let persisted = store.sqlite().is_some_and(|mut sqlite| {
            sqlite
                .save_context_with_semantics(
                    file_path,
                    fidelity,
                    "",
                    &binary,
                    source_hash,
                    compiled.version,
                    semantic_edges,
                    0,
                    0,
                )
                .is_ok()
        });
        if !persisted {
            return Err(
                "canonical baseline and semantic edges were not persisted atomically".to_string(),
            );
        }
    }
    Ok(())
}

pub(super) fn ensure_persisted_baseline(
    state: &McpState,
    file_path: &str,
    fidelity: crate::compression::Fidelity,
    compiled: &CompiledIR,
    source_hash: &str,
) -> Result<(), String> {
    let missing = {
        let guard = state.persistence_store_lock();
        let Some(store) = guard.as_ref() else {
            return Ok(());
        };
        let sqlite = store
            .sqlite()
            .ok_or_else(|| "Persistence DB is unavailable".to_string())?;
        if sqlite
            .baseline_binary(file_path)
            .map_err(|error| format!("Persistence lookup failed: {error}"))?
            .is_none()
        {
            true
        } else {
            let durable = sqlite
                .load_durable_context(file_path, None)
                .map_err(|error| format!("Invalid durable baseline: {error}"))?
                .ok_or_else(|| "Missing durable baseline owner".to_string())?;
            if durable.ir.file_id != file_path
                || durable.ir.version != compiled.version
                || durable.source_hash != source_hash
            {
                return Err("Durable baseline does not match session canonical state".to_string());
            }
            false
        }
    };
    if missing {
        let semantic_edges = state
            .semantic_edges(&compiled.file_id)
            .ok_or_else(|| "missing authoritative semantic-edge baseline".to_string())?;
        persist_baseline(
            state,
            file_path,
            fidelity,
            compiled,
            source_hash,
            &semantic_edges,
        )?;
        state.remember_persisted_path(&compiled.file_id, file_path);
    }
    Ok(())
}

pub(crate) fn ensure_apply_baseline(
    state: &McpState,
    alias: &str,
    file_path: &str,
    fidelity: crate::compression::Fidelity,
) -> Result<(), String> {
    if state.persistence_store_lock().is_none() {
        return Ok(());
    }
    let ir_ctx = state.ir_context_read();
    let instructions = ir_ctx
        .get_ir(alias)
        .cloned()
        .ok_or_else(|| "missing canonical baseline for delta application".to_string())?;
    let version = ir_ctx
        .file_version(alias)
        .ok_or_else(|| "missing canonical baseline version".to_string())?;
    let source_hash = ir_ctx
        .get_source_hash(alias)
        .cloned()
        .ok_or_else(|| "missing canonical baseline source hash".to_string())?;
    drop(ir_ctx);
    let compiled = compiled_from_tuples(alias.to_string(), version, instructions)?;
    ensure_persisted_baseline(state, file_path, fidelity, &compiled, &source_hash)
}

pub(crate) fn persisted_context_id(
    state: &McpState,
    file_path: &str,
) -> Result<Option<String>, String> {
    let guard = state.persistence_store_lock();
    let Some(store) = guard.as_ref() else {
        return Ok(None);
    };
    let sqlite = store
        .sqlite()
        .ok_or_else(|| "Persistence DB is unavailable".to_string())?;
    sqlite
        .current_context_id(file_path)
        .map_err(|error| format!("Persistence lookup failed: {error}"))?
        .map(Some)
        .ok_or_else(|| format!("No persisted baseline for delta file: {file_path}"))
}
