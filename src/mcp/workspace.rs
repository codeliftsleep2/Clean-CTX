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

use std::path::PathBuf;
use crate::compressor::compress_file;
use crate::compression::Fidelity;
use crate::mcp::McpState;

/// Scan a directory for .ts/.cs files and compress each one.
///
/// F-05: the per-file compressions now share the path dictionary
/// and cache in `state`, and the config's `exclude_patterns` are
/// consulted per entry. Per-file errors are collected into a
/// structured `errors` field instead of being inlined as
/// `// ERROR` comments.
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
) -> Result<String, Box<dyn std::error::Error>> {
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
                manifest.push_str(&format!("// ===== FILE: {} =====\n", entry));
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

    Ok(manifest)
}

/// Recursively collect .ts and .cs files from a directory.
pub(crate) fn collect_source_files(dir: &str, entries: &mut Vec<String>) {
    if let Ok(read_dir) = std::fs::read_dir(dir) {
        for entry in read_dir.flatten() {
            let path = entry.path();
            let name = path.file_name().unwrap_or_default().to_string_lossy();

            // Skip hidden dirs, node_modules, target, etc.
            if name.starts_with('.') || name == "node_modules" || name == "target" || name == "dist" {
                continue;
            }

            if path.is_dir() {
                collect_source_files(&path.to_string_lossy(), entries);
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
