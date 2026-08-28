// src/tests/gitdiff/engine.rs
//
// R-12 Phase 2: Tests for the multi-file git diff engine.
//
// Uses a real temp git repo with two commits covering modified
// compressible files (AST diff), modified non-compressible files
// (line-count fallback), added files, and deleted files.

use crate::compression::Fidelity;
use crate::gitdiff::engine::gitdiff_workspace;

/// Create a temp git repo with two commits:
///   - Commit 1: `src/app.ts` with a `UserService` class,
///     `notes.md`, `old.ts`
///   - Commit 2: `src/app.ts` gains a method, `notes.md` grows,
///     `old.ts` deleted, `new.ts` added
///
/// Returns the temp dir.
fn init_repo() -> tempfile::TempDir {
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
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(
        dir.path().join("src/app.ts"),
        "class UserService {\n  getUser(id: string): Promise<User> {\n    return api.get(id);\n  }\n}\n",
    )
    .unwrap();
    std::fs::write(dir.path().join("notes.md"), "line1\nline2\n").unwrap();
    std::fs::write(dir.path().join("old.ts"), "class Old { method() {} }\n").unwrap();
    git(&["add", "."]);
    git(&["commit", "-q", "-m", "commit1"]);

    // Commit 2
    std::fs::write(
        dir.path().join("src/app.ts"),
        "class UserService {\n  getUser(id: string): Promise<User> {\n    return api.get(id);\n  }\n  saveUser(u: User): void {\n    api.post(u);\n  }\n}\n",
    )
    .unwrap();
    std::fs::write(dir.path().join("notes.md"), "line1\nline2\nline3\nline4\n").unwrap();
    std::fs::remove_file(dir.path().join("old.ts")).unwrap();
    std::fs::write(dir.path().join("new.ts"), "class New { init() {} }\n").unwrap();
    git(&["add", "-A"]);
    git(&["commit", "-q", "-m", "commit2"]);

    dir
}

#[test]
fn gitdiff_workspace_header_and_counts() {
    let dir = init_repo();
    let root = dir.path().to_str().unwrap();

    let summary = gitdiff_workspace(root, "HEAD~1", Some("HEAD"), Fidelity::Medium, None, None)
        .expect("gitdiff_workspace");

    assert_eq!(summary.file_count, 4, "4 files changed");
    assert_eq!(
        summary.counts,
        (1, 1, 2, 0),
        "1 added, 1 deleted, 2 modified"
    );
    assert_eq!(summary.skipped, 0, "no skipped files");

    // Header line format.
    assert!(
        summary
            .manifest
            .starts_with("§GITDIFF HEAD~1..HEAD (4 files)"),
        "unexpected header: {}",
        summary.manifest.lines().next().unwrap_or("")
    );
}

#[test]
fn gitdiff_workspace_ts_modified_emits_ast_diff() {
    let dir = init_repo();
    let root = dir.path().to_str().unwrap();

    let summary = gitdiff_workspace(root, "HEAD~1", Some("HEAD"), Fidelity::Medium, None, None)
        .expect("gitdiff_workspace");

    // The TS file should produce a change-set with a `+` method marker.
    assert!(
        summary.manifest.contains("+ method saveUser"),
        "expected + method saveUser in manifest, got:\n{}",
        summary.manifest
    );
    // And reference the file path.
    assert!(
        summary.manifest.contains("src/app.ts"),
        "expected src/app.ts in manifest"
    );
}

#[test]
fn gitdiff_workspace_deleted_emits_one_line_entry() {
    let dir = init_repo();
    let root = dir.path().to_str().unwrap();

    let summary = gitdiff_workspace(root, "HEAD~1", Some("HEAD"), Fidelity::Medium, None, None)
        .expect("gitdiff_workspace");

    assert!(
        summary.manifest.contains("- FILE α"),
        "expected a deleted-file entry, got:\n{}",
        summary.manifest
    );
    assert!(
        summary.manifest.contains("old.ts (deleted)"),
        "expected old.ts deletion marker"
    );
}

#[test]
fn gitdiff_workspace_added_emits_skeleton() {
    let dir = init_repo();
    let root = dir.path().to_str().unwrap();

    let summary = gitdiff_workspace(root, "HEAD~1", Some("HEAD"), Fidelity::Medium, None, None)
        .expect("gitdiff_workspace");

    assert!(
        summary.manifest.contains("new.ts (+1 -0 ~0)"),
        "expected added new.ts entry, got:\n{}",
        summary.manifest
    );
    assert!(
        summary.manifest.contains("+ class New"),
        "expected added class skeleton"
    );
}

