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
//
// Phase 2: Angular file-triplet bundling. After all compressible
// files have been compressed, a post-compression bundling pass
// resolves file triplets (*.component.ts → .html + .scss),
// extracts template/style shape summaries, and emits ΦBUNDLE
// groups with a §ΦMAP footer.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use crate::compressor::compress_file;
use crate::compression::Fidelity;
use crate::mcp::McpState;
use crate::angular_meta::bundler;
use crate::angular_meta::footer::FooterBuilder;
use crate::angular_meta::template;
use crate::angular_meta::style;

/// F-17: maximum directory recursion depth.
const MAX_WALK_DEPTH: usize = 32;

/// File extensions eligible for direct compression.
const COMPRESSIBLE_EXTENSIONS: &[&str] = &["ts", "js", "cs"];

/// Angular-adjacent file extensions for shape extraction (not compressed).
const ANGULAR_EXTENSIONS: &[&str] = &["html", "scss", "css", "sass", "less"];

/// Structured result of a workspace compression pass.
#[derive(Debug, Clone)]
pub struct WorkspaceResult {
    pub manifest: String,
    pub errors: Vec<(String, String)>,
    pub excluded: Vec<String>,
}

/// Scan a directory, compress source files, and bundle Angular triplets.
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

    collect_source_files(dir_path, &mut entries);

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

    // Separate compressible from Angular-adjacent files.
    let mut compressible: Vec<String> = Vec::new();
    let mut angular_files: Vec<String> = Vec::new();
    for entry in &kept {
        let ext = Path::new(entry)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        if COMPRESSIBLE_EXTENSIONS.contains(&ext) {
            compressible.push(entry.clone());
        } else if ANGULAR_EXTENSIONS.contains(&ext) {
            angular_files.push(entry.clone());
        }
    }

    let McpState { dict, cache, config: _ } = state;

    // Compress compressible files (ts/js/cs).
    for entry in &compressible {
        match compress_file(PathBuf::from(entry), dict, cache, fidelity) {
            Ok(compressed) => {
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

    // Phase 2: Bundling pass.
    let mut footer_builder = FooterBuilder::new();
    let mut bundle_count = 0usize;

    for entry in &compressible {
        let path = Path::new(entry);
        if !bundler::is_component_ts(path) {
            continue;
        }

        let Some(triplet) = bundler::resolve_triplet(path) else {
            continue;
        };

        let component_abs = std::fs::canonicalize(entry)
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|_| entry.clone());
        let component_alias = dict.get_or_create_alias(component_abs);

        let mut file_aliases = vec![component_alias];
        let mut tpl_summary = None;
        let mut sty_summary = None;

        if let Some(ref tpl_path) = triplet.template {
            if let Ok(tpl_abs) = std::fs::canonicalize(tpl_path) {
                let a = dict.get_or_create_alias(tpl_abs.to_string_lossy().into_owned());
                file_aliases.push(a);
            }
            if let Ok(content) = std::fs::read_to_string(tpl_path) {
                let shape = template::extract_template_shape(&content);
                tpl_summary = Some(shape.to_marker_line());
            }
        }

        if let Some(ref sty_path) = triplet.style {
            if let Ok(sty_abs) = std::fs::canonicalize(sty_path) {
                let a = dict.get_or_create_alias(sty_abs.to_string_lossy().into_owned());
                file_aliases.push(a);
            }
            if let Ok(content) = std::fs::read_to_string(sty_path) {
                let shape = style::extract_style_shape(&content);
                sty_summary = Some(shape.to_marker_line());
            }
        }

        let _bundle_alias = dict.get_or_create_bundle_alias(triplet_name(path));

        bundle_count += 1;
        manifest.push_str(&format!(
            "// ===== Φ{}: {} =====\n",
            bundle_count,
            triplet_name(path),
        ));
        manifest.push_str(&format!("// files: {}\n", file_aliases.join(", ")));
        if let Some(ref t) = tpl_summary {
            manifest.push_str(&format!("// {}\n", t));
        }
        if let Some(ref s) = sty_summary {
            manifest.push_str(&format!("// {}\n", s));
        }
        manifest.push('\n');

        footer_builder.register_bundle(
            triplet_name(path),
            file_aliases,
            tpl_summary,
            sty_summary,
        );
    }

    // Register Angular-adjacent files not part of a bundle as α-aliases.
    for entry in &angular_files {
        let abs = std::fs::canonicalize(entry)
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|_| entry.clone());
        dict.get_or_create_alias(abs);
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

    manifest.push_str(&dict.format_footer());
    manifest.push_str(&footer_builder.build());

    Ok(WorkspaceResult {
        manifest,
        errors,
        excluded,
    })
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

/// Extract the triplet name from a component path.
fn triplet_name(path: &Path) -> String {
    path.file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string()
}

#[cfg(test)]
#[path = "../tests/mcp/workspace.rs"]
mod tests;