// src/gitdiff/refs.rs
//
// R-12 Phase 1: Git ref validation and resolution.
//
// Security-critical: refs are user-supplied strings that get passed to
// `git` subprocesses. A malicious ref like `--upload-pack=...` or
// `--output=/tmp/x` could be interpreted as a git flag (CVE-2022-39253
// style injection). We enforce a strict allowlist character set and
// reject any ref that could be parsed as a flag.
//
// Rules:
//   - First char must be `[A-Za-z0-9]` (never `-`, which would allow
//     flag injection).
//   - Remaining chars must be `[A-Za-z0-9._/\-~]` (git refname chars).
//   - No whitespace, no shell metacharacters (`;`, `|`, `&`, `$`, `` ` ``,
//     `(`, `)`, `<`, `>`, `'`, `"`, `\`, `*`, `?`, `[`, `]`, `{`, `}`).
//   - Empty refs are rejected.
//
// `resolve_ref` additionally verifies the ref exists via
// `git rev-parse --verify` so callers get a structured error for
// unknown refs rather than a confusing git stderr dump.

use crate::error::CleanCtxError;

/// Validate a git ref string against the strict allowlist.
///
/// Returns `Ok(())` if the ref is safe to pass to `git` as a positional
/// argument, or `Err` with a human-readable reason.
pub fn validate_ref(reference: &str) -> Result<(), CleanCtxError> {
    if reference.is_empty() {
        return Err(CleanCtxError::Config(
            "git ref must not be empty".to_string(),
        ));
    }
    let bytes = reference.as_bytes();
    // First char must be alphanumeric — never `-` (flag injection).
    if !bytes[0].is_ascii_alphanumeric() {
        return Err(CleanCtxError::Config(format!(
            "invalid git ref '{reference}': must start with an alphanumeric character"
        )));
    }
    // Remaining chars must be in the git refname allowlist.
    for &b in &bytes[1..] {
        let ok = b.is_ascii_alphanumeric()
            || b == b'.'
            || b == b'_'
            || b == b'/'
            || b == b'-'
            || b == b'~';
        if !ok {
            return Err(CleanCtxError::Config(format!(
                "invalid git ref '{reference}': contains disallowed character '{}'",
                b as char
            )));
        }
    }
    Ok(())
}

/// Resolve a git ref to a commit hash via `git rev-parse --verify`.
///
/// Validates the ref first (rejecting flag injection), then runs
/// `git rev-parse --verify <ref>^{commit}` in the given root directory.
/// Returns the full commit hash on success, or a structured error if the
/// ref is invalid or does not exist.
pub fn resolve_ref(root: &str, reference: &str) -> Result<String, CleanCtxError> {
    validate_ref(reference)?;
    let spec = format!("{reference}^{{commit}}");
    let output =
        match super::runner::run_git(root, &["rev-parse", "--verify", "--end-of-options", &spec]) {
            Ok(o) => o,
            // `rev-parse --verify` fails with a non-zero exit for unknown
            // refs — map that to a user-facing Config error rather than an
            // opaque internal git failure.
            Err(_) => {
                return Err(CleanCtxError::Config(format!(
                    "git ref '{reference}' does not resolve to a commit"
                )));
            }
        };
    let hash = output.trim().to_string();
    if hash.is_empty() {
        return Err(CleanCtxError::Config(format!(
            "git ref '{reference}' does not resolve to a commit"
        )));
    }
    Ok(hash)
}

#[cfg(test)]
#[path = "../tests/gitdiff/refs.rs"]
mod tests;
