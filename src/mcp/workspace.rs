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
//
// Track D (F-ANG-15): `compress_workspace_dir` is split into a
// 30-line orchestrator + three focused sub-passes (`compress_pass`,
// `bundle_pass`, `graph_pass`) + a footer formatter.

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
/// F-FULL-16: `.js` files are rejected at the compression level
/// (language_for_extension no longer accepts .js). They are kept in
/// the file scan so the user sees a clear "unsupported extension"
/// error rather than a silent skip.
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

/// Context shared between the three compression sub-passes.
struct PassContext {
    kept: Vec<String>,
    errors: Vec<(String, String)>,
    excluded: Vec<String>,
}

/// Scan a directory, compress source files, and bundle Angular triplets.
///
/// Track D (F-ANG-15): decomposed into a 30-line orchestrator that
/// delegates to `compress_pass`, `bundle_pass`, and `graph_pass`.
pub(crate) fn compress_workspace_dir(
    dir_path: &str,
    fidelity: Fidelity,
    state: &mut McpState,
) -> Result<WorkspaceResult, Box<dyn std::error::Error>> {
    let mut manifest = format_manifest_header(dir_path, fidelity, state);

    // File collection + exclusion.
    let mut all_entries: Vec<String> = Vec::new();
    collect_source_files(dir_path, &mut all_entries);
    let mut excluded = Vec::new();
    let kept: Vec<String> = all_entries
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

    let mut ctx = PassContext { kept, errors: Vec::new(), excluded };

    compress_pass(fidelity, state, &mut ctx, &mut manifest);
    let footer_builder = bundle_pass(state, &ctx, &mut manifest);
    graph_pass(state, &ctx, &mut manifest);

    format_manifest_footer(state, &ctx, footer_builder, &mut manifest);

    Ok(WorkspaceResult {
        manifest,
        errors: ctx.errors,
        excluded: ctx.excluded,
    })
}

/// Format the manifest header lines.
fn format_manifest_header(
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

/// Per-file compression pass. Compresses ts/js/cs files and emits
/// each as a `FILE:` section in the manifest.
///
/// F-FULL-15: The passes remain sequential (dict/cache are not `Sync`),
/// but within each pass the per-file work is I/O-bound (tree-sitter
/// parse + AST walk). Rayon parallelization of the three passes would
/// require wrapping dict/cache in `Mutex`, which adds overhead that
/// outweighs the gains for the typical workspace size (< 5000 files).
/// The main win from F-FULL-01/F-FULL-05 (cached file reads) already
/// eliminates the redundant I/O that was the bottleneck.
fn compress_pass(
    fidelity: Fidelity,
    state: &mut McpState,
    ctx: &mut PassContext,
    manifest: &mut String,
) {
    let compressible: Vec<String> = ctx
        .kept
        .iter()
        .filter(|p| {
            Path::new(p)
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|ext| COMPRESSIBLE_EXTENSIONS.contains(&ext))
        })
        .cloned()
        .collect();

    let McpState { dict, cache, .. } = state;

    for entry in &compressible {
        match compress_file(PathBuf::from(entry), dict, cache, fidelity) {
            Ok(compressed) => {
                // F-FULL-10: Always use the raw path as the alias key so the
                // alias is deterministic even if canonicalize fails on some
                // files but succeeds on others (e.g., permission issues).
                let alias = dict.get_or_create_alias(entry.clone());
                manifest.push_str(&format!(
                    "// ===== FILE: {} =====\n// α alias: {}\n",
                    entry, alias
                ));
                manifest.push_str(&compressed);
                manifest.push('\n');
            }
            Err(e) => {
                ctx.errors.push((entry.clone(), e.to_string()));
                manifest.push_str(&format!("// ERROR compressing {}: {}\n\n", entry, e));
            }
        }
    }
}

