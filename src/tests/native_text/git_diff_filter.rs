use super::*;

#[test]
fn removes_only_frozen_noise_and_preserves_context() {
    let input = "diff --git a/a b/a\nindex abc1234..def5678 100644\n--- a/a\n+++ b/a\n@@ -1 +1 @@\n context\n-old\n+new\n\\ No newline at end of file";
    let result = filter_git_diff(input);
    assert!(result.text.contains(" context\n-old\n+new"));
    assert!(!result.text.contains("index abc1234"));
    assert!(!result.text.contains("No newline"));
    assert_eq!(result.text.matches("§FILTERED git-diff:").count(), 1);
    assert_eq!(filter_git_diff(&result.text).text, result.text);
}

#[test]
fn caps_total_output_at_five_hundred_lines() {
    let input = (0..600)
        .map(|n| format!("+line {n}"))
        .collect::<Vec<_>>()
        .join("\n");
    let result = filter_git_diff(&input);
    let facts = result.facts.unwrap();
    assert_eq!(result.text.lines().count(), 500);
    assert!(facts.truncated);
    assert_eq!(result.text.matches("§FILTERED git-diff:").count(), 1);
}

#[test]
fn unchanged_patch_has_no_filter_facts_or_marker() {
    let input = "diff --git a/a b/a\n--- a/a\n+++ b/a\n@@ -1 +1 @@\n-old\n+new";
    let result = filter_git_diff(input);
    assert_eq!(result.text, input);
    assert!(result.facts.is_none());
}
