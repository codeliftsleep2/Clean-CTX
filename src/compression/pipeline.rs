// src/compression/pipeline.rs
//
// Non-streaming compression orchestrator (`compress_file`). It owns the
// shared helper functions `build_output_lines` and `assemble_body` that
// were historically private to `compressor.rs`. These are `pub(crate)` so
// the streaming variant can also call them.

use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::PathBuf;

use crate::analytics::calculate_savings;

/// F-18 (FAANG audit): maximum file size in bytes that `compress_file`
/// will read into memory. Files larger than this return a clean error
/// instead of risking an OOM. The streaming variant
/// (`compress_file_streaming`) can be used as a fallback for larger
/// files.
const MAX_FILE_BYTES: u64 = 10 * 1024 * 1024; // 10 MB

/// P1-10: Unified token metadata computation for cache-hit paths.
/// Ensures consistent token counting between cache-hit and cache-miss paths.
fn compute_token_metadata(raw_tokens: usize, compressed_text: &str) -> crate::analytics::TokenMetadata {
    let bpe = crate::analytics::bpe();
    let compressed_tokens = bpe.encode_with_special_tokens(compressed_text).len();
    let savings_percentage = if raw_tokens > 0 {
        let saved = raw_tokens.saturating_sub(compressed_tokens);
        (saved as f64 / raw_tokens as f64) * 100.0
    } else {
        0.0
    };
    crate::analytics::TokenMetadata {
        raw_tokens,
        compressed_tokens,
        savings_percentage,
    }
}
use crate::cache::LocalStateCache;
use crate::compaction::{
    compact_expression, compact_import, extract_class_name, extract_field,
    extract_method_sig, extract_rust_struct_name, format_class_entry,
    format_java_type_entry, format_rust_type_entry, simple_compact,
};
use crate::compaction::java::{
    compact_java_package, extract_java_constructor_sig, extract_java_type_name,
};
use crate::compression::capture_pipeline::run_capture_pipeline;
use crate::compression::language::language_for_extension;
use crate::compression::markers::build_marker;
use crate::compression::report::{format_compacted_body, format_final_output};
use crate::compression::micro_opcodes::apply_micro_opcodes;
use crate::compression::symbol_compression::apply_symbol_compression;
use crate::compression::type_aliases::apply_type_aliases;
use crate::compression::CapEntry;
use crate::compression::Fidelity;
use crate::dictionary::PathDictionary;

/// Output of [`build_output_lines`]. F-04 (FAANG audit): previously
/// the orchestrator counted classes/methods/imports by
/// `let class_count: usize = 0;` and bound it to `_`, then passed
/// `0, 0, 0` to `format_final_output`. The header always lied.
/// The struct now carries the real counts.
#[derive(Debug, Clone)]
pub struct BuildOutputResult {
    /// The compacted body lines, in document order, with the import
    /// block prepended.
    pub output_lines: Vec<String>,
    /// Number of `class.root` captures that emitted a class entry.
    pub class_count: usize,
    /// Number of `method.root` captures that emitted a method line.
    pub method_count: usize,
    /// Number of `import.root` captures that produced a non-empty
    /// import line.
    pub import_count: usize,
    /// Angular Meta-Layer block (Phase 1: Tier 1 decorator extraction).
    pub meta_block: Option<crate::angular_meta::MetaBlock>,
    /// Spring Boot Meta-Layer block (Phase 1: Tier 1 annotation extraction).
    pub spring_meta_block: Option<crate::spring_meta::MetaBlock>,
    /// .NET / C# Meta-Layer block.
    pub dotnet_meta_block: Option<crate::dotnet_meta::MetaBlock>,
}

