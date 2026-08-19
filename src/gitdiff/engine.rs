// src/gitdiff/engine.rs
//
// R-12 Phase 2: Diff engine.
//
// Orchestrates per-file diffs between two git refs, reusing the existing
// `src/diff` AST snapshot + change-set machinery for compressible files
// and falling back to a compact line-count delta for non-compressible
// files (html/css/json/etc.) where no tree-sitter grammar exists.
//
// Output format per file:
//   ┌ FILE αN <path> (+A -D ~M)
//   <format_diff output>
//
// For Added files the full compressed content is emitted (reusing the
// legacy pipeline). For Deleted files a one-line removal entry is
// emitted. Renamed files are diffed between the old path at `from` and
// the new path at `to`.
//
// R-12 Phase 3: `gitdiff_workspace` now accepts optional resource limits
// (`max_files` caps the number of changed files processed; `max_file_size`
// caps the per-file content size). Files exceeding either limit are
// counted in `skipped` and emitted as a one-line skip entry rather than
// failing the whole operation.

use crate::compression::Fidelity;
use crate::diff::{build_snapshot, diff_snapshots, format_diff};
use crate::gitdiff::workspace::{collect_changed_files, show_file, FileChange};
// ANGULAR_HTML_COMPRESSION_PLAN Phase 2: HTML template compression
// in the diff path. Only available when the `angular` feature is enabled.
#[cfg(feature = "angular")]
use crate::angular_meta::template_compress::compress_template_to_string;

/// Result of a multi-file git diff operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitDiffSummary {
    /// The rendered manifest text (the `content` payload for MCP).
    pub manifest: String,
    /// Total files changed (added + deleted + modified + renamed).
    pub file_count: usize,
    /// Per-change counts: (added, deleted, modified, renamed).
    pub counts: (usize, usize, usize, usize),
    /// Number of files skipped because they exceeded resource limits.
    pub skipped: usize,
}

