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
fn test_preserves_counts_and_success_summary_without_collapse() {
    let input = "\nTest Run Passed.\nTotal tests: 42\n     Passed: 42\n Total time: 1.234 Seconds";
    let result = filter_dotnet_diagnostics(input, DotnetOperation::Test);
    assert!(result.text.contains("Test Run Passed."));
    assert!(result.text.contains("Total tests: 42"));
    assert!(result.text.contains("Passed: 42"));
    assert!(result.text.contains("Total time: 1.234 Seconds"));
    assert!(result.text.contains("§FILTERED dotnet-test:"));
    let facts = result.facts.unwrap();
    assert_eq!(facts.filter_id, "dotnet-test-v1");
    assert!(!facts.collapsed);
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
    lines.push("Passed! - Failed: 0, Passed: 11, Skipped: 0, Total: 11, Duration: 1 s".into());
    let result = filter_dotnet_diagnostics(&lines.join("\n"), DotnetOperation::Test);
    assert!(
        result
            .text
            .contains("Passed! - Failed: 0, Passed: 11, Skipped: 0, Total: 11, Duration: 1 s")
    );
    assert_eq!(result.text.lines().count(), 100);
}
