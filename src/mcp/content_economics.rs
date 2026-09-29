//! Final model-visible content economics.
//!
//! This boundary is deliberately independent of encoding strategy. Callers
//! render a complete candidate, count it and the byte-exact raw source with the
//! same requested tokenizer, then use this selector. Raw wins ties so Clean-CTX
//! never adds model-input cost merely to expose derived navigation. Approximate
//! model tokenizers must also clear an uncertainty margin; failure to create a
//! local tokenizer fails safe to raw passthrough.

use crate::tokenizer::{Tokenizer, TokenizerKind};

const CLAUDE_ERROR_BASIS_POINTS: usize = 200;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SelectedRepresentation {
    Candidate,
    RawPassthrough,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EconomicContent {
    pub text: String,
    pub selected: SelectedRepresentation,
    pub raw_tokens: usize,
    pub candidate_tokens: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CountConfidence {
    ExactLocal,
    Approximate { error_basis_points: usize },
}

impl EconomicContent {
    pub(crate) fn selected_tokens(&self) -> usize {
        match self.selected {
            SelectedRepresentation::Candidate => self.candidate_tokens,
            SelectedRepresentation::RawPassthrough => self.raw_tokens,
        }
    }

    pub(crate) fn saved_tokens(&self) -> usize {
        self.raw_tokens.saturating_sub(self.selected_tokens())
    }
}

/// Apply the production token gate using only the already-created local
/// tokenizer. No model, network, or token-counting API call is permitted here.
///
/// Claude uses the repository's calibrated local cl100k approximation. Its
/// documented error is below two percent on typical code, so the comparison
/// reserves that uncertainty on both sides. Exact cl100k/o200k counters need
/// only show a strict saving. Tokenizers without a bounded local error model,
/// and tokenizer initialization failures, fail safe to byte-exact raw source.
pub(crate) fn select_with_local_tokenizer(
    raw: &str,
    candidate: String,
    tokenizer_kind: TokenizerKind,
    tokenizer: Option<&dyn Tokenizer>,
) -> EconomicContent {
    let raw_tokens = tokenizer.map_or_else(
        || crate::mcp::tool_helpers::estimate_tokens(raw),
        |tokenizer| tokenizer.count_tokens(raw),
    );
    select_with_local_tokenizer_and_raw_tokens(
        raw,
        candidate,
        tokenizer_kind,
        tokenizer,
        raw_tokens,
    )
}

/// Apply the production token gate with an already-counted raw document.
///
/// The raw count is content-addressed by the caller. Only the candidate still
/// requires tokenization, avoiding a second full-file encode in two-stage
/// economics paths.
pub(crate) fn select_with_local_tokenizer_and_raw_tokens(
    raw: &str,
    candidate: String,
    tokenizer_kind: TokenizerKind,
    tokenizer: Option<&dyn Tokenizer>,
    raw_tokens: usize,
) -> EconomicContent {
    let Some(tokenizer) = tokenizer else {
        return raw_passthrough_with_counts(
            raw,
            raw_tokens,
            crate::mcp::tool_helpers::estimate_tokens(&candidate),
        );
    };
    let confidence = match tokenizer_kind {
        TokenizerKind::Cl100k | TokenizerKind::O200k => CountConfidence::ExactLocal,
        TokenizerKind::Claude => CountConfidence::Approximate {
            error_basis_points: CLAUDE_ERROR_BASIS_POINTS,
        },
        TokenizerKind::Llama3 => {
            return raw_passthrough_with_counts(
                raw,
                raw_tokens,
                tokenizer.count_tokens(&candidate),
            );
        }
    };
    let candidate_tokens = tokenizer.count_tokens(&candidate);
    if candidate_is_economical(raw_tokens, candidate_tokens, confidence) {
        EconomicContent {
            text: candidate,
            selected: SelectedRepresentation::Candidate,
            raw_tokens,
            candidate_tokens,
        }
    } else {
        raw_passthrough_with_counts(raw, raw_tokens, candidate_tokens)
    }
}

fn candidate_is_economical(
    raw_tokens: usize,
    candidate_tokens: usize,
    confidence: CountConfidence,
) -> bool {
    match confidence {
        CountConfidence::ExactLocal => candidate_tokens < raw_tokens,
        CountConfidence::Approximate { error_basis_points } => {
            let scale = 10_000_u128;
            let error = error_basis_points.min(9_999) as u128;
            (candidate_tokens as u128) * (scale + error) < (raw_tokens as u128) * (scale - error)
        }
    }
}

fn raw_passthrough_with_counts(
    raw: &str,
    raw_tokens: usize,
    candidate_tokens: usize,
) -> EconomicContent {
    EconomicContent {
        text: raw.to_owned(),
        selected: SelectedRepresentation::RawPassthrough,
        raw_tokens,
        candidate_tokens,
    }
}

#[cfg(test)]
#[path = "../tests/mcp/content_economics.rs"]
mod tests;