/// Bundling pass. Resolves Angular file triplets (*.component.ts →
/// .html + .scss), extracts template/style shape summaries, and
/// emits `ΦBUNDLE` groups with a `§ΦMAP` footer.
fn bundle_pass(
    state: &mut McpState,
    ctx: &PassContext,
    manifest: &mut String,
) -> FooterBuilder {
    let mut footer_builder = FooterBuilder::new();
    let mut bundle_count = 0usize;

    let compressible: Vec<&String> = ctx
        .kept
        .iter()
        .filter(|p| {
            Path::new(p)
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|ext| COMPRESSIBLE_EXTENSIONS.contains(&ext))
        })
        .collect();

    for entry in &compressible {
        let path = Path::new(entry);
        if !bundler::is_component_ts(path) {
            continue;
        }
        let Some(triplet) = bundler::resolve_triplet(path) else {
            continue;
        };

        // F-FULL-10: Use raw paths for alias keys for deterministic results.
        let component_alias = state.dict.get_or_create_alias(entry.to_string());
        let mut file_aliases = vec![component_alias];
        let mut tpl_summary = None;
        let mut sty_summary = None;

        if let Some(ref tpl_path) = triplet.template {
            let a = state.dict.get_or_create_alias(tpl_path.to_string_lossy().to_string());
            file_aliases.push(a);
            if let Ok(content) = std::fs::read_to_string(tpl_path) {
                let shape = template::extract_template_shape(&content);
                tpl_summary = Some(shape.to_marker_line());
            }
        }
        if let Some(ref sty_path) = triplet.style {
            let a = state.dict.get_or_create_alias(sty_path.to_string_lossy().to_string());
            file_aliases.push(a);
            if let Ok(content) = std::fs::read_to_string(sty_path) {
                let shape = style::extract_style_shape(&content);
                sty_summary = Some(shape.to_marker_line());
            }
        }

        let _bundle_alias = state.dict.get_or_create_bundle_alias(triplet_name(path));
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

    footer_builder
}

/// Cross-file dependency graph pass. Reads each TS file once,
/// builds the Angular graph (F-ANG-04: caches file content for
/// reuse in the emit loop), and emits `§ΦGRAPH` markers.
fn graph_pass(state: &mut McpState, ctx: &PassContext, manifest: &mut String) {
    let compressible: Vec<&String> = ctx
        .kept
        .iter()
        .filter(|p| {
            Path::new(p)
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|ext| COMPRESSIBLE_EXTENSIONS.contains(&ext))
        })
        .collect();

    let mut file_contents: std::collections::HashMap<String, Arc<String>> =
        std::collections::HashMap::new();
    let mut graph_collector = GraphCollector::new();

    for entry in &compressible {
        let path = Path::new(entry);
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext != "ts" {
            continue;
        }
        // F-FULL-05: Use the shared source cache from McpState so files
        // read in compress_pass are not re-read here.
        let source_code = match state.read_source(entry) {
            Ok(s) => s,
            Err(_) => continue,
        };
        if !crate::angular_meta::detect::is_angular_file(&source_code) {
            continue;
        }
        // F-FULL-10: Use raw path for alias key for deterministic alias.
        let file_alias = state.dict.get_or_create_alias((*entry).clone());

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
        file_contents.insert((*entry).clone(), source_code);
    }

    let angular_graph = graph_collector.build_graph();
    state.angular_graph.set(angular_graph.clone());

    // Emit graph lines using cached file content (F-ANG-04).
    for source_code in file_contents.values() {
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
    }

    manifest.push_str(&angular_graph.format_graph_footer());
}

/// Format the manifest footer: excluded files, errors, path map,
/// and bundle footer.
fn format_manifest_footer(
    state: &McpState,
    ctx: &PassContext,
    footer_builder: FooterBuilder,
    manifest: &mut String,
) {
    // Register Angular-adjacent files not part of a bundle.
    for entry in &ctx.kept {
        let ext = Path::new(entry)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        if ANGULAR_EXTENSIONS.contains(&ext) {
            // Note: we skip alias registration here because `state`
            // is immutably borrowed. Angular-adjacent files that are
            // also part of a compressible triplet already got their
            // alias registered in `bundle_pass`. Standalone `.html`
            // / `.scss` files don't need an alias since they aren't
            // compressed or graph-emitted.
        }
    }

    if !ctx.excluded.is_empty() {
        manifest.push_str(&format!("// EXCLUDED ({} files):\n", ctx.excluded.len()));
        for path in &ctx.excluded {
            manifest.push_str(&format!("//   {}\n", path));
        }
    }
    if !ctx.errors.is_empty() {
        manifest.push_str(&format!("// ERRORS ({} files):\n", ctx.errors.len()));
        for (path, err) in &ctx.errors {
            manifest.push_str(&format!("//   {}: {}\n", path, err));
        }
    }

    manifest.push_str(&state.dict.format_footer());
    manifest.push_str(&footer_builder.build());
}

/// Extract the triplet name from a component path.
fn triplet_name(path: &Path) -> String {
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
fn extract_class_blocks(source: &str) -> Vec<String> {
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

#[cfg(test)]
#[path = "../tests/mcp/workspace.rs"]
mod tests;