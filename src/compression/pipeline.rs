// src/compression/pipeline.rs
//
// Non-streaming compression orchestrator (`compress_file`). It owns the
// shared helper functions `build_output_lines` and `assemble_body` that
// were historically private to `compressor.rs`. These are `pub(crate)` so
// the streaming variant can also call them.

use std::fs;
use std::path::PathBuf;

use crate::analytics::calculate_savings;

/// F-18 (FAANG audit): maximum file size in bytes that `compress_file`
/// will read into memory. Files larger than this return a clean error
/// instead of risking an OOM. The streaming variant
/// (`compress_file_streaming`) can be used as a fallback for larger
/// files.
const MAX_FILE_BYTES: u64 = 10 * 1024 * 1024; // 10 MB
use crate::cache::LocalStateCache;
use crate::compaction::{
    compact_expression, compact_import, extract_class_name, extract_field,
    extract_method_sig, format_class_entry, simple_compact,
};
use crate::compression::capture_pipeline::run_capture_pipeline;
use crate::compression::language::language_for_extension;
use crate::compression::markers::build_marker;
use crate::compression::report::{format_compacted_body, format_final_output};
use crate::compression::symbol_compression::apply_symbol_compression;
use crate::compression::CapEntry;
use crate::compression::Fidelity;
use crate::dictionary::PathDictionary;
use crate::angular_meta::run_meta_layer;

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
) -> Result<String, Box<dyn std::error::Error>> {
    // F-18: guard against reading an unbounded file into memory.
    let meta = fs::metadata(&file)?;
    if meta.len() > MAX_FILE_BYTES {
        return Err(format!(
            "File too large ({} bytes; max {} bytes). \
             Use compress_workspace or the streaming variant for large files.",
            meta.len(),
            MAX_FILE_BYTES,
        )
        .into());
    }
    let source_code = fs::read_to_string(&file)?;
    let source_bytes = source_code.as_bytes();

    let current_hash = cache.compute_hash(source_bytes);
    let absolute_path = fs::canonicalize(&file)?.to_string_lossy().into_owned();
    let path_alias = dict.get_or_create_alias(absolute_path.clone());

    // Include fidelity in the cache key so different fidelity levels don't share cached results
    let cache_key = format!("{}::{}", absolute_path, fidelity as u8);
    let is_modified = cache.update_and_verify(&cache_key, &current_hash);
    if !is_modified {
        let cached_notice = format!(
            "// [CACHE_HIT] {} unchanged. Use historic memory.\n",
            path_alias
        );

        // F-14: use the cached raw-token count to skip the BPE encode.
        // If the count is not in the cache (e.g. an older session), fall
        // through to a fresh `calculate_savings` call.
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
            calculate_savings(&source_code, &cached_notice)
        };

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

    // F-04: `build_output_lines` now returns real counts.
    let built = build_output_lines(&all_captures, &source_code, fidelity);
    let mut body_content = assemble_body(&built.output_lines, fidelity);
    // Angular Meta-Layer (Phase 1): inject the Φ block into the body
    // BEFORE symbol compression so the `Φ` markers stay untouched.
    if let Some(block) = &built.meta_block {
        body_content.push_str(&block.render());
    }
    let (display_body, sym_footer) = apply_symbol_compression(&body_content, fidelity);
    let compacted_body = format_compacted_body(&display_body, &sym_footer, &path_alias, fidelity);
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

// ---------------------------------------------------------------------------
// Shared pipeline helpers
// ---------------------------------------------------------------------------

/// Walk the captures in document order and build the output lines +
/// the per-fidelity counts. Shared between the streaming and
/// non-streaming variants.
///
/// F-04: the return type is now `BuildOutputResult` (with the real
/// counts) instead of a `(Vec<String>, Vec<String>)` tuple.
pub fn build_output_lines(
    all_captures: &[CapEntry],
    source_code: &str,
    fidelity: Fidelity,
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
        match cap.name.as_str() {
            "import.root" => {
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
            "method.root" => {
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
    let meta_block = run_meta_layer(
        source_code,
        &class_captures,
        fidelity,
    );
    BuildOutputResult {
        output_lines: output,
        class_count,
        method_count,
        import_count,
        meta_block,
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


