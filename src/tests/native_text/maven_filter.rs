use super::*;

#[test]
fn removes_transfer_and_plugin_noise_but_preserves_build_summary() {
    let input = "[INFO] Scanning for projects...\n[INFO] Building app 1.0.0\n[INFO] --- maven-compiler-plugin:3.8.0:compile ---\n[INFO] Downloading artifact\n[INFO] Downloaded artifact\n[WARNING] retained warning\n[INFO] BUILD SUCCESS\n[INFO] Total time: 4.123 s\n[INFO] Finished at: 2024-01-15T10:30:00Z";
    let result = filter_maven_diagnostics(input, MavenOperation::Package);
    assert!(result.text.contains("Scanning for projects"));
    assert!(result.text.contains("retained warning"));
    assert!(result.text.contains("BUILD SUCCESS"));
    assert!(result.text.contains("Total time: 4.123 s"));
    assert!(result.text.contains("Finished at:"));
    assert!(!result.text.contains("Building app"));
    assert!(!result.text.contains("maven-compiler-plugin"));
    assert!(!result.text.contains("Downloading artifact"));
    assert!(result.text.contains("§FILTERED maven-build:"));
    assert!(!result.facts.unwrap().collapsed);
}

#[test]
fn removes_only_numeric_module_progress_markers() {
    let input = "[INFO] [1/3] module-a\n[INFO] [custom/value] retained";
    let result = filter_maven_diagnostics(input, MavenOperation::Compile);
    assert!(!result.text.contains("[1/3]"));
    assert!(result.text.contains("[custom/value] retained"));
}

#[test]
fn applies_maven_bound_through_shared_kernel() {
    let input = (0..70)
        .map(|line| format!("[WARNING] retained {line}"))
        .collect::<Vec<_>>()
        .join("\n");
    let result = filter_maven_diagnostics(&input, MavenOperation::Install);
    assert_eq!(result.text.lines().count(), 50);
    assert!(result.facts.unwrap().truncated);
}

#[test]
fn truncation_retains_terminal_build_summaries() {
    let mut lines = (0..60)
        .map(|line| format!("[WARNING] retained {line}"))
        .collect::<Vec<_>>();
    lines.extend([
        "[INFO] BUILD SUCCESS".into(),
        "[INFO] Total time: 4.123 s".into(),
        "[INFO] Finished at: 2026-10-06T12:00:00Z".into(),
    ]);
    let result = filter_maven_diagnostics(&lines.join("\n"), MavenOperation::Package);
    assert!(result.text.contains("[INFO] BUILD SUCCESS"));
    assert!(result.text.contains("[INFO] Total time: 4.123 s"));
    assert!(result.text.contains("[INFO] Finished at:"));
    assert_eq!(result.text.lines().count(), 50);
}
