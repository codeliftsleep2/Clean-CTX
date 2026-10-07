use super::*;

#[test]
fn removes_build_boilerplate_and_preserves_success_summary() {
    let input = "Browser application bundle generation complete.\nProcessing assets...\nOutput location: dist/app\nBuild succeeded.";
    let result = filter_angular_diagnostics(input, AngularOperation::Build);
    assert_eq!(result.text.matches("§FILTERED angular-build:").count(), 1);
    assert!(result.text.contains("Build succeeded."));
    assert!(!result.text.contains("Processing assets"));
    assert!(!result.text.contains("Output location"));
    assert!(!result.facts.unwrap().collapsed);
}

#[test]
fn operations_keep_distinct_filter_identities() {
    for (operation, filter_id) in [
        (AngularOperation::Test, "angular-test-v1"),
        (AngularOperation::Lint, "angular-lint-v1"),
    ] {
        let result = filter_angular_diagnostics("\nsummary retained", operation);
        assert_eq!(result.facts.unwrap().filter_id, filter_id);
    }
}

#[test]
fn applies_angular_line_bound_through_shared_kernel() {
    let input = (0..90)
        .map(|line| format!("warning {line}: retained"))
        .collect::<Vec<_>>()
        .join("\n");
    let result = filter_angular_diagnostics(&input, AngularOperation::Lint);
    assert_eq!(result.text.lines().count(), 80);
    assert!(result.facts.unwrap().truncated);
}

#[test]
fn truncation_retains_terminal_build_summary() {
    let mut lines = (0..90)
        .map(|line| format!("warning {line}: retained"))
        .collect::<Vec<_>>();
    lines.push("Build succeeded.".into());
    let result = filter_angular_diagnostics(&lines.join("\n"), AngularOperation::Build);
    assert!(result.text.contains("Build succeeded."));
    assert_eq!(result.text.lines().count(), 80);
}