/// Reads a target source file and compiles it down into a highly compacted,
/// keyword-stripped structural signature stream with configurable fidelity.
///
/// F-18: returns a [`CompressionError::FileTooLarge`] if the file exceeds
/// [`MAX_FILE_BYTES`]. Use [`compress_file_streaming`](crate::compression::streaming::compress_file_streaming)
/// for larger files.
pub fn compress_file(
    file: PathBuf,
    dict: &mut PathDictionary,
    cache: &mut LocalStateCache,
    fidelity: Fidelity,
    config: Option<&crate::config::CleanCtxConfig>,
) -> Result<String, Box<dyn std::error::Error>> {
    compress_file_with_source(file, None, dict, cache, fidelity, config)
}

/// Like [`compress_file`], but accepts an optional pre-read source string
/// to avoid redundant disk reads. When `source_override` is `Some`, the
/// file-size check and `fs::read_to_string` are skipped.
///
/// Finding F (Token Efficiency Audit): callers that already have the
/// source content (e.g. via `state.read_source()`) can pass it here,
/// eliminating the double-read when `compile_file_ir` or other
/// downstream functions also need the source.
pub fn compress_file_with_source(
    file: PathBuf,
    source_override: Option<&str>,
    dict: &mut PathDictionary,
    cache: &mut LocalStateCache,
    fidelity: Fidelity,
    config: Option<&crate::config::CleanCtxConfig>,
) -> Result<String, Box<dyn std::error::Error>> {
    let source_code;
    if let Some(src) = source_override {
        source_code = src.to_string();
    } else {
        // A-13: guard against reading an unbounded file into memory.
        // Use centralized validation from ResourceLimits (SRP compliance).
        let meta = fs::metadata(&file)?;
        let file_size = meta.len();
        
        // Delegate to config validation method if available
        if let Some(cfg) = config {
            cfg.resource_limits.check_file_size(file_size)?;
        } else if file_size > MAX_FILE_BYTES {
            return Err(format!(
                "File too large ({} bytes; max {} bytes). \
                 Use compress_workspace or the streaming variant for large files.",
                file_size, MAX_FILE_BYTES,
            )
            .into());
        }
        
        source_code = fs::read_to_string(&file)?;
    }
    let source_bytes = source_code.as_bytes();

    let current_hash = cache.compute_hash(source_bytes);
    // F-FULL-10: Use the raw path as the alias key for deterministic results
    // across workspace passes and per-file tools.
    let absolute_path = file.to_string_lossy().to_string();
    let path_alias = dict.get_or_create_alias(absolute_path.clone());

    // Include fidelity in the cache key so different fidelity levels don't share cached results
    let cache_key = format!("{}::{}", absolute_path, fidelity as u8);
    let is_modified = cache.update_and_verify(&cache_key, &current_hash);
    if !is_modified {
        // P1-10: Use unified compute_token_metadata for both cache-hit paths
        // to ensure consistent token counting. Previously the cache-hit path
        // manually computed tokens inline while the miss path used calculate_savings(),
        // which could produce different results for the same input.
        let cached_notice = format!(
            "// [CACHE_HIT] {} unchanged. Use historic memory.\n",
            path_alias
        );

        let meta = if let Some(raw_tokens) = cache.get_raw_token_count(&current_hash) {
            compute_token_metadata(raw_tokens, &cached_notice)
        } else {
            calculate_savings(&source_code, &cached_notice, None)
        };

        // Phase III (Idea #8 — Progressive Header Elision):
        // Low fidelity uses the compact cache-hit format.
        if fidelity == Fidelity::Low {
            return Ok(crate::compression::report::format_compact_cache_hit(
                meta.raw_tokens,
                meta.compressed_tokens,
                meta.savings_percentage,
                &path_alias,
            ));
        }

        // F-04: also surface the Structures line on a cache hit so
        // the header is consistent with the miss path. We can't
        // know the real counts without re-parsing, so we report
        // "cached" for each — the LLM client knows the previous
        // output and can match by `path_alias`.
        let ratio_report = format!(
            "// Structures: cached, cached, cached | {}/{} tokens",
            meta.raw_tokens, meta.compressed_tokens
        );
        return Ok(format!(
            "// --- Token Optimization Report --- \n// Raw Tokens: {} | Retained Tokens: {} | Waste Reduced: {:.2}%\n// Fidelity: {:?}\n// {}\n{}",
            meta.raw_tokens, meta.compressed_tokens, meta.savings_percentage, fidelity, ratio_report, cached_notice
        ));
    }

    let extension = file.extension().and_then(|ext| ext.to_str()).unwrap_or("");
    let (language, query_string) = language_for_extension(extension)
        .ok_or_else(|| format!("Unsupported file extension: .{}", extension))?;

    // Run the SHARED capture pipeline. F-08: pass the real `fidelity`
    // through so the per-capture closures use it (instead of always
    // Low).
    let all_captures: Vec<CapEntry> = run_capture_pipeline(
        language,
        query_string,
        &source_code,
        fidelity,
        |capture_name, raw, f| {
            match capture_name {
                "class.root" => Some(extract_class_name(raw)),
                // Rust type declarations: struct, enum, trait, impl
                "struct.root" | "trait.root" | "impl.root" => {
                    Some(extract_rust_struct_name(raw))
                }
                // Java type declarations: interface, enum, record
                "interface.root" | "record.root" => {
                    Some(extract_java_type_name(raw, capture_name))
                }
                "method.root" => Some(extract_method_sig(raw, f)),
                "constructor.root" => Some(extract_java_constructor_sig(raw, f)),
                "field.root" => Some(extract_field(raw, f)),
                // Rust mod declarations are structural like imports
                "mod.root" => Some(compact_import(raw, f)),
                // Java package declarations
                "package.root" => Some(compact_java_package(raw, f)),
                // Rust type aliases
                "type.root" => Some(compact_expression(raw, f)),
                _ => Some(compact_expression(raw, f)),
            }
        },
    )?;

    // F-04: `build_output_lines` now returns real counts.
    let built = build_output_lines(&all_captures, &source_code, fidelity, None);
    let mut body_content = assemble_body(&built.output_lines, fidelity);
    // Angular Meta-Layer (Phase 1): inject the Φ block into the body
    // BEFORE symbol compression so the `Φ` markers stay untouched.
    if let Some(block) = &built.meta_block {
        body_content.push_str(&block.render());
    }
    // Spring Boot Meta-Layer (Phase 1): inject the Φ block into the body
    // BEFORE symbol compression so the `Φ` markers stay untouched.
    if let Some(block) = &built.spring_meta_block {
        body_content.push_str(&block.render());
    }
    // .NET / C# Meta-Layer: inject the Φ block into the body
    // BEFORE symbol compression so the `Φ` markers stay untouched.
    if let Some(block) = &built.dotnet_meta_block {
        body_content.push_str(&block.render());
    }
    // R-02: Apply type aliases before micro-opcodes and symbol
    // compression so `$uid` tokens are preserved. At Low fidelity types
    // are already stripped, so this is a natural no-op.
    let ta_footer = if let Some(cfg) = config
        && !cfg.type_aliases.is_empty()
    {
        let (substituted, footer) = apply_type_aliases(&body_content, &cfg.type_aliases);
        body_content = substituted;
        footer
    } else {
        String::new()
    };
    // Phase III (Idea #11 — Micro-Opcode Table for Text):
    // Apply micro-opcodes (§C, §P) before symbol compression so the
    // §-prefixed codes are included in the symbol dictionary analysis.
    body_content = apply_micro_opcodes(&body_content, fidelity);
    let (display_body, sym_footer) = apply_symbol_compression(&body_content, fidelity);
    let combined_footer = combine_footers(&sym_footer, &ta_footer);
    let compacted_body = format_compacted_body(&display_body, &combined_footer, &path_alias, fidelity);
    // F-14: store the raw-token count for this content hash so the
    // cache-hit path can skip the BPE encode on subsequent calls.
    let raw_tokens = crate::analytics::bpe()
        .encode_with_special_tokens(&source_code)
        .len();
    cache.store_raw_token_count(&current_hash, raw_tokens);

    let final_output = format_final_output(
        &source_code,
        &compacted_body,
        fidelity,
        built.class_count,
        built.method_count,
        built.import_count,
    );
    Ok(final_output)
}

