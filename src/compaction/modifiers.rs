// src/compaction/modifiers.rs
//
// SHARED modifier keyword lists used by the method and field compaction
// passes. The lists are exposed as `pub(crate)` constants so that callers
// in this crate can reuse the same vocabulary without redeclaring it.
//
// Phase 2 (FAANG audit F-07, F-16): `strip_modifiers` is the single
// helper that the class, method, and field extraction routines all
// share. The old implementation was three slightly-different copies
// (one in `class.rs`, one in `method.rs`, one in `field.rs`); the
// class-extraction copy in particular only ran a single pass, so a
// signature like `"public static abstract class Foo"` would stop
// after stripping `public ` and leave the `static abstract class`
// prefix intact. The new helper loops until stable.

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

/// Modifiers stripped from class declarations before extracting the
/// class name. The order matters when two prefixes overlap
/// (`"export default "` must be tried before `"export "`); the
/// `strip_modifiers` helper below picks the longest-matching prefix
/// on each pass.
pub(crate) const MODIFIERS_CLASS: &[&str] = &[
    "export default ",
    "export ",
    "abstract ",
    "sealed ",
    "public ",
    "private ",
    "protected ",
    "static ",
    "final ",
];

/// Modifiers stripped from Rust struct/trait/enum declarations.
pub(crate) const MODIFIERS_STRUCT_RS: &[&str] = &[
    "pub ", "pub(crate) ", "pub(super) ",
];

/// Repeatedly strip any of `modifiers` from the start of `s`, trimming
/// whitespace between prefixes. Loops until a pass removes nothing,
/// so an input like `"public static abstract class Foo"` produces
/// `"class Foo"` (not `"static abstract class Foo"`).
///
/// F-07: the previous `class.rs` implementation only walked the
/// modifier list once and returned the first match, which produced
/// incorrect output for multi-modifier declarations. F-16: this is
/// now the single source of truth — `class.rs`, `method.rs`, and
/// `field.rs` all call this helper instead of maintaining their own
/// copies.
///
/// The function is allocation-free per iteration (it uses
/// `str::strip_prefix`, which is `O(len(m))`) and only allocates a
/// single `String` if the input needs to be trimmed mid-loop.
pub(crate) fn strip_modifiers(s: &str, modifiers: &[&str]) -> String {
    let mut current = s.trim().to_string();
    loop {
        let mut stripped = false;
        for m in modifiers {
            if let Some(rest) = current.strip_prefix(m) {
                let trimmed = rest.trim_start().to_string();
                // If we made progress, restart the loop with the new
                // string. We compare lengths instead of full strings
                // to avoid an extra allocation in the common case
                // where the trim is a no-op.
                if trimmed.len() != current.len() {
                    current = trimmed;
                    stripped = true;
                    break;
                }
            }
        }
        if !stripped {
            break;
        }
    }
    current
}

#[cfg(test)]
#[path = "../tests/compaction/modifiers.rs"]
mod tests;

#[cfg(test)]
#[path = "../tests/proptest/modifier_stripper.rs"]
mod proptest_tests;
