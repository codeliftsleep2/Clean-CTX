// src/tests/gitdiff/refs.rs
//
// R-12 Phase 1: Tests for git ref validation and resolution.
//
// Security-critical: verifies that flag-injection attempts are rejected
// and that valid refs pass through. `resolve_ref` tests use a real temp
// git repo so they exercise the actual `git rev-parse` path.

use crate::error::CleanCtxError;
use crate::gitdiff::refs::{resolve_ref, validate_ref};

// ── validate_ref: valid refs ────────────────────────────────────────

#[test]
fn valid_refs_are_accepted() {
    for valid in [
        "HEAD",
        "HEAD~1",
        "main",
        "feature/x-2",
        "v1.0",
        "abc123",
        "origin/main",
        "release/1.0.0",
        "refs/heads/main",
    ] {
        assert!(
            validate_ref(valid).is_ok(),
            "expected '{valid}' to be a valid ref"
        );
    }
}

// ── validate_ref: invalid refs (flag injection) ─────────────────────

#[test]
fn empty_ref_is_rejected() {
    assert!(validate_ref("").is_err());
}

#[test]
fn leading_dash_refs_are_rejected() {
    // Flag-injection attempts — must all be rejected.
    for bad in [
        "--upload-pack",
        "-o",
        "--output=/tmp/x",
        "--output",
        "-c",
        "--config",
    ] {
        assert!(
            validate_ref(bad).is_err(),
            "expected '{bad}' to be rejected (flag injection)"
        );
    }
}

#[test]
fn shell_metacharacters_are_rejected() {
    for bad in [
        ";rm -rf /",
        "main;ls",
        "HEAD|cat",
        "HEAD&echo",
        "$(whoami)",
        "`id`",
        "HEAD>out",
        "HEAD<in",
        "HEAD'quote",
        "HEAD\"quote",
        "HEAD\\backslash",
        "HEAD*glob",
        "HEAD?glob",
        "HEAD[glob]",
        "HEAD{glob}",
    ] {
        assert!(
            validate_ref(bad).is_err(),
            "expected '{bad}' to be rejected (shell metacharacter)"
        );
    }
}

#[test]
fn whitespace_is_rejected() {
    for bad in ["HEAD HEAD", " main", "main ", "HEAD\tmain", "HEAD\nmain"] {
        assert!(
            validate_ref(bad).is_err(),
            "expected '{bad}' to be rejected (whitespace)"
        );
    }
}

#[test]
fn error_is_clean_ctx_config() {
    let err = validate_ref("--upload-pack").unwrap_err();
    assert!(matches!(err, CleanCtxError::Config(_)));
}

// ── resolve_ref: real temp git repo ─────────────────────────────────

/// Create a temp git repo with one commit and return its root path.
fn init_temp_repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().to_str().unwrap();

    // git init
    let init = std::process::Command::new("git")
        .arg("init")
        .arg("-q")
        .arg(root)
        .output()
        .expect("git init");
    assert!(init.status.success(), "git init failed: {:?}", init.stderr);

    // Configure a local identity (required for commit).
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

    // Write a file and commit.
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
fn resolve_ref_returns_commit_hash() {
    let dir = init_temp_repo();
    let root = dir.path().to_str().unwrap();

    let hash = resolve_ref(root, "HEAD").expect("resolve HEAD");
    // Full SHA-1 is 40 hex chars.
    assert_eq!(hash.len(), 40, "expected full commit hash, got '{hash}'");
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn resolve_ref_rejects_invalid_ref() {
    let dir = init_temp_repo();
    let root = dir.path().to_str().unwrap();

    let err = resolve_ref(root, "--upload-pack").unwrap_err();
    assert!(matches!(err, CleanCtxError::Config(_)));
}

#[test]
fn resolve_ref_rejects_unknown_ref() {
    let dir = init_temp_repo();
    let root = dir.path().to_str().unwrap();

    let err = resolve_ref(root, "nonexistent-branch").unwrap_err();
    assert!(matches!(err, CleanCtxError::Config(_)));
}