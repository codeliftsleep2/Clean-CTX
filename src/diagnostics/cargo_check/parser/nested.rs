//! Compile only approved nested selections; count omissions from the complete
//! validated record, including data belonging to omitted spans and children.
use super::super::model::{
    CargoDiagnostic, ChildDiagnostic, DiagnosticLevel, SanitizedSpan, Suggestion,
    TransformationFacts,
};
use super::super::policy::CargoCheckPolicy;
use super::super::sanitize::sanitize;
use super::retention::head_tail;
use serde_json::{Map, Value};

fn array<'a>(value: &'a Map<String, Value>, key: &str) -> &'a [Value] {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
}

fn replacement(span: &Value) -> Option<&str> {
    span.get("suggested_replacement").and_then(Value::as_str)
}

fn machine(span: &Value) -> bool {
    span.get("suggestion_applicability").and_then(Value::as_str) == Some("MachineApplicable")
}

fn suggestion_identity(span: &Value) -> String {
    Value::Array(
        [
            "file_name",
            "line_start",
            "line_end",
            "column_start",
            "column_end",
            "suggested_replacement",
            "suggestion_applicability",
        ]
        .iter()
        .map(|key| span.get(*key).cloned().unwrap_or(Value::Null))
        .collect(),
    )
    .to_string()
}

fn select_suggestions(spans: &[&Value], maximum: usize) -> Vec<String> {
    let mut selected = Vec::new();
    for priority in [true, false] {
        for span in spans
            .iter()
            .filter(|span| replacement(span).is_some() && machine(span) == priority)
        {
            if selected.len() == maximum {
                return selected;
            }
            let key = suggestion_identity(span);
            if !selected.contains(&key) {
                selected.push(key);
            }
        }
    }
    selected
}

pub(super) fn compile_diagnostic(
    message: &Map<String, Value>,
    level: DiagnosticLevel,
    text: &str,
    policy: CargoCheckPolicy,
    transformations: &mut TransformationFacts,
) -> CargoDiagnostic {
    let spans = array(message, "spans");
    let child_values = array(message, "children");
    let (child_indices, child_head, child_tail) = head_tail(
        (0..child_values.len()).collect(),
        policy.maximum_children,
        6,
    );
    let mut primary_seen = 0;
    let mut related_seen = 0;
    let mut kept_spans = Vec::new();
    for span in spans {
        let primary = span.get("is_primary").and_then(Value::as_bool) == Some(true);
        let (seen, maximum) = if primary {
            (&mut primary_seen, policy.maximum_primary_spans)
        } else {
            (&mut related_seen, policy.maximum_related_spans)
        };
        *seen += 1;
        if *seen <= maximum {
            kept_spans.push(span);
        }
    }
    let mut eligible = kept_spans.clone();
    for &index in &child_indices {
        eligible.extend(
            array(
                child_values[index].as_object().expect("validated child"),
                "spans",
            )
            .iter()
            .take(policy.maximum_related_spans),
        );
    }
    let selected_suggestions = select_suggestions(&eligible, policy.maximum_suggestions);
    let mut suggestions_seen = 0;
    let mut suggestion_bytes_seen = 0;
    let mut machine_seen = 0;
    let all_spans = spans.iter().chain(
        child_values
            .iter()
            .flat_map(|child| array(child.as_object().expect("validated child"), "spans").iter()),
    );
    for span in all_spans {
        if let Some(replacement) = replacement(span) {
            suggestions_seen += 1;
            suggestion_bytes_seen += replacement.len();
            machine_seen += usize::from(machine(span));
        }
    }
    let mut retained = 0;
    let mut retained_bytes = 0;
    let mut machine_retained = 0;
    let mut compile = |span: &Value| {
        let slot = replacement(span).and_then(|_| {
            selected_suggestions
                .iter()
                .position(|key| *key == suggestion_identity(span))
        });
        let allow = slot.is_some();
        if allow {
            retained += 1;
            retained_bytes += replacement(span).expect("selected replacement").len();
            machine_retained += usize::from(machine(span));
        }
        compile_span(span, slot, transformations)
    };
    let mut primary = Vec::new();
    let mut related = Vec::new();
    for span in kept_spans {
        let compiled = compile(span);
        if span.get("is_primary").and_then(Value::as_bool) == Some(true) {
            primary.push(compiled);
        } else {
            related.push(compiled);
        }
    }
    // Compile spans first while the shared suggestion quota is in scope.
    let child_spans: Vec<_> = child_indices
        .iter()
        .map(|&index| {
            array(
                child_values[index].as_object().expect("validated child"),
                "spans",
            )
            .iter()
            .take(policy.maximum_related_spans)
            .map(&mut compile)
            .collect::<Vec<_>>()
        })
        .collect();
    let children = child_indices
        .iter()
        .zip(child_spans)
        .map(|(&index, spans)| {
            let child = child_values[index].as_object().expect("validated child");
            let seen = array(child, "spans").len();
            ChildDiagnostic {
                level: DiagnosticLevel::from_producer(
                    child["level"].as_str().expect("validated level"),
                ),
                message: sanitize(
                    child["message"].as_str().expect("validated message"),
                    transformations,
                ),
                omitted_spans: seen.saturating_sub(spans.len()),
                spans_seen: seen,
                spans,
            }
        })
        .collect();
    CargoDiagnostic {
        selection: "unselected",
        level,
        message: sanitize(text, transformations),
        code: message
            .get("code")
            .and_then(|code| code.get("code"))
            .and_then(Value::as_str)
            .map(|code| sanitize(code, transformations)),
        omitted_primary_spans: primary_seen - primary.len(),
        omitted_related_spans: related_seen - related.len(),
        primary_spans: primary,
        related_spans: related,
        children,
        rendered_evidence: message
            .get("rendered")
            .and_then(Value::as_str)
            .map(|text| sanitize(text, transformations)),
        primary_spans_seen: primary_seen,
        related_spans_seen: related_seen,
        children_seen: child_values.len(),
        omitted_children: child_values.len() - child_indices.len(),
        children_head_retained: child_head,
        children_tail_retained: child_tail,
        suggestions_seen,
        suggestions_retained: retained,
        distinct_suggestions_retained: selected_suggestions.len(),
        omitted_suggestions: suggestions_seen - retained,
        suggestion_bytes_seen,
        omitted_suggestion_bytes: suggestion_bytes_seen - retained_bytes,
        machine_applicable_suggestions_seen: machine_seen,
        machine_applicable_suggestions_retained: machine_retained,
        repeat_count: 1,
    }
}

fn compile_span(
    value: &Value,
    slot: Option<usize>,
    facts: &mut TransformationFacts,
) -> SanitizedSpan {
    let number = |key| {
        value
            .get(key)
            .and_then(Value::as_u64)
            .expect("validated span coordinate")
    };
    SanitizedSpan {
        file: sanitize(
            value["file_name"].as_str().expect("validated span file"),
            facts,
        ),
        line_start: number("line_start"),
        line_end: number("line_end"),
        column_start: number("column_start"),
        column_end: number("column_end"),
        label: value
            .get("label")
            .and_then(Value::as_str)
            .map(|text| sanitize(text, facts)),
        suggestion: if let Some(retention_slot) = slot {
            replacement(value).map(|text| Suggestion {
                replacement: sanitize(text, facts),
                source_bytes: text.len(),
                retention_slot,
                applicability: value
                    .get("suggestion_applicability")
                    .and_then(Value::as_str)
                    .map(|text| sanitize(text, facts)),
            })
        } else {
            None
        },
    }
}
