// src/mcp/tool_helpers.rs
//
// Shared helper functions used by multiple tool handlers.
// Extracted from tools.rs during the Phase 1 module split.

use std::path::PathBuf;
use crate::compressor::Fidelity;
use crate::mcp::McpState;

/// Compress a file and extract the body lines (without header) for
/// delta comparison. Returns `(body_lines, full_output)`.
///
/// Thin MCP wrapper that reads source from state, then delegates to
/// the pure [`crate::compression::pipeline::compress_text`] function.
pub(super) fn compress_text_body(
    file_path: &str,
    fidelity: Fidelity,
    state: &McpState,
) -> Result<(Vec<String>, String), Box<dyn std::error::Error>> {
    // Use source_cache via state.read_source() — Finding 1
    let source_code_arc = state.read_source(file_path)?;
    let source_code = source_code_arc.as_ref().clone();
    let path_buf = std::path::PathBuf::from(file_path);
    let extension = path_buf.extension().and_then(|e| e.to_str()).unwrap_or("");
    let path_alias = state.get_or_create_alias(file_path.to_string());

    crate::compression::pipeline::compress_text(
        &source_code,
        extension,
        fidelity,
        &path_alias,
        Some(&state.config.type_aliases),
    )
}


/// Resolve a file path, handling relative paths with optional workspace root.
pub(super) fn resolve_file_path(path: &str, workspace_root: Option<&str>) -> String {
    let path_obj = std::path::Path::new(path);
    if path_obj.is_absolute() {
        path.to_string()
    } else if let Some(root) = workspace_root {
        let root_path = std::path::Path::new(root);
        if root_path.is_absolute() {
            root_path.join(path).to_string_lossy().into_owned()
        } else {
            // workspace root is also relative — join with CWD
            let cwd = std::env::current_dir().unwrap_or_default();
            cwd.join(root).join(path).to_string_lossy().into_owned()
        }
    } else {
        // Relative path with no workspace root — join with CWD
        let cwd = std::env::current_dir().unwrap_or_default();
        cwd.join(path).to_string_lossy().into_owned()
    }
}

/// Resolve a file path and enforce a workspace boundary (XPIA mitigation).
///
/// The boundary is anchored to the caller-supplied `workspace_root` when
/// provided, falling back to the **process CWD** when not. The trusted root
/// is canonicalized (resolving symlinks and `..`) and must be a real,
/// existing directory. The resolved path is also canonicalized and must
/// remain within the trusted root. Returns `Ok(resolved_path)` if inside the
/// boundary, or `Err(message)` if the path is outside or does not exist.
///
/// Security note: the caller-supplied `workspace_root` is itself
/// attacker-controlled, so it is canonicalized and required to exist before
/// being used as the boundary. This still prevents Cross-Prompt Injection
/// Attacks (XPIA) where source code with embedded instructions could direct
/// the LLM to read sensitive files outside the project (e.g. `/etc/passwd`,
/// `~/.ssh/id_rsa`) — the resolved path must remain within the (canonicalized)
/// root, so escaping via `..` or symlinks is rejected.
pub(super) fn resolve_file_path_checked(
    path: &str,
    workspace_root: Option<&str>,
    additional_roots: &[String],
) -> Result<String, String> {
    // 1. Resolve to absolute using the existing logic
    let resolved = resolve_file_path(path, workspace_root);
    // 2. Determine the trusted root: caller-supplied workspace_root when
    //    provided (canonicalized, must exist), else the process CWD.
    let trusted_root = match workspace_root {
        Some(root) => {
            let root_path = std::path::Path::new(root);
            if root_path.is_absolute() {
                root_path.to_path_buf()
            } else {
                std::env::current_dir().unwrap_or_default().join(root)
            }
        }
        None => std::env::current_dir()
            .map_err(|e| format!("cannot determine workspace root: {e}"))?,
    };
    let trusted_root_canon = trusted_root
        .canonicalize()
        .map_err(|_| format!("workspace root does not exist: {}", trusted_root.display()))?;
    // 3. Canonicalize the resolved path (resolves symlinks and `..`) for
    //    the boundary check. NOTE: we return the ORIGINAL resolved path,
    //    not the canonical form, so persistence keys and alias mappings
    //    remain stable (canonicalizing would change the DB key and break
    //    callers that expect the caller-supplied path form).
    let resolved_canon = std::path::Path::new(&resolved)
        .canonicalize()
        .map_err(|_| format!("path does not exist: {resolved}"))?;
    // 4. Boundary check: resolved must be within the trusted root, OR within
    //    one of the config-declared `additional_roots` (multi-repo support —
    //    see `CleanCtxConfig::additional_roots`). Each additional root is
    //    canonicalized here rather than at config-load time, since it must
    //    tolerate a repo that doesn't exist on this machine without erroring
    //    the whole config; such an entry is simply skipped.
    if resolved_canon.starts_with(&trusted_root_canon) {
        return Ok(resolved);
    }
    for extra_root in additional_roots {
        if let Ok(extra_root_canon) = std::path::Path::new(extra_root).canonicalize() {
            if resolved_canon.starts_with(&extra_root_canon) {
                return Ok(resolved);
            }
        }
    }
    // Audit fix: include the effective workspace roots in the error message
    // so the caller knows which roots were checked and can take corrective
    // action (pass `workspaceRoot` or configure `additional_roots`).
    let extra_roots_str = if additional_roots.is_empty() {
        String::new()
    } else {
        format!(
            " (additional_roots: {})",
            additional_roots.join(", ")
        )
    };
    Err(format!(
        "path outside workspace root: {resolved} (workspace root: {trusted_root}){extra_roots_str}",
        trusted_root = trusted_root_canon.display(),
    ))
}

