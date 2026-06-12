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
//
// Phase 3 (module split): utility functions (file collection,
// class block extraction, manifest formatting, constants) are
// extracted to `workspace_util.rs`.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use crate::compression::Fidelity;
use crate::compression::pipeline::compress_source;
use crate::compression::workspace_symbols::build_global_symbol_table;
use crate::mcp::McpState;
use crate::angular_meta::bundler;
use crate::angular_meta::decorators;
use crate::angular_meta::footer::FooterBuilder;
use crate::angular_meta::graph::GraphCollector;
use crate::angular_meta::template;
use crate::angular_meta::style;

use super::workspace_util::{
    COMPRESSIBLE_EXTENSIONS,
    collect_source_files,
    extract_class_blocks,
    format_manifest_header,
    format_manifest_footer,
    triplet_name,
    PassContextRef,
};

/// Structured result of a workspace compression pass.
#[derive(Debug, Clone)]
pub struct WorkspaceResult {
    pub manifest: String,
    pub errors: Vec<(String, String)>,
    /// F-FINAL-04: Each entry is `(file_path, matching_patterns)` so
    /// callers can surface *why* a file was excluded.
    pub excluded: Vec<(String, Vec<String>)>,
    /// F-FINAL-06: Non-fatal warnings emitted by sub-passes (e.g.
    /// duplicate Angular class name). Surfaced via the JSON-RPC
    /// `_warnings` field so MCP clients can see them.
    pub warnings: Vec<String>,
}

/// Context shared between the three compression sub-passes.
struct PassContext {
    kept: Vec<String>,
    errors: Vec<(String, String)>,
    /// F-FINAL-04: `(file_path, matching_patterns)` per excluded file.
    excluded: Vec<(String, Vec<String>)>,
    /// F-FINAL-06: Non-fatal warnings collected by the sub-passes
    /// (e.g. duplicate Angular class name in the graph pass). The
    /// orchestrator drains these into `state.warnings` at the end
    /// of the pass so the JSON-RPC response surfaces them.
    warnings: Vec<String>,
}

