// src/compression/streaming.rs
//
// Streaming compression orchestrator (`compress_file_streaming`). It is a
// thin wrapper around the same pipeline stages as the non-streaming variant
// (`pipeline::compress_file`), but reads the source file in chunks and
// reports progress via a caller-provided callback.

use std::fs;
use std::io::{BufReader, Read};
use std::path::PathBuf;

use crate::analytics::calculate_savings;
use crate::cache::LocalStateCache;
use crate::compaction::{
    compact_expression, extract_class_name, extract_field,
    extract_method_sig,
};
use crate::compression::capture_pipeline::run_capture_pipeline;
use crate::compression::language::language_for_extension;
use crate::compression::pipeline::{assemble_body, build_output_lines};
use crate::compression::report::{format_compacted_body, format_final_output};
use crate::compression::micro_opcodes::apply_micro_opcodes;
use crate::compression::symbol_compression::apply_symbol_compression;
use crate::compression::CapEntry;
use crate::compression::Fidelity;
use crate::dictionary::PathDictionary;

/// A progress event emitted by streaming compression. The `progress` is in
/// the range `0.0..=1.0`. `phase` describes the current step of the pipeline.
#[derive(Debug, Clone)]
pub struct CompressionProgress {
    /// Percentage complete in `[0.0, 1.0]`.
    pub progress: f64,
    /// Human-readable phase label, e.g. "reading", "parsing", "compressing".
    pub phase: String,
    /// Optional snippet or partial output for the current chunk.
    pub partial: Option<String>,
}

