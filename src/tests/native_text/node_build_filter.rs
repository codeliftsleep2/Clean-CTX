use super::*;

#[test]
fn removes_only_blank_lines_and_preserves_build_output() {
    let input = "✓ 123 modules transformed.\n\ndist/assets/app.js  120.4 kB\n✓ built in 1.23s";
    for operation in [
        NodeBuildOperation::Build,
        NodeBuildOperation::Compile,
        NodeBuildOperation::Bundle,
    ] {
        let result = filter_node_build_output(input, operation);
        assert!(result.text.contains("123 modules transformed"));
        assert!(result.text.contains("dist/assets/app.js"));
        assert!(result.text.contains("built in 1.23s"));
        assert!(result.text.contains("§FILTERED node-build:"));
        assert!(!result.facts.unwrap().collapsed);
    }
}

#[test]
fn applies_node_build_line_bound_through_shared_kernel() {
    let input = (0..130)
        .map(|line| format!("build output {line}"))
        .collect::<Vec<_>>()
        .join("\n");
    let result = filter_node_build_output(&input, NodeBuildOperation::Build);
    assert_eq!(result.text.lines().count(), 120);
    assert!(result.facts.unwrap().truncated);
}

#[test]
fn truncation_retains_terminal_build_summary() {
    let mut lines = (0..130)
        .map(|line| format!("module output {line}"))
        .collect::<Vec<_>>();
    lines.push("✓ built in 1.23s".into());
    let result = filter_node_build_output(&lines.join("\n"), NodeBuildOperation::Build);
    assert!(result.text.contains("✓ built in 1.23s"));
    assert_eq!(result.text.lines().count(), 120);
}
