use crate::diagnostics::cargo_check::{CargoCheckCompiler, CargoCheckPolicy};
use serde_json::{Value, json};

fn record(index: usize, level: &str) -> Value {
    json!({"reason":"compiler-message", "message":{
        "message":format!("{level} {index}"), "level":level, "spans":[]
    }})
}

fn observe(compiler: &mut CargoCheckCompiler, value: Value) {
    compiler.observe_stdout_frame(&serde_json::to_vec(&value).unwrap(), true, false);
}

#[test]
fn mixed_severity_flood_keeps_bounded_candidates_and_discloses_every_omission() {
    let mut compiler = CargoCheckCompiler::default();
    for index in 0..200 {
        for level in ["error", "warning", "note"] {
            observe(&mut compiler, record(index, level));
        }
    }
    let result = compiler.finish();
    let facts = result.retention;
    assert_eq!(facts.peak_candidates, 128);
    assert_eq!(facts.errors_retained, 64);
    assert_eq!(facts.warnings_retained, 0);
    assert_eq!(facts.errors_omitted, 136);
    assert_eq!(facts.warnings_omitted, 200);
    assert_eq!(facts.other_omitted, 200);
    assert_eq!(facts.head_retained, 48);
    assert_eq!(facts.tail_retained, 16);
    assert_eq!(result.diagnostics[48].message, "error 184");
    assert_eq!(
        facts.diagnostics_seen,
        facts.diagnostics_retained + facts.exact_repeats_collapsed + facts.diagnostics_omitted
    );
}

#[test]
fn warnings_fill_remaining_quota_with_their_own_causal_head_and_terminal_tail() {
    let mut compiler = CargoCheckCompiler::default();
    for index in 0..100 {
        observe(&mut compiler, record(index, "warning"));
    }
    for index in 0..10 {
        observe(&mut compiler, record(index, "error"));
    }
    let result = compiler.finish();
    assert_eq!(result.retention.errors_retained, 10);
    assert_eq!(result.retention.warnings_retained, 54);
    assert_eq!(result.retention.warnings_omitted, 46);
    assert_eq!(result.diagnostics[10].message, "warning 0");
    assert_eq!(result.diagnostics[49].message, "warning 39");
    assert_eq!(result.diagnostics[50].message, "warning 86");
    assert_eq!(result.diagnostics[63].message, "warning 99");
}

#[test]
fn evicted_repeat_occurrences_are_omissions_not_unrepresented_compaction() {
    let mut compiler = CargoCheckCompiler::default();
    for index in 0..100 {
        for _ in 0..if index == 60 { 10 } else { 1 } {
            observe(&mut compiler, record(index, "error"));
        }
    }
    observe(&mut compiler, record(0, "error"));
    let result = compiler.finish();
    assert_eq!(result.retention.diagnostics_seen, 110);
    assert_eq!(result.retention.exact_repeats_collapsed, 1);
    assert_eq!(result.retention.diagnostics_omitted, 45);
    assert_eq!(result.diagnostics[0].repeat_count, 2);
}

#[test]
fn rendering_changes_do_not_change_semantic_repeat_identity() {
    let mut compiler = CargoCheckCompiler::default();
    for rendering in ["first presentation", "second presentation"] {
        let mut value = record(0, "error");
        value["message"]["rendered"] = json!(rendering);
        observe(&mut compiler, value);
    }
    let result = compiler.finish();
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].repeat_count, 2);
}

#[test]
fn differing_child_semantics_never_collapse_as_exact_repeats() {
    let mut compiler = CargoCheckCompiler::default();
    for child in ["first fix", "second fix"] {
        let mut value = record(0, "error");
        value["message"]["children"] = json!([{"message":child,"level":"help"}]);
        observe(&mut compiler, value);
    }
    assert_eq!(compiler.finish().diagnostics.len(), 2);
}

#[test]
fn child_span_and_suggestion_omissions_include_unselected_children() {
    let mut value = record(0, "error");
    let children: Vec<_> = (0..10).map(|child| {
        let spans: Vec<_> = (0..12).map(|span| json!({
            "file_name":"src/main.rs", "is_primary":false,
            "line_start":child + 1, "line_end":child + 1,
            "column_start":span + 1, "column_end":span + 2,
            "suggested_replacement":format!("edit {child}/{span}"),
            "suggestion_applicability":if child == 9 && span == 7 { "MachineApplicable" } else { "MaybeIncorrect" }
        })).collect();
        json!({"message":format!("child {child}"), "level":"help", "spans":spans})
    }).collect();
    value["message"]["children"] = json!(children);
    let mut compiler = CargoCheckCompiler::default();
    observe(&mut compiler, value);
    let result = compiler.finish();
    let diagnostic = &result.diagnostics[0];
    assert_eq!(diagnostic.children_seen, 10);
    assert_eq!(diagnostic.children_head_retained, 6);
    assert_eq!(diagnostic.children_tail_retained, 2);
    assert_eq!(diagnostic.omitted_children, 2);
    assert_eq!(diagnostic.children[6].message, "child 8");
    assert!(
        diagnostic
            .children
            .iter()
            .all(|child| child.spans_seen == 12 && child.omitted_spans == 4)
    );
    assert_eq!(diagnostic.suggestions_seen, 120);
    assert_eq!(diagnostic.suggestions_retained, 4);
    assert_eq!(diagnostic.distinct_suggestions_retained, 4);
    assert_eq!(diagnostic.omitted_suggestions, 116);
    assert_eq!(diagnostic.machine_applicable_suggestions_seen, 1);
    assert_eq!(diagnostic.machine_applicable_suggestions_retained, 1);
    assert!(diagnostic.children[7].spans[7].suggestion.is_some());
    let retained_bytes: usize = diagnostic
        .children
        .iter()
        .flat_map(|child| &child.spans)
        .filter_map(|span| span.suggestion.as_ref())
        .map(|suggestion| suggestion.replacement.len())
        .sum();
    assert_eq!(
        diagnostic.suggestion_bytes_seen,
        diagnostic.omitted_suggestion_bytes + retained_bytes
    );
}

#[test]
fn duplicate_edits_share_a_distinct_suggestion_slot_without_hiding_occurrences() {
    let mut value = record(0, "error");
    let span = json!({"file_name":"src/main.rs","is_primary":true,"line_start":1,
        "line_end":1,"column_start":1,"column_end":2,"suggested_replacement":"same edit"});
    value["message"]["spans"] = json!(vec![span; 6]);
    let mut compiler = CargoCheckCompiler::default();
    observe(&mut compiler, value);
    let result = compiler.finish();
    let diagnostic = &result.diagnostics[0];
    assert_eq!(diagnostic.suggestions_seen, 6);
    assert_eq!(diagnostic.suggestions_retained, 4);
    assert_eq!(diagnostic.distinct_suggestions_retained, 1);
    assert_eq!(diagnostic.omitted_suggestions, 2);
    assert_eq!(diagnostic.omitted_suggestion_bytes, 18);
}

#[test]
fn zero_retention_policy_still_counts_producer_diagnostics() {
    let mut policy = CargoCheckPolicy::APPROVED;
    policy.maximum_diagnostics = 0;
    let mut compiler = CargoCheckCompiler::new(policy);
    observe(&mut compiler, record(0, "error"));
    let result = compiler.finish();
    assert!(result.diagnostics.is_empty());
    assert_eq!(result.retention.errors_seen, 1);
    assert_eq!(result.retention.errors_omitted, 1);
    assert_eq!(result.retention.peak_candidates, 0);
}
