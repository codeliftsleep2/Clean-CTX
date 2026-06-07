// src/compaction/mod.rs
//
// Extraction and compaction helpers used by the compressor and diff modules.
// Each function strips source text down to a structural signature whose
// verbosity is governed by the `Fidelity` level:
//   Low    – maximum compression, keywords/types/modifiers removed
//   Medium – balanced; retains async, types, and error annotations
//   High   – minimal compression; preserves full semantic shape
//
// The module is split by concern:
//   - `class`     : class name extraction and class-entry formatting
//   - `method`    : method signature compaction (low/medium/high)
//   - `field`     : field/property compaction
//   - `import`    : import declaration compaction
//   - `expression`: generic expression fallback + simple raw-text compaction
//   - `modifiers` : shared modifier keyword lists (used by method & field)

pub(crate) mod class;
pub(crate) mod expression;
pub(crate) mod field;
pub(crate) mod import;
pub(crate) mod method;
pub(crate) mod modifiers;

pub use class::{extract_class_name, format_class_entry};
pub use expression::{compact_expression, simple_compact};
pub use field::extract_field;
pub use import::{compact_import, extract_import_names};
pub use method::extract_method_sig;
