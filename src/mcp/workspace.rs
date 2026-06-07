// src/mcp/workspace.rs
//
// Workspace-level operations: scanning directories and compressing all files.
//
// F-05 (FAANG audit): the function used to construct a fresh
// `PathDictionary` and `LocalStateCache` per call and ignore the
// project config entirely. It now takes `&mut McpState`, which
// bundles all three — so the per-file path aliases are shared with
// the `compress_code_context` tool, the cache survives between
// calls, and `is_excluded` filters out files the user has
// configured to skip.
//
// F-09/F-13: the workspace result is now a structured
// [`WorkspaceResult`] instead of a bare `String`, and per-file
// alias cross-references are emitted in the manifest.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use crate::compressor::compress_file;
use crate::compression::Fidelity;
use crate::mcp::McpState;

/// F-17 (FAANG audit): maximum directory recursion depth for workspace
/// traversal. Prevents stack overflow on pathological directory trees.
const MAX_WALK_DEPTH: usize = 32;

/// Structured result of a workspace compression pass.
///
/// F-13 (FAANG audit): errors used to be inlined as `// ERROR`
/// comments inside the manifest string. They are now surfaced as
/// a separate `errors` field so MCP clients can programmatically
/// inspect failures without parsing comments.
#[derive(Debug, Clone)]
pub struct WorkspaceResult {
    /// The full manifest string with per-file compressed output,
    /// exclusion report, and path-alias footer.
    pub manifest: String,
    /// Per-file errors encountered during compression.
    pub errors: Vec<(String, String)>,
    /// Files skipped due to config exclusion patterns.
    pub excluded: Vec<String>,
}

/// Scan a directory for .ts/.cs files and compress each one.
///
/// F-05: the per-file compressions now share the path dictionary
/// and cache in `state`, and the config's `exclude_patterns` are
/// consulted per entry. Per-file errors are collected into a
/// structured `errors` field instead of being inlined as
/// `// ERROR` comments.
///
/// F-09: per-file alias cross-references (`// α1 = /path/to/file.ts`)
/// are emitted in the manifest so LLM clients can correlate the
/// per-file block with the global path map.
///
/// Borrow-checker note: the function destructures `state` to split
/// the dict and cache into two independent `&mut` references. The
/// `.dict_mut()` / `.cache_mut()` accessor pattern would not
/// compile here because Rust treats the two `state.field_mut()`
/// calls as overlapping mutable borrows of the same struct.
pub(crate) fn compress_workspace_dir(
    dir_path: &str,
    fidelity: Fidelity,
    state: &mut McpState,
) -> Result<WorkspaceResult, Box<dyn std::error::Error>> {
    let mut manifest = String::new();
    manifest.push_str("// Clean-CTX Workspace Manifest\n");
    manifest.push_str(&format!("// Directory: {}\n", dir_path));
    manifest.push_str(&format!("// Fidelity: {:?}\n", fidelity));
    manifest.push_str(&format!(
        "// Config: {} exclude patterns, {} fidelity overrides\n",
        state.config.exclude_patterns.len(),
        state.config.fidelity_overrides.len(),
    ));
    manifest.push('\n');

    let mut entries: Vec<String> = Vec::new();
    let mut excluded: Vec<String> = Vec::new();
    let mut errors: Vec<(String, String)> = Vec::new();

    // Collect all .ts and .cs files recursively.
    collect_source_files(dir_path, &mut entries);

    // Apply the user's exclude patterns up front so the manifest
    // can report the skip count and the per-file exclusions are
    // not silently absorbed into "ERROR ..." comments.
    let kept: Vec<String> = entries
        .into_iter()
        .filter(|p| {
            if state.config.is_excluded(p) {
                excluded.push(p.clone());
                false
            } else {
                true
            }
        })
        .collect();

    // Destructure to split-borrow dict and cache out of `state`
    // so we can call `compress_file` with both. (Rust's borrow
    // checker treats two `state.field_mut()` calls in the same
    // expression as overlapping mutable borrows; destructuring
    // gives us two independent `&mut` references.)
    let McpState { dict, cache, config: _ } = state;

    for entry in &kept {
        match compress_file(PathBuf::from(entry), dict, cache, fidelity) {
            Ok(compressed) => {
                // F-09: emit the per-file alias cross-reference so
                // LLM clients can correlate the block with the
                // path map footer.
                let absolute = std::fs::canonicalize(entry)
                    .map(|p| p.to_string_lossy().into_owned())
                    .unwrap_or_else(|_| entry.clone());
                let alias = dict.get_or_create_alias(absolute);
                manifest.push_str(&format!(
                    "// ===== FILE: {} =====\n// α alias: {}\n",
                    entry, alias
                ));
                manifest.push_str(&compressed);
                manifest.push('\n');
            }
            Err(e) => {
                errors.push((entry.clone(), e.to_string()));
                manifest.push_str(&format!("// ERROR compressing {}: {}\n\n", entry, e));
            }
        }
    }

    if !excluded.is_empty() {
        manifest.push_str(&format!("// EXCLUDED ({} files):\n", excluded.len()));
        for path in &excluded {
            manifest.push_str(&format!("//   {}\n", path));
        }
    }

    if !errors.is_empty() {
        manifest.push_str(&format!("// ERRORS ({} files):\n", errors.len()));
        for (path, err) in &errors {
            manifest.push_str(&format!("//   {}: {}\n", path, err));
        }
    }

    // Append the global path map.
    manifest.push_str(&dict.format_footer());

    Ok(WorkspaceResult {
        manifest,
        errors,
        excluded,
    })
}

/// Recursively collect .ts and .cs files from a directory.
///
/// F-17 (FAANG audit): tracks visited canonical paths to detect
/// symlink loops, and enforces a maximum recursion depth to prevent
/// stack overflow on pathological directory trees.
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

    // F-17: canonicalize the directory and check for loops.
    let canonical = match std::fs::canonicalize(dir_path) {
        Ok(p) => p,
        Err(_) => return, // inaccessible directory, skip silently
    };
    if !visited.insert(canonical) {
        return; // symlink loop detected
    }

    if let Ok(read_dir) = std::fs::read_dir(dir_path) {
        for entry in read_dir.flatten() {
            let path = entry.path();
            let name = path.file_name().unwrap_or_default().to_string_lossy();

            // Skip hidden dirs, node_modules, target, etc.
            if name.starts_with('.')
                || name == "node_modules"
                || name == "target"
                || name == "dist"
            {
                continue;
            }

            if path.is_dir() {
                // F-17: canonicalize the child dir; if it's a symlink
                // that points to an already-visited location, skip it.
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
                if ext == "ts" || ext == "js" || ext == "cs" {
                    entries.push(path.to_string_lossy().into_owned());
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "../tests/mcp/workspace.rs"]
mod tests;
