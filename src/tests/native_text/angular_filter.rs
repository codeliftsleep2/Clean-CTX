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

#[test]
fn truncation_retains_real_angular_application_builder_completion() {
    let mut lines = (0..90)
        .map(|line| format!("chunk-{line:03}.js | feature-{line:03}"))
        .collect::<Vec<_>>();
    lines.push(
        "Application bundle generation complete. [37.398 seconds] - 2026-01-01T00:00:00.000Z"
            .into(),
    );
    let result = filter_angular_diagnostics(&lines.join("\n"), AngularOperation::Build);
    assert!(
        result
            .text
            .contains("Application bundle generation complete.")
    );
    assert_eq!(result.text.lines().count(), 80);
}

#[test]
fn truncation_retains_real_vitest_summary_block() {
    let mut lines = (0..90)
        .map(|line| format!("spec-app-case-{line:03}.js | retained"))
        .collect::<Vec<_>>();
    lines.extend([
        " Test Files  40 passed (40)".into(),
        " Tests  347 passed (347)".into(),
        " Start at  10:37:20".into(),
        " Duration  36.12s (transform 7.66s, tests 7.95s)".into(),
    ]);
    let result = filter_angular_diagnostics(&lines.join("\n"), AngularOperation::Test);
    for summary in ["Test Files", "Tests", "Start at", "Duration"] {
        assert!(result.text.contains(summary));
    }
    assert_eq!(result.text.lines().count(), 80);
}
