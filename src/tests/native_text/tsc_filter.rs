use super::*;

#[test]
fn removes_decoration_but_preserves_diagnostics_and_summary() {
    let input =
        "Version 5.7.0\nsrc/main.ts(5,1): error TS2322: Type mismatch.\n  ~~~~~\nFound 1 error.";
    let result = filter_tsc_diagnostics(input);
    assert!(!result.text.contains("Version 5.7.0"));
    assert!(!result.text.contains("~~~~~"));
    assert!(result.text.contains("src/main.ts(5,1): error TS2322"));
    assert!(result.text.contains("Found 1 error."));
    assert!(result.text.contains("§FILTERED tsc:"));
}

#[test]
fn successful_summary_is_not_collapsed() {
    let input = "Found 0 errors. Watching for file changes.";
    let result = filter_tsc_diagnostics(input);
    assert_eq!(result.text, input);
    assert!(result.facts.is_none());
}

#[test]
fn applies_tsc_line_bound_through_shared_kernel() {
    let input = (0..120)
        .map(|line| format!("src/app.ts({line},1): warning TS1: retained"))
        .collect::<Vec<_>>()
        .join("\n");
    let result = filter_tsc_diagnostics(&input);
    assert_eq!(result.text.lines().count(), 100);
    assert!(result.facts.unwrap().truncated);
}
