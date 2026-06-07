// src/helpers.rs
//
// Re-export shim — preserves the old `crate::helpers` import path so that
// downstream consumers (and the old `compressor.rs` and `diff.rs` call
// sites) continue to compile unchanged.
//
// All actual logic now lives in `crate::compaction::*` (split by concern).
// The shim will be deleted in a later phase once all internal callers have
// been migrated to the new module path.

pub use crate::compaction::compact_expression;
pub use crate::compaction::compact_import;
pub use crate::compaction::extract_class_name;
pub use crate::compaction::extract_field;
pub use crate::compaction::extract_method_sig;
pub use crate::compaction::extract_import_names;
pub use crate::compaction::format_class_entry;
pub use crate::compaction::simple_compact;
