use super::*;

#[test]
fn build_removes_boilerplate_and_preserves_warnings_and_summary() {
    let input = "Microsoft (R) Build Engine\n  Determining projects to restore...\n  Restoring packages...\nwarning CS0168: retained warning\nBuild succeeded.\n    1 Warning(s)\n    0 Error(s)\nTime Elapsed 00:00:02.34";
    let result = filter_dotnet_diagnostics(input, DotnetOperation::Build);
    assert!(!result.text.contains("Microsoft (R)"));
    assert!(!result.text.contains("Determining projects"));
    assert!(result.text.contains("Time Elapsed"));
    assert!(result.text.contains("warning CS0168"));
    assert!(result.text.contains("Build succeeded."));
    assert!(result.text.contains("1 Warning(s)"));
    assert!(result.text.contains("0 Error(s)"));
    assert!(result.text.contains("§FILTERED dotnet-build:"));
    assert!(!result.facts.unwrap().collapsed);
}

#[test]
fn warning_prevents_success_collapse_and_preserves_diagnostics() {
    let input = "  Determining projects to restore...\nwarning NU1900: retained\nTest Run Passed.\nTotal tests: 42\n     Passed: 42\n Total time: 1.234 Seconds";
    let result = filter_dotnet_diagnostics(input, DotnetOperation::Test);
    assert!(result.text.contains("Test Run Passed."));
    assert!(result.text.contains("Total tests: 42"));
    assert!(result.text.contains("Passed: 42"));
    assert!(result.text.contains("Total time: 1.234 Seconds"));
    assert!(result.text.contains("warning NU1900: retained"));
    assert!(result.text.contains("§FILTERED dotnet-test:"));
    let facts = result.facts.unwrap();
    assert_eq!(facts.filter_id, "dotnet-test-v1");
    assert!(!facts.collapsed);
}

#[test]
fn clean_inline_success_collapses_to_one_disclosed_line() {
    let input = "Test run for app.dll\nPassed! -  Failed: 0, Passed: 43, Skipped: 0, Total: 43, Duration: 2.2 min";
    let result = filter_dotnet_diagnostics(input, DotnetOperation::Test);
    assert_eq!(
        result.text,
        "§FILTERED dotnet-test: 43 passed, 0 failed, 0 skipped, 2.2 min (2 → 1 lines)"
    );
    let facts = result.facts.unwrap();
    assert_eq!(facts.reduction_kind, ReductionKind::SuccessCollapse);
    assert!(facts.collapsed);
    assert!(!facts.truncated);
}

#[test]
fn clean_totals_block_success_collapses() {
    let input = "Test Run Passed.\nTotal tests: 42\nPassed: 42\nTotal time: 1.234 Seconds";
    let result = filter_dotnet_diagnostics(input, DotnetOperation::Test);
    assert!(result.text.contains("42 passed, 0 failed, 0 skipped"));
    assert!(result.text.contains("1.234 Seconds"));
    assert!(result.facts.unwrap().collapsed);
}

#[test]
fn skipped_failed_zero_test_and_unparseable_results_never_collapse() {
    for input in [
        "Passed! - Failed: 0, Passed: 42, Skipped: 1, Total: 43, Duration: 1 s",
        "Failed! - Failed: 1, Passed: 42, Skipped: 0, Total: 43, Duration: 1 s",
        "Passed! - Failed: 0, Passed: 0, Skipped: 0, Total: 0, Duration: 1 s",
        "No test is available in app.dll.",
    ] {
        let result = filter_dotnet_diagnostics(input, DotnetOperation::Test);
        assert!(!result.facts.as_ref().is_some_and(|facts| facts.collapsed));
        assert_eq!(result.text, input);
    }
}

#[test]
fn build_and_test_apply_distinct_shared_kernel_bounds() {
    let input = (0..120)
        .map(|line| format!("warning {line}: retained"))
        .collect::<Vec<_>>()
        .join("\n");
    let build = filter_dotnet_diagnostics(&input, DotnetOperation::Build);
    let test = filter_dotnet_diagnostics(&input, DotnetOperation::Test);
    assert_eq!(build.text.lines().count(), 40);
    assert_eq!(test.text.lines().count(), 100);
    assert!(build.facts.unwrap().truncated);
    assert!(test.facts.unwrap().truncated);
}

#[test]
fn build_truncation_retains_terminal_counts_and_timing() {
    let mut lines = (0..50)
        .map(|line| format!("warning CS{line:04}: retained warning"))
        .collect::<Vec<_>>();
    lines.extend([
        "Build succeeded.".into(),
        "    34 Warning(s)".into(),
        "    0 Error(s)".into(),
        "Time Elapsed 00:00:12.34".into(),
    ]);
    let result = filter_dotnet_diagnostics(&lines.join("\n"), DotnetOperation::Build);
    assert!(result.text.contains("Build succeeded."));
    assert!(result.text.contains("34 Warning(s)"));
    assert!(result.text.contains("0 Error(s)"));
    assert!(result.text.contains("Time Elapsed 00:00:12.34"));
    assert_eq!(result.text.lines().count(), 40);
}

#[test]
fn test_truncation_retains_terminal_passed_summary() {
    let mut lines = (0..110)
        .map(|line| format!("NuGet warning {line}: retained"))
        .collect::<Vec<_>>();
    // Real VSTest output can place two spaces after the dash. The anchor must
    // recognize semantic words rather than depend on one formatting space.
    lines.push("Passed! -  Failed: 0, Passed: 11, Skipped: 0, Total: 11, Duration: 1 s".into());
    let result = filter_dotnet_diagnostics(&lines.join("\n"), DotnetOperation::Test);
    assert!(
        result
            .text
            .contains("Passed! -  Failed: 0, Passed: 11, Skipped: 0, Total: 11, Duration: 1 s")
    );
    assert_eq!(result.text.lines().count(), 100);
}

#[test]
fn warning_without_removable_noise_stays_exactly_unchanged() {
    let input = "warning NU1900: retained\nTest Run Passed.\nTotal tests: 42\n     Passed: 42\n Total time: 1.234 Seconds";
    let result = filter_dotnet_diagnostics(input, DotnetOperation::Test);
    assert_eq!(result.text, input);
    assert!(result.facts.is_none());
}
