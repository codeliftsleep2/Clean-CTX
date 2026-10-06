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
fn git_filter_never_filters_stderr() {
    let stderr = "index abc1234..def5678 100644\nBinary files a and b differ";
    let (response, _) = process(&event("git diff", "\x1b[32mclean\x1b[0m", stderr));
    let output = response.expect("stdout normalization should produce a replacement");
    assert_eq!(
        output["hookSpecificOutput"]["updatedToolOutput"]["stderr"].as_str(),
        Some(stderr)
    );
}

#[test]
fn cargo_build_filters_only_its_approved_stderr_diagnostics() {
    let stdout = "application-owned stdout\n    Compiling must-stay";
    let stderr = "   Compiling clean-ctx v0.1.0\nwarning: retained\n --> src/lib.rs:1:1\n    Finished dev target(s)";
    let (response, facts) = process(&event("cargo build --workspace", stdout, stderr));
    let output = response.expect("Cargo progress removal should replace stderr");
    let updated = &output["hookSpecificOutput"]["updatedToolOutput"];
    assert_eq!(updated["stdout"].as_str(), Some(stdout));
    let transformed = updated["stderr"].as_str().unwrap();
    assert!(!transformed.contains("Compiling clean-ctx"));
    assert!(transformed.contains("warning: retained"));
    assert!(transformed.contains("--> src/lib.rs:1:1"));
    assert!(transformed.contains("Finished dev"));
    assert!(transformed.contains("§FILTERED cargo-build:"));
    assert_eq!(
        facts.fields[1].filter.as_ref().unwrap().filter_id,
        "cargo-build-v1"
    );
    assert!(facts.fields[0].filter.is_none());
}

#[test]
fn cargo_check_and_clippy_receive_distinct_operation_authority() {
    for (command, filter_id, marker) in [
        ("cargo check", "cargo-check-v1", "§FILTERED cargo-check:"),
        (
            "cargo clippy -- -D warnings",
            "cargo-clippy-v1",
            "§FILTERED cargo-clippy:",
        ),
    ] {
        let (response, facts) = process(&event(command, "", "    Checking crate v0.1.0\nFinished"));
        let output = response.expect("approved Cargo operation should filter stderr");
        assert!(
            output["hookSpecificOutput"]["updatedToolOutput"]["stderr"]
                .as_str()
                .unwrap()
                .contains(marker)
        );
        assert_eq!(
            facts.fields[1].filter.as_ref().unwrap().filter_id,
            filter_id
        );
    }
}

#[test]
fn cargo_test_run_unknown_and_compound_commands_receive_no_filter_authority() {
    let stderr = "    Compiling must-stay\n    Finished must-stay";
    for command in [
        "cargo test",
        "cargo run",
        "cargo metadata",
        "./cargo build",
        "command cargo check",
        "cargo +nightly clippy",
        "cargo build && echo done",
        "cargo check | tail -20",
        "FOO=bar cargo clippy",
    ] {
        let (response, facts) = process(&event(command, "", stderr));
        assert!(response.is_none(), "replaced ineligible command {command}");
        assert_eq!(
            facts.pass_through_reason,
            Some(PassThroughReason::Unchanged),
            "unexpected reason for {command}"
        );
    }
}

#[test]
fn authorized_target_isolates_producer_semantics_and_fields() {
    let cargo_noise = "    Compiling cargo-looking v0.1.0";
    let git_noise = "index abc1234..def5678 100644";

    let (git_response, git_facts) = process(&event("git diff", cargo_noise, git_noise));
    assert!(git_response.is_none());
    assert_eq!(
        git_facts.pass_through_reason,
        Some(PassThroughReason::Unchanged)
    );

    let (cargo_response, cargo_facts) = process(&event("cargo check", git_noise, cargo_noise));
    let updated = &cargo_response.unwrap()["hookSpecificOutput"]["updatedToolOutput"];
    assert_eq!(updated["stdout"].as_str(), Some(git_noise));
    assert!(!updated["stderr"].as_str().unwrap().contains("Compiling"));
    assert_eq!(
        cargo_facts.fields[1].filter.as_ref().unwrap().filter_id,
        "cargo-check-v1"
    );
}