/// Inject a `"baseline"` cache breakpoint into a JSON-RPC response.
///
/// Uses `compute_baseline_breaker(compressed_text)` as the breaker so the
/// cache is invalidated when the compressed output changes. This is the
/// primary consumer of the smart cache for per-file compression outputs.
///
/// No-op when cache is disabled in config.
pub(crate) fn inject_baseline_breakpoint(
    response: &mut serde_json::Value,
    state: &McpState,
    compressed_text: &str,
) {
    if !state.config.cache.enabled {
        return;
    }
    let ttl = state.config.cache.baseline_ttl.clone();
    let breaker = crate::mcp::cache_hints::compute_baseline_breaker(compressed_text);
    let tok_box = crate::tokenizer::create_tokenizer(
        crate::tokenizer::resolve_tokenizer_kind(None, Some(&state.config.tokenizer.to_string()))
    ).ok();
    let tok_ref: Option<&dyn crate::tokenizer::Tokenizer> = tok_box.as_deref();
    if let Some(result_obj) = response.get_mut("result") {
        crate::mcp::cache_hints::inject_cache_breakpoints(
            result_obj, state, "baseline", &ttl, &breaker, tok_ref,
        );
    }
}

/// Inject a `"tail"` cache breakpoint into a JSON-RPC response.
///
/// The tail is always rolling (breaker = "rolling") — it represents
/// dynamic content that changes each turn and should never be cached
/// across turns. Marks the tail as ephemeral in cache metrics.
///
/// No-op when cache is disabled in config.
pub(crate) fn inject_tail_breakpoint(
    response: &mut serde_json::Value,
    state: &McpState,
) {
    if !state.config.cache.enabled {
        return;
    }
    let ttl = state.config.cache.tail_ttl.clone();
    let tok_box = crate::tokenizer::create_tokenizer(
        crate::tokenizer::resolve_tokenizer_kind(None, Some(&state.config.tokenizer.to_string()))
    ).ok();
    let tok_ref: Option<&dyn crate::tokenizer::Tokenizer> = tok_box.as_deref();
    if let Some(result_obj) = response.get_mut("result") {
        crate::mcp::cache_hints::inject_cache_breakpoints(
            result_obj, state, "tail", &ttl, "rolling", tok_ref,
        );
    }
    crate::mcp::cache_hints::mark_tail_ephemeral(state);
}

/// Rough token estimation (chars / 4).
/// In production, use tiktoken-rs; this is a lightweight approximation.
pub(super) fn estimate_tokens(text: &str) -> usize {
    text.len() / 4
}

/// Count tokens using the provided pluggable tokenizer (R-19).
///
/// When `tokenizer` is `Some`, uses the pluggable tokenizer for accurate
/// token counting. When `None`, falls back to the rough chars/4 estimate.
pub(crate) fn count_tokens_with_tokenizer(
    text: &str,
    tokenizer: Option<&dyn crate::tokenizer::Tokenizer>,
) -> usize {
    match tokenizer {
        Some(tok) => tok.count_tokens(text),
        None => estimate_tokens(text),
    }
}