#[test]
fn gitdiff_workspace_non_compressible_falls_back_to_line_delta() {
    let dir = init_repo();
    let root = dir.path().to_str().unwrap();

    let summary = gitdiff_workspace(root, "HEAD~1", Some("HEAD"), Fidelity::Medium, None, None)
        .expect("gitdiff_workspace");

    // notes.md grew from 2 → 4 lines → +2 lines marker.
    assert!(
        summary.manifest.contains("notes.md"),
        "expected notes.md in manifest"
    );
    assert!(
        summary.manifest.contains("+2 lines (2 → 4)"),
        "expected line-count fallback, got:\n{}",
        summary.manifest
    );
}

#[test]
fn gitdiff_workspace_working_tree_diff() {
    let dir = init_repo();
    let root = dir.path().to_str().unwrap();

    // Uncommitted change: modify a file in the working tree.
    std::fs::write(
        dir.path().join("src/app.ts"),
        "class UserService {\n  getUser(id: string): Promise<User> {\n    return api.get(id);\n  }\n}\n",
    )
    .unwrap();

    let summary = gitdiff_workspace(root, "HEAD", None, Fidelity::Medium, None, None)
        .expect("gitdiff_workspace");

    assert_eq!(summary.file_count, 1, "1 uncommitted change");
    assert!(summary.manifest.contains("src/app.ts"));
    // Reverting the added method → a `-` (removed) marker.
    assert!(
        summary.manifest.contains("- method saveUser"),
        "expected removed method marker, got:\n{}",
        summary.manifest
    );
}

#[test]
fn gitdiff_workspace_working_tree_untracked_file_appears_as_added() {
    let dir = init_repo();
    let root = dir.path().to_str().unwrap();

    // Uncommitted new file — no git add, no intent-to-add. The new
    // untracked discovery path (`git ls-files --others --exclude-standard`)
    // should find it and treat it as FileChange::Added.
    std::fs::write(
        dir.path().join("brand_new.ts"),
        "class BrandNew { init() {} }\n",
    )
    .unwrap();

    let summary = gitdiff_workspace(root, "HEAD", None, Fidelity::Medium, None, None)
        .expect("gitdiff_workspace");

    assert_eq!(summary.file_count, 1, "1 untracked added file");
    assert_eq!(summary.counts, (1, 0, 0, 0), "1 added, 0 skipped");
    assert_eq!(summary.skipped, 0, "untracked file must not be skipped");
    assert!(
        summary.manifest.contains("brand_new.ts (+1 -0 ~0)"),
        "expected added brand_new.ts entry, got:\n{}",
        summary.manifest
    );
    assert!(
        summary.manifest.contains("+ class BrandNew"),
        "expected added class skeleton, got:\n{}",
        summary.manifest
    );
}

#[test]
fn gitdiff_workspace_size_limit_skip_not_double_counted() {
    let dir = init_repo();
    let root = dir.path().to_str().unwrap();

    // Force every content-bearing file to exceed the size limit (10 bytes).
    // Deleted files are never size-checked (one-line entry, no content
    // read) so old.ts remains a processed deletion. The critical invariant
    // is: `counts + skipped == file_count` — a skipped file must NOT also
    // appear in `counts` (double-counting bug).
    let summary = gitdiff_workspace(
        root,
        "HEAD~1",
        Some("HEAD"),
        Fidelity::Medium,
        Some(100),
        Some(10),
    )
    .expect("gitdiff_workspace");

    assert_eq!(summary.file_count, 4, "4 files changed");
    assert_eq!(
        summary.counts,
        (0, 1, 0, 0),
        "only the deletion is processed"
    );
    // new.ts (added), src/app.ts (modified), notes.md (modified) all
    // exceed 10 bytes → skipped. Deletions are never size-checked.
    assert_eq!(
        summary.skipped, 3,
        "3 files skipped (size limit): added + 2 modified"
    );
    assert_eq!(
        summary.counts.0 + summary.counts.1 + summary.counts.2 + summary.counts.3 + summary.skipped,
        summary.file_count,
        "counts + skipped must equal file_count (no double-counting)"
    );
    assert!(
        summary.manifest.contains("exceeds size limit"),
        "expected size-limit skip markers, got:\n{}",
        summary.manifest
    );
    // The delete entry must still be present.
    assert!(
        summary.manifest.contains("old.ts (deleted)"),
        "expected the deleted-file entry, got:\n{}",
        summary.manifest
    );
}

