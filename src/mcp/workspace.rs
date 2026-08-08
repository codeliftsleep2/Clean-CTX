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
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::SystemTime;
use crate::compression::Fidelity;
use crate::compression::pipeline::compress_source;
use crate::compression::workspace_symbols::build_global_symbol_table;
use crate::mcp::McpState;

// Phase 5: Angular meta-layer imports are gated by the `angular` feature.
// When disabled, these modules are not compiled and the bundling/graph
// passes become no-ops.
#[cfg(feature = "angular")]
use std::sync::Arc;
#[cfg(feature = "angular")]
use crate::angular_meta::bundler;
#[cfg(feature = "angular")]
use crate::angular_meta::decorators;
#[cfg(feature = "angular")]
use crate::angular_meta::footer::FooterBuilder;
#[cfg(feature = "angular")]
use crate::angular_meta::graph::GraphCollector;
#[cfg(feature = "angular")]
use crate::angular_meta::template;
#[cfg(feature = "angular")]
use crate::angular_meta::style;

use super::workspace_util::{
    COMPRESSIBLE_EXTENSIONS,
    collect_source_files,
    format_manifest_header,
};
#[cfg(feature = "angular")]
use super::workspace_util::{extract_class_blocks, triplet_name, PassContextRef};

// format_manifest_footer is only available when angular is enabled
#[cfg(feature = "angular")]
use super::workspace_util::format_manifest_footer;

/// F-22: Workspace compression result cache.
///
/// Computes a content hash from file paths + mtimes/sizes. If the hash
/// matches a previous call, returns the cached `WorkspaceResult` instantly
/// without re-compressing.
///
/// Invalidation: any file change (new, modified, deleted) produces a
/// different hash → cache miss.
struct WorkspaceCache {
    /// Maps content hash → serialized WorkspaceResult
    cache: std::collections::HashMap<u64, WorkspaceResult>,
}

impl WorkspaceCache {
    fn new() -> Self {
        Self {
            cache: std::collections::HashMap::new(),
        }
    }

    /// Compute a content hash from directory path, fidelity, file paths and their metadata.
    /// This is used as the cache key — any file change or fidelity switch produces a different hash.
    fn compute_hash(dir_path: &str, fidelity: &Fidelity, entries: &[String]) -> u64 {
        let mut hasher = DefaultHasher::new();
        dir_path.hash(&mut hasher);
        // Include fidelity in the hash so different fidelity levels don't collide
        format!("{:?}", fidelity).hash(&mut hasher);
        
        // Sort for determinism
        let mut sorted = entries.to_vec();
        sorted.sort();
        
        for entry in &sorted {
            entry.hash(&mut hasher);
            // Add mtime and size to the hash so file modifications invalidate the cache
            if let Ok(meta) = std::fs::metadata(entry) {
                if let Ok(mtime) = meta.modified() {
                    if let Ok(d) = mtime.duration_since(SystemTime::UNIX_EPOCH) {
                        d.as_secs().hash(&mut hasher);
                        d.subsec_nanos().hash(&mut hasher);
                    }
                }
                meta.len().hash(&mut hasher);
            }
        }
        
        hasher.finish()
    }

    fn get(&self, hash: u64) -> Option<&WorkspaceResult> {
        self.cache.get(&hash)
    }

    fn set(&mut self, hash: u64, result: WorkspaceResult) {
        self.cache.insert(hash, result);
    }
}

