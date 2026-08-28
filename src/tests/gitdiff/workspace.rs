// src/tests/gitdiff/workspace.rs
//
// R-12 Phase 1: Tests for changed-file collection and content retrieval.
//
// Uses a real temp git repo with two commits covering added, modified,
// deleted, and renamed files to verify `collect_changed_files` and
// `show_file` behave correctly.

use crate::error::CleanCtxError;
use crate::gitdiff::workspace::{FileChange, collect_changed_files, show_file};

/// Create a temp git repo with two commits:
///   - Commit 1: `a.txt`, `b.txt`, `keep.txt`
///   - Commit 2: `a.txt` modified, `b.txt` deleted, `c.txt` added,
///     `keep.txt` renamed to `renamed.txt`
///
/// Returns the temp dir.
fn init_two_commit_repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().to_str().unwrap();

    let git = |args: &[&str]| {
        let out = std::process::Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
            .output()
            .expect("git command");
        assert!(
            out.status.success(),
            "git {:?} failed: {:?}",
            args,
            String::from_utf8_lossy(&out.stderr)
        );
    };

    git(&["init", "-q"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);

    // Commit 1
    std::fs::write(dir.path().join("a.txt"), "hello\n").unwrap();
    std::fs::write(dir.path().join("b.txt"), "world\n").unwrap();
    std::fs::write(dir.path().join("keep.txt"), "keep\n").unwrap();
    git(&["add", "."]);
    git(&["commit", "-q", "-m", "commit1"]);

    // Commit 2
    std::fs::write(dir.path().join("a.txt"), "hello modified\n").unwrap();
    std::fs::remove_file(dir.path().join("b.txt")).unwrap();
    std::fs::write(dir.path().join("c.txt"), "new\n").unwrap();
    std::fs::rename(dir.path().join("keep.txt"), dir.path().join("renamed.txt")).unwrap();
    git(&["add", "-A"]);
    git(&["commit", "-q", "-m", "commit2"]);

    dir
}

#[test]
fn collect_changed_files_classifies_all_statuses() {
    let dir = init_two_commit_repo();
    let root = dir.path().to_str().unwrap();

    let changes = collect_changed_files(root, "HEAD~1", Some("HEAD")).expect("collect");

    // Expect: a.txt modified, b.txt deleted, c.txt added, keep.txt→renamed.txt.
    let mut added = Vec::new();
    let mut modified = Vec::new();
    let mut deleted = Vec::new();
    let mut renamed = Vec::new();
    for c in &changes {
        match c {
            FileChange::Added(p) => added.push(p.clone()),
            FileChange::Modified(p) => modified.push(p.clone()),
            FileChange::Deleted(p) => deleted.push(p.clone()),
            FileChange::Renamed(old, new) => renamed.push((old.clone(), new.clone())),
        }
    }

    assert_eq!(added, vec!["c.txt".to_string()]);
    assert_eq!(modified, vec!["a.txt".to_string()]);
    assert_eq!(deleted, vec!["b.txt".to_string()]);
    assert_eq!(
        renamed,
        vec![("keep.txt".to_string(), "renamed.txt".to_string())]
    );
}

#[test]
fn collect_changed_files_sorted_deterministically() {
    let dir = init_two_commit_repo();
    let root = dir.path().to_str().unwrap();

    let changes = collect_changed_files(root, "HEAD~1", Some("HEAD")).expect("collect");
    // Sorted by current path: a.txt, b.txt, c.txt, renamed.txt
    let paths: Vec<&str> = changes.iter().map(|c| c.current_path()).collect();
    assert_eq!(paths, vec!["a.txt", "b.txt", "c.txt", "renamed.txt"]);
}

#[test]
fn collect_changed_files_rejects_invalid_ref() {
    let dir = init_two_commit_repo();
    let root = dir.path().to_str().unwrap();

    let err = collect_changed_files(root, "--upload-pack", Some("HEAD")).unwrap_err();
    assert!(matches!(err, CleanCtxError::Config(_)));
}

#[test]
fn collect_changed_files_working_tree_diff() {
    let dir = init_two_commit_repo();
    let root = dir.path().to_str().unwrap();

    // Modify a file in the working tree (uncommitted).
    std::fs::write(dir.path().join("a.txt"), "uncommitted change\n").unwrap();

    // Diff HEAD against working tree (to = None).
    let changes = collect_changed_files(root, "HEAD", None).expect("collect");
    assert_eq!(changes.len(), 1, "expected 1 uncommitted change");
    assert!(matches!(&changes[0], FileChange::Modified(p) if p == "a.txt"));
}

#[test]
fn collect_changed_files_working_tree_untracked_file() {
    let dir = init_two_commit_repo();
    let root = dir.path().to_str().unwrap();

    // Create an untracked file — no git add, no intent-to-add.
    std::fs::write(dir.path().join("untracked.txt"), "new untracked content\n").unwrap();

    // Diff HEAD against working tree — untracked files must now appear.
    let changes = collect_changed_files(root, "HEAD", None).expect("collect");
    assert!(
        changes
            .iter()
            .any(|c| matches!(c, FileChange::Added(p) if p == "untracked.txt")),
        "untracked file must appear as FileChange::Added in working-tree diff"
    );
}

