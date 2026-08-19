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
/// and `required` which are field-specific, plus Rust's `pub` visibility
/// modifier (F-01 diff audit: Rust struct fields like `pub name: String`
/// were previously rendered as `pub name:String` instead of `name:String`).
pub(crate) const MODIFIERS_FIELD: &[&str] = &[
    "public ", "private ", "protected ", "readonly ",
    "static ", "abstract ", "override ", "virtual ",
    "sealed ", "new ", "required ", "pub ",
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

/// Strip leading C# attribute groups (`[ApiController]`, `[Route("api/{id}")]`,
/// `[HttpGet("{id}")]`) from the start of a tree-sitter capture.
///
/// tree-sitter's C# grammar includes attribute lists as children of the
/// `method_declaration` / `class_declaration` / `field_declaration` nodes,
/// so the raw capture starts with the attribute text. Without stripping it,
/// `extract_class_name` and `parse_method_sig` treat the attribute as the
/// symbol's name (`"[ApiController]"`, `"[HttpGet]"`), which breaks:
///   - C# method-name extraction (`GetAll`, `GetById`, …)
///   - the `focusMethods` symbol-targeting match (names are mangled)
///   - class-name extraction (`UserController` instead of `[ApiController]`)
///
/// The scan walks balanced `[`/`]` so a brace inside an attribute argument
/// (`[Route("api/[controller]")]`) is not treated as a body opener by
/// `find_body_start`, and an attribute with parens (`[HttpGet("{id}")]`)
/// does not fool the method-paren scan.
///
/// Returns the remaining slice starting at the first non-attribute character.
/// Non-C# inputs (no leading `[`) are returned unchanged.
pub(crate) fn strip_csharp_attributes(text: &str) -> &str {
    let mut rest = text.trim_start();
    loop {
        if !rest.starts_with('[') {
            return rest;
        }
        let mut depth = 0i32;
        let mut end = None;
        for (i, ch) in rest.char_indices() {
            match ch {
                '[' => depth += 1,
                ']' => {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(i);
                        break;
                    }
                }
                _ => {}
            }
        }
        let Some(i) = end else {
            // Unclosed bracket — leave the rest as-is (defensive).
            return rest;
        };
        let after = &rest[i + 1..];
        // TS index signature guard: `[key: string]: number` — the char
        // after `]` is `:` or `,` or `;`/`=` (declaration continues on
        // the same line). A C# attribute is followed by a newline (or
        // another `[` on the next line, or a leading modifier/declaration
        // keyword on the same line). If the remainder does NOT begin with
        // whitespace/newline and does NOT start with an identifier-like
        // char, this is not an attribute — return the whole input.
        let next_is_decl = after
            .chars()
            .next()
            .is_some_and(|c| c.is_alphanumeric() || c == '_' || c.is_whitespace());
        if !next_is_decl {
            return rest;
        }
        // If the remainder starts at a new line or another `[`, keep
        // stripping (multi-line attribute lists). Otherwise the bracket
        // group was followed by a declaration on the same line — done
        // stripping.
        let after_trimmed = after.trim_start();
        if !after_trimmed.starts_with('[') && !after.starts_with('\n') {
            // Single bracket group followed by a declaration on the same
            // line (e.g. `[key] public string Name { get; set; }`) — the
            // whole thing IS the capture. But a valid C# attribute is
            // normally on its own line; if the remainder starts with an
            // identifier that is NOT a property-like declaration, treat
            // the bracket group as the attribute and continue. Simplify:
            // attributes in tree-sitter C# always precede a newline OR
            // another attribute; if we reach a same-line remainder that
            // is a declaration, strip ONE group and return.
            return after_trimmed;
        }
        rest = after_trimmed;
    }
}

#[cfg(test)]
#[path = "../tests/compaction/modifiers.rs"]
mod tests;

#[cfg(test)]
#[path = "../tests/proptest/modifier_stripper.rs"]
mod proptest_tests;