/// Pure compression function with no MCP dependencies.
///
/// Takes source code, file extension, fidelity, and a path alias string.
/// Returns `(body_lines, full_output)` — the body lines for delta
/// comparison and the full formatted output with header.
///
/// This is the core compression logic extracted from `compress_text_body`
/// in the MCP layer, enabling reuse without MCP-specific state types.
pub fn compress_text(
    source_code: &str,
    extension: &str,
    fidelity: Fidelity,
    path_alias: &str,
    aliases: Option<&BTreeMap<String, String>>,
) -> Result<(Vec<String>, String), Box<dyn std::error::Error>> {
    let (language, query_string) = language_for_extension(extension)
        .ok_or_else(|| format!("Unsupported file extension: .{}", extension))?;

    let all_captures: Vec<CapEntry> = run_capture_pipeline(
        language,
        query_string,
        source_code,
        fidelity,
        |capture_name, raw, f| {
            match capture_name {
                "class.root" => Some(extract_class_name(raw)),
                // Rust type declarations: struct, enum, trait, impl
                "struct.root" | "trait.root" | "impl.root" => {
                    Some(extract_rust_struct_name(raw))
                }
                // Java type declarations: interface, enum, record
                "interface.root" | "record.root" => {
                    Some(extract_java_type_name(raw, capture_name))
                }
                "method.root" => Some(extract_method_sig(raw, f)),
                "constructor.root" => Some(extract_java_constructor_sig(raw, f)),
                "field.root" => Some(extract_field(raw, f)),
                // Rust mod declarations are structural like imports
                "mod.root" => Some(compact_import(raw, f)),
                // Java package declarations
                "package.root" => Some(compact_java_package(raw, f)),
                // Rust type aliases
                "type.root" => Some(compact_expression(raw, f)),
                _ => Some(compact_expression(raw, f)),
            }
        },
    )?;

    let built = build_output_lines(&all_captures, source_code, fidelity, None);
    let mut body_content = assemble_body(&built.output_lines, fidelity);
    if let Some(block) = &built.meta_block {
        body_content.push_str(&block.render());
    }
    if let Some(block) = &built.spring_meta_block {
        body_content.push_str(&block.render());
    }
    if let Some(block) = &built.dotnet_meta_block {
        body_content.push_str(&block.render());
    }
    // R-02: Apply type aliases before micro-opcodes and symbol
    // compression so `$uid` tokens are preserved.
    let ta_footer = if let Some(aliases) = aliases
        && !aliases.is_empty()
    {
        let (substituted, footer) = apply_type_aliases(&body_content, aliases);
        body_content = substituted;
        footer
    } else {
        String::new()
    };
    body_content = apply_micro_opcodes(&body_content, fidelity);
    let (display_body, sym_footer) = apply_symbol_compression(&body_content, fidelity);

    // Split the body into lines for delta comparison
    let body_lines: Vec<String> = display_body.lines().map(String::from).collect();

    // Build full output (header + body + symbol footer)
    let combined_footer = combine_footers(&sym_footer, &ta_footer);
    let compacted_body = format_compacted_body(&display_body, &combined_footer, path_alias, fidelity);
    let full_output = format_final_output(
        source_code,
        &compacted_body,
        fidelity,
        built.class_count,
        built.method_count,
        built.import_count,
    );

    Ok((body_lines, full_output))
}

