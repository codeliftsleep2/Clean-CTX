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
    let payload = crate::ir::compact_a::render_file_context(&normalized);
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

/// Select a correctness-complete focused representation without admitting the
/// full source document as a fallback. Full raw source is not semantically
/// equivalent to focused Edit because it exposes bodies outside the resolved
/// target set.
fn required_focused_content(
    raw_source: &str,
    candidate: String,
    tokenizer: Option<&dyn crate::tokenizer::Tokenizer>,
) -> crate::mcp::content_economics::EconomicContent {
    let raw_tokens = crate::mcp::tool_helpers::count_tokens_with_tokenizer(raw_source, tokenizer);
    let candidate_tokens =
        crate::mcp::tool_helpers::count_tokens_with_tokenizer(&candidate, tokenizer);
    crate::mcp::content_economics::EconomicContent {
        text: candidate,
        selected: crate::mcp::content_economics::SelectedRepresentation::Candidate,
        raw_tokens,
        candidate_tokens,
    }
}

pub(crate) fn select_complete_content(
    raw_source: &str,
    candidate: String,
    focused_edit: bool,
    tokenizer_kind: crate::tokenizer::TokenizerKind,
    tokenizer: Option<&dyn crate::tokenizer::Tokenizer>,
) -> crate::mcp::content_economics::EconomicContent {
    if focused_edit {
        required_focused_content(raw_source, candidate, tokenizer)
    } else {
        crate::mcp::content_economics::select_with_local_tokenizer(
            raw_source,
            candidate,
            tokenizer_kind,
            tokenizer,
        )
    }
}

pub(super) fn control_full_delta<T: Serialize>(
    file_id: &str,
    fidelity: Fidelity,
    delta: &T,
    _semantic_edges: &[SemanticEdge],
) -> String {
    let value = serde_json::json!({
        "schema": "clean-ctx/file-context-delta",
        "schema_version": 1,
        "file_id": file_id,
        "fidelity": fidelity,
        "delta": delta,
        "workspace_graph": "query workspace_query after apply when graph facts are required",
    });
    format!(
        "// FILE-CONTEXT-DELTA v1; apply to the acknowledged prior canonical state\n{}",
        serde_json::to_string_pretty(&value).expect("CONTROL-FULL delta is serializable")
    )
}