#[test]
fn cargo_filter_facts_do_not_leak_command_output_or_secrets() {
    let input = event(
        "cargo build --package command-sentinel",
        "",
        "    Compiling output-sentinel v0.1.0\npassword=secret-sentinel\nFinished",
    );
    let facts = serde_json::to_string(&process(&input).1).unwrap();
    for forbidden in [
        "command-sentinel",
        "output-sentinel",
        "secret-sentinel",
        "password=",
    ] {
        assert!(!facts.contains(forbidden), "facts leaked {forbidden}");
    }
    assert!(facts.contains("cargo-build-v1"));
}

#[test]
fn typescript_and_angular_targets_filter_only_stdout() {
    let cases = [
        (
            "tsc --noEmit",
            "Version 5.7.0\nsrc/app.ts(1,1): error TS1: retained",
            "§FILTERED tsc:",
            "tsc-v1",
        ),
        (
            "ng build",
            "Processing assets...\nBuild succeeded.",
            "§FILTERED angular-build:",
            "angular-build-v1",
        ),
        (
            "eslint src",
            "/repo/src/app.ts\n\n  1:1 warning retained",
            "§FILTERED eslint:",
            "eslint-v1",
        ),
        (
            "npm run build",
            "building...\n\n✓ done",
            "§FILTERED node-build:",
            "node-build-v1",
        ),
    ];
    let stderr = "    Compiling must-remain\nindex abc1234..def5678 100644";
    for (command, stdout, marker, filter_id) in cases {
        let (response, facts) = process(&event(command, stdout, stderr));
        let output = response.expect("approved stdout filter should replace output");
        let updated = &output["hookSpecificOutput"]["updatedToolOutput"];
        assert!(updated["stdout"].as_str().unwrap().contains(marker));
        assert_eq!(updated["stderr"].as_str(), Some(stderr));
        assert_eq!(
            facts.fields[0].filter.as_ref().unwrap().filter_id,
            filter_id
        );
        assert!(facts.fields[1].filter.is_none());
    }
}

#[test]
fn unsupported_typescript_and_node_commands_do_not_filter_resembling_output() {
    let stdout = "Version 5.7.0\nProcessing assets...\n\nretained";
    for command in [
        "pnpm tsc",
        "npx ng build",
        "npm run arbitrary",
        "npm install",
        "tsc --watch",
        "tsc --version",
        "eslint --print-config src/app.ts",
        "ng build --help",
        "npm run build | tail -20",
    ] {
        let (response, facts) = process(&event(command, stdout, ""));
        assert!(response.is_none(), "replaced ineligible command {command}");
        assert_eq!(
            facts.pass_through_reason,
            Some(PassThroughReason::Unchanged)
        );
    }
}

#[test]
fn dotnet_build_and_test_filter_only_stdout_with_distinct_semantics() {
    for (command, stdout, marker, filter_id) in [
        (
            "dotnet build CleanCtx.sln",
            "  Determining projects to restore...\nwarning CS0168: retained\nBuild succeeded.\nTime Elapsed 00:00:01",
            "§FILTERED dotnet-build:",
            "dotnet-build-v1",
        ),
        (
            "dotnet test tests/CleanCtx.Tests",
            "\nTest Run Passed.\nTotal tests: 42\nPassed: 42",
            "§FILTERED dotnet-test:",
            "dotnet-test-v1",
        ),
    ] {
        let stderr = "    Compiling must-remain\nindex abc1234..def5678 100644";
        let (response, facts) = process(&event(command, stdout, stderr));
        let output = response.expect("approved .NET operation should filter stdout");
        let updated = &output["hookSpecificOutput"]["updatedToolOutput"];
        assert!(updated["stdout"].as_str().unwrap().contains(marker));
        assert_eq!(updated["stderr"].as_str(), Some(stderr));
        assert_eq!(
            facts.fields[0].filter.as_ref().unwrap().filter_id,
            filter_id
        );
        assert!(facts.fields[1].filter.is_none());
    }
}

#[test]
fn ineligible_dotnet_operations_do_not_filter_resembling_output() {
    let stdout = "Microsoft (R) Build Engine\n  Determining projects\n\nretained";
    for command in [
        "dotnet run",
        "dotnet watch test",
        "dotnet test --list-tests",
        "dotnet build --getProperty:TargetFramework",
        "dotnet build | tail -20",
    ] {
        let (response, facts) = process(&event(command, stdout, ""));
        assert!(response.is_none(), "replaced ineligible command {command}");
        assert_eq!(
            facts.pass_through_reason,
            Some(PassThroughReason::Unchanged)
        );
    }
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
