//! Durable publication boundary for `provide_code_context` baselines.

use crate::ir::compiler::CompiledIR;
use crate::layers::meta::semantic::SemanticEdge;
use crate::mcp::McpState;

pub(super) fn read_checkpoint_required(
    state: &McpState,
    fidelity: crate::compression::Fidelity,
) -> bool {
    state.config.persistence.auto_save || fidelity == crate::compression::Fidelity::Edit
}

#[allow(clippy::too_many_arguments)]
pub(super) fn persist_read_baseline(
    state: &McpState,
    owner_path: &str,
    fidelity: crate::compression::Fidelity,
    compiled: &CompiledIR,
    semantic_edges: &[SemanticEdge],
    source_hash: &str,
    source: &str,
    raw_tokens: usize,
    compressed_tokens: usize,
) -> Result<(), &'static str> {
    if !read_checkpoint_required(state, fidelity) {
        return Ok(());
    }
    let guard = state.persistence_store_lock();
    let Some(store) = guard.as_ref() else {
        return Ok(());
    };
    let durable_ir = crate::mcp::persistence_ir::baseline(compiled, owner_path);
    let binary = crate::ir::binary_wire::encode(&durable_ir);
    let identities = crate::mcp::compatibility::derive_identities(
        source,
        std::path::Path::new(owner_path),
        &state.config,
    )
    .map_err(|_| "Durable compatibility identity derivation failed")?;
    let persisted = store.sqlite().is_some_and(|mut sqlite| {
        sqlite
            .save_context_with_compatibility(
                owner_path,
                fidelity,
                "",
                &binary,
                source_hash,
                compiled.version,
                semantic_edges,
                raw_tokens as u64,
                compressed_tokens as u64,
                &identities,
            )
            .is_ok()
    });
    if persisted {
        Ok(())
    } else {
        Err("Canonical IR and semantic edges could not be persisted atomically")
    }
}