#[test]
fn gitdiff_workspace_invalid_ref_rejected() {
    let dir = init_repo();
    let root = dir.path().to_str().unwrap();

    let err = gitdiff_workspace(
        root,
        "--upload-pack",
        Some("HEAD"),
        Fidelity::Low,
        None,
        None,
    )
    .unwrap_err();
    assert!(
        err.to_string().contains("invalid git ref"),
        "expected ref validation error, got: {err}"
    );
}

#[test]
fn gitdiff_workspace_untracked_ts_file_appears_in_manifest() {
    let dir = init_repo();
    let root = dir.path().to_str().unwrap();

    // Create an untracked TypeScript file (compressible) in the working tree.
    std::fs::write(
        dir.path().join("untracked_util.ts"),
        "class StringHelper {\n  static trim(input: string): string {\n    return input.trim();\n  }\n}\n",
    )
    .unwrap();

    let summary = gitdiff_workspace(root, "HEAD", None, Fidelity::Medium, None, None)
        .expect("gitdiff_workspace");

    assert!(
        summary.manifest.contains("untracked_util.ts"),
        "untracked file must appear in manifest:\n{}",
        summary.manifest
    );
    assert!(
        summary.manifest.contains("+ class StringHelper"),
        "untracked compressible file must produce AST skeleton:\n{}",
        summary.manifest
    );
    assert_eq!(summary.counts.0, 1, "untracked file must count as added");
}

#[test]
fn gitdiff_workspace_untracked_non_compressible_falls_back_to_line_delta() {
    let dir = init_repo();
    let root = dir.path().to_str().unwrap();

    // Untracked non-compressible file.
    std::fs::write(dir.path().join("data.json"), "[\"a\", \"b\"]\n").unwrap();

    let summary = gitdiff_workspace(root, "HEAD", None, Fidelity::Medium, None, None)
        .expect("gitdiff_workspace");

    assert!(
        summary.manifest.contains("data.json"),
        "untracked non-compressible file must appear in manifest:\n{}",
        summary.manifest
    );
    // Non-compressible → line-count delta from 0 lines.
    assert!(
        summary.manifest.contains("+1 lines (0 → 1)"),
        "expected line-count fallback, got:\n{}",
        summary.manifest
    );
}

#[test]
fn gitdiff_workspace_ignored_untracked_file_excluded() {
    let dir = init_repo();
    let root = dir.path().to_str().unwrap();

    // Write a .gitignore and create both ignored and non-ignored files.
    std::fs::write(dir.path().join(".gitignore"), "*.log\n").unwrap();
    std::fs::write(dir.path().join("trace.log"), "ignored\n").unwrap();
    std::fs::write(dir.path().join("lib.rs"), "pub fn helper() -> i32 { 42 }\n").unwrap();

    let summary = gitdiff_workspace(root, "HEAD", None, Fidelity::Medium, None, None)
        .expect("gitdiff_workspace");

    // Ignored file must not appear.
    assert!(
        !summary.manifest.contains("trace.log"),
        "ignored file (trace.log) must NOT appear in manifest:\n{}",
        summary.manifest
    );
    // Non-ignored untracked files must appear.
    assert!(
        summary.manifest.contains(".gitignore"),
        ".gitignore itself must appear in manifest"
    );
    assert!(
        summary.manifest.contains("lib.rs"),
        "non-ignored file (lib.rs) must appear in manifest"
    );
}

#[test]
fn gitdiff_workspace_untracked_plus_tracked_mixed() {
    let dir = init_repo();
    let root = dir.path().to_str().unwrap();

    // Modify a tracked file AND add an untracked file.
    std::fs::write(
        dir.path().join("src/app.ts"),
        "class UserService {\n  getUser(id: string): Promise<User> {\n    return api.get(id);\n  }\n}\n",
    )
    .unwrap();
    // Restore to the commit-1 shape (remove the added method from commit 2).
    // Now we diff HEAD (commit 2) against working tree, so saveUser should be
    // removed (deleted marker), and the untracked file should appear as added.
    std::fs::write(
        dir.path().join("new_util.ts"),
        "class NewUtil { run() {} }\n",
    )
    .unwrap();

    let summary = gitdiff_workspace(root, "HEAD", None, Fidelity::Medium, None, None)
        .expect("gitdiff_workspace");

    // Must report the tracked modified file and the untracked added file.
    assert!(
        summary.manifest.contains("new_util.ts"),
        "untracked file must appear in manifest:\n{}",
        summary.manifest
    );
    assert!(
        summary.manifest.contains("src/app.ts"),
        "tracked modified file must appear"
    );
    // file_count must include both.
    assert_eq!(
        summary.file_count, 2,
        "expected 2 changes: 1 tracked modified + 1 untracked added"
    );
}

