//! Exact, token-bounded MCP-result bytes. No tool registration or automatic injection.
use super::{
    Error, Result, identity, optional,
    profile::Trusted,
    query::{self, Acquisition, Request, State},
    require,
};
use crate::tokenizer::{Tokenizer, TokenizerKind, create_tokenizer};
use std::path::Path;

mod payload;

/// Diagnostic measurements are outside the counted response, never inserted into it.
#[derive(Debug, Default)]
pub struct Counts {
    pub tokenization_calls: usize,
    pub tokenizer_input_bytes: usize,
    pub citation_encodings: usize,
}

/// M4 must emit these exact result bytes, without an extra response representation.
#[derive(Debug)]
pub struct Response {
    pub serialized_result: String,
    pub used_tokens: usize,
    pub counts: Counts,
}

/// Only exact local tokenizers are accepted. No approximate or raw-text fallback.
pub fn project(
    profile: &Path,
    request: &Request,
    tokenizer: &str,
    token_budget: u64,
) -> Result<Response> {
    project_with(profile, request, tokenizer, token_budget, &mut || {})
}

fn project_with(
    profile: &Path,
    request: &Request,
    tokenizer: &str,
    token_budget: u64,
    after_pack: &mut dyn FnMut(),
) -> Result<Response> {
    let kind = match tokenizer {
        "cl100k" => TokenizerKind::Cl100k,
        "o200k" => TokenizerKind::O200k,
        _ => return Err(Error("unsupported_tokenizer")),
    };
    let engine = create_tokenizer(kind).map_err(|_| Error("tokenizer_unavailable"))?;
    let trusted = Trusted::load(profile)?;
    require(
        token_budget > 0 && token_budget <= trusted.profile.limits.token_budget,
        "request_limits",
    )?;
    let mut meter = Meter {
        engine,
        limit: token_budget,
        name: tokenizer,
        counts: Counts::default(),
    };
    let mut draft = None;
    let mut packing_error = None;
    let result = query::acquire_with(
        profile,
        request,
        &mut |reader, witnesses, work, proof, acquisition| {
            // The profile selected by acquisition must be exactly the profile used for the budget.
            require(
                identity::hash(
                    &serde_json::to_value(&reader.trusted.profile)
                        .map_err(|_| Error("profile_changed"))?,
                ) == identity::hash(
                    &serde_json::to_value(&trusted.profile)
                        .map_err(|_| Error("profile_changed"))?,
                ),
                "profile_changed",
            )?;
            if acquisition.reasons.is_empty() {
                let support = optional::collect(reader, witnesses, work, proof)?;
                match payload::prepare(acquisition, request, &trusted, support, &mut meter) {
                    Ok(value) => draft = Some(value),
                    Err(error) => packing_error = Some(error),
                }
            }
            after_pack();
            Ok(())
        },
    )?;
    if result.state == State::Ready {
        if let Some(error) = packing_error {
            return Err(error);
        }
        let (serialized_result, used_tokens) = draft.ok_or(Error("projection_unavailable"))?;
        return Ok(Response {
            serialized_result,
            used_tokens,
            counts: meter.counts,
        });
    }
    // A failed COMPLETION invalidates the entire draft, including optional material.
    // Build and meter a safe control result; never return the earlier authority bytes.
    let (serialized_result, used_tokens) =
        payload::control(&result, request, &trusted, &mut meter, false)?;
    Ok(Response {
        serialized_result,
        used_tokens,
        counts: meter.counts,
    })
}

struct Meter<'a> {
    engine: Box<dyn Tokenizer>,
    limit: u64,
    name: &'a str,
    counts: Counts,
}
impl Meter<'_> {
    fn count(&mut self, serialized: &str) -> usize {
        self.counts.tokenization_calls += 1;
        self.counts.tokenizer_input_bytes += serialized.len();
        self.engine.count_tokens(serialized)
    }
    fn fits(&self, used: usize) -> bool {
        used as u64 <= self.limit
    }
}

#[cfg(test)]
#[path = "../tests/pcg/projection.rs"]
mod tests;
