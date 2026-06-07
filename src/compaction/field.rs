// src/compaction/field.rs
//
// Field/property compaction across fidelity levels.

use crate::compaction::modifiers::{strip_modifiers, MODIFIERS_FIELD};
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
        Fidelity::High => compact_field_high(text),
    }
}

/// Medium-fidelity field: "name:type"
fn compact_field_medium(text: &str) -> String {
    let line = text.lines().next().unwrap_or(text).trim();
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
    let line = text.lines().next().unwrap_or(text).trim();
    // Drop initialiser and trailing semicolon
    let s = line.split('=').next().unwrap_or(line).trim();
    s.trim_end_matches(';').trim().to_string()
}

#[cfg(test)]
#[path = "../tests/compaction/field.rs"]
mod tests;