/// Diff an entire workspace between two git refs.
///
/// `from` and `to` are validated refs; `to == None` diffs against the
/// working tree. `fidelity` controls the per-file `format_diff` depth.
/// `max_files` caps the number of changed files processed (files beyond
/// the cap are counted in `skipped`). `max_file_size` caps the per-file
/// content size in bytes (oversized files are skipped). Returns a
/// [`GitDiffSummary`] with a rendered manifest.
pub fn gitdiff_workspace(
    root: &str,
    from: &str,
    to: Option<&str>,
    fidelity: Fidelity,
    max_files: Option<usize>,
    max_file_size: Option<usize>,
) -> Result<GitDiffSummary, crate::error::CleanCtxError> {
    let mut changes = collect_changed_files(root, from, to)?;

    // Resource limit: cap the number of changed files processed.
    // Files beyond the cap are counted as skipped (fail-closed, never
    // partial output).
    let mut skipped = 0usize;
    if let Some(max) = max_files
        && changes.len() > max
    {
        skipped = changes.len() - max;
        changes.truncate(max);
    }

    let mut manifest = String::new();
    // Header line: `§GITDIFF <from>..<to> (N files)`
    let to_label = to.unwrap_or("working-tree");
    manifest.push_str(&format!("§GITDIFF {from}..{to_label} ({} files)\n", changes.len()));

    let (mut added, mut deleted, mut modified, mut renamed) = (0usize, 0usize, 0usize, 0usize);

    for (alias, change) in (1usize..).zip(changes.iter()) {
        match change {
            FileChange::Added(path) => {
                // Emit a compact full-compress of the new content. When
                // diffing against the working tree (`to == None`), an
                // uncommitted new file does not exist in `HEAD` — read it
                // from disk instead of `git show HEAD:path`.
                let content = match to {
                    Some(t) => show_file(root, t, path),
                    None => std::fs::read_to_string(std::path::Path::new(root).join(path))
                        .map_err(|e| crate::error::CleanCtxError::Internal(e.to_string())),
                };
                match content {
                    Ok(content) => {
                        if let Some(max) = max_file_size
                            && content.len() > max
                        {
                            skipped += 1;
                            manifest.push_str(&format!(
                                "┌ FILE α{alias}: {path} (added — exceeds size limit, skipped)\n"
                            ));
                        } else {
                            added += 1;
                            let compressed = compress_added_file(&content, path, fidelity);
                            manifest.push_str(&format!(
                                "┌ FILE α{alias}: {path} (+1 -0 ~0)\n{compressed}\n"
                            ));
                        }
                    }
                    Err(_) => {
                        skipped += 1;
                        manifest.push_str(&format!(
                            "┌ FILE α{alias}: {path} (added — content unavailable, skipped)\n"
                        ));
                    }
                }
            }
            FileChange::Deleted(path) => {
                deleted += 1;
                manifest.push_str(&format!("- FILE α{alias}: {path} (deleted)\n"));
            }
            FileChange::Modified(path) => {
                let (body, err) =
                    diff_modified_file(root, from, to, path, fidelity, max_file_size);
                match err {
                    Some(msg) => {
                        skipped += 1;
                        manifest.push_str(&format!(
                            "┌ FILE α{alias}: {path} (~0 -0 +0 — {msg})\n"
                        ));
                    }
                    None => {
                        modified += 1;
                        // Count `+ - ~` lines in the body for the header.
                        let (m_adds, m_dels, m_mods) = count_change_markers(&body);
                        manifest.push_str(&format!(
                            "┌ FILE α{alias}: {path} (+{m_adds} -{m_dels} ~{m_mods})\n{body}\n"
                        ));
                    }
                }
            }
            FileChange::Renamed(old, new) => {
                let diff =
                    diff_renamed_file(root, from, to, old, new, fidelity, max_file_size);
                match diff {
                    Ok(body) => {
                        renamed += 1;
                        let (m_adds, m_dels, m_mods) = count_change_markers(&body);
                        manifest.push_str(&format!(
                            "~ FILE α{alias}: {old} → {new} (+{m_adds} -{m_dels} ~{m_mods})\n{body}\n"
                        ));
                    }
                    Err(msg) => {
                        skipped += 1;
                        manifest.push_str(&format!(
                            "~ FILE α{alias}: {old} → {new} (renamed — {msg})\n"
                        ));
                    }
                }
            }
        }
    }

    Ok(GitDiffSummary {
        manifest,
        file_count: changes.len(),
        counts: (added, deleted, modified, renamed),
        skipped,
    })
}

/// Diff a single modified file between two refs.
///
/// Returns `(body, Option<error_msg>)`. For compressible extensions the
/// body is the AST change-set from `format_diff`. For non-compressible
/// extensions the body is a line-count delta (`+N/-N/M`). On failure the
/// error message is returned in the second element. `max_file_size` caps
/// the per-file content size; oversized files return a skip error.
fn diff_modified_file(
    root: &str,
    from: &str,
    to: Option<&str>,
    path: &str,
    fidelity: Fidelity,
    max_file_size: Option<usize>,
) -> (String, Option<String>) {
    let from_content = match show_file(root, from, path) {
        Ok(c) => c,
        Err(e) => return (String::new(), Some(e.to_string())),
    };
    if let Some(max) = max_file_size
        && from_content.len() > max
    {
        return (
            String::new(),
            Some(format!("exceeds size limit ({max} bytes)")),
        );
    }
    let to_content = match to {
        Some(t) => match show_file(root, t, path) {
            Ok(c) => c,
            Err(e) => return (String::new(), Some(e.to_string())),
        },
        // Working tree — read from disk.
        None => match std::fs::read_to_string(std::path::Path::new(root).join(path)) {
            Ok(c) => c,
            Err(e) => return (String::new(), Some(e.to_string())),
        },
    };
    if let Some(max) = max_file_size
        && to_content.len() > max
    {
        return (
            String::new(),
            Some(format!("exceeds size limit ({max} bytes)")),
        );
    }

    diff_two_contents(&from_content, &to_content, path, fidelity)
}

