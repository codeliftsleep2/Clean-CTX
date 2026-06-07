// src/compressor.rs
//
// Public entry points for the compression pipeline. The actual
// pipeline is decomposed into focused submodules under
// `crate::compression`; this file is a thin orchestrator.
//
// Phase 2 notes:
//   - The capture-processing pipeline now lives in
//     `crate::compression::capture_pipeline::run_capture_pipeline`.
//   - Marker construction now lives in
//     `crate::compression::markers::build_marker`.
//   - Language detection now lives in `crate::compression::language`.
//   - The primitive opcode table now lives in
//     `crate::compression::opcodes::PRIMITIVE_OPCODES`.
//   - `Fidelity` now lives in `crate::compression::fidelity::Fidelity`.
//     `pub use crate::compression::Fidelity;` re-exports it here for
//     backward compatibility with the historical
//     `crate::compressor::Fidelity` import path.

use std::fs;
use std::io::{BufReader, Read};
use std::path::PathBuf;

use crate::analytics::calculate_savings;
use crate::cache::LocalStateCache;
use crate::compaction::{
    compact_expression, compact_import, extract_class_name, extract_field,
    extract_method_sig, format_class_entry, simple_compact,
};
use crate::compression::capture_pipeline::run_capture_pipeline;
use crate::compression::language::language_for_extension;
use crate::compression::markers::build_marker;
use crate::compression::CapEntry;
use crate::dictionary::{PathDictionary, SymbolDictionary};

// Re-export the shared `Fidelity` enum so external callers and the old
// `use crate::compressor::Fidelity;` import statements keep working.
// The type is the same one as `crate::compression::Fidelity`.
pub use crate::compression::Fidelity;

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

/// Reads a target source file and compiles it down into a highly compacted,
/// keyword-stripped structural signature stream with configurable fidelity.
pub fn compress_file(
    file: PathBuf,
    dict: &mut PathDictionary,
    cache: &mut LocalStateCache,
    fidelity: Fidelity,
) -> Result<String, Box<dyn std::error::Error>> {
    let source_code = fs::read_to_string(&file)?;
    let source_bytes = source_code.as_bytes();

    let current_hash = cache.compute_hash(source_bytes);
    let absolute_path = fs::canonicalize(&file)?.to_string_lossy().into_owned();
    let path_alias = dict.get_or_create_alias(absolute_path.clone());

    // Include fidelity in the cache key so different fidelity levels don't share cached results
    let cache_key = format!("{}::{:?}", absolute_path, fidelity);
    let is_modified = cache.update_and_verify(cache_key, current_hash);
    if !is_modified {
        let cached_notice = format!(
            "// [CACHE_HIT] {} unchanged. Use historic memory.\n",
            path_alias
        );
        let meta = calculate_savings(&source_code, &cached_notice);

        return Ok(format!(
            "// --- Token Optimization Report --- \n// Raw Tokens: {} | Retained Tokens: {} | Waste Reduced: {:.2}%\n// Fidelity: {:?}\n{}",
            meta.raw_tokens, meta.compressed_tokens, meta.savings_percentage, fidelity, cached_notice
        ));
    }

    let extension = file.extension().and_then(|ext| ext.to_str()).unwrap_or("");
    let (language, query_string) = language_for_extension(extension)
        .ok_or_else(|| format!("Unsupported file extension: .{}", extension))?;

    // Run the SHARED capture pipeline. The closure maps each
    // (capture_name, raw_text) pair to the normalised text the
    // compressor wants stored in the resulting CapEntry.
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

    let (output_lines, imports) = build_output_lines(&all_captures, &source_code, fidelity);
    let body_content = assemble_body(&output_lines, fidelity);
    let (display_body, sym_footer) = apply_symbol_compression(&body_content, fidelity);
    let compacted_body = format_compacted_body(&display_body, &sym_footer, &path_alias, fidelity);
    let final_output = format_final_output(&source_code, &compacted_body, fidelity, 0, 0, 0);
    Ok(final_output)
}

