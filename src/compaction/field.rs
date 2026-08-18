// src/compaction/field.rs
//
// Field/property compaction across fidelity levels.

use crate::compaction::modifiers::{strip_csharp_attributes, strip_modifiers, MODIFIERS_FIELD};
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

/// Medium-fidelity field: "name:type"
fn compact_field_medium(text: &str) -> String {
    // C# field captures may start with attribute lines (`[Key]`,
    // `[JsonPropertyName("id")]`); strip them so the declaration line
    // is the actual field, not the attribute.
    let stripped = strip_csharp_attributes(text);
    let line = stripped.lines().next().unwrap_or(stripped).trim();
    // F-16: use the shared `strip_modifiers` helper.
    let s = strip_modifiers(line, MODIFIERS_FIELD);
    // Drop initialiser (everything from `=` onwards) and trailing `;`
    let s = s.split('=').next().unwrap_or(&s).trim();
    let s = s.trim_end_matches(';').trim();
    // Collapse spaces around `:` and `?:`
    s.replace(" ?: ", "?:")
     .replace(" : ", ":")
     .replace(": ", ":")
}

/// High-fidelity field: preserve modifiers, strip only the initialiser.
fn compact_field_high(text: &str) -> String {
    // Strip leading C# attribute lines before taking the declaration line.
    let stripped = strip_csharp_attributes(text);
    let line = stripped.lines().next().unwrap_or(stripped).trim();
    // Drop initialiser and trailing semicolon
    let s = line.split('=').next().unwrap_or(line).trim();
    s.trim_end_matches(';').trim().to_string()
}

#[cfg(test)]
#[path = "../tests/compaction/field.rs"]
mod tests;
