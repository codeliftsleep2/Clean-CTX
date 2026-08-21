// src/compression/markers.rs
//
// SHARED marker construction & expansion. The compression pipeline uses
// single-character ⊕-prefixed markers to indicate control-flow behavior:
//
//   ⊕guard      (if / while)        — no-op in expanded output
//   ⊕loop       (for / while)       — no-op in expanded output
//   ⊕⇒          (return statement)  — expands to "→ "
//   ⊕!          (throw statement)   — expands to "throws: "
//
// Before Phase 2 the construction logic was duplicated inline in
// `compressor.rs` and `diff/builder.rs`:
//
//     let marker = match cap.name.as_str() {
//         "throw.root" => format!("⊕!{}", cap.text),
//         "for.root"   => "⊕loop".to_string(),
//         "if.root"    => "⊕guard".to_string(),
//         "while.root" => "⊕loop".to_string(),
//         "return.root"=> format!("⊕⇒{}", cap.text),
//         _            => continue,
//     };
//
// Phase 2 funnels both call sites through `build_marker`. The
// corresponding expansion side (`expand_markers_in_line`) was already
// factored out in Phase 1; it now lives next to its constructor so the
// two are visibly paired.

/// Build a marker string from a tree-sitter capture name and the captured
/// text. Returns `None` for capture names that are not control-flow
/// markers (class/method/field/import/etc.) so the caller can skip them
/// in the default match arm.
///
/// Capture-name → marker mapping:
///
/// | capture_name  | marker construction        |
/// |---------------|----------------------------|
/// | `throw.root`  | `⊕!<text>`                 |
/// | `for.root`    | `⊕loop`                    |
/// | `if.root`     | `⊕guard`                   |
/// | `while.root`  | `⊕loop`                    |
/// | `return.root` | `⊕⇒<text>`                 |
///
/// Anything else returns `None` and the caller leaves the capture alone.
pub fn build_marker(capture_name: &str, text: &str) -> Option<String> {
    match capture_name {
        "throw.root" => Some(format!("⊕!{}", text)),
        "for.root" => Some("⊕loop".to_string()),
        "if.root" => Some("⊕guard".to_string()),
        "while.root" => Some("⊕loop".to_string()),
        "return.root" => Some(format!("⊕⇒{}", text)),
        _ => None,
    }
}

/// Expand a single marker token. Returns the expanded text, or `None` if the
/// input is not a recognised marker (caller should pass through unchanged).
///
/// Only used in test code; production callers use `expand_markers_in_line`.
#[allow(dead_code)]
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
    for (from, to) in &[
        ("⊕guard", ""),
        ("⊕loop", ""),
        ("⊕⇒", "→ "),
        ("⊕!", "throws: "),
    ] {
        s = s.replace(from, to);
    }
    s
}

#[cfg(test)]
#[path = "../tests/compression/markers.rs"]
mod tests;
