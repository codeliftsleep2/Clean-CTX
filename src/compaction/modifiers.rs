// src/compaction/modifiers.rs
//
// SHARED modifier keyword lists used by the method and field compaction
// passes. The lists are exposed as `pub(crate)` constants so that callers
// in this crate can reuse the same vocabulary without redeclaring it.
//
// In Phase 2 these arrays are consumed by `method::compact_method_low`,
// `method::compact_method_medium`, and `field::compact_field_medium` —
// they are still duplicated inline in those files in Phase 1; the
// consolidation happens in the next phase.

/// Modifiers stripped at Low fidelity. Includes `async` and `readonly` —
/// everything not strictly required to identify the symbol.
pub(crate) const MODIFIERS_LOW: &[&str] = &[
    "public ", "private ", "protected ", "static ", "async ",
    "abstract ", "override ", "readonly ", "virtual ",
    "sealed ", "new ", "extern ",
];

/// Modifiers stripped at Medium fidelity. Keeps `async` and `readonly`
/// because they carry semantic information (concurrency model).
pub(crate) const MODIFIERS_MEDIUM: &[&str] = &[
    "public ", "private ", "protected ", "static ",
    "abstract ", "override ", "virtual ", "sealed ",
    "new ", "extern ",
];

/// Modifiers stripped from fields at Medium fidelity. Includes `readonly`
/// and `required` which are field-specific.
pub(crate) const MODIFIERS_FIELD: &[&str] = &[
    "public ", "private ", "protected ", "readonly ",
    "static ", "abstract ", "override ", "virtual ",
    "sealed ", "new ", "required ",
];
