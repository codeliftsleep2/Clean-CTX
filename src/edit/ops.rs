// src/edit/ops.rs
//
// Operation model for the `apply_edit` write path.
//
// A small, closed set of structural operations — deliberately NOT an
// arbitrary line-based patch format (that would reimplement `git apply`,
// worse, and lose the structural verification that makes this safe).
// Signature changes, renames, and cross-file effects are out of scope
// for v1 (see plan: "What This Does Not Replace").

use serde::{Deserialize, Serialize};

/// One structural edit targeting a single named unit (method-level body,
/// or an insertion anchored to one).
///
/// Serialized form (MCP JSON): tagged objects, e.g.
/// `{"type": "replace_body", "target": "UserService.processOrder",
///   "expectedOldText": "{ ... }", "newText": "{ ... }"}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum EditOperation {
    /// Replace the body of the named unit. `expected_old_text` must match
    /// the unit's current bytes exactly (optimistic-concurrency check).
    ReplaceBody {
        /// Signature-qualified unit name ("Class.method(type,...)") or a
        /// legacy qualified/bare name that is unique in the current source.
        target: String,
        /// Byte-exact text the caller last saw for this unit's body.
        expected_old_text: String,
        /// Replacement body text (spliced verbatim, no re-indentation).
        new_text: String,
    },
    /// Insert `unit_text` immediately after the anchor unit's span.
    InsertAfter {
        /// Signature-qualified anchor, or a legacy name that is unique.
        anchor: String,
        /// New unit text, spliced verbatim (caller supplies leading
        /// whitespace/newline as desired).
        unit_text: String,
    },
    /// Insert `unit_text` immediately before the anchor's authoritative
    /// declaration start (including owned attributes/decorators), not its body.
    /// Reject when declaration placement or leading comment ownership is unknown.
    InsertBefore {
        /// Signature-qualified anchor, or a legacy name that is unique.
        anchor: String,
        /// New unit text, spliced verbatim.
        unit_text: String,
    },
    /// Remove the named declaration entirely. `expected_old_text` may match
    /// the full declaration; the historical body-only form remains accepted
    /// for compatibility when it matches the same tracked unit.
    Delete {
        /// Signature-qualified unit name, or a legacy name that is unique.
        target: String,
        /// Byte-exact text the caller expects to remove.
        expected_old_text: String,
    },
}

impl EditOperation {
    /// Human-readable operation tag used in reports/errors.
    pub fn kind(&self) -> &'static str {
        match self {
            EditOperation::ReplaceBody { .. } => "replace_body",
            EditOperation::InsertAfter { .. } => "insert_after",
            EditOperation::InsertBefore { .. } => "insert_before",
            EditOperation::Delete { .. } => "delete",
        }
    }

    /// The unit this operation targets (or anchors to).
    pub fn target(&self) -> &str {
        match self {
            EditOperation::ReplaceBody { target, .. } | EditOperation::Delete { target, .. } => {
                target
            }
            EditOperation::InsertAfter { anchor, .. }
            | EditOperation::InsertBefore { anchor, .. } => anchor,
        }
    }
}

/// Result record for one applied operation (report row).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EditOutcome {
    /// Operation tag (`replace_body`, `insert_after`, …).
    pub kind: &'static str,
    /// Target/anchor unit name as supplied by the caller.
    pub target: String,
    /// Absolute start byte affected in the NEW file (insertions: the
    /// insertion point; replacements/deletions: the spliced range start).
    pub start_byte: u64,
    /// Absolute end byte affected in the NEW file (insertions: same as
    /// start; replacements/deletions: start + replacement length).
    pub end_byte: u64,
    /// Signed size change this operation contributed
    /// (new_len - old_len; insertions are positive, deletions negative).
    pub byte_delta: i64,
    /// Exact bytes inserted by this operation after EOL/separator adaptation.
    /// Internal receipt authority; omitted from direct serialization.
    #[serde(skip)]
    pub applied_text: Option<String>,
}

/// Upper bound for any single operation batch — keeps rejection paths
/// cheap and prevents pathological request shapes.
pub const MAX_OPERATIONS_PER_CALL: usize = 64;
