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

/// Determine if a line is the start of a §-section block that should be
/// skipped (PATHMAP, SYM, or any other §-prefixed footer).
pub fn is_section_start(line: &str) -> bool {
    let trimmed = line.trim();
    // F-FULL-18: reject lines that are commented out (e.g. // §PATHMAP).
    if trimmed.starts_with("//") {
        return false;
    }
    trimmed.starts_with('§')
}
