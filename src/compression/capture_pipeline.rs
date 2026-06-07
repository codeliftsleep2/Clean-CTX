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
    /// Byte offset of the start of the captured node in the source.
    pub start_byte: usize,
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
    parser.set_language(language)?;
    let tree = parser.parse(source, None).ok_or("AST Generation Error")?;
    let source_bytes = source.as_bytes();

    let query = Query::new(language, query_string)?;
    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(&query, tree.root_node(), source_bytes);

    let mut all_captures: Vec<CapEntry> = Vec::new();
    for mat in matches {
        for capture in mat.captures {
            let capture_name = query.capture_names()[capture.index as usize].to_string();
            if let Ok(text_slice) = capture.node.utf8_text(source_bytes) {
                let raw = text_slice.to_string();
                if let Some(processed) = process(&capture_name, &raw, fidelity) {
                    all_captures.push(CapEntry {
                        name: capture_name,
                        text: processed,
                        start_byte: capture.node.start_byte(),
                    });
                }
            }
        }
    }

    // Sort captures by document position so the caller can walk them
    // in source order. This is the same `Vec::sort_by` both original
    // call sites performed.
    all_captures.sort_by_key(|a| a.start_byte);
    Ok(all_captures)
}

#[cfg(test)]
#[path = "../tests/compression/capture_pipeline.rs"]
mod tests;