/// Diff the content between the old and new location of a renamed file.
fn diff_renamed_file(
    root: &str,
    from: &str,
    to: Option<&str>,
    old: &str,
    new: &str,
    fidelity: Fidelity,
    max_file_size: Option<usize>,
) -> Result<String, String> {
    let from_content = show_file(root, from, old).map_err(|e| e.to_string())?;
    if let Some(max) = max_file_size
        && from_content.len() > max
    {
        return Err(format!("exceeds size limit ({max} bytes)"));
    }
    let to_content = match to {
        Some(t) => show_file(root, t, new).map_err(|e| e.to_string())?,
        // Working tree — read the new path from disk.
        None => std::fs::read_to_string(std::path::Path::new(root).join(new))
            .map_err(|e| e.to_string())?,
    };
    if let Some(max) = max_file_size
        && to_content.len() > max
    {
        return Err(format!("exceeds size limit ({max} bytes)"));
    }
    Ok(diff_two_contents(&from_content, &to_content, new, fidelity).0)
}

/// Diff two content strings, returning `(body, Option<error>)`.
///
/// Compressible extensions (ts/js/cs) use the AST diff. Angular
/// `.component.html` files use the fidelity-gated template compressor
/// (ANGULAR_HTML_COMPRESSION_PLAN Phase 2). All others fall back to a
/// compact line-count delta so the tool never fails on grammar-missing
/// files.
fn diff_two_contents(
    from_content: &str,
    to_content: &str,
    path: &str,
    fidelity: Fidelity,
) -> (String, Option<String>) {
    if is_compressible(path) {
        match build_snapshot(from_content, fidelity) {
            Ok(base_snap) => match build_snapshot(to_content, fidelity) {
                Ok(cur_snap) => {
                    let actions = diff_snapshots(&base_snap, &cur_snap);
                    let body =
                        format_diff(&actions, fidelity).trim_end().to_string();
                    (body, None)
                }
                Err(e) => (String::new(), Some(e.to_string())),
            },
            Err(e) => (String::new(), Some(e.to_string())),
        }
    } else if is_angular_template(path) {
        // Angular `.component.html` — produce a compressed template
        // change-set. We emit the compressed form of both the old and
        // new content so the LLM can see what changed at the semantic
        // level (bindings, conditions, components) rather than a raw
        // line-count delta.
        let from_compressed = compress_template_to_string(from_content, fidelity);
        let to_compressed = compress_template_to_string(to_content, fidelity);
        if from_compressed == to_compressed {
            (format!("  ~ template unchanged ({} lines → {} lines)", from_content.lines().count(), to_content.lines().count()), None)
        } else {
            let mut body = String::new();
            body.push_str("  - template (old):\n");
            for line in from_compressed.lines() {
                body.push_str(&format!("    - {}\n", line));
            }
            body.push_str("  + template (new):\n");
            for line in to_compressed.lines() {
                body.push_str(&format!("    + {}\n", line));
            }
            (body, None)
        }
    } else {
        // Non-compressible — line-count delta.
        let from_lines = from_content.lines().count();
        let to_lines = to_content.lines().count();
        let delta = to_lines.abs_diff(from_lines);
        let marker = if to_lines > from_lines { "+" } else { "-" };
        (format!("  {marker}{delta} lines ({from_lines} → {to_lines})"), None)
    }
}

/// Compress an added file via the legacy pipeline (best-effort).
fn compress_added_file(content: &str, path: &str, fidelity: Fidelity) -> String {
    if is_angular_template(path) {
        // Angular `.component.html` — emit the compressed template skeleton.
        return compress_template_to_string(content, fidelity);
    }
    if !is_compressible(path) {
        return format_line_delta(0, content.lines().count());
    }
    // Best-effort: build a snapshot and render as a compact skeleton.
    match build_snapshot(content, fidelity) {
        Ok(snap) => {
            // Render imports + class names as a compact skeleton.
            let mut out = String::new();
            for imp in &snap.imports {
                out.push_str(&format!("+ import {imp}\n"));
            }
            for class in &snap.classes {
                out.push_str(&format!("+ class {}\n", class.name));
                for field in &class.fields {
                    out.push_str(&format!("  + field {field}\n"));
                }
                for method in &class.methods {
                    out.push_str(&format!("  + method {}\n", method.sig));
                }
            }
            for field in &snap.orphan_fields {
                out.push_str(&format!("+ field {field}\n"));
            }
            for method in &snap.orphan_methods {
                out.push_str(&format!("+ method {}\n", method.sig));
            }
            if out.is_empty() {
                // No classes/imports — just a line count.
                out = format_line_delta(0, content.lines().count());
            }
            out
        }
        Err(_) => format_line_delta(0, content.lines().count()),
    }
}

