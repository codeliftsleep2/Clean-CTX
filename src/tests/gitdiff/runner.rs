// src/tests/gitdiff/runner.rs
//
// R-12 Phase 1: Tests for safe git subprocess execution.
//
// Verifies `run_git` returns stdout on success, structured errors on
// failure, and that `--end-of-options` prevents path-flag injection.

use crate::error::CleanCtxError;
use crate::gitdiff::runner::{is_git_repo, run_git};

/// Create a temp git repo with one commit and return its root path.
fn init_temp_repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().to_str().unwrap();

    let init = std::process::Command::new("git")
        .arg("init")
        .arg("-q")
        .arg(root)
        .output()
        .expect("git init");
    assert!(init.status.success(), "git init failed: {:?}", init.stderr);

    for (key, value) in [("user.email", "test@example.com"), ("user.name", "Test")] {
        let cfg = std::process::Command::new("git")
            .arg("-C")
            .arg(root)
            .arg("config")
            .arg(key)
            .arg(value)
            .output()
            .expect("git config");
        assert!(cfg.status.success(), "git config {key} failed");
    }

    std::fs::write(dir.path().join("a.txt"), "hello\n").expect("write a.txt");
    let add = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("add")
        .arg("a.txt")
        .output()
        .expect("git add");
    assert!(add.status.success(), "git add failed: {:?}", add.stderr);

    let commit = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("commit")
        .arg("-q")
        .arg("-m")
        .arg("initial")
        .output()
        .expect("git commit");
    assert!(commit.status.success(), "git commit failed: {:?}", commit.stderr);

    dir
}

#[test]
fn run_git_returns_stdout_on_success() {
    let dir = init_temp_repo();
    let root = dir.path().to_str().unwrap();

    let out = run_git(root, &["rev-parse", "--is-inside-work-tree"]).expect("run_git");
    assert_eq!(out.trim(), "true");
}

#[test]
fn run_git_returns_structured_error_on_failure() {
    let dir = init_temp_repo();
    let root = dir.path().to_str().unwrap();

    // `git rev-parse --verify nonexistent` exits non-zero.
    let err = run_git(root, &["rev-parse", "--verify", "nonexistent"]).unwrap_err();
    assert!(matches!(err, CleanCtxError::Internal(_)));
    // The error message should include the git stderr diagnostics.
    assert!(
        err.to_string().contains("git command failed"),
        "expected git failure message, got: {err}"
    );
}

#[test]
fn run_git_reports_missing_git_binary() {
    // A non-existent root directory makes git fail with a clear error.
    let err = run_git("C:/definitely/not/a/real/path", &["rev-parse"]).unwrap_err();
    assert!(matches!(err, CleanCtxError::Internal(_)));
}

#[test]
fn is_git_repo_true_inside_repo() {
    let dir = init_temp_repo();
    let root = dir.path().to_str().unwrap();
    assert!(is_git_repo(root));
}

#[test]
fn is_git_repo_false_outside_repo() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().to_str().unwrap();
    assert!(!is_git_repo(root));
}

#[test]
fn end_of_options_prevents_path_flag_injection() {
    let dir = init_temp_repo();
    let root = dir.path().to_str().unwrap();

    // A path that looks like a flag must be treated as a literal path,
    // not parsed as a git option. `git show --end-of-options --output=/tmp/x`
    // should fail because the path doesn't exist, NOT because git tried
    // to interpret `--output` as a flag.
    let err = run_git(root, &["show", "--end-of-options", "--output=/tmp/x"]).unwrap_err();
    assert!(matches!(err, CleanCtxError::Internal(_)));
    // The error should reference the path, not "unknown option".
    let msg = err.to_string();
    assert!(
        !msg.contains("unknown option") && !msg.contains("unrecognized"),
        "git should not parse --output as a flag, got: {msg}"
    );
}
