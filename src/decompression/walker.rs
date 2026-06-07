// src/decompression/walker.rs
//
// Line-by-line section walker. Encapsulates the section/header detection
// state machine so the decompressor can stay focused on opcode expansion.
//
// Sections we recognise:
//   - Lines starting with "// ---"  (header separators)
//   - Lines starting with "// Raw" / "// Fidelity" / "// [CACHE"
//   - Lines starting with "§PATHMAP", "§SYM", or any other "§"-prefix
//   - Blank lines (close an open §-section)
//
// Each "header" prefix and each "§" line is fully skipped.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineKind {
    /// A blank line.
    Blank,
    /// A header / banner line that's filtered out (---, Raw, Fidelity, [CACHE).
    Header,
    /// The opening line of a §-prefixed section (footer block). The section
    /// continues until the next blank line.
    SectionStart,
    /// Any other line that should be passed through for expansion.
    Body,
    /// A line inside an open §-section (currently skipped).
    SectionBody,
}

/// Classify a single line of the compressed output.
pub fn classify(line: &str) -> LineKind {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return LineKind::Blank;
    }
    if trimmed.starts_with("// ---")
        || trimmed.starts_with("// Raw")
        || trimmed.starts_with("// Fidelity")
        || trimmed.starts_with("// [CACHE")
    {
        return LineKind::Header;
    }
    if trimmed.starts_with('§') {
        return LineKind::SectionStart;
    }
    LineKind::Body
}

/// Determine if a line is the start of a §-section block that should be
/// skipped (PATHMAP, SYM, or any other §-prefixed footer).
pub fn is_section_start(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with('§')
}
