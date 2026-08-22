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
    /// Why a modified method changed. One of:
    ///   `"body"`    — signature + markers identical, body fingerprint changed
    ///   `"markers"` — signature identical, behavior markers changed
    ///   `"sig"`     — signature changed
    ///   `""`        — not a method modification.
    /// G2-5 audit: previously the formatter couldn't distinguish a
    /// body-only change from a markers-only change (both kept the same
    /// `detail`), so markers-only changes were mislabeled "(body changed)".
    pub reason_hint: String,
}

/// Defaults to `(Unchanged, Class)` so `DiffAction::default()` works.
/// G2-5: these are derived (clippy `derivable-impls`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DiffKind {
    Added,
    Removed,
    Modified,
    #[default]
    Unchanged,
}

/// Serialized as `"class", "method", "field", "import"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DiffTarget {
    #[default]
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