/// Thread-safe holder for the workspace cache.
static WORKSPACE_CACHE: std::sync::Mutex<Option<WorkspaceCache>> = std::sync::Mutex::new(None);

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
    state: &McpState,
) -> Result<WorkspaceResult, Box<dyn std::error::Error>> {
    let _span = tracing::info_span!(
        "compress_workspace",
        dir_path = %dir_path,
        fidelity = %format!("{:?}", fidelity),
    ).entered();
    let overall_start = std::time::Instant::now();

    // Phase 4: Resolve relative `dir_path` against the caller's CWD, not
    // the process-global project root. This enables cross-repo workspace
    // compression in workspace-mode setups (e.g. `fe/src` from a parent
    // `Outcomes/` workspace root). Absolute paths pass through unchanged.
    let resolved_dir = if Path::new(dir_path).is_absolute() {
        dir_path.to_string()
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(dir_path).to_string_lossy().into_owned())
            .unwrap_or_else(|_| dir_path.to_string())
    };

    // F-22: Check workspace result cache before doing any work.
    // We collect entries first to compute the cache key, but if we get
    // a cache hit, we skip the entire compression pipeline.
    let mut all_entries: Vec<String> = Vec::new();
    collect_source_files(&resolved_dir, &mut all_entries);
    let file_count = all_entries.len();
    let cache_hash = WorkspaceCache::compute_hash(&resolved_dir, &fidelity, &all_entries);
    
    if let Ok(guard) = WORKSPACE_CACHE.lock() {
        if let Some(ref cache) = *guard {
            if let Some(cached) = cache.get(cache_hash) {
                #[cfg(debug_assertions)]
                eprintln!("[compress_workspace_dir] Cache HIT for {} ({} files)", resolved_dir, all_entries.len());
                tracing::info!(file_count = file_count, cached = true, "compress_workspace cache hit");
                return Ok(cached.clone());
            }
        }
    }

    #[cfg(debug_assertions)]
    eprintln!("[compress_workspace_dir] Cache MISS for {} ({} files)", resolved_dir, all_entries.len());

    let mut manifest = format_manifest_header(&resolved_dir, fidelity, state);

    // File collection + exclusion.
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

    // A-13: Check workspace file count against configured limit
    let file_count = kept.len() + excluded.len();
    if let Err(e) = state.config.resource_limits.check_workspace_file_count(file_count) {
        return Err(e.into());
    }

    let mut ctx = PassContext {
        kept,
        errors: Vec::new(),
        excluded,
        warnings: Vec::new(),
    };

    // A-13: Estimate memory usage and check against limit
    // Rough estimate: each file will need ~2x its size in memory during compression
    // (source + compressed output + AST structures)
    let estimated_memory: usize = ctx
        .kept
        .iter()
        .filter_map(|path| {
            std::fs::metadata(path)
                .ok()
                .map(|meta| (meta.len() * 2) as usize)
        })
        .sum();
    
    if let Err(e) = state.config.resource_limits.check_memory_usage(estimated_memory) {
        return Err(e.into());
    }

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

    // Angular-specific passes are only available when the feature is enabled
    #[cfg(feature = "angular")]
    {
        let footer_builder = bundle_pass(state, &ctx, &mut manifest);
        graph_pass(state, &mut ctx, &mut manifest);

        // Build a PassContextRef for format_manifest_footer
        let ctx_ref = PassContextRef {
            excluded: &ctx.excluded,
            errors: &ctx.errors,
        };
        format_manifest_footer(state, &ctx_ref, footer_builder, &mut manifest);
    }

    // F-FINAL-06: Merge the per-pass warnings collected in
    // `ctx.warnings` into the session-level buffer so the
    // `_warnings` field in the JSON-RPC response surfaces them. We
    // also drain `state.warnings` (in case any sub-system pushed
    // there directly) to ensure all sources of warnings are
    // captured in the result.
    let mut warnings = ctx.warnings;
    warnings.extend(state.drain_warnings());

    let result = WorkspaceResult {
        manifest,
        errors: ctx.errors,
        excluded: ctx.excluded,
        warnings,
    };

    let total_ms = overall_start.elapsed().as_millis() as u64;
    tracing::info!(
        dir_path = %dir_path,
        file_count = file_count,
        total_ms = total_ms,
        errors = result.errors.len(),
        excluded = result.excluded.len(),
        "compress_workspace complete"
    );

    // F-22: Store result in cache for future calls
    if let Ok(mut guard) = WORKSPACE_CACHE.lock() {
        if guard.is_none() {
            *guard = Some(WorkspaceCache::new());
        }
        if let Some(ref mut cache) = *guard {
            cache.set(cache_hash, result.clone());
        }
    }

    Ok(result)
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
    state: &McpState,
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

    #[cfg(debug_assertions)]
    eprintln!("[compress_pass] Starting compression of {} files", compressible.len());

    // C-1 fix: Create tokenizer once before the loop instead of per-file.
    let ws_tok = crate::tokenizer::create_tokenizer(
        crate::tokenizer::resolve_tokenizer_kind(None, Some(&state.config.tokenizer.to_string()))
    ).ok();
    let ws_tok_ref: Option<&dyn crate::tokenizer::Tokenizer> = ws_tok.as_deref();

    // F-21: Pre-assign aliases deterministically.
    // Before parallel work, iterate all files sequentially to assign α1, α2…αN
    // in a stable order. Once assigned, `get_or_create_alias` is a read-only
    // HashMap lookup (no mutation), safe for parallel calls.
    {
        let mut dict_guard = state.dict_lock();
        for entry in &compressible {
            dict_guard.get_or_create_alias(entry.clone());
        }
    }

    // F-20: Parallelize per-file compression with Rayon.
    // Alias lookups are now deterministic (pre-assigned above).
    use std::sync::Mutex;
    use rayon::prelude::*;
    let manifest_mutex = Mutex::new(manifest);
    let errors_mutex = Mutex::new(&mut ctx.errors);

    compressible.par_iter().for_each(|entry| {
        // Pre-read via source_cache so bundle_pass/graph_pass
        // get cache hits instead of re-reading from disk.
        let source_arc = state.read_source(entry).ok();
        let source_ref = source_arc.as_ref().map(|s| s.as_str());

        let mut dict_guard = state.dict_lock();
        let mut cache_guard = state.cache_write();

        match compress_file_with_source(
            PathBuf::from(&entry),
            source_ref,
            &mut dict_guard,
            &mut cache_guard,
            fidelity,
            Some(&state.config),
        ) {
            Ok(compressed) => {
                // Alias already pre-assigned — this is a read-only lookup.
                let alias = dict_guard.get_or_create_alias(entry.clone());
                let mut manifest_guard = manifest_mutex.lock().unwrap_or_else(|p| p.into_inner());
                manifest_guard.push_str(&format!(
                    "// ===== FILE: {} =====\n// α alias: {}\n",
                    entry, alias
                ));
                manifest_guard.push_str(&compressed);
                manifest_guard.push('\n');

                // Record per-file stats for workspace compression (MED-2 fix: use pluggable tokenizer)
                let ws_source = source_ref.unwrap_or("");
                let ws_raw = super::tool_helpers::count_tokens_with_tokenizer(ws_source, ws_tok_ref);
                let ws_compressed = super::tool_helpers::count_tokens_with_tokenizer(&compressed, ws_tok_ref);
                state.record_compression(
                    entry,
                    ws_raw,
                    ws_compressed,
                    &format!("{:?}", fidelity).to_lowercase(),
                    false,
                    "workspace",
                    None,
                    "ir_compression",
                );
            }
            Err(e) => {
                let mut errors_guard = errors_mutex.lock().unwrap_or_else(|p| p.into_inner());
                errors_guard.push((entry.clone(), e.to_string()));
                let mut manifest_guard = manifest_mutex.lock().unwrap_or_else(|p| p.into_inner());
                manifest_guard.push_str(&format!("// ERROR compressing {}: {}\n\n", entry, e));
            }
        }
    });
}

