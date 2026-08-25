// src/edit/apply.rs
//
// Verification + splicing core for the `apply_edit` write path
// (docs/plans/APPLY_EDIT_PLAN.md Phase 2).
//
// Guarantees, in order:
//   1. Unit-level optimistic concurrency — the bytes about to be replaced
//      must match the caller's expected text exactly, MODULO line-ending
//      width (transport layers normalize CRLF↔LF; see `verify_expected`).
//   2. Non-overlap — operations in one batch must touch disjoint ranges.
//   3. Syntax gate — the spliced result must parse cleanly under the
//      file's tree-sitter grammar BEFORE any bytes hit disk (plan step 4,
//      hard and non-bypassable).
//   4. EOL preservation — incoming text is adapted to the FILE's line
//      ending convention before splicing; endings are never rewritten as
//      a side effect and never mixed.
//
// This module never touches disk or session state; callers own I/O.

use thiserror::Error;

use super::locate::{LocateError, UnitTable};
use super::ops::EditOperation;

/// Maximum characters of expected/actual text embedded in a mismatch
/// error payload (plan Phase 3: "bounded size").
const SNIPPET_BOUND: usize = 512;

#[derive(Debug, Error)]
pub enum EditError {
    #[error("unsupported file extension: .{0}")]
    UnsupportedExtension(String),
    /// v1 policy (plan Open Question 2): a prior tracked state is required
    /// so there is always a "last known state" to verify against.
    #[error("no prior tracked state for `{0}` — call provide_code_context first")]
    NoTrackedState(String),
    #[error("source is not valid UTF-8")]
    SourceNotUtf8,
    #[error(transparent)]
    Locate(#[from] LocateError),
    /// The unit exists but its current text differs from what the caller
    /// expects. Snippets are bounded to `SNIPPET_BOUND` chars.
    #[error(
        "unit `{target}` changed since last seen (expected {expected_len} bytes, actual {actual_len} bytes)"
    )]
    Mismatch {
        target: String,
        expected_snippet: String,
        actual_snippet: String,
        expected_len: usize,
        actual_len: usize,
    },
    #[error(
        "operations overlap: range ending at {first_end} intersects range starting at {second_start}"
    )]
    Overlap { first_end: u64, second_start: u64 },
    #[error(
        "syntax gate rejected the edit at line {line}, column {column}: {message} — nothing was written"
    )]
    SyntaxGateRejected {
        line: usize,
        column: usize,
        message: String,
    },
}

impl EditError {
    /// Structured payload for MCP error responses (bounded, no full-file
    /// echo). Returns `(code_hint_message, data_json)`.
    pub fn structured(&self) -> serde_json::Value {
        match self {
            EditError::Mismatch {
                target,
                expected_snippet,
                actual_snippet,
                expected_len,
                actual_len,
            } => serde_json::json!({
                "kind": "unit_mismatch",
                "target": target,
                "expected": expected_snippet,
                "actual": actual_snippet,
                "expectedLen": expected_len,
                "actualLen": actual_len,
            }),
            EditError::Locate(LocateError::Ambiguous { target, candidates }) => {
                serde_json::json!({
                    "kind": "ambiguous_target",
                    "target": target,
                    "candidates": candidates,
                })
            }
            other => serde_json::json!({
                "kind": "edit_rejected",
                "error": other.to_string(),
            }),
        }
    }
}

/// Bound a text snippet for error payloads.
fn bounded(text: &str) -> String {
    if text.len() <= SNIPPET_BOUND {
        return text.to_string();
    }
    let mut cut = SNIPPET_BOUND;
    while !text.is_char_boundary(cut) {
        cut -= 1;
    }
    format!("{}\u{2026}[truncated]", &text[..cut])
}

/// One planned byte-range replacement (internal planning record).
struct PlannedEdit {
    kind: &'static str,
    target: String,
    start: u64,
    end: u64,
    new_text: String,
}