// ---------------------------------------------------------------------------
// Shared pipeline helpers
// ---------------------------------------------------------------------------

/// Walk the captures in document order and build the output lines +
/// the per-fidelity counts. Shared between the streaming and
/// non-streaming variants.
///
/// F-04: the return type is now `BuildOutputResult` (with the real
/// counts) instead of a `(Vec<String>, Vec<String>)` tuple.
///
/// `skip_set`: optional set of symbol names to exclude from the output.
/// When a class, method, or field name matches an entry in this set,
/// the capture is dropped entirely. Used by the CBM filter-first
/// architecture to exclude low-importance symbols.
pub fn build_output_lines(
    all_captures: &[CapEntry],
    source_code: &str,
    fidelity: Fidelity,
    skip_set: Option<&HashSet<String>>,
) -> BuildOutputResult {
    let mut output_lines: Vec<String> = Vec::new();
    let mut fields: Vec<String> = Vec::new();
    let mut markers: Vec<String> = Vec::new();
    let mut imports: Vec<String> = Vec::new();
    let mut class_count: usize = 0;
    let mut method_count: usize = 0;
    let mut import_count: usize = 0;
    let mut class_captures: Vec<String> = Vec::new();

    for cap in all_captures {
        // CBM filter-first: skip low-importance symbols
        if let Some(skip) = skip_set {
            if !skip.is_empty() && should_skip_capture(cap, skip) {
                continue;
            }
        }
        match cap.name.as_str() {
            "import.root" | "mod.root" | "package.root" => {
                let compact = compact_import(&cap.text, fidelity);
                if !compact.is_empty() {
                    imports.push(compact);
                    import_count += 1;
                }
            }
            "class.root" => {
                if !output_lines.is_empty() && (fidelity == Fidelity::High || fidelity == Fidelity::Medium) {
                    output_lines.push(String::new());
                }
                output_lines.push(format_class_entry(&cap.text, &fields, fidelity));
                class_captures.push(cap.text.clone());
                class_count += 1;
                fields.clear();
                markers.clear();
            }
            // Rust type declarations: struct, enum, trait, impl
            // Use Rust-specific formatting (no "class" prefix)
            "struct.root" | "trait.root" | "impl.root" => {
                if !output_lines.is_empty() && (fidelity == Fidelity::High || fidelity == Fidelity::Medium) {
                    output_lines.push(String::new());
                }
                output_lines.push(format_rust_type_entry(&cap.text, &fields, fidelity));
                class_captures.push(cap.text.clone());
                class_count += 1;
                fields.clear();
                markers.clear();
            }
            // Java type declarations: interface, enum, record
            // Use Java-specific formatting (preserves type keyword)
            "interface.root" | "enum.root" | "record.root" => {
                if !output_lines.is_empty() && (fidelity == Fidelity::High || fidelity == Fidelity::Medium) {
                    output_lines.push(String::new());
                }
                output_lines.push(format_java_type_entry(&cap.text, &cap.name, &fields, fidelity));
                class_captures.push(cap.text.clone());
                class_count += 1;
                fields.clear();
                markers.clear();
            }
            "method.root" | "constructor.root" => {
                let sig = &cap.text;
                if !markers.is_empty() {
                    let marker_str = markers.join(" ");
                    if fidelity == Fidelity::High {
                        output_lines.push(format!("  {} {{ {} }}", sig, marker_str));
                    } else if fidelity == Fidelity::Medium {
                        output_lines.push(format!("{} {}", sig, marker_str));
                    } else {
                        output_lines.push(sig.clone());
                    }
                } else {
                    if fidelity == Fidelity::High {
                        output_lines.push(format!("  {}", sig));
                    } else {
                        output_lines.push(sig.clone());
                    }
                }
                method_count += 1;
                markers.clear();
            }
            "field.root" => {
                if !cap.text.is_empty() {
                    fields.push(cap.text.clone());
                }
            }
            _ => {
                if fidelity == Fidelity::Low {
                    continue;
                }
                if let Some(marker) = build_marker(&cap.name, &cap.text)
                    && markers.last().map(|m| m != &marker).unwrap_or(true) {
                        markers.push(marker);
                    }
            }
        }
    }

    // Orphaned fields with no class context
    if !fields.is_empty() && output_lines.is_empty() && fidelity != Fidelity::Low {
        output_lines.push(format!("⊕fields {{ {} }}", fields.join("; ")));
    }

    // Raw fallback when nothing was captured
    if output_lines.is_empty()
        && let Some(first_line) = source_code.lines().next() {
            let trimmed = first_line.trim().to_string();
            if !trimmed.is_empty() {
                output_lines.push(simple_compact(&trimmed, fidelity));
            }
        }

    // Prepend imports
    let mut output = output_lines;
    if !imports.is_empty() {
        let import_block = match fidelity {
            Fidelity::Low => imports.join("; "),
            _ => imports.join("\n"),
        };
        output.insert(0, import_block);
    }
    // Phase 4: Dispatch meta-layers through the registry.
    // When a feature is disabled, the corresponding meta-layer is not
    // registered and the tree-sitter grammar is not linked.
    let registry = crate::layers::LayerRegistry::global();
    let meta_results = registry.run_meta_layers_pipeline(source_code, &class_captures, fidelity);
    
    let meta_block = meta_results.iter()
        .find(|(name, _)| name == "angular")
        .map(|(_, text)| {
            let mut lines = Vec::new();
            for line in text.lines() {
                if line.starts_with("Φ") {
                    lines.push(line.to_string());
                }
            }
            crate::angular_meta::MetaBlock { lines }
        });
    
    let spring_meta_block = meta_results.iter()
        .find(|(name, _)| name == "spring_boot")
        .map(|(_, text)| {
            let mut lines = Vec::new();
            for line in text.lines() {
                if line.starts_with("Φ") {
                    lines.push(line.to_string());
                }
            }
            crate::spring_meta::MetaBlock { lines }
        });
    
    let dotnet_meta_block = meta_results.iter()
        .find(|(name, _)| name == "dotnet")
        .map(|(_, text)| {
            let mut lines = Vec::new();
            for line in text.lines() {
                if line.starts_with("Φ") {
                    lines.push(line.to_string());
                }
            }
            crate::dotnet_meta::MetaBlock { lines }
        });
    
    BuildOutputResult {
        output_lines: output,
        class_count,
        method_count,
        import_count,
        meta_block,
        spring_meta_block,
        dotnet_meta_block,
    }
}

