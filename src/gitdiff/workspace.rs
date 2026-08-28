// src/gitdiff/workspace.rs
//
// R-12 Phase 1: Changed-file collection and content retrieval.
//
// Uses `git diff --name-status --find-renames` to enumerate files that
// changed between two refs, classifying each as Added / Deleted /
// Modified / Renamed. Provides `show_file` to retrieve a file's content
// at a specific ref via `git show <ref>:<path>`.
//
// Security: paths returned by git are validated against the same
// allowlist as refs (no absolute escapes, no flag injection). The
// `--end-of-options` sentinel in `run_git` guards the path argument.

use crate::error::CleanCtxError;

/// Classification of a file that changed between two refs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileChange {
    /// File was added between `from` and `to`.
    Added(String),
    /// File was deleted between `from` and `to`.
    Deleted(String),
    /// File content changed between `from` and `to`.
    Modified(String),
    /// File was renamed from `old_path` to `new_path`.
    Renamed(String, String),
}

impl FileChange {
    /// The path to use for content retrieval at the `to` ref.
    pub fn current_path(&self) -> &str {
        match self {
            FileChange::Added(p) | FileChange::Modified(p) | FileChange::Deleted(p) => p,
            FileChange::Renamed(_, new) => new,
        }
    }

    /// The path to use for content retrieval at the `from` ref.
    pub fn baseline_path(&self) -> &str {
        match self {
            FileChange::Added(p) | FileChange::Modified(p) | FileChange::Deleted(p) => p,
            FileChange::Renamed(old, _) => old,
        }
    }
}

/// Collect the set of files that changed between two refs.
///
/// Runs `git diff --name-status --find-renames <from> <to>` and parses
/// the output into a sorted `Vec<FileChange>`. The `to` ref may be
/// `None` to diff against the working tree (uncommitted changes).
///
/// When `to == None`, additionally discovers untracked non-ignored files
/// via `git ls-files --others --exclude-standard` and treats them as
/// `FileChange::Added`. This makes working-tree diffs include files that
/// exist on disk but have never been staged, while respecting `.gitignore`,
/// `.git/info/exclude`, and global gitignore rules.
///
/// Output format of `git diff --name-status`:
///   - `A\tpath`          → Added
///   - `M\tpath`          → Modified
///   - `D\tpath`          → Deleted
///   - `R100\told\tnew`   → Renamed (100 = similarity score)
pub fn collect_changed_files(
    root: &str,
    from: &str,
    to: Option<&str>,
) -> Result<Vec<FileChange>, CleanCtxError> {
    // Validate refs before passing to git.
    crate::gitdiff::refs::validate_ref(from)?;
    if let Some(t) = to {
        crate::gitdiff::refs::validate_ref(t)?;
    }

    // ── Tracked changes via git diff ──
    let mut args: Vec<String> = vec![
        "diff".to_string(),
        "--name-status".to_string(),
        "--find-renames".to_string(),
        "--end-of-options".to_string(),
        from.to_string(),
    ];
    if let Some(t) = to {
        args.push(t.to_string());
    }

    // Convert to &str slice for run_git.
    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let output = super::runner::run_git(root, &arg_refs)?;

    let mut changes: Vec<FileChange> = Vec::new();
    for line in output.lines() {
        let line = line.trim_end();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split('\t');
        let status = parts.next().unwrap_or("");
        let path1 = parts.next().unwrap_or("");
        let path2 = parts.next();

        // Validate paths: reject absolute escapes and flag-like paths.
        validate_git_path(path1)?;
        if let Some(p2) = path2 {
            validate_git_path(p2)?;
        }

        match status {
            "A" => changes.push(FileChange::Added(path1.to_string())),
            "M" => changes.push(FileChange::Modified(path1.to_string())),
            "D" => changes.push(FileChange::Deleted(path1.to_string())),
            // Rename status is `R<similarity>` e.g. R100.
            s if s.starts_with('R') => {
                let new_path = path2.unwrap_or("");
                changes.push(FileChange::Renamed(path1.to_string(), new_path.to_string()));
            }
            // Unknown status — skip (defensive; git only emits A/M/D/R/C/T/U/X).
            _ => {}
        }
    }

    // ── Untracked non-ignored files (working-tree diff only) ──
    // `git diff` does not enumerate untracked files. When diffing against
    // the working tree, discover them via `git ls-files --others
    // --exclude-standard`, which respects .gitignore, .git/info/exclude,
    // and global gitignore rules. Each untracked file is treated as
    // FileChange::Added — semantically a newly added file in the working
    // tree — and enters the existing pipeline unchanged.
    if to.is_none() {
        let ls_output = super::runner::run_git(
            root,
            &[
                "ls-files",
                "--others",
                "--exclude-standard",
                "--end-of-options",
            ],
        )?;

        // Build a set of paths already discovered by `git diff` to avoid
        // duplicating files that happen to be reported by both commands
        // (defensive; `git diff` should never report untracked files).
        let existing_paths: std::collections::HashSet<String> = changes
            .iter()
            .map(|c| c.current_path().to_string())
            .collect();

        for line in ls_output.lines() {
            let path = line.trim_end();
            if path.is_empty() || existing_paths.contains(path) {
                continue;
            }
            validate_git_path(path)?;
            changes.push(FileChange::Added(path.to_string()));
        }
    }

    // Sort by current path for deterministic output.
    changes.sort_by(|a, b| a.current_path().cmp(b.current_path()));
    Ok(changes)
}

/// Retrieve a file's content at a specific ref.
///
/// Runs `git show <ref>:<path>` and returns the raw content. The ref is
/// validated and the path is validated to prevent flag injection.
pub fn show_file(root: &str, reference: &str, path: &str) -> Result<String, CleanCtxError> {
    crate::gitdiff::refs::validate_ref(reference)?;
    validate_git_path(path)?;
    let spec = format!("{reference}:{path}");
    // `--end-of-options` after the `show` subcommand prevents the
    // `<ref>:<path>` spec from being parsed as a flag.
    super::runner::run_git(root, &["show", "--end-of-options", &spec])
}

/// Validate a git output path.
///
/// Rejects absolute paths (leading `/` or Windows drive letters) and
/// paths that could be parsed as flags (leading `-`). Git paths are
/// always relative to the repo root, so these are never legitimate.
fn validate_git_path(path: &str) -> Result<(), CleanCtxError> {
    if path.is_empty() {
        return Err(CleanCtxError::Config(
            "git path must not be empty".to_string(),
        ));
    }
    if path.starts_with('/') || path.starts_with('-') {
        return Err(CleanCtxError::Config(format!(
            "invalid git path '{path}': must be relative and not start with '-'"
        )));
    }
    // Reject Windows drive letters (C:\...) and UNC paths.
    let bytes = path.as_bytes();
    if bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
        return Err(CleanCtxError::Config(format!(
            "invalid git path '{path}': absolute paths are not allowed"
        )));
    }
    Ok(())
}

#[cfg(test)]
#[path = "../tests/gitdiff/workspace.rs"]
mod tests;
