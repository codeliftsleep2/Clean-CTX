// src/compression/capture_pipeline.rs
//
// SHARED tree-sitter capture pipeline. Both `compressor::compress_file`
// and `diff/builder::try_build_with` walked a tree-sitter query in
// exactly the same way:
//
//   1. Build a `TSParser`, set the language, parse the source.
//   2. Compile a `Query`.
//   3. Run a `QueryCursor` over the root node.
//   4. For each capture, slice the source, normalise the text via
//      `compaction::*`, and collect a `Vec<CapEntry>` sorted by
//      document position.
//
// Phase 2 funnels both call sites through `run_capture_pipeline`. The
// orchestrators then walk the returned `Vec<CapEntry>` and emit output
// in their own formats (compact layout vs. structured snapshot).
//
// The shared entry point takes a closure (`process`) so the per-capture
// text normalisation stays with the caller — `compress_file` runs
// `extract_method_sig` etc. differently from `try_build_with` (the diff
// path also handles `import.root` here, the compressor does not).

use streaming_iterator::StreamingIterator;
use tree_sitter::{Language, Parser as TSParser, Query, QueryCursor};

use crate::compression::Fidelity;

/// A single tree-sitter capture, with the text already sliced from the
/// source and normalised. `start_byte` is the byte offset in the source
/// (not the character offset) and is used to sort captures in document
/// order before the orchestrator walks them.
#[derive(Debug, Clone)]
pub struct CapEntry {
    /// Capture name (e.g. "class.root", "method.root", "throw.root").
    pub name: String,
    /// Normalised text — what the caller chose to put here for this
    /// capture. For `class.root` it might be just the class name; for
    /// `method.root` it might be the compacted signature; for control
    /// flow it might be the marker string.
    pub text: String,
    /// Raw (unprocessed) text of the captured node in the source.
    /// Used by the IR compiler's language layers which need the full
    /// original text (e.g., class head with extends/implements) to
    /// extract relationships.
    pub raw_text: String,
    /// Byte offset of the start of the captured node in the source.
    pub start_byte: usize,
    /// Byte offset one past the end of the captured node in the source.
    ///
    /// Orchestrators use this for span-containment tests: the diff
    /// snapshot builder owns a member by the declaration whose
    /// `[start_byte, end_byte)` window contains the member, so a nested
    /// type that closed before the member starts can never steal it.
    pub end_byte: usize,
}

// F-08 (FAANG audit): the closure previously received a hard-coded
// `Fidelity::Low` regardless of what the caller asked for. That meant
// `compress_code_context` with `fidelity: "high"` still produced
// Low-fidelity method/field signatures. The function now takes a
// real `Fidelity` argument and threads it through to `process`.
pub fn run_capture_pipeline<F>(
    language: Language,
    query_string: &str,
    source: &str,
    fidelity: Fidelity,
    mut process: F,
) -> Result<Vec<CapEntry>, Box<dyn std::error::Error>>
where
    F: FnMut(&str, &str, Fidelity) -> Option<String>,
{
    let mut parser = TSParser::new();
    parser.set_language(&language)?;
    tracing::debug!("[run_capture_pipeline] language set");
    let tree = parser.parse(source, None).ok_or("AST Generation Error")?;
    tracing::debug!("[run_capture_pipeline] parsed, {} bytes", source.len());
    let source_bytes = source.as_bytes();

    let query = Query::new(&language, query_string)?;
    tracing::debug!("[run_capture_pipeline] query compiled");
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source_bytes);

    let mut all_captures: Vec<CapEntry> = Vec::new();
    while let Some(mat) = matches.next() {
        for capture in mat.captures.iter() {
            let capture_name = query.capture_names()[capture.index as usize].to_string();
            if let Ok(text_slice) = capture.node.utf8_text(source_bytes) {
                let raw = text_slice.to_string();
                if let Some(processed) = process(&capture_name, &raw, fidelity) {
                    all_captures.push(CapEntry {
                        name: capture_name,
                        text: processed,
                        raw_text: raw,
                        start_byte: capture.node.start_byte(),
                        end_byte: capture.node.end_byte(),
                    });
                }
            }
        }
    }

    // Sort captures by document position so the caller can walk them
    // in source order. This is the same `Vec::sort_by` both original
    // call sites performed.
    all_captures.sort_by_key(|a| a.start_byte);
    tracing::debug!("[rcp] DONE — returning {} captures", all_captures.len());
    Ok(all_captures)
}

#[cfg(test)]
#[path = "../tests/compression/capture_pipeline.rs"]
mod tests;