// ══════════════════════════════════════════════════════════════════
// ANGULAR_HTML_COMPRESSION_PLAN Phase 2: `.component.html` tests
// ══════════════════════════════════════════════════════════════════

/// Create a temp git repo where commit 2 changes ONLY a method body
/// (same signature) — the critical false-negative regression for
/// `diff_commits`.
fn init_body_only_repo() -> tempfile::TempDir {
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

    // Commit 1: a method with body `return api.get(id);`
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(
        dir.path().join("src/app.ts"),
        "class UserService {\n  getUser(id: string): Promise<User> {\n    return api.get(id);\n  }\n}\n",
    )
    .unwrap();
    git(&["add", "."]);
    git(&["commit", "-q", "-m", "commit1"]);

    // Commit 2: same signature, body changed to `return api.fetch(id);`
    std::fs::write(
        dir.path().join("src/app.ts"),
        "class UserService {\n  getUser(id: string): Promise<User> {\n    return api.fetch(id);\n  }\n}\n",
    )
    .unwrap();
    git(&["add", "-A"]);
    git(&["commit", "-q", "-m", "commit2"]);

    dir
}

/// Create a temp git repo with a `.component.html` file that changes
/// between two commits.
fn init_html_repo() -> tempfile::TempDir {
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

    // Commit 1: simple template
    std::fs::create_dir_all(dir.path().join("src/app")).unwrap();
    std::fs::write(
        dir.path().join("src/app/user-card.component.html"),
        "<div class=\"container\"><app-card [data]=\"cardData\"></app-card></div>\n",
    )
    .unwrap();
    git(&["add", "."]);
    git(&["commit", "-q", "-m", "commit1"]);

    // Commit 2: template gains a binding and a condition
    std::fs::write(
        dir.path().join("src/app/user-card.component.html"),
        "<div class=\"container\"><app-card *ngIf=\"showCard\" [data]=\"cardData\" (select)=\"onSelect($event)\"></app-card></div>\n",
    )
    .unwrap();
    git(&["add", "-A"]);
    git(&["commit", "-q", "-m", "commit2"]);

    dir
}

/// Regression: a body-only change (same method signature) must be
/// detected. Previously `diff_snapshots` compared only sig + markers,
/// so this scenario produced `= class Foo (unchanged)` — a false
/// negative.
#[test]
fn gitdiff_workspace_body_only_change_emits_ast_diff() {
    let dir = init_body_only_repo();
    let root = dir.path().to_str().unwrap();

    let summary = gitdiff_workspace(root, "HEAD~1", Some("HEAD"), Fidelity::Medium, None, None)
        .expect("gitdiff_workspace");

    assert_eq!(summary.file_count, 1, "exactly 1 file changed");
    assert_eq!(summary.counts, (0, 0, 1, 0), "1 modified file");

    // The manifest must contain the body-change marker, not `= (unchanged)`.
    assert!(
        summary.manifest.contains("~ method getUser (body changed)"),
        "expected body-change marker, got:\n{}",
        summary.manifest
    );
    assert!(
        !summary.manifest.contains("= class UserService (unchanged)"),
        "class with body-changed method must not be reported unchanged, got:\n{}",
        summary.manifest
    );
}

#[test]
fn gitdiff_workspace_component_html_emits_template_changeset() {
    let dir = init_html_repo();
    let root = dir.path().to_str().unwrap();

    let summary = gitdiff_workspace(root, "HEAD~1", Some("HEAD"), Fidelity::Medium, None, None)
        .expect("gitdiff_workspace");

    // The .component.html file should produce a compressed template
    // change-set, not a line-count delta.
    assert!(
        summary.manifest.contains("user-card.component.html"),
        "expected .component.html in manifest, got:\n{}",
        summary.manifest
    );
    assert!(
        summary.manifest.contains("template (old)"),
        "expected old template section, got:\n{}",
        summary.manifest
    );
    assert!(
        summary.manifest.contains("template (new)"),
        "expected new template section, got:\n{}",
        summary.manifest
    );
    // The new template should show the *ngIf condition.
    assert!(
        summary.manifest.contains("showCard"),
        "expected *ngIf condition in compressed template, got:\n{}",
        summary.manifest
    );
}

