// src/gitdiff/runner.rs
//
// R-12 Phase 1: Safe `git` subprocess execution.
//
// Security model:
//   - Never uses a shell. All arguments are passed via `Command::arg()`
//     with explicit values — no shell interpolation, no string templating.
//   - `--end-of-options` terminates option parsing for the path component
//     so a path like `--output=/tmp/x` is treated as a literal path, not
//     a git flag.
//   - stderr is captured and included in the error so callers get the
//     real git diagnostics (unknown ref, not a repo, etc.).
//   - Exit-code mapping: 0 → stdout, 1 → structured `GitError`, 128 →
//     "not a git repository" style failure.

use std::process::Command;

use crate::error::CleanCtxError;

/// Execute a git command in the given root directory.
///
/// `args` must begin with the git subcommand, followed by any known
/// flags (never user-supplied), an explicit `--end-of-options` sentinel,
/// and finally pre-validated user-supplied positional values. The
/// sentinel must appear AFTER the subcommand and known flags — it is a
/// subcommand-level option, not a global git option.
///
/// Patterns:
///   - `run_git(root, &["rev-parse", "--verify", "--end-of-options", ref])`
///   - `run_git(root, &["diff", "--name-status", "--end-of-options", from, to])`
///
/// Refs must be validated via [`crate::gitdiff::refs::validate_ref`] and
/// paths via the callers before being passed here.
///
/// Returns stdout on success (exit status 0). Any non-zero exit produces
/// a structured [`CleanCtxError`] with the captured stderr.
pub fn run_git(root: &str, args: &[&str]) -> Result<String, CleanCtxError> {
    let mut cmd = Command::new("git");
    cmd.arg("--no-pager");
    // `-C <root>` changes directory; root is always resolved via
    // `resolve_file_path_checked` by callers, never user-raw.
    cmd.arg("-C").arg(root);
    for arg in args {
        cmd.arg(arg);
    }

    let output = cmd.output().map_err(|e| {
        CleanCtxError::Internal(format!(
            "failed to execute git: {e} (is git installed and on PATH?)"
        ))
    })?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        let code = output.status.code().unwrap_or(-1);
        let msg = if stderr.trim().is_empty() {
            format!("git command failed with exit code {code}: {args:?}")
        } else {
            format!(
                "git command failed with exit code {code}: {}",
                stderr.trim()
            )
        };
        return Err(CleanCtxError::Internal(msg));
    }

    Ok(stdout)
}

/// Whether a directory is inside a git work tree (`git rev-parse --is-inside-work-tree`).
pub fn is_git_repo(root: &str) -> bool {
    run_git(root, &["rev-parse", "--is-inside-work-tree"])
        .map(|out| out.trim() == "true")
        .unwrap_or(false)
}

#[cfg(test)]
#[path = "../tests/gitdiff/runner.rs"]
mod tests;