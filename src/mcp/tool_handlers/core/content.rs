//! Portable model-visible content assembly for core context handlers.

use crate::compression::Fidelity;
use crate::ir::{CompiledIR, HierarchicalIR};
use crate::layers::meta::semantic::SemanticEdge;
use crate::mcp::McpState;
use serde::Serialize;

pub(crate) fn compact_a_document(
    ir: &CompiledIR,
    hierarchy: &HierarchicalIR,
    semantic_edges: &[SemanticEdge],
    fidelity: Fidelity,
    source_path: &str,
    state: &McpState,
) -> String {
    let normalized = crate::ir::normalize_control_full(
        &ir.file_id,
        source_path,
        ir.version,
        fidelity,
        hierarchy,
        semantic_edges,
    );
    let payload = crate::ir::compact_a::render(&normalized);
    let footer = state.format_dict_footer_for_aliases(&[&ir.file_id]);
    // Keep the body-frame terminator emitted by A1. The additional newline is
    // the document/footer boundary; trimming the payload would corrupt the
    // final exact body frame.
    format!("{}\n{}", payload, footer.trim())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn economical_compact_a_document(
    ir: &CompiledIR,
    hierarchy: &HierarchicalIR,
    semantic_edges: &[SemanticEdge],
    fidelity: Fidelity,
    source_path: &str,
    raw_source: &str,
    state: &McpState,
    tokenizer_kind: crate::tokenizer::TokenizerKind,
    tokenizer: Option<&dyn crate::tokenizer::Tokenizer>,
) -> crate::mcp::content_economics::EconomicContent {
    let candidate = compact_a_document(ir, hierarchy, semantic_edges, fidelity, source_path, state);
    crate::mcp::content_economics::select_with_local_tokenizer(
        raw_source,
        candidate,
        tokenizer_kind,
        tokenizer,
    )
}

pub(super) fn control_full_delta<T: Serialize>(
    file_id: &str,
    fidelity: Fidelity,
    delta: &T,
    semantic_edges: &[SemanticEdge],
) -> String {
    let value = serde_json::json!({
        "schema": "clean-ctx/control-full-delta",
        "schema_version": crate::ir::CONTROL_FULL_VERSION,
        "file_id": file_id,
        "fidelity": fidelity,
        "delta": delta,
        "semantic_edges_after_apply": semantic_edges,
        "navigation": {
            "schema": crate::ir::CONTROL_FULL_NAVIGATION_SCHEMA,
            "schema_version": crate::ir::CONTROL_FULL_NAVIGATION_VERSION,
            "occurrence_groups": [],
            "semantic_edge_provenance": crate::ir::semantic_edge_navigation(
                semantic_edges,
                "semantic_edges_after_apply",
            ),
        },
    });
    format!(
        "// CONTROL-FULL-DELTA v{}; apply to the acknowledged prior canonical state\n{}",
        crate::ir::CONTROL_FULL_VERSION,
        serde_json::to_string_pretty(&value).expect("CONTROL-FULL delta is serializable")
    )
}
