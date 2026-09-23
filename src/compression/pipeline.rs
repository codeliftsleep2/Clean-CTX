// src/compression/pipeline.rs
//
// Non-streaming compression orchestrators (`compress_file`,
// `compress_file_with_source`, `compress_text`, `compress_source`). Output
// assembly (`build_output_lines`, `assemble_body`, `combine_footers`) lives in
// `output.rs`; CBM filter-first capture skipping lives in `skip.rs`.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use crate::analytics::calculate_savings;

/// F-18 (FAANG audit): maximum file size in bytes that `compress_file`
/// will read into memory. Files larger than this return a clean error
/// instead of risking an OOM.
const MAX_FILE_BYTES: u64 = 10 * 1024 * 1024; // 10 MB

/// P1-10: Unified token metadata computation for cache-hit paths.
/// Ensures consistent token counting between cache-hit and cache-miss paths.
fn compute_token_metadata(
    raw_tokens: usize,
    compressed_text: &str,
) -> crate::analytics::TokenMetadata {
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
use crate::compaction::java::{
    compact_java_package, extract_java_constructor_sig, extract_java_type_name,
};
use crate::compaction::{
    compact_expression, compact_import, extract_class_name, extract_field, extract_method_sig,
    extract_rust_struct_name,
};
use crate::compression::CapEntry;
use crate::compression::Fidelity;
use crate::compression::capture_pipeline::run_capture_pipeline;
use crate::compression::language::language_for_extension;
use crate::compression::micro_opcodes::apply_micro_opcodes;
pub(crate) use crate::compression::output::{assemble_body, build_output_lines, combine_footers};
use crate::compression::report::{format_compacted_body, format_final_output};
pub(crate) use crate::compression::skip::should_skip_capture;
use crate::compression::symbol_compression::apply_symbol_compression;
use crate::compression::type_aliases::apply_type_aliases;
use crate::dictionary::PathDictionary;

/// Reads a target source file and compiles it down into a highly compacted,
/// keyword-stripped structural signature stream with configurable fidelity.
///
/// F-18: returns a [`CompressionError::FileTooLarge`] if the file exceeds
/// [`MAX_FILE_BYTES`].
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
        let meta = fs::metadata(&file)?;
        let file_size = meta.len();
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
    let absolute_path = file.to_string_lossy().to_string();
    let path_alias = dict.get_or_create_alias(absolute_path.clone());

    let cache_key = format!("{}::{}", absolute_path, fidelity as u8);
    let is_modified = cache.update_and_verify(&cache_key, &current_hash);
    if !is_modified {
        // C-8: at Edit/Verbatim a cache hit must return the raw source
        // byte-exact (no token-report notice that would break byte-exactness).
        if fidelity == Fidelity::Edit || fidelity == Fidelity::Verbatim {
            return Ok(source_code);
        }
        let cached_notice = format!(
            "// [CACHE_HIT] {} unchanged. Use historic memory.\n",
            path_alias
        );

        let meta = if let Some(raw_tokens) = cache.get_raw_token_count(&current_hash) {
            compute_token_metadata(raw_tokens, &cached_notice)
        } else {
            calculate_savings(&source_code, &cached_notice, None)
        };

        if fidelity == Fidelity::Low {
            return Ok(crate::compression::report::format_compact_cache_hit(
                meta.raw_tokens,
                meta.compressed_tokens,
                meta.savings_percentage,
                &path_alias,
            ));
        }

        let ratio_report = format!(
            "// Structures: cached, cached, cached | {}/{} tokens",
            meta.raw_tokens, meta.compressed_tokens
        );
        return Ok(format!(
            "// --- Token Optimization Report --- \n// Raw Tokens: {} | Retained Tokens: {} | Waste Reduced: {:.2}%\n// Fidelity: {:?}\n// {}\n{}",
            meta.raw_tokens,
            meta.compressed_tokens,
            meta.savings_percentage,
            fidelity,
            ratio_report,
            cached_notice
        ));
    }

    let extension = file.extension().and_then(|ext| ext.to_str()).unwrap_or("");
    let (language, query_string) = language_for_extension(extension)
        .ok_or_else(|| format!("Unsupported file extension: .{}", extension))?;

    let all_captures: Vec<CapEntry> = run_capture_pipeline(
        language,
        query_string,
        &source_code,
        fidelity,
        |capture_name, raw, f| match capture_name {
            "class.root" => Some(extract_class_name(raw)),
            "struct.root" | "trait.root" | "impl.root" => Some(extract_rust_struct_name(raw)),
            "interface.root" | "record.root" => Some(extract_java_type_name(raw, capture_name)),
            "method.root" => Some(extract_method_sig(raw, f)),
            "constructor.root" => Some(extract_java_constructor_sig(raw, f)),
            "field.root" => Some(extract_field(raw, f)),
            "mod.root" => Some(compact_import(raw, f)),
            "package.root" => Some(compact_java_package(raw, f)),
            "type.root" => Some(compact_expression(raw, f)),
            _ => Some(compact_expression(raw, f)),
        },
    )?;

    let built = build_output_lines(&all_captures, &source_code, fidelity, None, config);
    let mut body_content = assemble_body(&built.output_lines, fidelity);
    // C-11: at Edit/Verbatim the Φ meta blocks must NOT be injected (they
    // would corrupt byte-exact method bodies).
    if fidelity != Fidelity::Edit && fidelity != Fidelity::Verbatim {
        if let Some(block) = &built.meta_block {
            body_content.push_str(&block.render());
        }
        if let Some(block) = &built.spring_meta_block {
            body_content.push_str(&block.render());
        }
        if let Some(block) = &built.dotnet_meta_block {
            body_content.push_str(&block.render());
        }
    }
    // C-5: at Edit/Verbatim type-alias substitution must be skipped so
    // byte-exact method bodies are never rewritten.
    let ta_footer = if let Some(cfg) = config
        && !cfg.type_aliases.is_empty()
        && fidelity != Fidelity::Edit
        && fidelity != Fidelity::Verbatim
    {
        let (substituted, footer) = apply_type_aliases(&body_content, &cfg.type_aliases);
        body_content = substituted;
        footer
    } else {
        String::new()
    };
    body_content = apply_micro_opcodes(&body_content, fidelity);
    let (display_body, sym_footer) = apply_symbol_compression(&body_content, fidelity);
    let combined_footer = combine_footers(&sym_footer, &ta_footer);
    let compacted_body =
        format_compacted_body(&display_body, &combined_footer, &path_alias, fidelity);
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
        |capture_name, raw, f| match capture_name {
            "class.root" => Some(extract_class_name(raw)),
            "struct.root" | "trait.root" | "impl.root" => Some(extract_rust_struct_name(raw)),
            "interface.root" | "record.root" => Some(extract_java_type_name(raw, capture_name)),
            "method.root" => Some(extract_method_sig(raw, f)),
            "constructor.root" => Some(extract_java_constructor_sig(raw, f)),
            "field.root" => Some(extract_field(raw, f)),
            "mod.root" => Some(compact_import(raw, f)),
            "package.root" => Some(compact_java_package(raw, f)),
            "type.root" => Some(compact_expression(raw, f)),
            _ => Some(compact_expression(raw, f)),
        },
    )?;

    let built = build_output_lines(&all_captures, source_code, fidelity, None, None);
    let mut body_content = assemble_body(&built.output_lines, fidelity);
    // C-11: at Edit/Verbatim the Φ meta blocks must NOT be injected (they
    // would corrupt byte-exact method bodies). Mirrors the guard already
    // present in `compress_file_with_source`.
    if fidelity != Fidelity::Edit && fidelity != Fidelity::Verbatim {
        if let Some(block) = &built.meta_block {
            body_content.push_str(&block.render());
        }
        if let Some(block) = &built.spring_meta_block {
            body_content.push_str(&block.render());
        }
        if let Some(block) = &built.dotnet_meta_block {
            body_content.push_str(&block.render());
        }
    }
    // C-5: at Edit/Verbatim type-alias substitution must be skipped so
    // byte-exact method bodies are never rewritten.
    let ta_footer = if let Some(aliases) = aliases
        && !aliases.is_empty()
        && fidelity != Fidelity::Edit
        && fidelity != Fidelity::Verbatim
    {
        let (substituted, footer) = apply_type_aliases(&body_content, aliases);
        body_content = substituted;
        footer
    } else {
        String::new()
    };
    body_content = apply_micro_opcodes(&body_content, fidelity);
    let (display_body, sym_footer) = apply_symbol_compression(&body_content, fidelity);

    let body_lines: Vec<String> = display_body.lines().map(String::from).collect();

    let combined_footer = combine_footers(&sym_footer, &ta_footer);
    let compacted_body =
        format_compacted_body(&display_body, &combined_footer, path_alias, fidelity);
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

