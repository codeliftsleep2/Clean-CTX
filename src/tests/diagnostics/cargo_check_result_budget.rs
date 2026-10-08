use crate::diagnostics::cargo_check::{CargoCheckCompiler, CargoCheckPolicy};
use serde_json::json;

#[test]
fn result_budget_regression_optional_rendering_cannot_exceed_structured_limit() {
    let mut compiler = CargoCheckCompiler::default();
    let record = json!({"reason":"compiler-message", "message":{
        "message":"first causal error", "level":"error", "rendered":"x".repeat(600 * 1024)
    }});
    compiler.observe_stdout_frame(&serde_json::to_vec(&record).unwrap(), true, false);
    let result = compiler.finish();
    assert_eq!(result.retention.diagnostics_seen, 1);
    assert_eq!(result.diagnostics[0].message, "first causal error");
    assert!(
        serde_json::to_vec(&result).unwrap().len()
            <= CargoCheckPolicy::APPROVED.structured_content_bytes
    );
}

#[test]
fn result_budget_regression_json_escaping_is_counted_in_serialized_budget() {
    let mut compiler = CargoCheckCompiler::default();
    let record = json!({"reason":"compiler-message", "message":{
        "message":"first causal error", "level":"error", "rendered":"\"\\\n".repeat(100 * 1024)
    }});
    compiler.observe_stdout_frame(&serde_json::to_vec(&record).unwrap(), true, false);
    let result = compiler.finish();
    assert_eq!(result.retention.diagnostics_seen, 1);
    assert!(
        serde_json::to_vec(&result).unwrap().len()
            <= CargoCheckPolicy::APPROVED.structured_content_bytes
    );
}

#[test]
fn result_budget_regression_evidence_metadata_counts_toward_structured_limit() {
    let mut compiler = CargoCheckCompiler::default();
    for _ in 0..20000 {
        compiler.observe_stdout_frame(b"x", true, false);
    }
    let result = compiler.finish();
    assert_eq!(result.parser_coverage.stdout_frames, 20000);
    assert!(
        serde_json::to_vec(&result).unwrap().len()
            <= CargoCheckPolicy::APPROVED.structured_content_bytes
    );
}

#[test]
fn result_budget_regression_diagnostic_messages_cannot_exceed_structured_limit() {
    let mut compiler = CargoCheckCompiler::default();
    for index in 0..64 {
        let record = json!({"reason":"compiler-message", "message":{
            "message":format!("error {index}: {}", "x".repeat(12 * 1024)), "level":"error"
        }});
        compiler.observe_stdout_frame(&serde_json::to_vec(&record).unwrap(), true, false);
    }
    let result = compiler.finish();
    assert_eq!(result.retention.diagnostics_seen, 64);
    assert!(result.diagnostics[0].message.starts_with("error 0:"));
    assert!(
        serde_json::to_vec(&result).unwrap().len()
            <= CargoCheckPolicy::APPROVED.structured_content_bytes
    );
}
