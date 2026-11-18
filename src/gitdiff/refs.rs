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
//   - Base chars must be `[A-Za-z0-9._/\-]`.
//   - Optional revision suffixes are `~N` or `^N`, where N is decimal.
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
    const GRAMMAR: &str = "<name-or-sha>[~N|^N ...] (for example HEAD~2, HEAD^1, or abc123^2)";
    if reference.is_empty() {
        return Err(CleanCtxError::Config(format!(
            "git ref must not be empty; supported grammar: {GRAMMAR}"
        )));
    }
    let bytes = reference.as_bytes();
    // First char must be alphanumeric — never `-` (flag injection).
    if !bytes[0].is_ascii_alphanumeric() {
        return Err(CleanCtxError::Config(format!(
            "invalid git ref '{reference}': must start with an alphanumeric character; supported grammar: {GRAMMAR}"
        )));
    }

    let suffix_start = reference.find(['~', '^']).unwrap_or(reference.len());
    for &b in &bytes[1..suffix_start] {
        let ok = b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'/' | b'-');
        if !ok {
            return Err(CleanCtxError::Config(format!(
                "invalid git ref '{reference}': contains disallowed character '{}'; supported grammar: {GRAMMAR}",
                b as char,
            )));
        }
    }

    let mut suffix = &reference[suffix_start..];
    while !suffix.is_empty() {
        let selector = suffix.as_bytes()[0];
        debug_assert!(selector == b'~' || selector == b'^');
        suffix = &suffix[1..];
        let digits = suffix.bytes().take_while(u8::is_ascii_digit).count();
        if digits == 0 {
            return Err(CleanCtxError::Config(format!(
                "invalid git ref '{reference}': revision selectors require a decimal number; supported grammar: {GRAMMAR}"
            )));
        }
        suffix = &suffix[digits..];
        if !suffix.is_empty() && !matches!(suffix.as_bytes()[0], b'~' | b'^') {
            return Err(CleanCtxError::Config(format!(
                "invalid git ref '{reference}': unsupported revision syntax; supported grammar: {GRAMMAR}"
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
