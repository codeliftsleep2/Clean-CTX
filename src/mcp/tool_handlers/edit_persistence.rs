//! Durable edit intent and semantic commit boundary.
use crate::compression::Fidelity;
use crate::mcp::McpState;

pub(super) fn establish_edit_intent(
    state: &McpState,
    intent: &crate::mcp::sqlite_store::EditIntent,
    authority: &crate::mcp::semantic_compilation::retained::RetainedAuthority,
    compatibility: &crate::mcp::compatibility::identity::CompatibilityIdentities,
) -> Result<(), String> {
    let guard = state.persistence_store_lock();
    let Some(store) = guard.as_ref() else {
        return Ok(());
    };
    store
        .sqlite()
        .ok_or_else(|| "Persistence DB is unavailable".to_string())?
        .establish_retained_edit_intent(intent, authority, compatibility)
        .map_err(|error| format!("Cannot establish durable edit intent: {error}"))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn commit_edit_semantics(
    state: &McpState,
    file_path: &str,
    transition_id: &str,
    target_binary: &[u8],
    target_hash: &str,
    target_version: u64,
    semantic_edges: &[crate::layers::meta::semantic::SemanticEdge],
    semantic_dependencies: &[crate::layers::meta::semantic::SemanticDependency],
    authority: &crate::mcp::semantic_compilation::retained::RetainedAuthority,
    compatibility: &crate::mcp::compatibility::identity::CompatibilityIdentities,
) -> Result<(), String> {
    let guard = state.persistence_store_lock();
    let Some(store) = guard.as_ref() else {
        return Ok(());
    };
    let mut sqlite = store
        .sqlite()
        .ok_or_else(|| "Persistence DB is unavailable".to_string())?;
    sqlite
        .save_context_with_retained_semantics(
            file_path,
            Fidelity::Edit,
            "",
            target_binary,
            target_hash,
            target_version,
            semantic_edges,
            semantic_dependencies,
            authority,
            0,
            0,
            compatibility,
        )
        .map_err(|error| format!("Atomic durable semantic commit failed: {error}"))?;
    if let Err(error) = sqlite.clear_edit_intent(file_path, transition_id) {
        tracing::warn!(%error, %file_path, "committed edit intent retained for idempotent recovery");
    }
    Ok(())
}

pub(super) fn clear_edit_intent_best_effort(
    state: &McpState,
    file_path: &str,
    transition_id: &str,
) {
    let guard = state.persistence_store_lock();
    if let Some(store) = guard.as_ref()
        && let Some(mut sqlite) = store.sqlite()
    {
        let _ = sqlite.clear_edit_intent(file_path, transition_id);
    }
}