/// Streaming variant of [`compress_file`](crate::compression::pipeline::compress_file).
/// Reads the source file in chunks, reports progress via a caller-provided
/// callback, and delegates all structural logic to the shared pipeline
/// helpers.
pub fn compress_file_streaming<F>(
    file: PathBuf,
    dict: &mut PathDictionary,
    cache: &mut LocalStateCache,
    fidelity: Fidelity,
    chunk_bytes: usize,
    mut on_progress: F,
) -> Result<String, Box<dyn std::error::Error>>
where
    F: FnMut(CompressionProgress) -> Result<(), Box<dyn std::error::Error>>,
{
    let metadata = fs::metadata(&file)?;
    let total_bytes = metadata.len() as usize;
    // F-FINAL-05: Use the raw path as the alias key (not the
    // canonicalized path) for consistency with the non-streaming
    // variant and `bundle_pass` / `graph_pass`. On Windows, `canonicalize`
    // returns UNC paths that would produce a different alias from the
    // raw-path aliases used everywhere else.
    let absolute_path = file.to_string_lossy().into_owned();
    let path_alias = dict.get_or_create_alias(absolute_path.clone());

    // --- Phase 1: streaming read with progress reporting -----------------
    on_progress(CompressionProgress {
        progress: 0.0,
        phase: "reading".to_string(),
        partial: Some(format!("// Streaming read: {} ({} bytes)", path_alias, total_bytes)),
    })?;

    let f = fs::File::open(&file)?;
    let mut reader = BufReader::with_capacity(chunk_bytes.max(4096), f);
    let mut source_code = String::new();
    let mut buf = vec![0u8; chunk_bytes.max(4096)];
    let mut bytes_read: usize = 0;
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 { break; }
        let chunk = std::str::from_utf8(&buf[..n])
            .map_err(|e| format!("Invalid UTF-8 in source file: {}", e))?;
        source_code.push_str(chunk);
        bytes_read += n;
        let p = if total_bytes > 0 { (bytes_read as f64 / total_bytes as f64) * 0.2 } else { 0.2 };
        on_progress(CompressionProgress {
            progress: p,
            phase: "reading".to_string(),
            partial: None,
        })?;
    }

    let source_bytes = source_code.as_bytes();
    let current_hash = cache.compute_hash(source_bytes);

    // --- Cache short-circuit (consistent with non-streaming variant) ------
    let cache_key = format!("{}::{}", absolute_path, fidelity as u8);
    let is_modified = cache.update_and_verify(&cache_key, &current_hash);
    if !is_modified {
        // Phase III (Idea #8 — Progressive Header Elision):
        // Low fidelity uses the compact cache-hit format.
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
                calculate_savings(&source_code, "// [CACHE_HIT]", None)
            };
            let compact = crate::compression::report::format_compact_cache_hit(
                meta.raw_tokens,
                meta.compressed_tokens,
                meta.savings_percentage,
                &path_alias,
            );
            on_progress(CompressionProgress {
                progress: 1.0,
                phase: "cache-hit".to_string(),
                partial: Some(compact.clone()),
            })?;
            return Ok(compact);
        }

        let cached_notice = format!(
            "// [CACHE_HIT] {} unchanged. Use historic memory.\n",
            path_alias
        );

        // F-14: use the cached raw-token count to skip the BPE encode.
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
            calculate_savings(&source_code, &cached_notice, None)
        };

        on_progress(CompressionProgress {
            progress: 1.0,
            phase: "cache-hit".to_string(),
            partial: Some(cached_notice.clone()),
        })?;
        // F-04: same `cached, cached, cached` placeholder as the
        // non-streaming path so LLM clients see a consistent header.
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

    // --- Phase 2: parse AST ----------------------------------------------
    on_progress(CompressionProgress {
        progress: 0.22,
        phase: "parsing".to_string(),
        partial: None,
    })?;

    // --- Phase 3: extract captures with the SHARED pipeline -------------
    // F-08: pass the real `fidelity` through so the per-capture
    // closures use it (instead of always Low).
    let all_captures: Vec<CapEntry> = run_capture_pipeline(
        language,
        query_string,
        &source_code,
        fidelity,
        |capture_name, raw, f| {
            if capture_name == "class.root" {
                Some(extract_class_name(raw))
            } else if capture_name == "method.root" {
                Some(extract_method_sig(raw, f))
            } else if capture_name == "field.root" {
                Some(extract_field(raw, f))
            } else {
                Some(compact_expression(raw, f))
            }
        },
    )?;

    on_progress(CompressionProgress {
        progress: 0.4,
        phase: "compressing".to_string(),
        partial: None,
    })?;

    // Walk the captures and emit incremental progress events.
    // F-04: `build_output_lines` now returns `BuildOutputResult` with
    // real counts; we destructure only what we need here.
    let total_captures = all_captures.len();
    let built = build_output_lines(&all_captures, &source_code, fidelity, None);
    for idx in 0..total_captures {
        let p = 0.4 + (idx as f64 / total_captures.max(1) as f64) * 0.5;
        on_progress(CompressionProgress {
            progress: p,
            phase: "compressing".to_string(),
            partial: None,
        })?;
    }

    let mut body_content = assemble_body(&built.output_lines, fidelity);
    // Angular Meta-Layer (Phase 1): inject the Φ block into the body
    // BEFORE symbol compression so the `Φ` markers stay untouched.
    if let Some(block) = &built.meta_block {
        body_content.push_str(&block.render());
    }

    // Phase III (Idea #11 — Micro-Opcode Table for Text):
    // Apply micro-opcodes before symbol compression.
    body_content = apply_micro_opcodes(&body_content, fidelity);

    // --- Phase 4: symbol opcode compression (Low fidelity only) ----------
    let (display_body, sym_footer) = apply_symbol_compression(&body_content, fidelity);

    // --- Phase 5: assemble final report ----------------------------------
    on_progress(CompressionProgress {
        progress: 0.9,
        phase: "assembling".to_string(),
        partial: None,
    })?;

    let compacted_body = format_compacted_body(&display_body, &sym_footer, &path_alias, fidelity);
    // F-14: store the raw-token count for this content hash so the
    // cache-hit path can skip the BPE encode on subsequent calls.
    let raw_tokens = crate::analytics::bpe()
        .encode_with_special_tokens(&source_code)
        .len();
    cache.store_raw_token_count(&current_hash, raw_tokens);

    // F-04: pass real counts from `built`.
    let final_output = format_final_output(
        &source_code,
        &compacted_body,
        fidelity,
        built.class_count,
        built.method_count,
        built.import_count,
    );

    on_progress(CompressionProgress {
        progress: 1.0,
        phase: "done".to_string(),
        partial: Some(final_output.clone()),
    })?;

    Ok(final_output)
}

#[cfg(test)]
#[path = "../tests/compression/streaming.rs"]
mod tests;