#[test]
fn collect_changed_files_untracked_file_only_in_working_tree_diff() {
    let dir = init_two_commit_repo();
    let root = dir.path().to_str().unwrap();

    // Create an untracked file.
    std::fs::write(dir.path().join("untracked.txt"), "new content\n").unwrap();

    // Commit-to-commit diff must NOT include the untracked file.
    let changes = collect_changed_files(root, "HEAD~1", Some("HEAD")).expect("collect");
    assert!(
        !changes.iter().any(|c| c.current_path() == "untracked.txt"),
        "untracked file must NOT appear in commit-to-commit diff"
    );
}

#[test]
fn collect_changed_files_tracked_and_untracked_in_working_tree_diff() {
    let dir = init_two_commit_repo();
    let root = dir.path().to_str().unwrap();

    // Both a tracked modified file and an untracked file.
    std::fs::write(dir.path().join("a.txt"), "modified\n").unwrap();
    std::fs::write(dir.path().join("untracked.txt"), "new\n").unwrap();

    let changes = collect_changed_files(root, "HEAD", None).expect("collect");
    // a.txt modified, untracked.txt added
    assert_eq!(
        changes.len(),
        2,
        "expected tracked modified + untracked added"
    );

    assert!(
        changes
            .iter()
            .any(|c| matches!(c, FileChange::Modified(p) if p == "a.txt"))
    );
    assert!(
        changes
            .iter()
            .any(|c| matches!(c, FileChange::Added(p) if p == "untracked.txt"))
    );
}

#[test]
fn collect_changed_files_ignored_file_excluded() {
    let dir = init_two_commit_repo();
    let root = dir.path().to_str().unwrap();

    // Write a .gitignore that ignores *.log files.
    std::fs::write(dir.path().join(".gitignore"), "*.log\n").unwrap();
    // Create an ignored file and a non-ignored file.
    std::fs::write(dir.path().join("build.log"), "ignored content\n").unwrap();
    std::fs::write(dir.path().join("main.rs"), "fn main() {}\n").unwrap();

    // Diff HEAD against working tree.
    let changes = collect_changed_files(root, "HEAD", None).expect("collect");

    // The ignored file must NOT appear.
    assert!(
        !changes.iter().any(|c| c.current_path() == "build.log"),
        "ignored file (build.log) must NOT appear when .gitignore is active"
    );

    // The non-ignored file (and .gitignore itself) must appear.
    assert!(
        changes.iter().any(|c| c.current_path() == ".gitignore"),
        ".gitignore itself must appear in working-tree diff"
    );
    // main.rs should appear as untracked and not ignored.
    assert!(
        changes.iter().any(|c| c.current_path() == "main.rs"),
        "non-ignored file (main.rs) must appear in working-tree diff"
    );
}

#[test]
fn collect_changed_files_no_changes_untracked_only() {
    let dir = init_two_commit_repo();
    let root = dir.path().to_str().unwrap();

    // No tracked changes, but one untracked file.
    std::fs::write(dir.path().join("fresh.ts"), "let x = 1;\n").unwrap();

    let changes = collect_changed_files(root, "HEAD", None).expect("collect");
    // Only the untracked file should appear.
    assert_eq!(changes.len(), 1, "expected only the untracked file");
    assert!(matches!(&changes[0], FileChange::Added(p) if p == "fresh.ts"));
}

#[test]
fn show_file_returns_content_at_ref() {
    let dir = init_two_commit_repo();
    let root = dir.path().to_str().unwrap();

    // a.txt at HEAD~1 is "hello\n".
    let old = show_file(root, "HEAD~1", "a.txt").expect("show old");
    assert_eq!(old, "hello\n");

    // a.txt at HEAD is "hello modified\n".
    let new = show_file(root, "HEAD", "a.txt").expect("show new");
    assert_eq!(new, "hello modified\n");
}

#[test]
fn show_file_rejects_invalid_path() {
    let dir = init_two_commit_repo();
    let root = dir.path().to_str().unwrap();

    let err = show_file(root, "HEAD", "--output=/tmp/x").unwrap_err();
    assert!(matches!(err, CleanCtxError::Config(_)));
}

#[test]
fn show_file_rejects_absolute_path() {
    let dir = init_two_commit_repo();
    let root = dir.path().to_str().unwrap();

    let err = show_file(root, "HEAD", "/etc/passwd").unwrap_err();
    assert!(matches!(err, CleanCtxError::Config(_)));
}

#[test]
fn file_change_path_helpers() {
    let added = FileChange::Added("a.txt".into());
    assert_eq!(added.current_path(), "a.txt");
    assert_eq!(added.baseline_path(), "a.txt");

    let renamed = FileChange::Renamed("old.txt".into(), "new.txt".into());
    assert_eq!(renamed.current_path(), "new.txt");
    assert_eq!(renamed.baseline_path(), "old.txt");
}