/// Compile a file to IR, detecting language and running the full
/// 4-layer compilation pipeline.
///
/// Phase A (FAANG remediation): The compiler now instantiates the
/// appropriate language layers (TypeScriptLayer, CSharpLayer) and
/// meta layers (AngularMetaLayer) based on the detected language.
///
/// NF-02 fix: The version is set based on the previous version in the
/// context state, ensuring a monotonic version chain across successive
/// `delta_code_context` calls. If the file was previously tracked at
/// version N, the new compiled IR gets version N+1. If untracked,
/// version starts at 1.
///
/// NF-01 fix: The consumptive `CompressingPatternRecognizer` is wired
/// into the compile path *after* the additive `CodePatternRecognizer`,
/// so flags are emitted first, then patterns are consumed for wire-size
/// reduction. This enables the Phase H 30% compression on edits.
///
/// A-08: Returns the source hash along with the compiled IR to enable
/// source change detection in the delta path.
pub(super) fn compile_file_ir(
    file_path: &str,
    fidelity: Fidelity,
    state: &McpState,
) -> Result<(crate::ir::compiler::CompiledIR, String), crate::error::CleanCtxError> {
    compile_file_ir_focused(file_path, fidelity, state, None)
}

/// Compile a file to IR with symbol targeting (`focus`).
///
/// `focus`: optional set of method names that should receive full verbatim
/// bodies at `Edit` fidelity. When `Some(set)`, only those methods get their
/// body extracted into the IR (`CoreOp::Body`); all other methods are emitted
/// signature-only. This is the compile-time counterpart to the render-time
/// `focus` gate in `render_llm.rs` — it avoids extracting/storing body text
/// for methods that will be filtered out at render time (memory/CPU
/// optimization). When `None`, every method's body is extracted (legacy
/// behavior, byte-identical to `compile_file_ir`).
pub(super) fn compile_file_ir_focused(
    file_path: &str,
    fidelity: Fidelity,
    state: &McpState,
    focus: Option<&std::collections::HashSet<String>>,
) -> Result<(crate::ir::compiler::CompiledIR, String), crate::error::CleanCtxError> {
    use crate::ir::compiler::IRCompiler;
    use crate::ir::layers::typescript::TypeScriptLayer;
    use crate::ir::layers::csharp::CSharpLayer;
    use crate::ir::layers::rust::RustLayer;
    use crate::ir::layers::java::JavaLayer;
    // P0-4: Meta-layers are now handled by LayerRegistry::global() inside IRCompiler.
    // The old ir::layers::angular/spring/dotnet modules have been removed.
    use crate::compression::language::language_for_extension;

    // Use source_cache via state.read_source() — Finding 1
    let source_arc = state.read_source(file_path)?;
    let source = source_arc.as_str();
    let path_buf = PathBuf::from(file_path);
    let extension = path_buf.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    let (language, query_string) = language_for_extension(extension)
        .ok_or_else(|| crate::error::CleanCtxError::Ir(format!("Unsupported file extension: .{}", extension)))?;

    // F-FULL-10: Use raw path for alias key for deterministic results.
    // Canonicalize is still performed for the `α alias: <path>` footer
    // display, but the alias key itself uses the raw path.
    let path_alias = state.get_or_create_alias(file_path.to_string());

    // NF-02: Determine the next version based on the previous context state
    let prev_version = state.file_version(&path_alias).unwrap_or(0);

    // A-08: Compute source hash for change detection
    let source_hash = {
        let cache = state.cache_read();
        cache.compute_hash(source.as_bytes())
    };

    let mut compiler = IRCompiler::new();

    // Add language-specific layers (Layer 2)
    match extension {
        "ts" | "js" => {
            compiler.add_language_layer(Box::new(TypeScriptLayer::new()));
        }
        "cs" => {
            compiler.add_language_layer(Box::new(CSharpLayer::new()));
        }
        "rs" => {
            compiler.add_language_layer(Box::new(RustLayer::new()));
        }
        "java" => {
            compiler.add_language_layer(Box::new(JavaLayer::new()));
        }
        _ => {}
    }

    // P0-4: Framework meta-layers (Layer 3) are now handled by LayerRegistry::global()
    // inside IRCompiler::compile(). Meta-layers are registered in src/layers/meta/
    // and wired via McpState -> LayerRegistry. No manual add_meta_layer() needed.

    // F-07 (FAANG audit): Wire the additive CodePatternRecognizer into
    // the compile path. This is the Layer 4 additive recognizer that
    // emits CTOR/OBSERVABLE/GETTER/SETTER flags alongside the original
    // instructions. The recognizer is always-on because it adds context
    // without removing any instructions (zero regression).
    compiler.add_pattern_recognizer(Box::new(
        crate::ir::layers::patterns::CodePatternRecognizer::new(),
    ));

    // NF-01: Wire the consumptive CompressingPatternRecognizer *after*
    // the additive recognizer. This enables the Phase H 30% compression
    // on edits by consuming recognised patterns into single PAT ops.
    // The additive recognizer's flags (CTOR/OBSERVABLE/GETTER/SETTER)
    // are emitted first, then the consumptive recognizer collapses them
    // where possible. This ordering ensures maximum compression.
    compiler.add_pattern_recognizer(Box::new(
        crate::ir::patterns::CompressingPatternRecognizer::new(),
    ));

    // CBM filter-first: pass the skip set so low-importance symbols
    // are excluded from IR output entirely.
    let skip_set = state.get_skip_set(file_path);
    let mut compiled = compiler.compile_focused(
        source,
        &path_alias,
        language,
        query_string,
        fidelity,
        skip_set.as_ref(),
        focus,
    )?;

    // R-02 Phase 3: Apply type aliases to the IR instruction stream.
    // Replaces type names in FieldType, Return, and Param ops with
    // alias tokens, and appends CoreOp::TypeAlias ops for used aliases.
    crate::ir::type_aliases::apply_type_aliases_to_ir(
        &mut compiled.instructions,
        &state.config.type_aliases,
    );

    // NF-02: Override the version with the next monotonic value.
    // The compiler always sets version=1; we fix it here.
    compiled.version = prev_version.saturating_add(1);

    // A-08: Return source hash along with compiled IR
    Ok((compiled, source_hash))
}

