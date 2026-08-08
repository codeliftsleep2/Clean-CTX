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
    assert_eq!(summary.counts, (1, 1, 2, 0), "1 added, 1 deleted, 2 modified");
    assert_eq!(summary.skipped, 0, "no skipped files");

    // Header line format.
    assert!(
        summary.manifest.starts_with("§GITDIFF HEAD~1..HEAD (4 files)"),
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
fn gitdiff_workspace_working_tree_added_file_read_from_disk() {
    let dir = init_repo();
    let root = dir.path().to_str().unwrap();

    // Uncommitted new file (does not exist in HEAD). `git add -N` (intent
    // to add) makes git report it as `A` in the working-tree diff without
    // creating a commit — exercising the `to == None` disk-read path.
    std::fs::write(
        dir.path().join("brand_new.ts"),
        "class BrandNew { init() {} }\n",
    )
    .unwrap();
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["add", "-N", "brand_new.ts"])
        .output()
        .expect("git add -N");
    assert!(
        out.status.success(),
        "git add -N failed: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );

    let summary = gitdiff_workspace(root, "HEAD", None, Fidelity::Medium, None, None)
        .expect("gitdiff_workspace");

    assert_eq!(summary.file_count, 1, "1 uncommitted added file");
    assert_eq!(summary.counts, (1, 0, 0, 0), "1 added, 0 skipped");
    assert_eq!(summary.skipped, 0, "added file must not be skipped");
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
    let summary = gitdiff_workspace(root, "HEAD~1", Some("HEAD"), Fidelity::Medium, Some(100), Some(10))
        .expect("gitdiff_workspace");

    assert_eq!(summary.file_count, 4, "4 files changed");
    assert_eq!(summary.counts, (0, 1, 0, 0), "only the deletion is processed");
    // new.ts (added), src/app.ts (modified), notes.md (modified) all
    // exceed 10 bytes → skipped. Deletions are never size-checked.
    assert_eq!(summary.skipped, 3, "3 files skipped (size limit): added + 2 modified");
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

    let err = gitdiff_workspace(root, "--upload-pack", Some("HEAD"), Fidelity::Low, None, None)
        .unwrap_err();
    assert!(
        err.to_string().contains("invalid git ref"),
        "expected ref validation error, got: {err}"
    );
}

// ══════════════════════════════════════════════════════════════════
// ANGULAR_HTML_COMPRESSION_PLAN Phase 2: `.component.html` tests
// ══════════════════════════════════════════════════════════════════

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