/// Compress source code from a string (not from a file path).
///
/// `config` is threaded through to the meta-layer registry so per-framework
/// `enabled` flags and sub-layer settings (min_pipe_operators,
/// include_dispatch_sites, etc.) are honored. When `None`, all meta-layers
/// run with their defaults.
/// Only used by the workspace compressor (retired in Phase C1);
/// kept for test-only stats integration.
#[allow(dead_code)]
pub fn compress_source(
    source_code: &str,
    absolute_path: &str,
    dict: &mut PathDictionary,
    cache: &mut LocalStateCache,
    fidelity: Fidelity,
    config: Option<&crate::config::CleanCtxConfig>,
    aliases: Option<&BTreeMap<String, String>>,
) -> Result<String, Box<dyn std::error::Error>> {
    let source_bytes = source_code.as_bytes();
    let current_hash = cache.compute_hash(source_bytes);
    let path_alias = dict.get_or_create_alias(absolute_path.to_string());

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
            meta.raw_tokens,
            meta.compressed_tokens,
            meta.savings_percentage,
            fidelity,
            ratio_report,
            cached_notice
        ));
    }

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
        |capture_name, raw, f| match capture_name {
            "class.root" => Some(extract_class_name(raw)),
            "struct.root" | "trait.root" | "impl.root" => Some(extract_rust_struct_name(raw)),
            "interface.root" | "record.root" => Some(extract_java_type_name(raw, capture_name)),
            "method.root" => Some(extract_method_sig(raw, f)),
            "constructor.root" => Some(extract_java_constructor_sig(raw, f)),
            "field.root" => Some(extract_field(raw, f)),
            "mod.root" => Some(compact_import(raw, f)),
            "package.root" => Some(compact_java_package(raw, f)),
            "type.root" => Some(compact_expression(raw, f)),
            _ => Some(compact_expression(raw, f)),
        },
    )?;

    // F-04/compress_source: append ALL meta-layer blocks (Angular, Spring
    // Boot, and .NET), not just the Angular one. Previously the Spring and
    // .NET blocks were silently dropped in this workspace global-symbol path.
    // The `config` is threaded through so per-framework `enabled` flags and
    // sub-layer settings are honored (previously hardcoded to `None`).
    let built = build_output_lines(&all_captures, source_code, fidelity, None, config);
    let mut body_content = assemble_body(&built.output_lines, fidelity);
    // C-11: at Edit/Verbatim the Φ meta blocks must NOT be injected (they
    // would corrupt byte-exact method bodies). Mirrors the guard already
    // present in `compress_file_with_source`.
    if fidelity != Fidelity::Edit && fidelity != Fidelity::Verbatim {
        if let Some(block) = &built.meta_block {
            body_content.push_str(&block.render());
        }
        if let Some(block) = &built.spring_meta_block {
            body_content.push_str(&block.render());
        }
        if let Some(block) = &built.dotnet_meta_block {
            body_content.push_str(&block.render());
        }
    }
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
    let combined_footer = combine_footers("", &ta_footer);
    let compacted_body = crate::compression::report::format_compacted_body(
        &body_content,
        &combined_footer,
        &path_alias,
        fidelity,
    );
    let raw_tokens = crate::analytics::bpe()
        .encode_with_special_tokens(source_code)
        .len();
    cache.store_raw_token_count(&current_hash, raw_tokens);

    let final_output = crate::compression::report::format_final_output(
        source_code,
        &compacted_body,
        fidelity,
        built.class_count,
        built.method_count,
        built.import_count,
    );
    Ok(final_output)
}

#[cfg(test)]
#[path = "../tests/compression/pipeline.rs"]
mod tests;

#[cfg(test)]
#[path = "../tests/compression/pipeline_anatomy.rs"]
mod pipeline_anatomy_tests;