/// Create a temp git repo where commit 2 adds a C# property to a class.
/// This is the critical false-negative regression: previously `CS_QUERY`
/// only captured `(field_declaration)`, so C# properties were invisible
/// to the diff. F-01 diff audit.
fn init_csharp_property_repo() -> tempfile::TempDir {
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

    // Commit 1: a C# class with a method but no properties.
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(
        dir.path().join("src/UserService.cs"),
        "public class UserService {\n  public void GetUser(int id) {\n    // no-op\n  }\n}\n",
    )
    .unwrap();
    git(&["add", "."]);
    git(&["commit", "-q", "-m", "commit1"]);

    // Commit 2: same class with a new property added.
    std::fs::write(
        dir.path().join("src/UserService.cs"),
        "public class UserService {\n  public string Name { get; set; }\n  public void GetUser(int id) {\n    // no-op\n  }\n}\n",
    )
    .unwrap();
    git(&["add", "-A"]);
    git(&["commit", "-q", "-m", "commit2"]);

    dir
}

/// Create a temp git repo with a Rust file that changes between two
/// commits. Previously `.rs` files fell back to line-count deltas even
/// though the codebase has full tree-sitter support for Rust. F-05 diff
/// audit.
#[cfg(feature = "rust")]
fn init_rust_repo() -> tempfile::TempDir {
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

    // Commit 1: a Rust struct with one field.
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(
        dir.path().join("src/models.rs"),
        "pub struct User {\n  pub id: u32,\n}\n",
    )
    .unwrap();
    git(&["add", "."]);
    git(&["commit", "-q", "-m", "commit1"]);

    // Commit 2: same struct with a new field.
    std::fs::write(
        dir.path().join("src/models.rs"),
        "pub struct User {\n  pub id: u32,\n  pub name: String,\n}\n",
    )
    .unwrap();
    git(&["add", "-A"]);
    git(&["commit", "-q", "-m", "commit2"]);

    dir
}

/// Create a temp git repo where commit 2 adds a method to a Java class.
/// G3-2: end-to-end regression for the G2-3 fix (Java in the fallback
/// chain). Previously Java files fell back to the wrong parser and the
/// added method was invisible.
#[cfg(feature = "java")]
fn init_java_repo() -> tempfile::TempDir {
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

    // Commit 1: a Java class with one method.
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(
        dir.path().join("src/UserService.java"),
        "package com.example;\n\npublic class UserService {\n  public void getUser(int id) {\n    // no-op\n  }\n}\n",
    )
    .unwrap();
    git(&["add", "."]);
    git(&["commit", "-q", "-m", "commit1"]);

    // Commit 2: same class with a new method added.
    std::fs::write(
        dir.path().join("src/UserService.java"),
        "package com.example;\n\npublic class UserService {\n  public void getUser(int id) {\n    // no-op\n  }\n  public void saveUser(String name) {\n    // no-op\n  }\n}\n",
    )
    .unwrap();
    git(&["add", "-A"]);
    git(&["commit", "-q", "-m", "commit2"]);

    dir
}

/// G3-2: Java files must produce AST-level diffs (not line-count deltas
/// or a wrong-parser fallback). Regression for the G2-3 fix.
#[cfg(feature = "java")]
#[test]
fn gitdiff_workspace_java_file_emits_ast_diff() {
    let dir = init_java_repo();
    let root = dir.path().to_str().unwrap();

    let summary = gitdiff_workspace(root, "HEAD~1", Some("HEAD"), Fidelity::Medium, None, None)
        .expect("gitdiff_workspace");

    assert_eq!(summary.file_count, 1, "exactly 1 file changed");
    assert_eq!(summary.counts, (0, 0, 1, 0), "1 modified file");

    // The manifest must contain the added method, not a line-count delta.
    assert!(
        summary.manifest.contains("+ method saveUser"),
        "expected + method saveUser in manifest, got:\n{}",
        summary.manifest
    );
    assert!(
        !summary.manifest.contains("lines ("),
        "Java file must not fall back to line-count delta, got:\n{}",
        summary.manifest
    );
}

/// Create a temp git repo where commit 2 changes a top-level function
/// (no class). G3-3: end-to-end regression for the G2-2 fix (orphan
/// methods). Previously top-level functions were dropped entirely.
fn init_top_level_fn_repo() -> tempfile::TempDir {
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

    // Commit 1: a top-level exported function.
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(
        dir.path().join("src/utils.ts"),
        "export function formatName(name: string): string {\n  return name.trim();\n}\n",
    )
    .unwrap();
    git(&["add", "."]);
    git(&["commit", "-q", "-m", "commit1"]);

    // Commit 2: same function, body changed (same signature).
    std::fs::write(
        dir.path().join("src/utils.ts"),
        "export function formatName(name: string): string {\n  return name.toUpperCase();\n}\n",
    )
    .unwrap();
    git(&["add", "-A"]);
    git(&["commit", "-q", "-m", "commit2"]);

    dir
}

