// src/decompression/decompressor.rs
//
// The `Decompressor` struct: parses footer blocks (PATHMAP, SYM) out of a
// compressed string and rewrites opcodes + markers back into human-readable
// tokens.

use std::collections::BTreeMap;

use super::markers::expand_markers_in_line;
use super::opcodes::builtin_opcode_map;
use super::walker::is_section_start;

pub struct Decompressor {
    custom_symbols: BTreeMap<String, String>,
    path_aliases: BTreeMap<String, String>,
    builtin_opcodes: BTreeMap<&'static str, &'static str>,
}

/// F-06 (FAANG audit): a word boundary for opcode expansion is "the
/// adjacent char is not a Unicode alphanumeric or `_`". The previous
/// implementation used `is_ascii_alphanumeric`, which silently
/// treated every non-ASCII character as a boundary — meaning a
/// token like `α1` (Greek alpha, ASCII '1') would expand correctly
/// but `αétat` (Greek alpha + Latin é + t + a + t) would not.
pub(crate) fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Replace all occurrences of `pattern` in `text` at word boundaries.
///
/// F-06: the previous byte-based ASCII check has been replaced with
/// a char-based Unicode check. The new algorithm is:
///   1. Find the next occurrence of `pattern` in the remainder of
///      `text`. Record its absolute byte offset.
///   2. If neither the preceding char nor the following char is a
///      "word char" (per `is_word_char` above), replace the match
///      with `replacement` and resume after it.
///   3. Otherwise advance by one char and re-scan.
///
/// Because `pattern` is expected to be short (an opcode like
/// `"$ctor"`) the inner `find` is cheap; the outer loop is
/// `O(len(text) + num_replacements)`.
pub(crate) fn word_boundary_replace(text: &str, pattern: &str, replacement: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut start = 0;

    while let Some(pos) = text[start..].find(pattern) {
        let abs = start + pos;
        // The char immediately before the match, if any.
        let before_ok = text[..abs]
            .chars()
            .next_back()
            .is_none_or(|c| !is_word_char(c));
        // The char immediately after the match, if any.
        let after_ok = text[abs + pattern.len()..]
            .chars()
            .next()
            .is_none_or(|c| !is_word_char(c));

        if before_ok && after_ok {
            result.push_str(&text[start..abs]);
            result.push_str(replacement);
            start = abs + pattern.len();
        } else {
            // Advance one Unicode char (not one byte) so we don't
            // infinite-loop on a multi-byte character.
            let next_char_boundary = text[start..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| start + i)
                .unwrap_or(text.len());
            result.push_str(&text[start..next_char_boundary]);
            start = next_char_boundary;
        }
    }
    result.push_str(&text[start..]);
    result
}

impl Default for Decompressor {
    fn default() -> Self {
        Self::new()
    }
}

impl Decompressor {
    pub fn new() -> Self {
        Self {
            custom_symbols: BTreeMap::new(),
            path_aliases: BTreeMap::new(),
            builtin_opcodes: builtin_opcode_map(),
        }
    }

    pub fn parse(&mut self, compressed: &str) {
        self.custom_symbols.clear();
        self.path_aliases.clear();
        for line in compressed.lines() {
            let trimmed = line.trim();
            if (trimmed.starts_with('α') || trimmed.starts_with('β') || trimmed.starts_with('γ'))
                && trimmed.contains(" = ")
                && let Some(eq_pos) = trimmed.find(" = ") {
                    let alias = trimmed[..eq_pos].trim().to_string();
                    let path = trimmed[eq_pos + 3..].trim().to_string();
                    self.path_aliases.insert(alias, path);
                }
            if trimmed.starts_with('$') && trimmed.contains(" = ")
                && let Some(eq_pos) = trimmed.find(" = ") {
                    let opcode = trimmed[..eq_pos].trim().to_string();
                    let token = trimmed[eq_pos + 3..].trim().to_string();
                    self.custom_symbols.insert(opcode, token);
                }
        }
    }

    pub fn decompress(&self, compressed: &str) -> String {
        let mut output = String::new();
        let mut skip_section = false;

        for line in compressed.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("// ---") || trimmed.starts_with("// Raw")
                || trimmed.starts_with("// Fidelity") || trimmed.starts_with("// [CACHE")
            {
                continue;
            }
            if is_section_start(trimmed) {
                skip_section = true;
                continue;
            }
            if skip_section {
                if trimmed.is_empty() { skip_section = false; }
                continue;
            }

            let mut expanded = line.to_string();

            // Expand path aliases
            for (alias, path) in &self.path_aliases {
                expanded = expanded.replace(alias.as_str(), path.as_str());
            }

            // Build sorted opcode list (longest first) and expand
            let mut all_opcodes: Vec<(&str, &str)> = Vec::new();
            for (opcode, token) in &self.custom_symbols {
                all_opcodes.push((opcode.as_str(), token.as_str()));
            }
            for (&opcode, &token) in &self.builtin_opcodes {
                all_opcodes.push((opcode, token));
            }
            all_opcodes.sort_by_key(|b| std::cmp::Reverse(b.0.len()));

            for (opcode, token) in &all_opcodes {
                expanded = word_boundary_replace(&expanded, opcode, token);
            }

            // Expand markers to human-readable comments
            expanded = expand_markers_in_line(&expanded);

            let cleaned = expanded.trim().to_string();
            if !cleaned.is_empty() {
                output.push_str(&cleaned);
                output.push('\n');
            }
        }

        if output.trim().is_empty() {
            self.strip_sections(compressed)
        } else {
            output
        }
    }

    fn strip_sections(&self, text: &str) -> String {
        let mut result = String::new();
        let mut skip = false;
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("// ---") || trimmed.starts_with("// Raw")
                || trimmed.starts_with("// Fidelity") || trimmed.starts_with("// [CACHE")
            {
                continue;
            }
            if trimmed.starts_with('§') {
                skip = !skip;
                continue;
            }
            if skip { continue; }
            result.push_str(line);
            result.push('\n');
        }
        result.trim().to_string()
    }

    pub fn quick_decompress(&mut self, compressed: &str) -> String {
        self.parse(compressed);
        self.decompress(compressed)
    }
}

#[cfg(test)]
#[path = "../tests/decompression/decompressor.rs"]
mod integration_tests;
