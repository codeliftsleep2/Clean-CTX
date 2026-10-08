use super::CargoCheckCompiler;
use serde_json::{Value, json};

fn diagnostic(index: usize) -> Value {
    json!({"reason":"compiler-message","message":{
    "message":format!("error {index}"),"level":"error","spans":[{
        "file_name":"src/main.rs","is_primary":true,
        "line_start":1,"line_end":1,"column_start":1,"column_end":2
    }]}})
}

fn observe(compiler: &mut CargoCheckCompiler, value: Value) {
    compiler.observe_stdout_frame(&serde_json::to_vec(&value).unwrap(), true, false);
}

#[test]
fn retention_regression_columns_distinguish_semantic_locations() {
    let mut compiler = CargoCheckCompiler::default();
    let first = diagnostic(0);
    let mut second = first.clone();
    second["message"]["spans"][0]["column_start"] = json!(5);
    observe(&mut compiler, first);
    observe(&mut compiler, second);
    assert_eq!(compiler.finish().diagnostics.len(), 2);
}

#[test]
fn retention_regression_redaction_cannot_make_distinct_diagnostics_exact_repeats() {
    let mut compiler = CargoCheckCompiler::default();
    for secret in ["firstsecret123456", "secondsecret123456"] {
        let mut value = diagnostic(0);
        value["message"]["message"] = json!(format!("token={secret}"));
        observe(&mut compiler, value);
    }
    assert_eq!(compiler.finish().diagnostics.len(), 2);
}

#[test]
fn retention_regression_omitted_spans_still_disclose_omitted_suggestions() {
    let mut compiler = CargoCheckCompiler::default();
    let mut value = diagnostic(0);
    let mut span = value["message"]["spans"][0].clone();
    span["suggested_replacement"] = json!("replacement");
    value["message"]["spans"] = json!(vec![span; 6]);
    observe(&mut compiler, value);
    let result = compiler.finish();
    assert_eq!(result.diagnostics[0].omitted_primary_spans, 2);
    assert_eq!(result.diagnostics[0].omitted_suggestions, 2);
}

#[test]
fn retention_regression_candidate_storage_stays_bounded_before_finish() {
    let mut compiler = CargoCheckCompiler::default();
    for index in 0..1000 {
        observe(&mut compiler, diagnostic(index));
    }
    assert!(compiler.diagnostics.len() <= 3 * 64);
}

#[test]
fn retention_regression_machine_applicable_suggestion_precedes_earlier_alternatives() {
    let mut compiler = CargoCheckCompiler::default();
    let mut value = diagnostic(0);
    let spans: Vec<_> = (0..5)
        .map(|index| {
            let mut span = value["message"]["spans"][0].clone();
            span["is_primary"] = json!(false);
            span["suggested_replacement"] = json!(format!("replacement {index}"));
            span["suggestion_applicability"] = json!(if index == 4 {
                "MachineApplicable"
            } else {
                "MaybeIncorrect"
            });
            span
        })
        .collect();
    value["message"]["spans"] = json!(spans);
    observe(&mut compiler, value);
    let result = compiler.finish();
    assert!(result.diagnostics[0].related_spans[4].suggestion.is_some());
    assert!(result.diagnostics[0].related_spans[3].suggestion.is_none());
}
