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

// Test-only instrumentation: full-file parses performed on the CURRENT
// thread. It exists to prove the invariant that adding capture queries (the
// native invocation captures) never adds a `Parser::parse` — the producer
// rides the existing walk.
//
// Thread-local so concurrently running tests cannot pollute each other's
// measurement. Compiled only for C#-enabled test builds, which is exactly
// where it is consumed.
#[cfg(all(test, feature = "csharp"))]
thread_local! {
    static PARSE_COUNT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Test-only: parses performed on this thread since the last reset.
#[cfg(all(test, feature = "csharp"))]
pub(crate) fn parse_count() -> usize {
    PARSE_COUNT.with(|count| count.get())
}

/// Test-only: reset this thread's parse counter.
#[cfg(all(test, feature = "csharp"))]
pub(crate) fn reset_parse_count() {
    PARSE_COUNT.with(|count| count.set(0));
}

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

/// A single capture together with the identity of the query match that
/// produced it.
///
/// One syntactic construct can bind several captures (a query pattern like
/// `(invocation_expression function: (…) @call.callee arguments: (…) @call.argument)`
/// binds a callee and its arguments in the SAME match). A consumer that must
/// assemble one fact from several captures groups by `match_index`.
///
/// `match_index` is a document-order counter over the matches visited during
/// one walk: it is stable and comparable only within the `Vec` returned by a
/// single [`run_capture_pipeline_nodes`] call. Captures of one match are
/// emitted by the pipeline in ascending `start_byte` order, and — because the
/// sort is stable — in pattern order when start bytes are equal, so a field
/// capture always precedes the child captures nested inside it.
#[derive(Debug, Clone)]
pub struct CapturedNode {
    /// Capture name (e.g. "class.root", "call.callee").
    pub name: String,
    /// Normalised text chosen by the caller's `process` closure.
    pub text: String,
    /// Raw (unprocessed) text of the captured node in the source.
    pub raw_text: String,
    /// Byte offset of the start of the captured node in the source.
    pub start_byte: usize,
    /// Byte offset one past the end of the captured node in the source.
    pub end_byte: usize,
    /// Identity of the query match that produced this capture.
    pub match_index: usize,
}

impl CapturedNode {
    /// Project this node onto the public [`CapEntry`] shape, dropping the
    /// query-match identity (consumers that only need one capture per
    /// construct never consult it).
    pub fn to_entry(&self) -> CapEntry {
        CapEntry {
            name: self.name.clone(),
            text: self.text.clone(),
            raw_text: self.raw_text.clone(),
            start_byte: self.start_byte,
            end_byte: self.end_byte,
        }
    }
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
    process: F,
) -> Result<Vec<CapEntry>, Box<dyn std::error::Error>>
where
    F: FnMut(&str, &str, Fidelity) -> Option<String>,
{
    Ok(
        run_capture_pipeline_nodes(language, query_string, source, fidelity, process)?
            .iter()
            .map(CapturedNode::to_entry)
            .collect(),
    )
}

/// Walk `query_string` over ONE tree-sitter parse of `source` and return every
/// capture with its query-match identity.
///
/// This is the single capture-walk implementation; [`run_capture_pipeline`] is
/// the [`CapEntry`] projection of it. Callers that must assemble a fact from
/// several captures of one construct (the generic call-fact producer) use this
/// entry point; they still parse the source exactly once.
pub fn run_capture_pipeline_nodes<F>(
    language: Language,
    query_string: &str,
    source: &str,
    fidelity: Fidelity,
    mut process: F,
) -> Result<Vec<CapturedNode>, Box<dyn std::error::Error>>
where
    F: FnMut(&str, &str, Fidelity) -> Option<String>,
{
    let mut parser = TSParser::new();
    parser.set_language(&language)?;
    tracing::debug!("[run_capture_pipeline] language set");
    #[cfg(all(test, feature = "csharp"))]
    PARSE_COUNT.with(|count| count.set(count.get() + 1));
    let tree = parser.parse(source, None).ok_or("AST Generation Error")?;
    tracing::debug!("[run_capture_pipeline] parsed, {} bytes", source.len());
    let source_bytes = source.as_bytes();

    let query = Query::new(&language, query_string)?;
    tracing::debug!("[run_capture_pipeline] query compiled");
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source_bytes);

    let mut all_captures: Vec<CapturedNode> = Vec::new();
    let mut match_index: usize = 0;
    while let Some(mat) = matches.next() {
        for capture in mat.captures.iter() {
            let capture_name = query.capture_names()[capture.index as usize].to_string();
            if let Ok(text_slice) = capture.node.utf8_text(source_bytes) {
                let raw = text_slice.to_string();
                if let Some(processed) = process(&capture_name, &raw, fidelity) {
                    all_captures.push(CapturedNode {
                        name: capture_name,
                        text: processed,
                        raw_text: raw,
                        start_byte: capture.node.start_byte(),
                        end_byte: capture.node.end_byte(),
                        match_index,
                    });
                }
            }
        }
        match_index += 1;
    }

    // Sort captures by document position so the caller can walk them
    // in source order. This is the same `Vec::sort_by` both original
    // call sites performed. The sort is STABLE, so captures that start at the
    // same byte keep their pattern order (a field capture precedes the child
    // captures nested inside it).
    all_captures.sort_by_key(|a| a.start_byte);
    tracing::debug!("[rcp] DONE — returning {} captures", all_captures.len());
    Ok(all_captures)
}

#[cfg(test)]
#[path = "../tests/compression/capture_pipeline.rs"]
mod tests;
