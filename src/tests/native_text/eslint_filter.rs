use super::*;

#[test]
fn removes_blank_lines_without_hiding_warnings() {
    let input = "/repo/src/app.ts\n\n  1:1  warning  Unexpected any  rule-name\n\n1 problem (0 errors, 1 warning)";
    let result = filter_eslint_diagnostics(input);
    assert!(result.text.contains("warning  Unexpected any"));
    assert!(result.text.contains("1 problem (0 errors, 1 warning)"));
    assert!(!result.text.contains("eslint: ok"));
    assert!(result.text.contains("§FILTERED eslint:"));
    assert!(!result.facts.unwrap().collapsed);
}

#[test]
fn applies_eslint_line_bound_through_shared_kernel() {
    let input = (0..150)
        .map(|line| format!("{line}:1 warning retained rule-name"))
        .collect::<Vec<_>>()
        .join("\n");
    let result = filter_eslint_diagnostics(&input);
    assert_eq!(result.text.lines().count(), 140);
    assert!(result.facts.unwrap().truncated);
}

#[test]
fn truncation_retains_terminal_problem_summary() {
    let mut lines = (0..150)
        .map(|line| format!("{line}:1 warning retained rule-name"))
        .collect::<Vec<_>>();
    lines.push("✖ 150 problems (0 errors, 150 warnings)".into());
    let result = filter_eslint_diagnostics(&lines.join("\n"));
    assert!(
        result
            .text
            .contains("✖ 150 problems (0 errors, 150 warnings)")
    );
    assert_eq!(result.text.lines().count(), 140);
}
