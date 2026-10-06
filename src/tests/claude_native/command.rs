use super::*;
use crate::native_text::cargo_filter::CargoOperation;

#[test]
fn selects_only_approved_operation_and_field_pairs() {
    assert_eq!(
        diagnostic_target("  cargo build --workspace  "),
        Some(DiagnosticTarget::CargoStderr(CargoOperation::Build))
    );
    assert_eq!(
        diagnostic_target("cargo check --all-targets"),
        Some(DiagnosticTarget::CargoStderr(CargoOperation::Check))
    );
    assert_eq!(
        diagnostic_target("cargo clippy -- -D warnings"),
        Some(DiagnosticTarget::CargoStderr(CargoOperation::Clippy))
    );
    assert_eq!(
        diagnostic_target("git show --format='value with spaces' HEAD"),
        Some(DiagnosticTarget::GitDiffStdout)
    );
}

#[test]
fn cargo_test_run_and_unknown_operations_have_no_authority() {
    for command in [
        "cargo test",
        "cargo run",
        "cargo metadata",
        "cargo",
        "./cargo build",
        "command cargo build",
        "cargo +nightly check",
        "'cargo' build",
        "FOO=bar cargo build",
    ] {
        assert_eq!(diagnostic_target(command), None, "admitted {command}");
    }
}

#[test]
fn shell_control_constructs_are_conservatively_rejected() {
    for command in [
        "cargo check && echo done",
        "cargo build; echo done",
        "cargo clippy || true",
        "cargo check | tail -20",
        "cargo check &",
        "cargo check > result.txt",
        "cargo check 2> errors.txt",
        "cargo check 2>&1",
        "cargo check < args.txt",
        "cargo check $(printf arg)",
        "cargo check `printf arg`",
        "(cargo check)",
        "{ cargo check; }",
        "cargo check\necho done",
    ] {
        assert_eq!(diagnostic_target(command), None, "admitted {command}");
    }
}

#[test]
fn quoted_metacharacters_in_arguments_do_not_create_shell_control() {
    assert_eq!(
        diagnostic_target("cargo check --message-format 'value;still-an-argument'"),
        Some(DiagnosticTarget::CargoStderr(CargoOperation::Check))
    );
    assert_eq!(
        diagnostic_target("git diff -- 'path with spaces'"),
        Some(DiagnosticTarget::GitDiffStdout)
    );
}

#[test]
fn unbalanced_quotes_and_substitution_inside_double_quotes_are_rejected() {
    assert_eq!(diagnostic_target("cargo check 'unterminated"), None);
    assert_eq!(diagnostic_target("cargo check \"$(printf arg)\""), None);
    assert_eq!(diagnostic_target("cargo check \"`printf arg`\""), None);
}
