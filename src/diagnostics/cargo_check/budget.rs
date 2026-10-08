//! Serialized-byte enforcement, including escaping and disclosure metadata.
use super::CargoCheckPolicy;
use super::model::{CargoCheckSemanticResult, CausalPreview, DetailCounts, DiagnosticLevel};
use serde::Serialize;
use std::collections::BTreeSet;
use std::io::{self, Write};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum BoundedResultError {
    #[error("CargoCheck mandatory facts cannot fit the approved result budget")]
    MandatoryFacts,
    #[error("CargoCheck result serialization failed")]
    Serialization,
    #[error("CargoCheck mandatory text cannot fit the approved text budget")]
    MandatoryText,
}

#[derive(Default)]
struct Counter(usize);
impl Write for Counter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0 = self
            .0
            .checked_add(bytes.len())
            .ok_or_else(|| io::Error::other("result size overflow"))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub(crate) fn serialized_size(value: &impl Serialize) -> Result<usize, BoundedResultError> {
    let mut counter = Counter::default();
    serde_json::to_writer(&mut counter, value).map_err(|_| BoundedResultError::Serialization)?;
    Ok(counter.0)
}

pub(crate) fn prefix(text: &str, maximum: usize) -> &str {
    let mut end = text.len().min(maximum);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}

pub(crate) fn initialize(result: &mut CargoCheckSemanticResult, policy: CargoCheckPolicy) {
    let before = DetailCounts {
        diagnostics: result.diagnostics.len(),
        rendered: result
            .diagnostics
            .iter()
            .filter(|item| item.rendered_evidence.is_some())
            .count(),
        evidence_items: result.evidence.len(),
        evidence_bytes: result.evidence_facts.retained_bytes,
        children: result
            .diagnostics
            .iter()
            .map(|item| item.children.len())
            .sum(),
        related_spans: result
            .diagnostics
            .iter()
            .map(|item| item.related_spans.len())
            .sum(),
        suggestions: result
            .diagnostics
            .iter()
            .map(|item| item.suggestions_retained)
            .sum(),
    };
    result.result_budget.limit_bytes = policy.structured_content_bytes;
    result.result_budget.before = before;
    result.result_budget.causal_preview = result.diagnostics.first().map(|item| CausalPreview {
        level: item.level,
        message: prefix(&item.message, policy.content_bytes).to_owned(),
        source_sanitized_bytes: item.message.len(),
        truncated: item.message.len() > policy.content_bytes,
    });
    // Pre-reduction size includes its own disclosure, measured to a fixed point.
    while let Ok(size) = serialized_size(result) {
        if size == result.result_budget.pre_reduction_bytes {
            break;
        }
        result.result_budget.pre_reduction_bytes = size;
    }
}

pub(crate) fn enforce(
    result: &mut CargoCheckSemanticResult,
    limit: usize,
    measure: impl Fn(&CargoCheckSemanticResult) -> Result<usize, BoundedResultError>,
) -> Result<(), BoundedResultError> {
    result.result_budget.limit_bytes = limit;
    result.result_budget.mandatory_facts_exceed_budget = false;
    loop {
        // Count the complete result, including this size field, to a fixed point.
        let size = measure(result)?;
        if result.result_budget.serialized_bytes != size {
            result.result_budget.serialized_bytes = size;
            continue;
        }
        if size <= limit {
            result.result_budget.mandatory_facts_exceed_budget = false;
            return Ok(());
        }
        result.result_budget.activated = true;
        if !reduce(result) {
            result.result_budget.mandatory_facts_exceed_budget = true;
            return Err(BoundedResultError::MandatoryFacts);
        }
    }
}

fn record_reduction(result: &mut CargoCheckSemanticResult, field: &'static str) {
    if !result.result_budget.fields_reduced.contains(&field) {
        result.result_budget.fields_reduced.push(field);
    }
}

