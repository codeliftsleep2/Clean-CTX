//! Durable publication boundary for `provide_code_context` baselines.

use crate::ir::compiler::CompiledIR;
use crate::layers::meta::semantic::SemanticEdge;
use crate::mcp::McpState;

#[allow(clippy::too_many_arguments)]
pub(super) fn persist_edit_baseline(
    state: &McpState,
    resolved_path: &str,
    compiled: &CompiledIR,
    semantic_edges: &[SemanticEdge],
    source_hash: &str,
    raw_tokens: usize,
    compressed_tokens: usize,
) -> Result<(), &'static str> {
    let guard = state.persistence_store_lock();
    let Some(store) = guard.as_ref() else {
        return Ok(());
    };
    let durable_ir = crate::mcp::persistence_ir::baseline(compiled, resolved_path);
    let binary = crate::ir::binary_wire::encode(&durable_ir);
    let persisted = store.sqlite().is_some_and(|mut sqlite| {
        sqlite
            .save_context_with_semantics(
                resolved_path,
                crate::compression::Fidelity::Edit,
                "",
                &binary,
                source_hash,
                compiled.version,
                semantic_edges,
                raw_tokens as u64,
                compressed_tokens as u64,
            )
            .is_ok()
    });
    if persisted {
        Ok(())
    } else {
        Err("Canonical IR and semantic edges could not be persisted atomically")
    }
}
