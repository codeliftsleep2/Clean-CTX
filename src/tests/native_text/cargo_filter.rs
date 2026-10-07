use super::*;

#[test]
fn removes_progress_and_preserves_diagnostic_and_summary() {
    let input = "   Compiling clean-ctx v0.1.0\nwarning: useful warning\n --> src/lib.rs:4:1\n    Finished dev [unoptimized] target(s) in 1.2s";
    let result = filter_cargo_diagnostics(input, CargoOperation::Build);
    assert!(!result.text.contains("Compiling clean-ctx"));
    assert!(result.text.contains("warning: useful warning"));
    assert!(result.text.contains("--> src/lib.rs:4:1"));
    assert!(result.text.contains("Finished dev"));
    assert!(result.text.ends_with("§FILTERED cargo-build: 4 → 3 lines"));
    let facts = result.facts.unwrap();
    assert_eq!(facts.filter_id, "cargo-build-v1");
    assert!(!facts.collapsed);
}

#[test]
fn operation_owns_disclosure_and_repeat_protection() {
    for (operation, label) in [
        (CargoOperation::Build, "cargo-build"),
        (CargoOperation::Check, "cargo-check"),
        (CargoOperation::Clippy, "cargo-clippy"),
    ] {
        let result = filter_cargo_diagnostics("    Checking crate v0.1.0\nFinished", operation);
        assert_eq!(result.text.matches("§FILTERED").count(), 1);
        assert!(result.text.contains(label));
        assert_eq!(
            filter_cargo_diagnostics(&result.text, operation).text,
            result.text
        );
    }
}

#[test]
fn caps_total_output_at_one_hundred_lines_without_collapsing() {
    let input = (0..120)
        .map(|line| format!("warning: retained {line}"))
        .collect::<Vec<_>>()
        .join("\n");
    let result = filter_cargo_diagnostics(&input, CargoOperation::Clippy);
    let facts = result.facts.unwrap();
    assert_eq!(result.text.lines().count(), 100);
    assert!(facts.truncated);
    assert!(!facts.collapsed);
}

#[test]
fn semantic_only_output_is_unchanged() {
    let input = "warning: retained\n --> src/lib.rs:1:1\nFinished";
    let result = filter_cargo_diagnostics(input, CargoOperation::Check);
    assert_eq!(result.text, input);
    assert!(result.facts.is_none());
}

#[test]
fn truncation_retains_terminal_finished_summary() {
    let mut lines = (0..120)
        .map(|line| format!("warning: retained diagnostic {line}"))
        .collect::<Vec<_>>();
    lines.push("    Finished dev [unoptimized] target(s) in 2.34s".into());
    let result = filter_cargo_diagnostics(&lines.join("\n"), CargoOperation::Build);
    assert!(result.text.contains("Finished dev [unoptimized]"));
    assert_eq!(result.text.lines().count(), 100);
}
