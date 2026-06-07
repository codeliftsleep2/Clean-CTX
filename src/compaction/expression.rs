// src/compaction/expression.rs
//
// Generic expression-level compaction and raw-text fallback.

use crate::compression::Fidelity;

/// Compact an arbitrary expression (used for captures that aren't class,
/// method, field, or import nodes — e.g. throw expressions, return values).
///
/// Low:    single identifier or very short form
/// Medium: trimmed single-line form, no body braces
/// High:   first meaningful line, trimmed
pub fn compact_expression(text: &str, fidelity: Fidelity) -> String {
    let first_line = text.lines().next().unwrap_or(text).trim();
    match fidelity {
        Fidelity::Low => {
            // Take the first identifier/word only
            first_line
                .split(|c: char| !c.is_alphanumeric() && c != '_' && c != '$')
                .find(|s| !s.is_empty())
                .unwrap_or(first_line)
                .to_string()
        }
        Fidelity::Medium => {
            // Strip body brace and anything after it
            first_line.split('{').next().unwrap_or(first_line).trim().to_string()
        }
        Fidelity::High => first_line.to_string(),
    }
}

/// Minimal compaction used as a raw fallback when no AST captures are found.
/// Just trims whitespace and collapses internal runs of spaces.
pub fn simple_compact(text: &str, _fidelity: Fidelity) -> String {
    // Collapse runs of whitespace to a single space
    let mut result = String::with_capacity(text.len());
    let mut last_was_space = false;
    for ch in text.trim().chars() {
        if ch.is_whitespace() {
            if !last_was_space {
                result.push(' ');
            }
            last_was_space = true;
        } else {
            result.push(ch);
            last_was_space = false;
        }
    }
    result
}
