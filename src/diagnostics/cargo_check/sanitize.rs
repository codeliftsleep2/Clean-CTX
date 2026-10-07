use super::model::TransformationFacts;
use crate::native_text::{ansi::normalize_terminal_text, redaction::redact_recognized_secrets};

pub(crate) fn sanitize(value: &str, facts: &mut TransformationFacts) -> String {
    let normalized = normalize_terminal_text(value);
    facts.normalization.sequences_removed += normalized.facts.sequences_removed;
    facts.normalization.bytes_removed += normalized.facts.bytes_removed;

    let redacted = redact_recognized_secrets(&normalized.text);
    for (class, count) in redacted.facts.counts {
        *facts.redaction.counts.entry(class).or_insert(0) += count;
    }
    redacted.text
}