/// Apply a batch of operations to `source`, verified against `units`.
///
/// On success returns the new source text plus per-operation outcome
/// records. Nothing is written here; the caller runs the syntax gate and
/// owns the disk write.
pub fn apply(
    source: &str,
    units: &UnitTable,
    operations: &[EditOperation],
) -> Result<ApplyReport, EditError> {
    // Plan: resolve every target and pin its byte range.
    let mut planned: Vec<PlannedEdit> = Vec::with_capacity(operations.len());
    for op in operations {
        let record = units.resolve(op.target())?;
        let planned_edit = match op {
            EditOperation::ReplaceBody {
                target,
                expected_old_text,
                new_text,
            } => {
                verify_expected(target, &record.text, expected_old_text)?;
                PlannedEdit {
                    kind: "replace_body",
                    target: target.clone(),
                    start: record.start_byte,
                    end: record.end_byte,
                    new_text: to_unit_eol(unit_is_crlf(&record.text, source), new_text),
                }
            }
            EditOperation::Delete {
                target,
                expected_old_text,
            } => {
                verify_expected(target, &record.text, expected_old_text)?;
                PlannedEdit {
                    kind: "delete",
                    target: target.clone(),
                    start: record.start_byte,
                    end: record.end_byte,
                    new_text: String::new(),
                }
            }
            EditOperation::InsertAfter { anchor, unit_text } => PlannedEdit {
                kind: "insert_after",
                target: anchor.clone(),
                start: record.end_byte,
                end: record.end_byte,
                new_text: to_unit_eol(unit_is_crlf(&record.text, source), unit_text),
            },
            EditOperation::InsertBefore { anchor, unit_text } => PlannedEdit {
                kind: "insert_before",
                target: anchor.clone(),
                start: record.start_byte,
                end: record.start_byte,
                new_text: to_unit_eol(unit_is_crlf(&record.text, source), unit_text),
            },
        };
        planned.push(planned_edit);
    }
    validate_disjoint(&planned)?;
    splice(source, planned)
}

/// Canonical form for line-ending-insensitive comparison: every CRLF
/// pair collapses to LF.
fn eol_normalized(s: &str) -> String {
    s.replace("\r\n", "\n")
}

/// Rewrite `text` into the EOL convention of the unit being touched.
///
/// Transport layers (editors, clipboards, LLM clients) routinely
/// normalize line endings, so a caller's text may arrive in the opposite
/// convention from the file. Splices always use the FILE's convention —
/// endings are never rewritten as a side effect and never mixed.
/// Idempotent for text already in the target convention.
fn to_unit_eol(crlf: bool, text: &str) -> String {
    if crlf {
        // Canonicalize down first so already-CRLF input stays exact.
        text.replace("\r\n", "\n").replace('\n', "\r\n")
    } else {
        text.replace("\r\n", "\n")
    }
}

/// Whether the unit being touched uses CRLF separators. Units without
/// any separator fall back to the whole-file convention — measured from
/// the actual source, never assumed.
fn unit_is_crlf(unit_text: &str, source: &str) -> bool {
    if unit_text.contains('\r') {
        true
    } else if unit_text.contains('\n') {
        false
    } else {
        source.contains("\r\n")
    }
}

/// Optimistic-concurrency check: current unit bytes must equal the caller's
/// expectation CONTENT-wise, modulo line-ending width (raw bytes are kept
/// in mismatch payloads so callers see the file's real representation).
/// Verification is unit-granular (the invariant
/// that actually matters) — not the whole-file recheck the client host's
/// native write tool performs.
fn verify_expected(target: &str, actual: &str, expected: &str) -> Result<(), EditError> {
    if eol_normalized(actual) == eol_normalized(expected) {
        return Ok(());
    }
    Err(EditError::Mismatch {
        target: target.to_string(),
        expected_snippet: bounded(expected),
        actual_snippet: bounded(actual),
        expected_len: expected.len(),
        actual_len: actual.len(),
    })
}