/// Streaming variant of [`compress_file`]. See `compress_file` for the
/// pipeline; the streaming variant adds chunked reads and a progress
/// callback. Phase 2 unifies the capture pipeline and marker
/// construction with the non-streaming variant — both call into the
/// same `crate::compression::*` modules.
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

    // Walk the captures and emit incremental progress events. We can't
    // emit from inside the capture closure (it's `FnMut` and the
    // `on_progress` callback would re-enter it), so we walk twice: once
    // to produce output, once to emit progress.
    let total_captures = all_captures.len();
    let (output_lines, imports) = build_output_lines(&all_captures, &source_code, fidelity);
    for (idx, _cap) in all_captures.iter().enumerate() {
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

// ---------------------------------------------------------------------------
// Pipeline helpers
// ---------------------------------------------------------------------------
//
// These pure helpers are the inner building blocks of the orchestrators.
// Phase 2 extracts them from the 600-line monolith so each step is testable
// in isolation; the orchestrators are now 50–100 lines each.

/// Walk the captures in document order and build the output lines + the
/// imports list. Shared between the streaming and non-streaming variants.
fn build_output_lines(
    all_captures: &[CapEntry],
    source_code: &str,
    fidelity: Fidelity,
) -> (Vec<String>, Vec<String>) {
    let mut output_lines: Vec<String> = Vec::new();
    let mut fields: Vec<String> = Vec::new();
    let mut markers: Vec<String> = Vec::new();
    let mut imports: Vec<String> = Vec::new();
    let mut import_count: usize = 0;
    let mut class_count: usize = 0;
    let _ = (import_count, class_count);

    for cap in all_captures {
        match cap.name.as_str() {
            "import.root" => {
                imports.push(compact_import(&cap.text, fidelity));
            }
            "class.root" => {
                if !output_lines.is_empty() && (fidelity == Fidelity::High || fidelity == Fidelity::Medium) {
                    output_lines.push(String::new());
                }
                output_lines.push(format_class_entry(&cap.text, &fields, fidelity));
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
                if let Some(marker) = build_marker(&cap.name, &cap.text) {
                    if markers.last().map(|m| m != &marker).unwrap_or(true) {
                        markers.push(marker);
                    }
                }
            }
        }
    }

    // Orphaned fields with no class context
    if !fields.is_empty() && output_lines.is_empty() && fidelity != Fidelity::Low {
        output_lines.push(format!("⊕fields {{ {} }}", fields.join("; ")));
    }

    // Raw fallback when nothing was captured
    if output_lines.is_empty() {
        if let Some(first_line) = source_code.lines().next() {
            let trimmed = first_line.trim().to_string();
            if !trimmed.is_empty() {
                output_lines.push(simple_compact(&trimmed, fidelity));
            }
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
    (output, imports)
}

/// Join the body lines using the per-fidelity separator.
fn assemble_body(output_lines: &[String], fidelity: Fidelity) -> String {
    match fidelity {
        Fidelity::Low => output_lines.join(";"),
        Fidelity::Medium => output_lines.join("\n"),
        Fidelity::High => output_lines.join("\n"),
    }
}

/// Apply the symbol-dictionary opcode pass (Low fidelity only). Higher
/// fidelities don't need it — the structural markers already provide
/// sufficient density.
fn apply_symbol_compression(body_content: &str, fidelity: Fidelity) -> (String, String) {
    if fidelity != Fidelity::Low {
        return (body_content.to_string(), String::new());
    }
    let mut sym_dict = SymbolDictionary::new();
    for token in body_content.split_whitespace() {
        let clean = token.trim_matches(|c: char| {
            c == '(' || c == ')' || c == '[' || c == ']' || c == '{' || c == '}'
                || c == '<' || c == '>' || c == ':' || c == ';' || c == ',' || c == '.'
        });
        if !clean.is_empty() {
            sym_dict.register(clean);
        }
        if let Some(rest) = token.strip_prefix('⊕') {
            if !rest.is_empty() {
                sym_dict.register(rest);
            }
        }
    }
    let encoded = sym_dict.encode(body_content);
    let footer = sym_dict.format_footer();
    (encoded, footer)
}

fn format_compacted_body(
    display_body: &str,
    sym_footer: &str,
    path_alias: &str,
    fidelity: Fidelity,
) -> String {
    let layout_header = match fidelity {
        Fidelity::Low => format!("// --- Compacted Layout (Low Fidelity): {} ---", path_alias),
        Fidelity::Medium => format!("// --- Enhanced Layout (Medium Fidelity): {} ---", path_alias),
        Fidelity::High => format!("// --- Full Layout (High Fidelity): {} ---", path_alias),
    };
    if sym_footer.is_empty() {
        format!("{}\n{}\n", layout_header, display_body)
    } else {
        format!("{}\n{}\n{}", layout_header, display_body, sym_footer)
    }
}

fn format_final_output(
    source_code: &str,
    compacted_body: &str,
    fidelity: Fidelity,
    class_count: usize,
    method_count: usize,
    import_count: usize,
) -> String {
    let meta = calculate_savings(source_code, compacted_body);
    let ratio_report = format!(
        "// Structures: {} classes, {} methods, {} imports | {}/{} raw tokens",
        class_count, method_count, import_count, meta.raw_tokens, meta.raw_tokens
    );
    format!(
        "// --- Token Optimization Report --- \n// Raw Tokens: {} | Retained Tokens: {} | Waste Reduced: {:.2}%\n// Fidelity: {:?}\n// {}\n{}",
        meta.raw_tokens, meta.compressed_tokens, meta.savings_percentage, fidelity, ratio_report, compacted_body
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compressor::Fidelity;

    #[test]
    fn assemble_body_uses_semicolon_at_low() {
        let lines = vec!["a".to_string(), "b".to_string()];
        assert_eq!(assemble_body(&lines, Fidelity::Low), "a;b");
    }

    #[test]
    fn assemble_body_uses_newline_at_medium() {
        let lines = vec!["a".to_string(), "b".to_string()];
        assert_eq!(assemble_body(&lines, Fidelity::Medium), "a\nb");
    }

    #[test]
    fn assemble_body_uses_newline_at_high() {
        let lines = vec!["a".to_string(), "b".to_string()];
        assert_eq!(assemble_body(&lines, Fidelity::High), "a\nb");
    }

    #[test]
    fn format_compacted_body_omits_footer_when_empty() {
        let out = format_compacted_body("BODY", "", "a1", Fidelity::Low);
        assert!(out.contains("Compacted Layout (Low Fidelity): a1"));
        assert!(out.contains("BODY"));
        assert!(!out.contains("§SYM"));
    }

    #[test]
    fn format_compacted_body_includes_footer_when_present() {
        let out = format_compacted_body("BODY", "§SYM\n  $1 = Foo", "a1", Fidelity::Low);
        assert!(out.contains("§SYM"));
    }
}