use super::*;
use crate::native_text::redaction::SecretClass;
use serde_json::json;

fn event(command: &str, stdout: &str, stderr: &str) -> Value {
    json!({
        "hook_event_name": "PostToolUse", "tool_name": "Bash",
        "tool_input": {"command": command, "secret_input": "must-not-log"},
        "tool_response": {"stdout": stdout, "stderr": stderr, "interrupted": false,
            "isImage": false, "unknown": {"ordered": [1, 2, 2, 3]}}
    })
}

#[test]
fn pipeline_orders_normalize_redact_filter_and_preserves_shape() {
    let stdout = "diff --git a/a b/a\nindex abc1234..def5678 100644 password=removed-secret\n\x1b[31m+Authorization: Bearer sentinel-secret\x1b[0m";
    let input = event("git show --color=always HEAD", stdout, "password=hunter2");
    let (response, facts) = process(&input);
    let output = response.unwrap();
    let updated = &output["hookSpecificOutput"]["updatedToolOutput"];
    let rendered = updated.to_string();
    assert!(!updated["stdout"].as_str().unwrap().contains('\u{1b}'));
    assert!(!rendered.contains("sentinel-secret"));
    assert!(!rendered.contains("removed-secret"));
    assert!(!rendered.contains("hunter2"));
    assert!(!rendered.contains("index abc1234"));
    assert!(rendered.contains("§FILTERED git-diff:"));
    assert_eq!(updated["unknown"], json!({"ordered": [1, 2, 2, 3]}));
    assert_eq!(input["tool_input"]["secret_input"], "must-not-log");
    assert_eq!(
        facts.fields[0]
            .redaction
            .counts
            .get(&SecretClass::SecretAssignment),
        Some(&1)
    );
}

#[test]
fn git_looking_output_from_other_command_is_not_filtered() {
    let input = event("printf data", "index abc1234..def5678 100644", "");
    let (response, facts) = process(&input);
    assert!(response.is_none());
    assert_eq!(
        facts.pass_through_reason,
        Some(PassThroughReason::Unchanged)
    );
}

#[test]
fn failure_and_unsupported_shapes_never_replace() {
    let mut failure = event("git diff", "index abc1234..def5678", "");
    failure["hook_event_name"] = json!("PostToolUseFailure");
    assert!(process(&failure).0.is_none());
    let mut image = event("git diff", "x", "");
    image["tool_response"]["isImage"] = json!(true);
    assert!(process(&image).0.is_none());
}

#[test]
fn no_change_means_no_updated_output() {
    let (response, facts) = process(&event("echo ok", "ok", ""));
    assert!(response.is_none());
    assert_eq!(
        facts.pass_through_reason,
        Some(PassThroughReason::Unchanged)
    );
}

#[test]
fn first_slice_never_filters_stderr() {
    let stderr = "index abc1234..def5678 100644\nBinary files a and b differ";
    let (response, _) = process(&event("git diff", "\x1b[32mclean\x1b[0m", stderr));
    let output = response.expect("stdout normalization should produce a replacement");
    assert_eq!(
        output["hookSpecificOutput"]["updatedToolOutput"]["stderr"].as_str(),
        Some(stderr)
    );
}

#[test]
fn facts_are_allowlisted_and_exclude_sensitive_payloads() {
    let input = event(
        "git diff --password command-secret",
        "password=sentinel-secret",
        "",
    );
    let facts = serde_json::to_string(&process(&input).1).unwrap();
    for forbidden in [
        "command-secret",
        "sentinel-secret",
        "password=",
        "must-not-log",
        "tool_response",
    ] {
        assert!(!facts.contains(forbidden), "facts leaked {forbidden}");
    }
    assert!(facts.contains("redaction"));
}

#[test]
fn forced_pipeline_failures_return_no_replacement_and_preserve_input() {
    let input = event("git diff", "index abc1234..def5678", "password=hunter2");
    for (fault, reason) in [
        (PipelineFault::Transform, PassThroughReason::TransformError),
        (
            PipelineFault::Reconstruction,
            PassThroughReason::ReconstructionFailed,
        ),
        (
            PipelineFault::Validation,
            PassThroughReason::ValidationFailed,
        ),
    ] {
        let original = input.clone();
        let (response, facts) = process_with_fault(&input, fault);
        assert!(response.is_none());
        assert_eq!(facts.pass_through_reason, Some(reason));
        assert_eq!(input, original);
    }
}