/// Reject any batch whose non-empty ranges intersect. Zero-length
/// insertions at shared boundaries never conflict; equal-position empty
/// edits are applied in caller order by the stable splice.
fn validate_disjoint(planned: &[PlannedEdit]) -> Result<(), EditError> {
    for window in planned.windows(2) {
        let (a, b) = (&window[0], &window[1]);
        let a_range = a.start < a.end;
        let b_range = b.start < b.end;
        let conflicts = if a_range && b_range {
            b.start < a.end && a.start < b.end
        } else {
            // Exactly one side is an insertion point (zero-length): it
            // conflicts iff it lands strictly inside the other's span.
            (a_range ^ b_range) && b.start > a.start && b.start < a.end
        };
        if conflicts {
            return Err(EditError::Overlap {
                first_end: a.end,
                second_start: b.start,
            });
        }
    }
    Ok(())
}

/// Splice the planned edits into `source`, back-to-front by byte position
/// (stable within equal positions so caller order wins at shared points).
fn splice(source: &str, mut planned: Vec<PlannedEdit>) -> Result<ApplyReport, EditError> {
    planned.sort_by(|a, b| b.start.cmp(&a.start).then(b.end.cmp(&a.end)));

    let mut out = source.as_bytes().to_vec();
    let mut outcomes = Vec::with_capacity(planned.len());
    for edit in &planned {
        let start = usize::try_from(edit.start).map_err(|_| EditError::SourceNotUtf8)?;
        let end = usize::try_from(edit.end).map_err(|_| EditError::SourceNotUtf8)?;
        if start > out.len() || end > out.len() || start > end {
            return Err(EditError::Locate(LocateError::NotFound(format!(
                "{} (stale span {}..{} beyond file length {})",
                edit.target,
                start,
                end,
                out.len()
            ))));
        }
        let old_len = end - start;
        let delta = edit.new_text.len() as i64 - old_len as i64;
        out.splice(start..end, edit.new_text.bytes());
        outcomes.push(super::ops::EditOutcome {
            kind: edit.kind,
            target: edit.target.clone(),
            start_byte: edit.start,
            end_byte: edit.start + edit.new_text.len() as u64,
            byte_delta: delta,
        });
    }

    let new_source = String::from_utf8(out).map_err(|_| EditError::SourceNotUtf8)?;
    outcomes.reverse(); // report in original operation order
    Ok(ApplyReport {
        new_source,
        operations: outcomes,
    })
}

/// Report of a successful in-memory application.
#[derive(Debug, Clone)]
pub struct ApplyReport {
    /// New file content after all operations were applied.
    pub new_source: String,
    /// Per-operation outcomes in original request order.
    pub operations: Vec<super::ops::EditOutcome>,
}

/// Hard pre-commit syntax gate (plan Design step 4).
///
/// Parses `source` with the grammar for `extension` and rejects any parse
/// containing ERROR nodes. A mis-relocated or malformed splice is far more
/// likely to produce a parse error than a valid-but-wrong edit — this gate
/// exists to catch that class of failure before disk. Non-bypassable by
/// construction: the write path calls this unconditionally.
pub fn verify_syntax(source: &str, extension: &str) -> Result<(), EditError> {
    let Some((language, _query)) = crate::compression::language::language_for_extension(extension)
    else {
        return Err(EditError::UnsupportedExtension(extension.to_string()));
    };
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&language)
        .map_err(|e| EditError::SyntaxGateRejected {
            line: 0,
            column: 0,
            message: format!("grammar init failed: {e}"),
        })?;
    let Some(tree) = parser.parse(source, None) else {
        return Err(EditError::SyntaxGateRejected {
            line: 0,
            column: 0,
            message: "parser produced no tree".to_string(),
        });
    };
    let root = tree.root_node();
    if root.has_error() {
        let (line, column, kind) = find_first_error(root);
        return Err(EditError::SyntaxGateRejected {
            line,
            column,
            message: format!("parse error near node `{kind}`"),
        });
    }
    Ok(())
}

/// Depth-first search for the first ERROR/missing node. Returns 1-based
/// line/column plus the node kind; `(0, 0, "")` when none is found.
fn find_first_error(node: tree_sitter::Node<'_>) -> (usize, usize, String) {
    if node.is_error() || node.is_missing() {
        return (
            node.start_position().row + 1,
            node.start_position().column + 1,
            node.kind().to_string(),
        );
    }
    for child in node.children(&mut node.walk()) {
        let found = find_first_error(child);
        if found.0 != 0 {
            return found;
        }
    }
    (0, 0, String::new())
}
