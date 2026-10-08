//! Online causal head/terminal tail selection. Identity is private, producer
//! semantic data, never sanitized display text and never serialized in results.
use super::super::model::{CargoDiagnostic, DiagnosticLevel, RetentionFacts};
use serde_json::{Map, Value};

pub(super) struct Candidate {
    pub diagnostic: CargoDiagnostic,
    pub identity: String,
}

pub(super) fn identity(message: &Map<String, Value>) -> String {
    let mut semantic = message.clone();
    semantic.remove("rendered");
    if let Some(children) = semantic.get_mut("children").and_then(Value::as_array_mut) {
        for child in children {
            if let Some(child) = child.as_object_mut() {
                child.remove("rendered");
            }
        }
    }
    Value::Object(semantic).to_string()
}

pub(super) fn observe_repeat(candidates: &mut [Candidate], identity: &str) -> bool {
    if let Some(candidate) = candidates.iter_mut().find(|item| item.identity == identity) {
        candidate.diagnostic.repeat_count += 1;
        true
    } else {
        false
    }
}

fn error(candidate: &Candidate) -> bool {
    candidate.diagnostic.level.is_error()
}

pub(super) fn admit(
    candidates: &mut Vec<Candidate>,
    candidate: Candidate,
    maximum: usize,
    facts: &mut RetentionFacts,
) {
    if maximum == 0 {
        return;
    }
    let is_error = error(&candidate);
    let count = candidates
        .iter()
        .filter(|item| error(item) == is_error)
        .count();
    if count == maximum {
        // Keep the first 3/4 stable and rotate the tail in producer order.
        let tail_start = maximum.saturating_mul(3) / 4;
        if let Some(index) = candidates
            .iter()
            .enumerate()
            .filter(|(_, item)| error(item) == is_error)
            .nth(tail_start)
            .map(|(index, _)| index)
        {
            candidates.remove(index);
        }
    }
    candidates.push(candidate);
    facts.peak_candidates = facts.peak_candidates.max(candidates.len());
}

pub(super) fn head_tail<T>(
    values: Vec<T>,
    maximum: usize,
    preferred_head: usize,
) -> (Vec<T>, usize, usize) {
    if values.len() <= maximum {
        let length = values.len();
        return (values, length, 0);
    }
    let head = preferred_head.min(maximum);
    let tail = maximum - head;
    let tail_start = values.len() - tail;
    (
        values
            .into_iter()
            .enumerate()
            .filter_map(|(index, value)| (index < head || index >= tail_start).then_some(value))
            .collect(),
        head,
        tail,
    )
}

pub(super) fn finish(
    candidates: Vec<Candidate>,
    maximum: usize,
    facts: &mut RetentionFacts,
) -> Vec<CargoDiagnostic> {
    let (errors, warnings): (Vec<_>, Vec<_>) = candidates.into_iter().partition(error);
    let mut selected = Vec::new();
    let mut represented_errors = 0;
    let mut represented_warnings = 0;
    for (pool, seen, represented) in [
        (errors, facts.errors_seen, &mut represented_errors),
        (warnings, facts.warnings_seen, &mut represented_warnings),
    ] {
        let remaining = maximum.saturating_sub(selected.len());
        let pool_length = pool.len();
        let occurrences: usize = pool.iter().map(|item| item.diagnostic.repeat_count).sum();
        let (pool, mut head, mut tail) =
            head_tail(pool, remaining, remaining.saturating_mul(3) / 4);
        // A full online pool can already have a gap even when final selection
        // consumes it whole; disclose the stable head and rotating tail then.
        if tail == 0 && seen > occurrences && pool_length == maximum && remaining >= maximum {
            head = maximum.saturating_mul(3) / 4;
            tail = maximum - head;
        }
        facts.head_retained += head;
        facts.tail_retained += tail;
        for (index, mut candidate) in pool.into_iter().enumerate() {
            candidate.diagnostic.selection = if index < head { "head" } else { "tail" };
            *represented += candidate.diagnostic.repeat_count;
            facts.exact_repeats_collapsed += candidate.diagnostic.repeat_count - 1;
            selected.push(candidate.diagnostic);
        }
    }
    facts.errors_retained = selected.iter().filter(|item| item.level.is_error()).count();
    facts.warnings_retained = selected
        .iter()
        .filter(|item| item.level == DiagnosticLevel::Warning)
        .count();
    facts.errors_omitted = facts.errors_seen.saturating_sub(represented_errors);
    facts.warnings_omitted = facts.warnings_seen.saturating_sub(represented_warnings);
    facts.other_omitted = facts.other_seen;
    facts.diagnostics_retained = selected.len();
    facts.diagnostics_omitted = facts.errors_omitted + facts.warnings_omitted + facts.other_omitted;
    facts.selection_policy = "errors_then_warnings_causal_head_terminal_tail_3_to_1";
    facts.repeat_scope = "exact_pre_redaction_semantic_records_in_retained_candidates";
    selected
}

#[cfg(test)]
#[path = "../../../tests/diagnostics/cargo_check_retention_accounting.rs"]
mod tests;