/// Scan a directory, compress source files, and bundle Angular triplets.
///
/// Track D (F-ANG-15): decomposed into a 30-line orchestrator that
/// delegates to `compress_pass`, `bundle_pass`, and `graph_pass`.
///
/// Phase III (Idea #9): At Low fidelity, uses a two-pass approach
/// with global symbol deduplication for 15-30% additional savings.
pub(crate) fn compress_workspace_dir(
    dir_path: &str,
    fidelity: Fidelity,
    state: &mut McpState,
) -> Result<WorkspaceResult, Box<dyn std::error::Error>> {
    let mut manifest = format_manifest_header(dir_path, fidelity, state);

    // File collection + exclusion.
    let mut all_entries: Vec<String> = Vec::new();
    collect_source_files(dir_path, &mut all_entries);
    let mut excluded: Vec<(String, Vec<String>)> = Vec::new();
    let kept: Vec<String> = all_entries
        .into_iter()
        .filter(|p| {
            if let Some(patterns) = state.config.matching_exclude_patterns(p) {
                excluded.push((p.clone(), patterns));
                false
            } else {
                true
            }
        })
        .collect();

    let mut ctx = PassContext {
        kept,
        errors: Vec::new(),
        excluded,
        warnings: Vec::new(),
    };

    // Phase III (Idea #9): At Low fidelity, use the two-pass approach
    // with global symbol deduplication. This builds a workspace-level
    // frequency table across all files, assigns globally-optimised
    // opcodes, and emits a shared §GSYM dictionary instead of per-file
    // symbol footers.
    if fidelity == Fidelity::Low {
        compress_pass_with_global_symbols(fidelity, state, &mut ctx, &mut manifest);
    } else {
        compress_pass(fidelity, state, &mut ctx, &mut manifest);
    }

    let footer_builder = bundle_pass(state, &ctx, &mut manifest);
    graph_pass(state, &mut ctx, &mut manifest);

    // Build a PassContextRef for format_manifest_footer
    let ctx_ref = PassContextRef {
        excluded: &ctx.excluded,
        errors: &ctx.errors,
    };
    format_manifest_footer(state, &ctx_ref, footer_builder, &mut manifest);

    // F-FINAL-06: Merge the per-pass warnings collected in
    // `ctx.warnings` into the session-level buffer so the
    // `_warnings` field in the JSON-RPC response surfaces them. We
    // also drain `state.warnings` (in case any sub-system pushed
    // there directly) to ensure all sources of warnings are
    // captured in the result.
    let mut warnings = ctx.warnings;
    warnings.extend(state.drain_warnings());

    Ok(WorkspaceResult {
        manifest,
        errors: ctx.errors,
        excluded: ctx.excluded,
        warnings,
    })
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
    use crate::compression::pipeline::compress_file_with_source;

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

    for entry in &compressible {
        // Pre-read via source_cache so bundle_pass/graph_pass
        // get cache hits instead of re-reading from disk.
        let source_arc = state.read_source(entry).ok();
        let source_ref = source_arc.as_ref().map(|s| s.as_str());

        match compress_file_with_source(
            PathBuf::from(&entry),
            source_ref,
            &mut state.dict,
            &mut state.cache,
            fidelity,
        ) {
            Ok(compressed) => {
                let alias = state.dict.get_or_create_alias(entry.clone());
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

/// Two-pass compression with global symbol deduplication (Phase III, Idea #9).
///
/// Pass 1: Compress each file without symbol compression, collect bodies.
/// Pass 2: Build global symbol table, re-encode each file with shared
///         dictionary, emit global §GSYM dictionary in manifest header.
fn compress_pass_with_global_symbols(
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

    // Pass 1: Compress all files and collect the output.
    // We use `compress_source` (which skips per-file symbol compression)
    // so we can later apply the global symbol dictionary instead.
    struct CompressedEntry {
        path: String,
        alias: String,
        compressed: String,
        /// The body portion (after header, before footer) for global encoding.
        body: String,
    }

    let mut entries: Vec<CompressedEntry> = Vec::new();

    for entry in &compressible {
        // Read the source code via source_cache (Finding 1 / workspace.rs).
        let source_code = match state.read_source(entry) {
            Ok(arc) => (*arc).clone(),
            Err(e) => {
                ctx.errors.push((entry.clone(), e.to_string()));
                manifest.push_str(&format!(
                    "// ERROR reading {}: {}\n\n",
                    entry, e
                ));
                continue;
            }
        };

        // Compress without per-file symbol compression.
        match compress_source(&source_code, entry, &mut state.dict, &mut state.cache, fidelity) {
            Ok(compressed) => {
                let alias = state.dict.get_or_create_alias(entry.clone());
                // Extract the body from the compressed output.
                // The compressed output has the report header + body.
                // We need the body for global symbol encoding.
                let body = extract_body_from_compressed(&compressed);
                entries.push(CompressedEntry {
                    path: entry.clone(),
                    alias,
                    compressed,
                    body,
                });
            }
            Err(e) => {
                ctx.errors.push((entry.clone(), e.to_string()));
                manifest.push_str(&format!(
                    "// ERROR compressing {}: {}\n\n",
                    entry, e
                ));
            }
        }
    }

    if entries.is_empty() {
        return;
    }

    // Pass 2: Build global symbol table from all file bodies.
    let body_pairs: Vec<(String, String)> = entries
        .iter()
        .map(|e| (e.path.clone(), e.body.clone()))
        .collect();
    let mut global_table = build_global_symbol_table(&body_pairs);

    // Emit the global dictionary in the manifest.
    let global_footer = global_table.format_global_footer();
    if !global_footer.is_empty() {
        manifest.push_str(&global_footer);
        manifest.push('\n');
    }

    // Re-encode each file with the global dictionary.
    for entry in &entries {
        manifest.push_str(&format!(
            "// ===== FILE: {} =====\n// α alias: {}\n",
            entry.path, entry.alias
        ));

        if entry.body.is_empty() {
            // No body to encode — emit the original compressed output.
            manifest.push_str(&entry.compressed);
        } else {
            // Encode the body with global symbols.
            global_table.begin_file();
            let encoded_body = global_table.encode_body(&entry.body);
            let file_refs = global_table.format_file_refs();

            // Reconstruct the output: header + encoded body + global refs.
            // We use the original compressed output's header (everything
            // before the body starts) and append the encoded body.
            let header = extract_header_from_compressed(&entry.compressed);
            manifest.push_str(&header);
            manifest.push_str(&encoded_body);
            if !file_refs.is_empty() {
                manifest.push_str(&file_refs);
            }
        }
        manifest.push('\n');
    }
}

/// Extract the body portion from a compressed output string.
///
/// The compressed output from `compress_source` has the format:
/// ```text
/// // --- Token Optimization Report ---
/// // Raw Tokens: X | ...
/// // Fidelity: Low
/// // Structures: ...
/// // α1
/// <body>
/// ```
///
/// This function returns everything after the header (after the `// αN` line).
fn extract_body_from_compressed(compressed: &str) -> String {
    let lines = compressed.lines();
    let mut header_ended = false;
    let mut body = String::new();

    for line in lines {
        if header_ended {
            if !body.is_empty() {
                body.push('\n');
            }
            body.push_str(line);
        } else if line.starts_with("// α") || line.starts_with("// [CACHE_HIT]") {
            header_ended = true;
        }
    }

    body
}

/// Extract the header portion from a compressed output string.
///
/// Returns everything up to and including the `// αN` line.
fn extract_header_from_compressed(compressed: &str) -> String {
    let mut header = String::new();
    for line in compressed.lines() {
        header.push_str(line);
        header.push('\n');
        if line.starts_with("// α") || line.starts_with("// [CACHE_HIT]") {
            break;
        }
    }
    header
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
            // F-FINAL-01: Use the shared source cache so files already
            // read in compress_pass / graph_pass are not re-read here.
            if let Ok(content) = state.read_source(&tpl_path.to_string_lossy()) {
                let shape = template::extract_template_shape(&content);
                tpl_summary = Some(shape.to_marker_line());
            }
        }
        if let Some(ref sty_path) = triplet.style {
            let a = state.dict.get_or_create_alias(sty_path.to_string_lossy().to_string());
            file_aliases.push(a);
            // F-FINAL-01: Same shared-cache fix as the template branch.
            if let Ok(content) = state.read_source(&sty_path.to_string_lossy()) {
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
///
/// F-FINAL-06: `graph_pass` now takes `&mut PassContext` so it can
/// push non-fatal warnings (currently: duplicate Angular class
/// names from `AngularGraphBuilder`) into `ctx.warnings` for the
/// orchestrator to merge into the JSON-RPC response.
fn graph_pass(state: &mut McpState, ctx: &mut PassContext, manifest: &mut String) {
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

    let mut angular_graph = graph_collector.build_graph();

    // F-FINAL-06: Drain the graph's warnings (currently: duplicate
    // class names) into the pass-level warning buffer so the
    // orchestrator can surface them via the JSON-RPC `_warnings`
    // field. This replaces the previous `eprintln!` in the builder.
    ctx.warnings.extend(angular_graph.take_warnings());

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

#[cfg(test)]
#[path = "../tests/mcp/workspace.rs"]
mod tests;