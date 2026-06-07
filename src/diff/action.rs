// src/diff/action.rs
//
// DiffAction / DiffKind / DiffTarget — the value types emitted by the diff.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffAction {
    /// "+" added, "-" removed, "~" modified, "=" unchanged.
    pub kind: DiffKind,
    /// What kind of structural element this affects.
    pub target: DiffTarget,
    /// Human-readable label, e.g. "class FooService" or "method process".
    pub label: String,
    /// Compact representation of the new value (or baseline value for `-`).
    pub detail: String,
    /// For `~` actions, the prior compact representation. Empty otherwise.
    pub previous_detail: String,
}

/// Serialized as `"+", "-", "~", "="`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffKind {
    Added,
    Removed,
    Modified,
    Unchanged,
}

/// Serialized as `"class", "method", "field", "import"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffTarget {
    Class,
    Method,
    Field,
    Import,
}

impl DiffKind {
    pub fn symbol(self) -> &'static str {
        match self {
            DiffKind::Added => "+",
            DiffKind::Removed => "-",
            DiffKind::Modified => "~",
            DiffKind::Unchanged => "=",
        }
    }
}
