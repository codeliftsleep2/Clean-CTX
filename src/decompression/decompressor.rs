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

/// Replace all occurrences of `pattern` in `text` at word boundaries.
/// Word boundary = adjacent char is not alphanumeric.
fn word_boundary_replace(text: &str, pattern: &str, replacement: &str) -> String {
    let mut result = String::new();
    let mut start = 0;
    while let Some(pos) = text[start..].find(pattern) {
        let abs_pos = start + pos;
        let before_ok = abs_pos == 0 || !text.as_bytes()[abs_pos - 1].is_ascii_alphanumeric();
        let after_pos = abs_pos + pattern.len();
        let after_ok = after_pos >= text.len() || !text.as_bytes()[after_pos].is_ascii_alphanumeric();

        if before_ok && after_ok {
            result.push_str(&text[start..abs_pos]);
            result.push_str(replacement);
            start = after_pos;
        } else {
            start = abs_pos + 1;
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
mod tests {
    use super::*;
    use super::super::walker::LineKind;

    #[test]
    fn test_decompress_low() {
        let input = "// --- Compacted Layout (Low Fidelity): α1 ---\n$c SampleService;$ctor();$b isInitialized\n\n§PATHMAP\n  α1 = C:\\project\\Service.ts";
        let mut d = Decompressor::new();
        let result = d.quick_decompress(input);
        assert!(result.contains("class SampleService"));
        assert!(result.contains("constructor()"));
        assert!(result.contains("boolean isInitialized"));
    }

    #[test]
    fn test_line_classification() {
        assert_eq!(classify_line_kind(""), LineKind::Blank);
        assert_eq!(classify_line_kind("   "), LineKind::Blank);
        assert_eq!(classify_line_kind("// --- header"), LineKind::Header);
        assert_eq!(classify_line_kind("§PATHMAP"), LineKind::SectionStart);
        assert_eq!(classify_line_kind("hello world"), LineKind::Body);
    }

    fn classify_line_kind(line: &str) -> LineKind {
        super::super::walker::classify(line)
    }
}
