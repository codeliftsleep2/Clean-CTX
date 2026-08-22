// src/compaction/field.rs
//
// Field/property compaction across fidelity levels.

use crate::compaction::modifiers::{MODIFIERS_FIELD, strip_csharp_attributes, strip_modifiers};
use crate::compression::Fidelity;

/// Extract a compact field/property signature.
///
/// Input:  "private readonly userId: string = '';"
/// Low:    ""  (fields are suppressed at Low fidelity)
/// Medium: "userId:string"
/// High:   "private readonly userId: string"
pub fn extract_field(text: &str, fidelity: Fidelity) -> String {
    match fidelity {
        Fidelity::Low => String::new(),
        Fidelity::Medium => compact_field_medium(text),
        // C-12 (FAANG audit): At Edit/Verbatim the field text must be
        // byte-exact (including the initializer and trailing semicolon)
        // so `replace_in_file` SEARCH blocks match. High keeps the
        // stripped form.
        Fidelity::Edit | Fidelity::Verbatim => text.to_string(),
        Fidelity::High => compact_field_high(text),
    }
}

/// Strip a C# property accessor block (`{ get; set; }`, `{ get; }`,
/// `{ get; private set; }`) from a property declaration so the compacted
/// form is `Name:string` not `Name:string { get; set; }`.
///
/// The accessor block is the LAST balanced `{...}` group at depth 0.
/// Expression-bodied properties (`=> value`) and auto-property
/// initializers (`= new();`) are handled by the existing `=` split.
fn strip_property_accessors(text: &str) -> &str {
    let mut depth = 0i32;
    let mut last_open = None;
    let mut last_close = None;
    for (i, ch) in text.char_indices() {
        match ch {
            '{' => {
                if depth == 0 {
                    last_open = Some(i);
                }
                depth += 1;
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    last_close = Some(i);
                }
            }
            _ => {}
        }
    }
    match (last_open, last_close) {
        (Some(open), Some(close)) if open < close => &text[..open],
        _ => text,
    }
}

/// Medium-fidelity field: "name:type"
fn compact_field_medium(text: &str) -> String {
    // C# field captures may start with attribute lines (`[Key]`,
    // `[JsonPropertyName("id")]`); strip them so the declaration line
    // is the actual field, not the attribute.
    let stripped = strip_csharp_attributes(text);
    // C# property captures include the `{ get; set; }` accessor block.
    // Strip it so the compacted form is `Name:string` not
    // `Name:string { get; set; }`.
    let stripped = strip_property_accessors(stripped);
    let line = stripped.lines().next().unwrap_or(stripped).trim();
    // F-16: use the shared `strip_modifiers` helper.
    let s = strip_modifiers(line, MODIFIERS_FIELD);
    // Drop initializer='everything from `=` onwards' and trailing `;`
    let s = s.split('=').next().unwrap_or(&s).trim();
    let s = s.trim_end_matches(';').trim();
    // C# uses type-first syntax (`string Name`), TS uses name-first
    // (`Name: string`). Normalise C# to name-first (`Name:string`).
    // F-01 diff audit: property captures from C# need this so the
    // compacted form matches the TS convention.
    let s = normalize_csharp_type(s);
    // Collapse spaces around `:` and `?:`
    s.replace(" : ", ":").replace(": ", ":")
}

/// Normalise a C# type-first field declaration to name-first,
/// e.g. `string Name` → `Name:string`.
///
/// TS fields already use name-first syntax (`userId: string`), so a line
/// containing a `:` is left untouched. Only lines WITHOUT `:` (C#
/// type-first syntax `string Name`) are reordered to name-first.
fn normalize_csharp_type(line: &str) -> String {
    let line = line.trim();
    // TS/Java name-first — already has `:`, leave untouched.
    if line.contains(':') {
        return line.to_string();
    }
    // C# type-first: split on the last whitespace — the last token is
    // the field name, the rest is the type.
    let Some((type_part, name_part)) = line.rsplit_once(|c: char| c.is_whitespace()) else {
        return line.to_string();
    };
    let name = name_part.trim();
    let ty = type_part.trim();
    if name.is_empty() || ty.is_empty() {
        return line.to_string();
    }
    format!("{}:{}", name, ty)
}

/// High-fidelity field: preserve modifiers, strip only the initialiser.
fn compact_field_high(text: &str) -> String {
    // Strip leading C# attribute lines before taking the declaration line.
    let stripped = strip_csharp_attributes(text);
    // C# property captures include the `{ get; set; }` accessor block.
    // Strip it so the compacted form is `Name:string` not
    // `Name:string { get; set; }`.
    let stripped = strip_property_accessors(stripped);
    let line = stripped.lines().next().unwrap_or(stripped).trim();
    // Drop initialiser and trailing semicolon
    let s = line.split('=').next().unwrap_or(line).trim();
    s.trim_end_matches(';').trim().to_string()
}

#[cfg(test)]
#[path = "../tests/compaction/field.rs"]
mod tests;