/// Compact line-count delta, e.g. `(+42 lines new file)`.
fn format_line_delta(from: usize, to: usize) -> String {
    if to > from {
        format!("+{} lines ({from} → {to})", to - from)
    } else {
        format!("-{} lines ({from} → {to})", from - to)
    }
}

/// Count `+ - ~` change markers in a diff body (one line each).
fn count_change_markers(body: &str) -> (usize, usize, usize) {
    let mut adds = 0;
    let mut dels = 0;
    let mut mods = 0;
    for line in body.lines() {
        let t = line.trim_start();
        if let Some(rest) = t.strip_prefix('+') {
            let after = rest.trim_start();
            // G2-7 audit: the old `starts_with("import")` also skipped
            // identifiers like `importData` or `implementation`. Only skip
            // actual import lines (`import Foo` / `import<end>`).
            let is_import_line = after.starts_with("import")
                && after[6..]
                    .chars()
                    .next()
                    .is_none_or(|c| c.is_whitespace() || c == '\n');
            if !after.is_empty() && !is_import_line {
                adds += 1;
            }
        } else if let Some(rest) = t.strip_prefix('-') {
            let after = rest.trim_start();
            // G3-1 audit: mirror the `+` branch — skip `- import` removal
            // lines so the header counts are symmetric. The old code only
            // skipped `+ import` (G2-7), so a file that removed an import
            // reported a spurious `-1` in the header.
            let is_import_line = after.starts_with("import")
                && after[6..]
                    .chars()
                    .next()
                    .is_none_or(|c| c.is_whitespace() || c == '\n');
            if !after.is_empty() && !is_import_line {
                dels += 1;
            }
        } else if t.starts_with('~') {
            mods += 1;
        }
    }
    (adds, dels, mods)
}

/// Whether a file path has a compressible extension.
///
/// F-05 diff audit: previously only `ts`/`js`/`cs` were compressible.
/// Rust (`.rs`) and Java (`.java`) files fell back to line-count deltas
/// even though the codebase has full tree-sitter support for both —
/// a significant gap for a Rust project. TSX/JSX are also added since
/// the TypeScript grammar handles them.
///
/// `rs` and `java` are gated on their Cargo features — when the feature
/// is disabled, `safe_rust_language()`/`safe_java_language()` return
/// `None`, so routing the file into `build_snapshot` would panic on the
/// `Language must be Some` expect. Feature-gating keeps the fallback to
/// a line-count delta for unsupported languages.
fn is_compressible(path: &str) -> bool {
    let Some(ext) = path.rsplit('.').next() else {
        return false;
    };
    match ext {
        "ts" | "js" | "tsx" | "jsx" | "cs" => true,
        #[cfg(feature = "rust")]
        "rs" => true,
        #[cfg(feature = "java")]
        "java" => true,
        _ => false,
    }
}

/// Whether a file path is an Angular `.component.html` template.
///
/// ANGULAR_HTML_COMPRESSION_PLAN Phase 2: these files are treated as
/// compressible when the `angular` feature is enabled, producing
/// AST-level change-sets instead of line-count deltas.
fn is_angular_template(path: &str) -> bool {
    #[cfg(feature = "angular")]
    {
        let lower = path.to_lowercase();
        lower.ends_with(".component.html")
    }
    #[cfg(not(feature = "angular"))]
    {
        let _ = path;
        false
    }
}

#[cfg(test)]
#[path = "../tests/gitdiff/engine.rs"]
mod tests;