/// Compute an AST-level diff between the file's in-session baseline and
/// its current on-disk state.
///
/// A-08 (Token Efficiency Audit, H-01/L-01): Now accepts source text
/// directly instead of reading from disk, so callers control the read
/// path and can use `state.read_source()` for source_cache integration.
///
/// F-21 (FAANG audit): before calling the expensive `build_snapshot`,
/// the handler hashes the source and checks if a baseline exists *and*
/// the hash matches. On match, it returns a "no changes" message
/// without re-parsing the file with tree-sitter.
pub(crate) fn diff_code_context_handler(
    file: PathBuf,
    source: &str,
    cache: &mut crate::cache::LocalStateCache,
    fidelity: Fidelity,
) -> Result<String, Box<dyn std::error::Error>> {
    use crate::diff::{build_snapshot, diff_snapshots, format_diff, diff_summary};

    let absolute_path = match std::fs::canonicalize(&file) {
        Ok(p) => p.to_string_lossy().into_owned(),
        Err(_) => file.to_string_lossy().into_owned(),
    };
    let cache_key = format!("{}::{}", absolute_path, fidelity as u8);

    // F-21: hash the source content and check if the baseline is
    // still valid before paying for the expensive tree-sitter parse.
    let source_hash = cache.compute_hash(source.as_bytes());
    if let Some(stored_hash) = cache.get_baseline_hash(&cache_key)
        && stored_hash == source_hash
        && let Some(baseline_snap) = cache.get_baseline(&cache_key).cloned()
    {
        // Content is byte-identical to the stored baseline — no
        // structural changes possible.
        let class_count = baseline_snap.classes.len();
        return Ok(format!(
            "// --- AST Diff ---\n// No changes since last snapshot ({} classes).\n// Hash: {}",
            class_count, &source_hash[..12],
        ));
    }

    let current = build_snapshot(source, fidelity)?;

    let baseline = cache.get_baseline(&cache_key).cloned();
    let body = match baseline {
        None => {
            let class_count = current.classes.len();
            // F-21: store the hash BEFORE `store_baseline` takes
            // ownership of `cache_key`.
            cache.store_baseline_hash(&cache_key, &source_hash);
            cache.store_baseline(cache_key, current);
            format!(
                "// --- AST Diff ---\n// No baseline snapshot for this file yet.\n// Current state stored as baseline ({} classes).\n// Call diff_code_context again after the file changes to see the delta.",
                class_count
            )
        }
        Some(baseline_snap) => {
            let actions = diff_snapshots(&baseline_snap, &current);
            let (added, removed, modified, unchanged) = diff_summary(&actions);
            let header = format!(
                "// --- AST Diff: {} ---\n// +{} -{} ~{} ={} (classes/methods/fields/imports)\n",
                absolute_path, added, removed, modified, unchanged
            );
            let body = format_diff(&actions, fidelity);
            cache.store_baseline_hash(&cache_key, &source_hash);
            cache.store_baseline(cache_key, current);
            format!("{}{}", header, body)
        }
    };
    Ok(body)
}

#[cfg(test)]
#[path = "../tests/mcp/tool_helpers.rs"]
mod tests;
