// src/decompression/markers.rs
//
// Marker expansion. The compression pipeline uses single-character ⊕-prefixed
// markers to indicate control-flow behavior:
//   ⊕guard      (if / while)        — no-op in expanded output
//   ⊕loop       (for / while)       — no-op in expanded output
//   ⊕⇒          (return statement)  — expands to "→ "
//   ⊕!          (throw statement)   — expands to "throws: "
//
// In Phase 2 this is consolidated with the corresponding marker-construction
// logic in `compressor.rs` and `diff/builder.rs` into a shared
// `crate::compression::markers` module.

/// Expand a single marker token. Returns the expanded text, or `None` if the
/// input is not a recognised marker (caller should pass through unchanged).
pub fn expand_marker(token: &str) -> Option<&'static str> {
    match token {
        "⊕guard" | "⊕loop" => Some(""),
        "⊕⇒" => Some("→ "),
        "⊕!" => Some("throws: "),
        _ => None,
    }
}

/// Expand every recognised marker in a line. Replaces non-overlapping
/// matches; unknown tokens are left untouched.
pub fn expand_markers_in_line(line: &str) -> String {
    // We only operate on a small fixed set, so a per-marker replace is fine.
    let mut s = line.to_string();
    for (from, to) in &[("⊕guard", ""), ("⊕loop", ""), ("⊕⇒", "→ "), ("⊕!", "throws: ")] {
        s = s.replace(from, to);
    }
    s
}
