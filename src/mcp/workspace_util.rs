// src/mcp/workspace_util.rs
//
// Utility functions extracted from workspace.rs during Phase 3
// module split. Contains file collection, class block extraction,
// manifest formatting helpers, and shared constants.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::angular_meta::decorators;
use crate::compression::Fidelity;
use crate::mcp::McpState;
use crate::angular_meta::footer::FooterBuilder;

/// F-17: maximum directory recursion depth.
pub(crate) const MAX_WALK_DEPTH: usize = 32;

/// File extensions eligible for direct compression.
/// F-FULL-16: `.js` files are rejected at the compression level
/// (language_for_extension no longer accepts .js). They are kept in
/// the file scan so the user sees a clear "unsupported extension"
/// error rather than a silent skip.
pub(crate) const COMPRESSIBLE_EXTENSIONS: &[&str] = &["ts", "js", "cs"];

/// Angular-adjacent file extensions for shape extraction (not compressed).
pub(crate) const ANGULAR_EXTENSIONS: &[&str] = &["html", "scss", "css", "sass", "less"];

/// Format the manifest header lines.
pub(crate) fn format_manifest_header(
    dir_path: &str,
    fidelity: Fidelity,
    state: &McpState,
) -> String {
    let mut m = String::new();
    m.push_str("// Clean-CTX Workspace Manifest\n");
    m.push_str(&format!("// Directory: {}\n", dir_path));
    m.push_str(&format!("// Fidelity: {:?}\n", fidelity));
    m.push_str(&format!(
        "// Config: {} exclude patterns, {} fidelity overrides\n",
        state.config.exclude_patterns.len(),
        state.config.fidelity_overrides.len(),
    ));
    m.push('\n');
    m
}

/// Format the manifest footer: excluded files, errors, path map,
/// and bundle footer.
pub(crate) fn format_manifest_footer(
    state: &McpState,
    ctx: &PassContextRef,
    footer_builder: FooterBuilder,
    manifest: &mut String,
) {
    // MED-05: Angular-adjacent files (*.html, *.scss, *.css) that are part
    // of a compressible triplet already got their alias registered in
    // `bundle_pass`. Standalone uncompressible files don't need registry
    // entries since they are neither compressed nor graph-emitted.
    // The previous dead loop over ctx.kept has been removed — it performed
    // no work because `state` is immutably borrowed here.

    if !ctx.excluded.is_empty() {
        manifest.push_str(&format!("// EXCLUDED ({} files):\n", ctx.excluded.len()));
        for (path, patterns) in ctx.excluded {
            // F-FINAL-04: surface *which* pattern(s) excluded this file
            // so the user can debug a misconfigured exclude list.
            manifest.push_str(&format!(
                "//   {} [matched: {}]\n",
                path,
                patterns.join(", "),
            ));
        }
    }
    if !ctx.errors.is_empty() {
        manifest.push_str(&format!("// ERRORS ({} files):\n", ctx.errors.len()));
        for (path, err) in ctx.errors {
            manifest.push_str(&format!("//   {}: {}\n", path, err));
        }
    }

    manifest.push_str(&state.dict.format_footer());
    manifest.push_str(&footer_builder.build());
}

/// Minimal reference to PassContext fields needed by format_manifest_footer.
/// Avoids a circular dependency on the full PassContext struct.
pub(crate) struct PassContextRef<'a> {
    pub excluded: &'a [(String, Vec<String>)],
    pub errors: &'a [(String, String)],
}

/// Extract the triplet name from a component path.
pub(crate) fn triplet_name(path: &Path) -> String {
    path.file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string()
}

/// Recursively collect source files from a directory.
pub(crate) fn collect_source_files(dir: &str, entries: &mut Vec<String>) {
    let mut visited = HashSet::new();
    collect_source_files_inner(dir, entries, &mut visited, 0);
}