/// Compress source code from a string (not from a file path).
///
/// This is used by workspace-level compression where the source code
/// has already been read. The `apply_symbol_compression` step is
/// skipped when a global symbol table will be used instead.
pub fn compress_source(
    source_code: &str,
    absolute_path: &str,
    dict: &mut PathDictionary,
    cache: &mut LocalStateCache,
    fidelity: Fidelity,
    aliases: Option<&BTreeMap<String, String>>,
) -> Result<String, Box<dyn std::error::Error>> {
    let source_bytes = source_code.as_bytes();
    let current_hash = cache.compute_hash(source_bytes);
    let path_alias = dict.get_or_create_alias(absolute_path.to_string());
    eprintln!("[compress_source] got alias {} for {}", path_alias, absolute_path);

    let cache_key = format!("{}::{}", absolute_path, fidelity as u8);
    let is_modified = cache.update_and_verify(&cache_key, &current_hash);
    if !is_modified {
        if fidelity == Fidelity::Low {
            let meta = if let Some(raw_tokens) = cache.get_raw_token_count(&current_hash) {
                let bpe = crate::analytics::bpe();
                let cached_notice = format!(
                    "// [CACHE_HIT] {} unchanged. Use historic memory.\n",
                    path_alias
                );
                let compressed_tokens = bpe.encode_with_special_tokens(&cached_notice).len();
                let savings_percentage = if raw_tokens > 0 {
                    let saved = raw_tokens.saturating_sub(compressed_tokens);
                    (saved as f64 / raw_tokens as f64) * 100.0
                } else {
                    0.0
                };
                crate::analytics::TokenMetadata {
                    raw_tokens,
                    compressed_tokens,
                    savings_percentage,
                }
            } else {
                crate::analytics::calculate_savings(source_code, "// [CACHE_HIT]", None)
            };
            return Ok(crate::compression::report::format_compact_cache_hit(
                meta.raw_tokens,
                meta.compressed_tokens,
                meta.savings_percentage,
                &path_alias,
            ));
        }

        let cached_notice = format!(
            "// [CACHE_HIT] {} unchanged. Use historic memory.\n",
            path_alias
        );
        let meta = if let Some(raw_tokens) = cache.get_raw_token_count(&current_hash) {
            let bpe = crate::analytics::bpe();
            let compressed_tokens = bpe.encode_with_special_tokens(&cached_notice).len();
            let savings_percentage = if raw_tokens > 0 {
                let saved = raw_tokens.saturating_sub(compressed_tokens);
                (saved as f64 / raw_tokens as f64) * 100.0
            } else {
                0.0
            };
            crate::analytics::TokenMetadata {
                raw_tokens,
                compressed_tokens,
                savings_percentage,
            }
        } else {
            crate::analytics::calculate_savings(source_code, &cached_notice, None)
        };

        let ratio_report = format!(
            "// Structures: cached, cached, cached | {}/{} tokens",
            meta.raw_tokens, meta.compressed_tokens
        );
        return Ok(format!(
            "// --- Token Optimization Report --- \n// Raw Tokens: {} | Retained Tokens: {} | Waste Reduced: {:.2}%\n// Fidelity: {:?}\n// {}\n{}",
            meta.raw_tokens, meta.compressed_tokens, meta.savings_percentage, fidelity, ratio_report, cached_notice
        ));
    }

    // Determine file extension from path
    let extension = std::path::Path::new(absolute_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");
    let (language, query_string) = crate::compression::language::language_for_extension(extension)
        .ok_or_else(|| format!("Unsupported file extension: .{}", extension))?;

     let all_captures: Vec<CapEntry> = run_capture_pipeline(
         language,
         query_string,
         source_code,
         fidelity,
         |capture_name, raw, f| {
             match capture_name {
                 "class.root" => Some(extract_class_name(raw)),
                 // Rust type declarations: struct, enum, trait, impl
                 "struct.root" | "trait.root" | "impl.root" => {
                     Some(extract_rust_struct_name(raw))
                 }
                 // Java type declarations: interface, enum, record
                 "interface.root" | "record.root" => {
                     Some(extract_java_type_name(raw, capture_name))
                 }
                 "method.root" => Some(extract_method_sig(raw, f)),
                 "constructor.root" => Some(extract_java_constructor_sig(raw, f)),
                 "field.root" => Some(extract_field(raw, f)),
                 // Rust mod declarations are structural like imports
                 "mod.root" => Some(compact_import(raw, f)),
                 // Java package declarations
                 "package.root" => Some(compact_java_package(raw, f)),
                 // Rust type aliases
                 "type.root" => Some(compact_expression(raw, f)),
                 _ => Some(compact_expression(raw, f)),
             }
         },
     )?;

     #[cfg(debug_assertions)]
     eprintln!("[compress_source] run_capture_pipeline returned {} captures", all_captures.len());

     let built = build_output_lines(&all_captures, source_code, fidelity, None);
     #[cfg(debug_assertions)]
     eprintln!("[compress_source] build_output_lines done");
     let mut body_content = assemble_body(&built.output_lines, fidelity);
     #[cfg(debug_assertions)]
     eprintln!("[compress_source] assemble_body done");
     if let Some(block) = &built.meta_block {
         body_content.push_str(&block.render());
     }
     #[cfg(debug_assertions)]
     eprintln!("[compress_source] meta_block rendered");
     // R-02: Apply type aliases before micro-opcodes so `$uid` tokens
     // are preserved. At Low fidelity types are already stripped.
     let ta_footer = if let Some(aliases) = aliases
        && !aliases.is_empty()
    {
        let (substituted, footer) = apply_type_aliases(&body_content, aliases);
        body_content = substituted;
        footer
    } else {
        String::new()
    };
     // Phase III (Idea #11 — Micro-Opcode Table for Text):
     // Apply micro-opcodes (§C, §P) for workspace-level compression too.
     body_content = apply_micro_opcodes(&body_content, fidelity);
     #[cfg(debug_assertions)]
     eprintln!("[compress_source] apply_micro_opcodes done");
     // NOTE: Symbol compression is NOT applied here — it will be applied
     // by the workspace-level global symbol table instead.
     let combined_footer = combine_footers("", &ta_footer);
     let compacted_body = crate::compression::report::format_compacted_body(
         &body_content,
         &combined_footer,
         &path_alias,
         fidelity,
     );
     #[cfg(debug_assertions)]
     eprintln!("[compress_source] format_compacted_body done");
     let raw_tokens = crate::analytics::bpe()
         .encode_with_special_tokens(source_code)
         .len();
     #[cfg(debug_assertions)]
     eprintln!("[compress_source] bpe encode done, {} tokens", raw_tokens);
     cache.store_raw_token_count(&current_hash, raw_tokens);

     #[cfg(debug_assertions)]
     eprintln!("[compress_source] store_raw_token_count done");
     let final_output = crate::compression::report::format_final_output(
         source_code,
         &compacted_body,
         fidelity,
         built.class_count,
         built.method_count,
         built.import_count,
     );
     #[cfg(debug_assertions)]
     eprintln!("[compress_source] format_final_output done");
     Ok(final_output)
}

/// Check if a capture should be skipped due to CBM filter-first rules.
///
/// Returns `true` if the capture's symbol name matches an entry in the
/// `skip_set`. Class, struct, enum, trait, impl, and interface captures
/// are checked by their `text` field (the extracted name). Method and
/// field captures are checked by their `text` field (the signature/name
/// which typically starts with the symbol name).
///
/// `cap.name` is the capture type (e.g. "class.root", "method.root")
/// and `cap.text` is the processed text (e.g. the class name or method
/// signature).
pub(crate) fn should_skip_capture(cap: &crate::compression::CapEntry, skip_set: &HashSet<String>) -> bool {
    // Type-like captures (class, struct, enum, trait, impl, interface, record):
    // check the full text against the skip set
    if matches!(
        cap.name.as_str(),
        "class.root" | "struct.root" | "enum.root" | "trait.root"
            | "impl.root" | "interface.root" | "record.root"
    ) {
        return skip_set.contains(cap.text.trim());
    }

    // Method captures: check the signature (first word is usually the method name)
    if matches!(cap.name.as_str(), "method.root" | "constructor.root" | "func.root" | "arrow.root") {
        if let Some(first_word) = cap.text.split_whitespace().next() {
            return skip_set.contains(first_word);
        }
        return skip_set.contains(cap.text.trim());
    }

    // Field captures: check the field name
    if cap.name == "field.root" {
        // Fields are typically "name: type" — take part before ":"
        if let Some(field_name) = cap.text.split(':').next() {
            return skip_set.contains(field_name.trim());
        }
        return skip_set.contains(cap.text.trim());
    }

    false
}

/// Combine the symbol-dictionary footer and the type-alias footer into
/// a single string. The `§TA` footer is appended after the `§SYM` footer
/// so both are present in the output. If either is empty, the other is
/// returned as-is.
fn combine_footers(sym_footer: &str, ta_footer: &str) -> String {
    if ta_footer.is_empty() {
        sym_footer.to_string()
    } else if sym_footer.is_empty() {
        ta_footer.to_string()
    } else {
        format!("{}\n{}", sym_footer, ta_footer)
    }
}

/// Join the body lines using the per-fidelity separator.
pub fn assemble_body(output_lines: &[String], fidelity: Fidelity) -> String {
    match fidelity {
        Fidelity::Low => output_lines.join(";"),
        Fidelity::Medium => output_lines.join("\n"),
        Fidelity::High => output_lines.join("\n"),
    }
}

#[cfg(test)]
#[path = "../tests/compression/pipeline.rs"]
mod tests;