fn reduce(result: &mut CargoCheckSemanticResult) -> bool {
    if result
        .diagnostics
        .iter()
        .any(|item| item.rendered_evidence.is_some())
    {
        for item in &mut result.diagnostics {
            item.rendered_evidence = None;
        }
        record_reduction(result, "rendered_diagnostics");
    } else if !result.evidence.is_empty() {
        result.evidence.clear();
        result.evidence_facts.retained_bytes = 0;
        for facts in result.evidence_facts.categories.values_mut() {
            facts.retained_records = 0;
            facts.retained_bytes = 0;
            facts.omitted_records = facts
                .original_records
                .saturating_sub(facts.sanitized_empty_records);
            facts.omitted_sanitized_bytes = facts.sanitized_bytes;
            facts.omitted_bytes = facts
                .withheld_source_bytes
                .saturating_add(facts.omitted_sanitized_bytes);
            facts.partially_retained_records = 0;
            facts.head_bytes = 0;
            facts.tail_bytes = 0;
            facts.exemplar_bytes = 0;
            facts.limit_activated |= facts.omitted_records != 0;
        }
        record_reduction(result, "evidence");
    } else if result
        .diagnostics
        .iter()
        .any(|item| !item.children.is_empty())
    {
        for item in &mut result.diagnostics {
            for child in &item.children {
                item.omitted_suggestion_bytes += child
                    .spans
                    .iter()
                    .filter_map(|span| span.suggestion.as_ref())
                    .map(|suggestion| suggestion.source_bytes)
                    .sum::<usize>();
            }
            item.children.clear();
            item.omitted_children = item.children_seen;
            item.children_head_retained = 0;
            item.children_tail_retained = 0;
            refresh_suggestions(item);
        }
        record_reduction(result, "children");
    } else if result
        .diagnostics
        .iter()
        .any(|item| !item.related_spans.is_empty())
    {
        for item in &mut result.diagnostics {
            item.omitted_suggestion_bytes += item
                .related_spans
                .iter()
                .filter_map(|span| span.suggestion.as_ref())
                .map(|suggestion| suggestion.source_bytes)
                .sum::<usize>();
            item.related_spans.clear();
            item.omitted_related_spans = item.related_spans_seen;
            refresh_suggestions(item);
        }
        record_reduction(result, "related_spans");
    } else if result
        .diagnostics
        .iter()
        .any(|item| item.suggestions_retained != 0)
    {
        for item in &mut result.diagnostics {
            for span in &mut item.primary_spans {
                if let Some(suggestion) = span.suggestion.take() {
                    item.omitted_suggestion_bytes += suggestion.source_bytes;
                }
            }
            refresh_suggestions(item);
        }
        record_reduction(result, "suggestions");
    } else if !result.diagnostics.is_empty() {
        let index = result
            .diagnostics
            .iter()
            .rposition(|item| !item.level.is_error())
            .unwrap_or_else(|| {
                if result.diagnostics.len() > 2 {
                    result.diagnostics.len() - 2
                } else {
                    result.diagnostics.len() - 1
                }
            });
        let removed = result.diagnostics.remove(index);
        let facts = &mut result.retention;
        facts.diagnostics_retained -= 1;
        facts.diagnostics_omitted += removed.repeat_count;
        facts.exact_repeats_collapsed -= removed.repeat_count - 1;
        if removed.selection == "head" {
            facts.head_retained -= 1;
        } else if removed.selection == "tail" {
            facts.tail_retained -= 1;
        }
        if removed.level.is_error() {
            facts.errors_retained -= 1;
            facts.errors_omitted += removed.repeat_count;
        } else if removed.level == DiagnosticLevel::Warning {
            facts.warnings_retained -= 1;
            facts.warnings_omitted += removed.repeat_count;
        }
        record_reduction(result, "lower_priority_diagnostics");
    } else {
        return false;
    }
    true
}

fn refresh_suggestions(item: &mut super::model::CargoDiagnostic) {
    let suggestions: Vec<_> = item
        .primary_spans
        .iter()
        .chain(&item.related_spans)
        .chain(item.children.iter().flat_map(|child| &child.spans))
        .filter_map(|span| span.suggestion.as_ref())
        .collect();
    item.suggestions_retained = suggestions.len();
    item.distinct_suggestions_retained = suggestions
        .iter()
        .map(|suggestion| suggestion.retention_slot)
        .collect::<BTreeSet<_>>()
        .len();
    item.machine_applicable_suggestions_retained = suggestions
        .iter()
        .filter(|suggestion| suggestion.applicability.as_deref() == Some("MachineApplicable"))
        .count();
    item.omitted_suggestions = item.suggestions_seen - item.suggestions_retained;
}