fn collect_source_files_inner(
    dir: &str,
    entries: &mut Vec<String>,
    visited: &mut HashSet<PathBuf>,
    depth: usize,
) {
    if depth > MAX_WALK_DEPTH {
        return;
    }

    let dir_path = Path::new(dir);

    let canonical = match std::fs::canonicalize(dir_path) {
        Ok(p) => p,
        Err(_) => return,
    };
    if !visited.insert(canonical) {
        return;
    }

    if let Ok(read_dir) = std::fs::read_dir(dir_path) {
        for entry in read_dir.flatten() {
            let path = entry.path();
            let name = path.file_name().unwrap_or_default().to_string_lossy();

            if name.starts_with('.')
                || name == "node_modules"
                || name == "target"
                || name == "dist"
            {
                continue;
            }

            if path.is_dir() {
                if let Ok(child_canonical) = std::fs::canonicalize(&path)
                    && visited.contains(&child_canonical)
                {
                    continue;
                }
                collect_source_files_inner(
                    &path.to_string_lossy(),
                    entries,
                    visited,
                    depth + 1,
                );
            } else if path.is_file() {
                let ext = path.extension().unwrap_or_default().to_string_lossy();
                if COMPRESSIBLE_EXTENSIONS.contains(&ext.as_ref())
                    || ANGULAR_EXTENSIONS.contains(&ext.as_ref())
                {
                    entries.push(path.to_string_lossy().into_owned());
                }
            }
        }
    }
}

// --- F-ANG-03: extract_class_blocks rewrite ---
//
// The previous 137-line function duplicated the brace-matching
// state machine from `decorators.rs`. This rewrite delegates to
// the Track A-promoted helpers (`find_class_body_open`,
// `find_matching_brace`) and is ~20 lines.

/// Textually extract class declaration blocks from TypeScript source
/// code, including any leading decorators.
///
/// Each block spans from the first `@` (or `class` if no decorator)
/// through the closing `}` of the class body. This is a lightweight
/// text scanner that mirrors the approach used in `decorators.rs`.
///
/// Used by Phase 3 (cross-file graph) to feed class text into
/// `decorators::extract_graph_entries`.
pub(crate) fn extract_class_blocks(source: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut cursor = 0;
    // F-FULL-06: Guard against infinite loop on degenerate input
    // (e.g., unterminated class with repeated `class` keyword).
    // If we've advanced more times than the source length, break.
    let source_len = source.len();
    let mut iterations = 0usize;
    while let Some(class_pos) = find_next_class_keyword(&source[cursor..]) {
        iterations += 1;
        if iterations > source_len.saturating_add(1) {
            break;
        }
        let abs = cursor + class_pos;
        // Look backwards for decorator start.
        let block_start = find_decorator_start(source, abs);
        if let Some(open) = decorators::find_class_body_open(&source[block_start..]) {
            let abs_open = block_start + open;
            if let Some(close) = decorators::find_matching_brace(source, abs_open) {
                blocks.push(source[block_start..=close].to_string());
                cursor = close + 1;
                continue;
            }
        }
        cursor = abs + 6;
    }
    blocks
}

/// Find the next occurrence of `class ` (with trailing space) in `text`.
fn find_next_class_keyword(text: &str) -> Option<usize> {
    text.find("class ")
}

/// Scan backwards from `class_pos` to find a preceding `@` decorator.
/// Returns the start of the block (decorator `@` position, or
/// `class_pos` if no decorator found). Handles TypeScript modifier
/// keywords (`export`, `abstract`, `default`, `declare`) that may
/// appear between the decorator and the class keyword.
fn find_decorator_start(source: &str, class_pos: usize) -> usize {
    let bytes = source.as_bytes();
    let mut i = class_pos;

    // Skip backwards through whitespace.
    while i > 0 && bytes[i - 1].is_ascii_whitespace() {
        i -= 1;
    }

    // Check for modifier keywords before class (export, abstract, etc.).
    let word_end = i;
    while i > 0 && (bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'_') {
        i -= 1;
    }
    let word = &source[i..word_end];
    if matches!(word, "export" | "abstract" | "default" | "declare") {
        // Skip whitespace before the modifier.
        while i > 0 && bytes[i - 1].is_ascii_whitespace() {
            i -= 1;
        }
    } else {
        i = word_end; // Not a modifier, restore position.
    }

    // If we're at ')' (end of decorator call), find matching '@'.
    if i > 0 && bytes[i - 1] == b')' {
        let mut depth = 0i32;
        let mut j = i - 1;
        loop {
            match bytes[j] {
                b')' => depth += 1,
                b'(' => {
                    depth -= 1;
                    if depth == 0 {
                        // Scan backwards through the decorator name to find '@'.
                        let mut k = j;
                        while k > 0 && (bytes[k - 1].is_ascii_alphanumeric()
                            || bytes[k - 1] == b'_'
                            || bytes[k - 1] == b'$')
                        {
                            k -= 1;
                        }
                        if k > 0 && bytes[k - 1] == b'@' {
                            return k - 1;
                        }
                    }
                }
                _ => {}
            }
            if j == 0 { break; }
            j -= 1;
        }
    }

    class_pos
}