/// Two-pass compression with global symbol deduplication (Phase III, Idea #9).
///
/// Pass 1: Compress each file without symbol compression, collect bodies.
/// Pass 2: Build global symbol table, re-encode each file with shared
///         dictionary, emit global §GSYM dictionary in manifest header.
fn compress_pass_with_global_symbols(
    fidelity: Fidelity,
    state: &McpState,
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

    // C-1 fix: Create tokenizer once before the loop instead of per-file.
    let gs_tok = crate::tokenizer::create_tokenizer(
        crate::tokenizer::resolve_tokenizer_kind(None, Some(&state.config.tokenizer.to_string()))
    ).ok();
    let gs_tok_ref: Option<&dyn crate::tokenizer::Tokenizer> = gs_tok.as_deref();

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

        let mut dict_guard = state.dict_lock();
        let mut cache_guard = state.cache_write();

        match compress_source(&source_code, entry, &mut dict_guard, &mut cache_guard, fidelity, Some(&state.config.type_aliases)) {
            Ok(compressed) => {
                // Use the already-held dict_guard instead of re-locking through state
                let alias = dict_guard.get_or_create_alias(entry.clone());
                // Extract the body from the compressed output.
                // The compressed output has the report header + body.
                // We need the body for global symbol encoding.
                let body = extract_body_from_compressed(&compressed);

                // Record per-file stats for global-symbol workspace compression (MED-2 fix: use pluggable tokenizer)
                let gs_raw = super::tool_helpers::count_tokens_with_tokenizer(&source_code, gs_tok_ref);
                let gs_compressed = super::tool_helpers::count_tokens_with_tokenizer(&compressed, gs_tok_ref);
                state.record_compression(
                    entry,
                    gs_raw,
                    gs_compressed,
                    &format!("{:?}", fidelity).to_lowercase(),
                    false,
                    "workspace_gsym",
                    None,
                    "ir_compression",
                );

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
///
/// This function is only available when the `angular` feature is enabled.
/// When disabled, it returns an empty `FooterBuilder` stub.
#[cfg(feature = "angular")]
fn bundle_pass(
    state: &McpState,
    ctx: &PassContext,
    manifest: &mut String,
) -> FooterBuilder {
    let mut footer_builder = FooterBuilder::new();
    let mut bundle_count = 0usize;
    // ANGULAR_HTML_COMPRESSION_PLAN: use the config's default fidelity
    // for template rendering. The workspace pass doesn't receive an
    // explicit fidelity, so we use the config default (Low for the
    // single-line shape summary, Medium/High for structural output).
    let fidelity = state.config.default_fidelity;

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
        let component_alias = state.get_or_create_alias(entry.to_string());
        let mut file_aliases = vec![component_alias];
        let mut tpl_summary = None;
        let mut sty_summary = None;

        if let Some(ref tpl_path) = triplet.template {
            let a = state.get_or_create_alias(tpl_path.to_string_lossy().to_string());
            file_aliases.push(a);
            // F-FINAL-01: Use the shared source cache so files already
            // read in compress_pass / graph_pass are not re-read here.
            if let Ok(content) = state.read_source(&tpl_path.to_string_lossy()) {
                let shape = template::extract_template_shape(&content);
                // Fidelity-gated rendering: Low → single-line summary,
                // Medium/High → multi-line structural output.
                let lines = shape.to_marker_lines(fidelity);
                tpl_summary = Some(lines.join("\n"));
            }
        }
        if let Some(ref sty_path) = triplet.style {
            let a = state.get_or_create_alias(sty_path.to_string_lossy().to_string());
            file_aliases.push(a);
            // F-FINAL-01: Same shared-cache fix as the template branch.
            if let Ok(content) = state.read_source(&sty_path.to_string_lossy()) {
                let shape = style::extract_style_shape(&content);
                sty_summary = Some(shape.to_marker_line());
            }
        }

        let _bundle_alias = state.get_or_create_bundle_alias(triplet_name(path));
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
///
/// This function is only available when the `angular` feature is enabled.
/// When disabled, it is a no-op.
#[cfg(feature = "angular")]
fn graph_pass(state: &McpState, ctx: &mut PassContext, manifest: &mut String) {
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
        let file_alias = state.get_or_create_alias((*entry).clone());

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

    state.angular_graph_lock().set(angular_graph.clone());

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

// Stub implementations when `angular` feature is disabled.
// These return empty/no-op results so the orchestrator doesn't need
// to change its call sites.

/// Stub FooterBuilder for when Angular is disabled.
/// Provides the same API but all methods are no-ops.
#[cfg(not(feature = "angular"))]
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct FooterBuilder {
    _private: (),
}

#[cfg(not(feature = "angular"))]
#[allow(dead_code)]
impl FooterBuilder {
    pub fn new() -> Self {
        Self { _private: () }
    }
    
    pub fn register_bundle(
        &mut self,
        _name: String,
        _aliases: Vec<String>,
        _template: Option<String>,
        _style: Option<String>,
    ) {
        // no-op
    }
    
    pub fn is_empty(&self) -> bool {
        true
    }
    
    pub fn format_footer(&self) -> String {
        String::new()
    }
}

#[cfg(not(feature = "angular"))]
#[allow(dead_code)]
fn bundle_pass(
    _state: &McpState,
    _ctx: &PassContext,
    _manifest: &mut String,
) -> FooterBuilder {
    FooterBuilder::new()
}

#[cfg(not(feature = "angular"))]
#[allow(dead_code)]
fn graph_pass(_state: &McpState, _ctx: &mut PassContext, _manifest: &mut String) {
    // no-op
}

#[cfg(all(test, feature = "rust"))]
#[path = "../tests/mcp/workspace.rs"]
mod tests;
