use crate::diagnostics::cargo_check::{
    CargoCheckCompiler, CargoCheckPolicy, DiagnosticLevel, EvidenceCategory,
};
use serde_json::{Value, json};

fn compiler_message(index: usize, level: &str) -> Value {
    json!({
        "reason": "compiler-message",
        "message": {
            "message": format!("diagnostic {index}"),
            "code": { "code": format!("E{index:04}") },
            "level": level,
            "spans": [{
                "file_name": format!("src/file_{index}.rs"),
                "line_start": index + 1,
                "line_end": index + 1,
                "column_start": 2,
                "column_end": 8,
                "is_primary": true,
                "label": "primary",
                "suggested_replacement": "replacement",
                "suggestion_applicability": "MachineApplicable"
            }],
            "children": [{
                "message": "try this",
                "level": "help",
                "spans": []
            }],
            "rendered": format!("error: diagnostic {index}")
        }
    })
}

fn observe_json(compiler: &mut CargoCheckCompiler, value: Value) {
    let line = serde_json::to_vec(&value).expect("fixture serializes");
    compiler.observe_stdout_frame(&line, true, false);
}

#[test]
fn approved_policy_matches_owner_decision() {
    let policy = CargoCheckPolicy::APPROVED;
    assert_eq!(policy.stdout_capture_bytes, 8 * 1024 * 1024);
    assert_eq!(policy.stderr_capture_bytes, 4 * 1024 * 1024);
    assert_eq!(policy.aggregate_capture_bytes, 12 * 1024 * 1024);
    assert_eq!(policy.maximum_frame_bytes, 2 * 1024 * 1024);
    assert_eq!(policy.maximum_diagnostics, 64);
    assert_eq!(policy.evidence_bytes, 96 * 1024);
    assert_eq!(policy.structured_content_bytes, 512 * 1024);
    assert_eq!(policy.content_bytes, 24 * 1024);
    assert_eq!(policy.content_lines, 240);
}

#[test]
fn recognized_records_contribute_separate_facts() {
    let mut compiler = CargoCheckCompiler::default();
    observe_json(&mut compiler, compiler_message(7, "error"));
    observe_json(&mut compiler, json!({ "reason": "compiler-artifact" }));
    observe_json(&mut compiler, json!({ "reason": "build-script-executed" }));
    observe_json(
        &mut compiler,
        json!({ "reason": "build-finished", "success": false }),
    );

    let result = compiler.finish();
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].level, DiagnosticLevel::Error);
    assert_eq!(result.diagnostics[0].code.as_deref(), Some("E0007"));
    assert_eq!(result.parser_coverage.compiler_messages, 1);
    assert_eq!(result.parser_coverage.compiler_artifacts, 1);
    assert_eq!(result.parser_coverage.build_scripts, 1);
    assert_eq!(result.parser_coverage.build_finished, 1);
    assert_eq!(result.cargo_evidence.build_finished_success, Some(false));
}

#[test]
fn mixed_and_malformed_evidence_never_disappears() {
    let mut compiler = CargoCheckCompiler::default();
    compiler.observe_stdout_frame(b"ordinary producer text", true, false);
    compiler.observe_stdout_frame(b"{broken", true, false);
    compiler.observe_stdout_frame(b"{\"reason\":\"compiler-message\"", false, false);
    compiler.observe_stdout_frame(b"oversized", true, true);
    compiler.observe_stdout_frame(&[0xff, 0xfe], true, false);
    observe_json(
        &mut compiler,
        json!({ "reason": "future-cargo-message", "secret": "token=abc123456789" }),
    );

    let result = compiler.finish();
    // Invalid UTF-8 is withheld as a decoding fault, not compiled as text.
    assert_eq!(result.parser_coverage.non_json_stdout, 1);
    assert_eq!(result.parser_coverage.malformed_json, 1);
    assert_eq!(result.parser_coverage.truncated_frames, 1);
    assert_eq!(result.parser_coverage.over_limit_frames, 1);
    assert_eq!(result.parser_coverage.invalid_utf8_frames, 1);
    assert_eq!(result.parser_coverage.unknown_structured, 1);
    assert!(
        result
            .evidence
            .iter()
            .any(|item| { item.category == EvidenceCategory::MalformedOrTruncated })
    );
}

#[test]
fn every_retained_producer_string_is_sanitized() {
    let mut value = compiler_message(1, "error");
    value["message"]["message"] = json!("token=abc123456789");
    value["message"]["spans"][0]["label"] = json!("Authorization: Bearer abc123456789");
    value["message"]["rendered"] = json!("\u{001b}[31mtoken=abc123456789\u{001b}[0m");
    let mut compiler = CargoCheckCompiler::default();
    observe_json(&mut compiler, value);
    compiler.observe_stderr_frame(b"password=abc123456789", true, false);

    let serialized = serde_json::to_string(&compiler.finish()).expect("result serializes");
    assert!(!serialized.contains("abc123456789"));
    assert!(!serialized.contains("\\u001b"));
    assert!(serialized.contains("[REDACTED]"));
}

#[test]
fn diagnostic_selection_keeps_causal_head_and_terminal_tail() {
    let mut compiler = CargoCheckCompiler::default();
    for index in 0..100 {
        observe_json(&mut compiler, compiler_message(index, "error"));
    }
    let result = compiler.finish();
    assert_eq!(result.diagnostics.len(), 64);
    assert_eq!(result.diagnostics.first().unwrap().message, "diagnostic 0");
    assert_eq!(result.diagnostics[47].message, "diagnostic 47");
    assert_eq!(result.diagnostics[48].message, "diagnostic 84");
    assert_eq!(result.diagnostics.last().unwrap().message, "diagnostic 99");
    assert_eq!(result.retention.diagnostics_omitted, 36);
}

#[test]
fn exact_repeats_collapse_without_hiding_the_count() {
    let mut compiler = CargoCheckCompiler::default();
    for _ in 0..10 {
        observe_json(&mut compiler, compiler_message(3, "warning"));
    }
    let result = compiler.finish();
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].repeat_count, 10);
    assert_eq!(result.retention.exact_repeats_collapsed, 9);
    assert_eq!(result.retention.diagnostics_omitted, 0);
}

#[test]
fn conflicting_completion_records_are_disclosed() {
    let mut compiler = CargoCheckCompiler::default();
    observe_json(
        &mut compiler,
        json!({ "reason": "build-finished", "success": true }),
    );
    observe_json(
        &mut compiler,
        json!({ "reason": "build-finished", "success": false }),
    );
    let result = compiler.finish();
    assert_eq!(result.cargo_evidence.conflicting_build_finished_records, 1);
    assert!(
        result
            .evidence
            .iter()
            .any(|item| { item.category == EvidenceCategory::AuthorityMismatch })
    );
}
