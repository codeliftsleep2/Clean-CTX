// src/angular_meta/template_compress.rs
//
// Fidelity-gated Angular template compression entry point.
//
// This module provides the public `compress_template` function that
// compresses an Angular HTML template at the given fidelity level:
//   - Low    → single-line shape summary (current behavior)
//   - Medium → multi-line structural Angular semantics
//   - High   → near-full template with HTML scaffolding stripped
//
// It also provides PrimeNG component recognition (`Φp-<name>:` markers)
// for the Phase 4 deliverable.

use crate::angular_meta::template::{extract_template_shape, TemplateShape};
use crate::compression::Fidelity;

/// Compress an Angular template at the given fidelity level.
///
/// - `Fidelity::Low` → single-line shape summary (current behavior)
/// - `Fidelity::Medium` → multi-line structural Angular semantics
/// - `Fidelity::High` → near-full template with HTML scaffolding stripped
///
/// Returns the compressed marker lines. For Low fidelity this is a
/// single line; for Medium/High it is a multi-line block.
pub fn compress_template(html: &str, fidelity: Fidelity) -> Vec<String> {
    let shape = extract_template_shape(html);
    shape.to_marker_lines(fidelity)
}

/// Compress an Angular template and return the joined string form.
///
/// Convenience wrapper for callers that want a single `String` rather
/// than a `Vec<String>` of lines.
pub fn compress_template_to_string(html: &str, fidelity: Fidelity) -> String {
    compress_template(html, fidelity).join("\n")
}

/// Check if a tag is a PrimeNG component (starts with `p-`).
///
/// PrimeNG components follow the `p-<name>` convention (e.g. `p-table`,
/// `p-card`, `p-inputtext`, `p-button`). These are custom elements that
/// carry significant semantic weight for LLM consumption.
pub fn is_prime_ng_component(tag: &str) -> bool {
    tag.starts_with("p-")
}

/// Extract PrimeNG component markers from a template shape.
///
/// Returns a `Vec<String>` of `Φp-<name>:` markers for each unique
/// PrimeNG component found in the template. The markers are sorted
/// and deduplicated.
pub fn extract_prime_ng_markers(shape: &TemplateShape) -> Vec<String> {
    let mut markers: Vec<String> = shape
        .custom_elements
        .iter()
        .filter(|tag| is_prime_ng_component(tag))
        .map(|tag| format!("Φ{}:", tag))
        .collect();
    markers.sort();
    markers.dedup();
    markers
}

/// Compress an Angular template with PrimeNG markers appended.
///
/// This is the full Phase 4 entry point: it produces the fidelity-gated
/// template compression and appends any PrimeNG component markers as a
/// trailing line.
pub fn compress_template_with_prime_ng(html: &str, fidelity: Fidelity) -> Vec<String> {
    let shape = extract_template_shape(html);
    let mut lines = shape.to_marker_lines(fidelity);
    let prime_ng = extract_prime_ng_markers(&shape);
    if !prime_ng.is_empty() {
        lines.push(prime_ng.join(" "));
    }
    lines
}

#[cfg(all(test, feature = "angular"))]
#[path = "../tests/angular_meta/template_compress.rs"]
mod tests;