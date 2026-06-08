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
use std::sync::Arc;
use crate::compressor::compress_file;
use crate::compression::Fidelity;
use crate::mcp::McpState;
use crate::angular_meta::bundler;
use crate::angular_meta::decorators;
use crate::angular_meta::footer::FooterBuilder;
use crate::angular_meta::graph::GraphCollector;
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

    let McpState { dict, cache, config: _, angular_graph: _ } = state;

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

    // Phase 3: Cross-file dependency graph.
    // After all files are compressed and bundled, build the Angular
    // graph from raw source text, resolving DI and selector linkages.
    // F-ANG-04: read each TS file ONCE and cache the content in
    // `file_contents` so the graph-build pass and the graph-emit
    // pass share the read. The previous code did the read twice.
    let mut file_contents: std::collections::HashMap<String, Arc<String>> =
        std::collections::HashMap::new();
    let mut graph_collector = GraphCollector::new();
    for entry in &compressible {
        let path = Path::new(entry);
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext != "ts" {
            continue;
        }
        let source_code = match std::fs::read_to_string(path) {
            Ok(s) => Arc::new(s),
            Err(_) => continue,
        };
        if !crate::angular_meta::detect::is_angular_file(&source_code) {
            continue;
        }
        let file_alias = {
            let abs = std::fs::canonicalize(path)
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|_| entry.clone());
            dict.get_or_create_alias(abs)
        };

        // Extract class captures using text-based approach: find each
        // `class Name {` block with its preceding decorators.
        let class_captures: Vec<String> = extract_class_blocks(&source_code);
        for raw_class in &class_captures {
            if let Some((class_name, kind, selector, injects, pipe_name)) =
                decorators::extract_graph_entries(raw_class)
            {
                graph_collector.push(
                    &class_name,
                    &file_alias,
                    kind,
                    selector.as_deref(),
                    &injects,
                    pipe_name.as_deref(),
                );
            }
        }
        file_contents.insert(entry.clone(), source_code);
    }
    let angular_graph = graph_collector.build_graph();
    state.angular_graph.set(angular_graph.clone());

    // Emit graph lines in manifest for each compressible file.
    // Reuses the cached content from `file_contents` (F-ANG-04).
    for (entry, source_code) in &file_contents {
        let class_captures: Vec<String> = extract_class_blocks(source_code);
        for raw_class in &class_captures {
            if let Some((class_name, _, _, _, _)) =
                decorators::extract_graph_entries(raw_class)
            {
                if let Some(graph_line) = angular_graph.format_graph_line(&class_name) {
                    manifest.push_str(&format!("// {}\n", graph_line));
                }
            }
        }
        let _ = entry; // silence unused warning if entry is only used as key
    }

    // Append the §ΦGRAPH footer section.
    manifest.push_str(&angular_graph.format_graph_footer());

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

/// Textually extract class declaration blocks from TypeScript source
/// code, including any leading decorators.
///
/// Each block spans from the first `@` (or `class` if no decorator)
/// through the closing `}` of the class body. This is a lightweight
/// text scanner that mirrors the approach used in `decorators.rs`.
///
/// Used by Phase 3 (cross-file graph) to feed class text into
/// `decorators::extract_graph_entries`.
fn extract_class_blocks(source: &str) -> Vec<String> {
    let mut blocks: Vec<String> = Vec::new();
    let bytes = source.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        // Find the start of a potential class declaration.
        // We look for either `@` (decorator) or `class ` (bare class).
        let mut start = None;

        // Scan for `@` decorators or `class ` keyword.
        if bytes[i] == b'@' {
            start = Some(i);
            // Consume the decorator chain: skip to after the `@Name(...)`.
            let mut depth = 0i32;
            let mut j = i;
            while j < len {
                match bytes[j] {
                    b'(' => depth += 1,
                    b')' => {
                        depth -= 1;
                        if depth < 0 {
                            break;
                        }
                    }
                    b'@' if depth == 0 && j != i => {
                        // Multiple decorators: continue scanning.
                    }
                    _ => {}
                }
                j += 1;
                if depth == 0 && (bytes[j - 1] == b')' || bytes[j - 1] == b'\n') {
                    // Check if next non-whitespace is `@` or `class`.
                    let mut k = j;
                    while k < len && (bytes[k] == b' ' || bytes[k] == b'\t' || bytes[k] == b'\n') {
                        k += 1;
                    }
                    if k < len && bytes[k] == b'@' {
                        i = k;
                        break;
                    }
                    if k < len && source[k..].starts_with("class ") {
                        start = Some(i);
                        i = k;
                        break;
                    }
                    // Not followed by decorator or class — skip ahead.
                    i = j;
                    break;
                }
            }
            if i > start.unwrap_or(0) {
                continue;
            }
        }

        if start.is_none() && source[i..].starts_with("class ") {
            start = Some(i);
        }

        let block_start = match start {
            Some(s) => s,
            None => {
                i += 1;
                continue;
            }
        };

        // Now find the opening `{` of the class body.
        // Scan from `class` keyword forward, tracking brace depth
        // so decorator braces are skipped (depth starts at 0).
        let class_pos = source[block_start..].find("class ").map(|p| block_start + p);
        let class_pos = match class_pos {
            Some(p) => p,
            None => {
                i = block_start + 1;
                continue;
            }
        };

        // Find opening `{` of the class body (depth 0, skipping decorator braces).
        let mut depth = 0i32;
        let mut open_brace = None;
        let mut j = class_pos + 6; // after "class "
        while j < len {
            match bytes[j] {
                b'{' => {
                    if depth == 0 {
                        open_brace = Some(j);
                        break;
                    }
                    depth += 1;
                }
                b'}' => depth -= 1,
                b'"' | b'\'' => {
                    let quote = bytes[j];
                    j += 1;
                    while j < len && bytes[j] != quote {
                        if bytes[j] == b'\\' && j + 1 < len {
                            j += 2;
                        } else {
                            j += 1;
                        }
                    }
                }
                b'`' => {
                    j += 1;
                    while j < len && bytes[j] != b'`' {
                        if bytes[j] == b'\\' && j + 1 < len {
                            j += 2;
                        } else {
                            j += 1;
                        }
                    }
                }
                _ => {}
            }
            j += 1;
        }

        match open_brace {
            Some(ob) => {
                // Find matching close brace.
                let close = find_matching_brace_text(source, ob);
                let block_end = close + 1;
                blocks.push(source[block_start..block_end].to_string());
                i = block_end;
            }
            None => {
                i = class_pos + 6;
            }
        }
    }

    blocks
}

/// Find the byte offset of the `}` matching the `{` at `open_brace`.
fn find_matching_brace_text(text: &str, open_brace: usize) -> usize {
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut depth = 0i32;
    let mut i = open_brace;
    while i < len {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return i;
                }
            }
            b'"' | b'\'' => {
                let quote = bytes[i];
                i += 1;
                while i < len && bytes[i] != quote {
                    if bytes[i] == b'\\' && i + 1 < len {
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
            }
            b'`' => {
                i += 1;
                while i < len && bytes[i] != b'`' {
                    if bytes[i] == b'\\' && i + 1 < len {
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }
    len.saturating_sub(1)
}

#[cfg(test)]
#[path = "../tests/mcp/workspace.rs"]
mod tests;