/// G3-3: a top-level function (no class) body change must be detected.
/// Regression for the G2-2 fix (orphan methods). Also exercises the
/// G3-5 fix (method_key must not key on `export`).
#[test]
fn gitdiff_workspace_top_level_function_change_emits_ast_diff() {
    let dir = init_top_level_fn_repo();
    let root = dir.path().to_str().unwrap();

    let summary = gitdiff_workspace(root, "HEAD~1", Some("HEAD"), Fidelity::Medium, None, None)
        .expect("gitdiff_workspace");

    assert_eq!(summary.file_count, 1, "exactly 1 file changed");
    assert_eq!(summary.counts, (0, 0, 1, 0), "1 modified file");

    // The manifest must contain the body-change marker keyed on the
    // actual function name (not "export").
    assert!(
        summary
            .manifest
            .contains("~ method formatName (body changed)"),
        "expected ~ method formatName (body changed), got:\n{}",
        summary.manifest
    );
    assert!(
        !summary.manifest.contains("~ method export"),
        "method must not be keyed on 'export', got:\n{}",
        summary.manifest
    );
}

/// Regression: a C# property-only change (adding a property to a class)
/// must be detected by `diff_commits`. Previously the class appeared
/// unchanged — a critical false negative. F-01 diff audit.
#[test]
fn gitdiff_workspace_csharp_property_change_emits_ast_diff() {
    let dir = init_csharp_property_repo();
    let root = dir.path().to_str().unwrap();

    let summary = gitdiff_workspace(root, "HEAD~1", Some("HEAD"), Fidelity::Medium, None, None)
        .expect("gitdiff_workspace");

    assert_eq!(summary.file_count, 1, "exactly 1 file changed");
    assert_eq!(summary.counts, (0, 0, 1, 0), "1 modified file");

    // The manifest must contain a field add marker, not `= (unchanged)`.
    assert!(
        summary.manifest.contains("+ field Name"),
        "expected + field Name in manifest, got:\n{}",
        summary.manifest
    );
    assert!(
        !summary.manifest.contains("= class UserService (unchanged)"),
        "class with a new property must not be reported unchanged, got:\n{}",
        summary.manifest
    );
}

/// Regression: Rust files must produce AST-level diffs, not line-count
/// deltas. F-05 diff audit.
#[cfg(feature = "rust")]
#[test]
fn gitdiff_workspace_rust_file_emits_ast_diff() {
    let dir = init_rust_repo();
    let root = dir.path().to_str().unwrap();

    let summary = gitdiff_workspace(root, "HEAD~1", Some("HEAD"), Fidelity::Medium, None, None)
        .expect("gitdiff_workspace");

    assert_eq!(summary.file_count, 1, "exactly 1 file changed");
    assert_eq!(summary.counts, (0, 0, 1, 0), "1 modified file");

    // The manifest must contain a field add marker, not a line-count delta.
    assert!(
        summary.manifest.contains("+ field name"),
        "expected + field name in manifest, got:\n{}",
        summary.manifest
    );
    assert!(
        !summary.manifest.contains("lines ("),
        "Rust file must not fall back to line-count delta, got:\n{}",
        summary.manifest
    );
}

#[test]
fn gitdiff_workspace_component_html_added_emits_skeleton() {
    let dir = init_html_repo();
    let root = dir.path().to_str().unwrap();

    // Add a new .component.html file in the working tree.
    std::fs::write(
        dir.path().join("src/app/new-card.component.html"),
        "<p-card [value]=\"rows\"><p-inputtext /></p-card>\n",
    )
    .unwrap();
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["add", "-N", "src/app/new-card.component.html"])
        .output()
        .expect("git add -N");
    assert!(
        out.status.success(),
        "git add -N failed: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );

    let summary = gitdiff_workspace(root, "HEAD", None, Fidelity::Medium, None, None)
        .expect("gitdiff_workspace");

    assert!(
        summary.manifest.contains("new-card.component.html"),
        "expected added .component.html in manifest, got:\n{}",
        summary.manifest
    );
    // The added template should be compressed (not a line-count delta).
    assert!(
        summary.manifest.contains("Φtpl:"),
        "expected compressed template skeleton, got:\n{}",
        summary.manifest
    );
}

