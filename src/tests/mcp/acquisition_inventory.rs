use super::{SourceKind, scope_policy_fingerprint};
use crate::mcp::tool_handlers::hydration::source_inventory;
use std::path::PathBuf;

fn state_with_exclusions(exclusions: &[&str]) -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config.exclude_patterns = exclusions.iter().map(|value| value.to_string()).collect();
    crate::mcp::McpState::new(config)
}

#[test]
fn acquisition_inventory_is_deterministic_and_respects_admission_policy() {
    let root = tempfile::tempdir().expect("workspace");
    std::fs::create_dir(root.path().join(".git")).expect("Git root marker");
    std::fs::create_dir(root.path().join("excluded")).expect("excluded directory");
    std::fs::write(root.path().join("B.cs"), "public class B {}\n").expect("B source");
    std::fs::write(root.path().join("A.cs"), "public class A {}\n").expect("A source");
    std::fs::write(root.path().join("ignored.txt"), "ContextA\n").expect("unsupported source");
    std::fs::write(
        root.path().join("excluded").join("Hidden.cs"),
        "public class Hidden {}\n",
    )
    .expect("excluded source");
    let state = state_with_exclusions(&["excluded"]);
    let roots = [PathBuf::from(root.path())];

    let first = source_inventory(&state, &roots);
    let second = source_inventory(&state, &roots);

    assert!(first.attempted && first.complete);
    assert_eq!(first.digest, second.digest);
    assert_eq!(first.entries, second.entries);
    assert_eq!(first.entries.len(), 2);
    assert!(
        first
            .entries
            .windows(2)
            .all(|pair| pair[0].owner < pair[1].owner)
    );
    assert!(
        first
            .entries
            .iter()
            .all(|entry| entry.source_kind == SourceKind::CSharp)
    );
}

#[test]
fn acquisition_inventory_digest_changes_with_source_content() {
    let root = tempfile::tempdir().expect("workspace");
    std::fs::create_dir(root.path().join(".git")).expect("Git root marker");
    let source = root.path().join("Context.cs");
    std::fs::write(&source, "public class ContextA {}\n").expect("initial source");
    let state = state_with_exclusions(&[]);
    let roots = [PathBuf::from(root.path())];
    let before = source_inventory(&state, &roots);

    std::fs::write(&source, "public class ContextB {}\n").expect("changed source");
    state.invalidate_source_cache(&source.to_string_lossy());
    let after = source_inventory(&state, &roots);

    assert_ne!(before.digest, after.digest);
    assert_ne!(before.entries[0].source_hash, after.entries[0].source_hash);
}

#[test]
fn acquisition_inventory_missing_root_never_proves_completion() {
    let state = state_with_exclusions(&[]);
    let root = tempfile::tempdir().expect("parent");
    let missing = root.path().join("missing");

    let inventory = source_inventory(&state, &[missing]);

    assert!(!inventory.attempted);
    assert!(!inventory.complete);
    assert!(inventory.entries.is_empty());
}

#[test]
fn scope_policy_fingerprint_is_order_independent_and_policy_sensitive() {
    let left = scope_policy_fingerprint(
        ["C:/repo-b".into(), "C:/repo-a".into()],
        ["dist".into(), "generated*".into()],
    );
    let reordered = scope_policy_fingerprint(
        ["C:/repo-a".into(), "C:/repo-b".into()],
        ["generated*".into(), "dist".into()],
    );
    assert_eq!(left, reordered);
    assert_ne!(
        left,
        scope_policy_fingerprint(["C:/repo-a".into(), "C:/repo-b".into()], ["dist".into()],)
    );
    assert_ne!(
        left,
        scope_policy_fingerprint(["C:/repo-a".into()], ["dist".into(), "generated*".into()],)
    );
}
