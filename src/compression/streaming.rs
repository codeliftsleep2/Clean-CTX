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
    let absolute_path = fs::canonicalize(&file)?.to_string_lossy().into_owned();
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
    let cache_key = format!("{}::{:?}", absolute_path, fidelity);
    let is_modified = cache.update_and_verify(cache_key, current_hash);
    if !is_modified {
        let cached_notice = format!(
            "// [CACHE_HIT] {} unchanged. Use historic memory.\n",
            path_alias
        );
        let meta = calculate_savings(&source_code, &cached_notice);
        on_progress(CompressionProgress {
            progress: 1.0,
            phase: "cache-hit".to_string(),
            partial: Some(cached_notice.clone()),
        })?;
        return Ok(format!(
            "// --- Token Optimization Report --- \n// Raw Tokens: {} | Retained Tokens: {} | Waste Reduced: {:.2}%\n// Fidelity: {:?}\n{}",
            meta.raw_tokens, meta.compressed_tokens, meta.savings_percentage, fidelity, cached_notice
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
    let all_captures: Vec<CapEntry> = run_capture_pipeline(
        language,
        query_string,
        &source_code,
        |capture_name, raw, _low| {
            if capture_name == "class.root" {
                Some(extract_class_name(raw))
            } else if capture_name == "method.root" {
                Some(extract_method_sig(raw, fidelity))
            } else if capture_name == "field.root" {
                Some(extract_field(raw, fidelity))
            } else {
                Some(compact_expression(raw, fidelity))
            }
        },
    )?;

    on_progress(CompressionProgress {
        progress: 0.4,
        phase: "compressing".to_string(),
        partial: None,
    })?;

    // Walk the captures and emit incremental progress events
    let total_captures = all_captures.len();
    let (output_lines, _imports) = build_output_lines(&all_captures, &source_code, fidelity);
    for idx in 0..total_captures {
        let p = 0.4 + (idx as f64 / total_captures.max(1) as f64) * 0.5;
        on_progress(CompressionProgress {
            progress: p,
            phase: "compressing".to_string(),
            partial: None,
        })?;
    }

    let body_content = assemble_body(&output_lines, fidelity);

    // --- Phase 4: symbol opcode compression (Low fidelity only) ----------
    let (display_body, sym_footer) = apply_symbol_compression(&body_content, fidelity);

    // --- Phase 5: assemble final report ----------------------------------
    on_progress(CompressionProgress {
        progress: 0.9,
        phase: "assembling".to_string(),
        partial: None,
    })?;

    let compacted_body = format_compacted_body(&display_body, &sym_footer, &path_alias, fidelity);
    let final_output = format_final_output(&source_code, &compacted_body, fidelity, 0, 0, 0);

    on_progress(CompressionProgress {
        progress: 1.0,
        phase: "done".to_string(),
        partial: Some(final_output.clone()),
    })?;

    Ok(final_output)
}