// ══════════════════════════════════════════════════════════════════
// Non-CBM Tool Audit 2026-08-25, finding #1: changed-class labels
//
// Ground truth from the audit: an `internal static class` rendered as
// `~ class internal` and a `public enum`-style declaration rendered as
// `~ class public`. Access modifiers must never appear as class labels;
// changed classes must carry their actual identifier.
// ══════════════════════════════════════════════════════════════════

/// Temp git repo mirroring the audited change shape: ONE changed file
/// containing BOTH an unchanged public class and a changing
/// `internal static class` with a tuple-returning async method (the audit
/// observed `= class SampleRecordData (unchanged)` and
/// `~ class internal` in the same per-file output).
fn init_csharp_internal_class_repo() -> tempfile::TempDir {
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

    let factory_v1 = concat!(
        "using System.Threading.Tasks;\n",
        "namespace MyApp.Tests.Support\n",
        "{\n",
        "    internal static class TestDataFactory\n",
        "    {\n",
        "        internal static async Task<(int First, int Second)> CreateRecordWithDefaults(int id)\n",
        "        {\n",
        "            return (1, id);\n",
        "        }\n",
        "    }\n",
        "\n",
        "    public class SampleRecordData\n",
        "    {\n",
        "        public int Id { get; set; }\n",
        "    }\n",
        "}\n"
    );

    git(&["init", "-q"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);

    // Commit 1
    std::fs::create_dir_all(dir.path().join("Support")).unwrap();
    std::fs::write(dir.path().join("Support/TestDataFactory.cs"), factory_v1).unwrap();
    git(&["add", "."]);
    git(&["commit", "-q", "-m", "commit1"]);

    // Commit 2: body-only change to the tuple-returning method; the
    // sibling class in the same file is untouched.
    let factory_v2 = factory_v1.replace("return (1, id);", "return (2, id);");
    std::fs::write(dir.path().join("Support/TestDataFactory.cs"), factory_v2).unwrap();
    git(&["add", "-A"]);
    git(&["commit", "-q", "-m", "commit2"]);

    dir
}

/// Regression: a changed `internal static class` must be labeled with its
/// actual identifier, never with the access modifier (`~ class internal`)
/// and never as a visibility token (`~ class public`).
#[test]
fn gitdiff_changed_internal_class_label_uses_identifier() {
    let dir = init_csharp_internal_class_repo();
    let root = dir.path().to_str().unwrap();

    let summary = gitdiff_workspace(root, "HEAD~1", Some("HEAD"), Fidelity::Medium, None, None)
        .expect("gitdiff_workspace");

    assert!(
        summary.manifest.contains("~ class TestDataFactory"),
        "expected '~ class TestDataFactory', got:\n{}",
        summary.manifest
    );
    assert!(
        !summary.manifest.contains("~ class internal"),
        "access modifier leaked into the class label:\n{}",
        summary.manifest
    );
    assert!(
        !summary.manifest.contains("~ class public"),
        "visibility token leaked into the class label:\n{}",
        summary.manifest
    );
}

/// The unchanged-class branch must keep its existing correct rendering
/// while the changed-class branch is fixed.
#[test]
fn gitdiff_unchanged_class_label_remains_correct() {
    let dir = init_csharp_internal_class_repo();
    let root = dir.path().to_str().unwrap();

    let summary = gitdiff_workspace(root, "HEAD~1", Some("HEAD"), Fidelity::Medium, None, None)
        .expect("gitdiff_workspace");

    assert!(
        summary
            .manifest
            .contains("= class SampleRecordData (unchanged)"),
        "unchanged class rendering regressed:\n{}",
        summary.manifest
    );
}

// ══════════════════════════════════════════════════════════════════
// RED regression: an interpolated string must never bleed into a member's
// signature / the diff signature line.
//
// The original bug report used a brace-bodied constructor
// (`public Example(string value) { Console.WriteLine($"Value: {value}"); }`).
// That exact shape parses cleanly on the current snapshot (the body `{`
// precedes the interpolation `{`, so `split('{').next()` still stops at
// the body brace). The defect reproduces with the adjusted fixture below,
// where the interpolated parameter name surfaces in an expression-bodied
// member — a C# record whose primary constructor parameter `Value` is
// interpolated inside an expression-bodied member of the record symbol.
// (See the discussion in the task: the fixture must be adjusted to trigger
// the real, existing defect.)
#[test]
fn gitdiff_interpolation_does_not_bleed_into_signature_line() {
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

    // Commit 1: the record with only its primary constructor.
    let v1 = "public sealed record Example(string Value);\n";
    std::fs::write(dir.path().join("Example.cs"), v1).unwrap();
    git(&["add", "."]);
    git(&["commit", "-q", "-m", "commit1"]);

    // Commit 2: add an expression-bodied member whose body interpolates
    // the primary-constructor parameter name, `Value`.
    let v2 = concat!(
        "public sealed record Example(string Value)\n",
        "{\n",
        "    public string Display() => $\"Value: {Value}\";\n",
        "}\n"
    );
    std::fs::write(dir.path().join("Example.cs"), v2).unwrap();
    git(&["add", "-A"]);
    git(&["commit", "-q", "-m", "commit2"]);

    let summary = gitdiff_workspace(root, "HEAD~1", Some("HEAD"), Fidelity::Medium, None, None)
        .expect("gitdiff_workspace");
    let manifest = &summary.manifest;

    // Guard against a false NEGATIVE: the added member must surface at all.
    assert!(
        manifest.contains("Display"),
        "added member 'Display' must appear in the diff:\n{manifest}"
    );

    // VIOLATED by the current implementation: the `=>` expression-body
    // leaks onto the signature / diff line.
    assert!(
        !manifest.contains("=>"),
        "expression body leaked onto a signature/diff line:\n{manifest}"
    );

    // VIOLATED by the current implementation: the string-interpolation
    // fragment `$"Value:` bleeds into the rendered signature line.
    assert!(
        !manifest.contains("$\"Value:"),
        "interpolated-string fragment bled into the signature line:\n{manifest}"
    );
}

// ══════════════════════════════════════════════════════════════════
// RED regression (ff2a29a): a brace-bodied constructor whose
// base-initializer argument is an INTERPOLATED string must not have its
// signature truncated at the interpolation hole inside
// `: base($"Unexpected value: {value}, ...")`. The literal-unaware
// `stripped.find('{')` in `extract_method_sig` stopped mid-literal, so
// every interpolation hole plus the initializer tail vanished from the
// rendered member label in the diff manifest.
#[test]
fn gitdiff_ctor_base_initializer_interpolation_not_truncated() {
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

    // Commit 1: the class shell with no members.
    let v1 = "public class ExampleException : Exception\n{\n}\n";
    std::fs::write(dir.path().join("ExampleException.cs"), v1).unwrap();
    git(&["add", "."]);
    git(&["commit", "-q", "-m", "commit1"]);

    // Commit 2: add the ff2a29a repro shape — a constructor whose
    // base-initializer argument interpolates both parameters.
    let v2 = concat!(
        "public class ExampleException : Exception\n",
        "{\n",
        "    public ExampleException(string value, object context)\n",
        "        : base($\"Unexpected value: {value}, context: {context}\")\n",
        "    {\n",
        "        Value = value;\n",
        "        Context = context;\n",
        "    }\n",
        "\n",
        "    public string Value { get; }\n",
        "    public object Context { get; }\n",
        "}\n"
    );
    std::fs::write(dir.path().join("ExampleException.cs"), v2).unwrap();
    git(&["add", "-A"]);
    git(&["commit", "-q", "-m", "commit2"]);

    let summary = gitdiff_workspace(root, "HEAD~1", Some("HEAD"), Fidelity::Medium, None, None)
        .expect("gitdiff_workspace");
    let manifest = &summary.manifest;

    // Guard against a false NEGATIVE: the added constructor must surface.
    assert!(
        manifest.contains("ExampleException("),
        "added constructor must appear in the diff:\n{manifest}"
    );

    // Revised by the base-initializer label fix: the Medium-tier member
    // label is the bare declaration — the initializer clause is call-site
    // metadata and no longer renders onto the label. (ff2a29a asserted
    // hole survival because the truncation bug destroyed the initializer
    // outright; the label tier now drops the clause instead, while High
    // keeps the byte-exact header.)
    assert!(
        manifest.contains("ExampleException(string value,object context)"),
        "constructor label must be the bare declaration:\n{manifest}"
    );
    assert!(
        !manifest.contains("{value}"),
        "interpolation hole {{value}} must not render onto any diff/signature line:\n{manifest}"
    );
    assert!(
        !manifest.contains("{context}"),
        "interpolation hole {{context}} must not render onto any diff/signature line:\n{manifest}"
    );

    // Body statements belong to the body, never to a signature/label line.
    assert!(
        !manifest.contains("Value = value;"),
        "constructor body leaked onto a signature/diff line:\n{manifest}"
    );
    assert!(
        !manifest.contains("Context = context;"),
        "constructor body leaked onto a signature/diff line:\n{manifest}"
    );
}
