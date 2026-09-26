//! Portable model-visible content assembly for core context handlers.

use crate::compression::Fidelity;
use crate::ir::{CompiledIR, HierarchicalIR};
use crate::mcp::McpState;

/// Render the model-visible presentation for a compiled file.
///
/// ARCH-003: `content` is a compact projection — class/interface/method names,
/// typed ownership, signatures, collapsed modifiers, extends/implements,
/// imports, type aliases, and (at Edit) exact bodies. The reversible codec
/// (CTX-001, `compact_a`/`compact_a3`) is code-side and must never appear in
/// the model-visible content.
pub(crate) fn presentation_document(
    ir: &CompiledIR,
    hierarchy: &HierarchicalIR,
    fidelity: Fidelity,
    state: &McpState,
) -> String {
    let llm_text = crate::ir::render_hierarchical_for_llm(hierarchy, fidelity);
    let footer = state.format_dict_footer_for_aliases(&[&ir.file_id]);
    format!("{}\n// {}\n{}", llm_text.trim(), ir.file_id, footer.trim())
}

pub(crate) fn economical_presentation_document(
    ir: &CompiledIR,
    hierarchy: &HierarchicalIR,
    fidelity: Fidelity,
    raw_source: &str,
    state: &McpState,
    tokenizer_kind: crate::tokenizer::TokenizerKind,
    tokenizer: Option<&dyn crate::tokenizer::Tokenizer>,
) -> crate::mcp::content_economics::EconomicContent {
    let candidate = presentation_document(ir, hierarchy, fidelity, state);
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

// `control_full_delta` was removed. Delta operations are returned only in the
// code-side `result.delta`; model-visible content is a minimal acknowledgement
// (or the explicitly classified raw-source economics fallback).

#[cfg(test)]
#[path = "../../../tests/mcp/presentation_footer.rs"]
mod presentation_footer_tests;

// Presentation boundary guard: the model-visible `content` must be a compact
// projection (SCHEMA v5 presentation), never the CONTROL-FULL codec — whose
// decoder contract (preamble, grammar legend, envelope schema id, body
// framing) is code-side machinery. These tests pin that boundary rather than
// any particular presentation shape.
#[cfg(all(test, feature = "typescript"))]
#[path = "../../../tests/mcp/presentation_boundary.rs"]
mod presentation_boundary_tests;
