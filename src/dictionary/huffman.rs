// src/dictionary/huffman.rs
//
// Phase III (Idea #10 — Huffman-Coded Symbol Dictionary).
//
// Frequency-weighted symbol assignment: the most frequently used tokens get
// the shortest opcodes ($a, $b, ...) instead of sequential $1, $2, ...
//
// The Huffman tree structure is encoded in the footer so decompressors can
// reverse the mapping. The codes are prefix-free by construction (each
// code is a single character after the `$` prefix), so no tree walk is
// needed — just a flat lookup table sorted by frequency.

use std::collections::HashMap;

use crate::compression::opcodes::{PRIMITIVE_OPCODES, is_primitive_opcode};
use crate::dictionary::symbol::tokenize_for_symbols;

/// Short opcodes ordered by frequency weight: `$a` is the most frequent,
/// `$b` the second-most, etc. The `$` prefix keeps them syntactically
/// distinct from raw tokens. Using letters instead of digits avoids
/// clashing with the existing `$1`, `$2`, … sequential scheme.
const HUFFMAN_CODES: &[&str] = &[
    "$a", "$b", "$c", "$d", "$e", "$f", "$g", "$h", "$i", "$j", "$k", "$l", "$m", "$n", "$o", "$p",
    "$q", "$r", "$s", "$t", "$u", "$v", "$w", "$x", "$y", "$z", "$A", "$B", "$C", "$D", "$E", "$F",
    "$G", "$H", "$I", "$J", "$K", "$L", "$M", "$N", "$O", "$P", "$Q", "$R", "$S", "$T", "$U", "$V",
    "$W", "$X", "$Y", "$Z",
];

/// A Huffman-coded symbol dictionary that assigns the shortest codes to the
/// most frequently used tokens.
///
/// Unlike the sequential `SymbolDictionary` (which assigns `$1`, `$2`, … in
/// order of first occurrence), this dictionary counts all token frequencies
/// first, then assigns codes sorted by frequency. The result is that the
/// most common tokens get the shortest codes, maximising savings.
///
/// The footer encodes the mapping so decompressors can reverse it:
/// ```text
/// §HUF
///   $a = Service (42)
///   $b = Observable (38)
///   $c = HttpClient (15)
/// ```
///
/// The count in parentheses lets the decompressor know when the mapping
/// is complete (optional; the footer ends at the next `§` or EOF).
#[derive(Debug, Clone)]
pub struct HuffmanSymbolDictionary {
    /// token → opcode (e.g. "Service" → "$a")
    forward: HashMap<String, String>,
    /// opcode → token (e.g. "$a" → "Service")
    reverse: HashMap<String, String>,
    /// token → frequency count
    frequency: HashMap<String, usize>,
}

impl HuffmanSymbolDictionary {
    pub fn new() -> Self {
        let mut dict = Self {
            forward: HashMap::new(),
            reverse: HashMap::new(),
            frequency: HashMap::new(),
        };

        // Load built-in primitive opcodes so they're always available.
        for (opcode, token) in PRIMITIVE_OPCODES {
            dict.forward.insert(token.to_string(), opcode.to_string());
            dict.reverse.insert(opcode.to_string(), token.to_string());
        }

        dict
    }

    /// Count a token occurrence. Does not assign an opcode yet — call
    /// [`build_codes`] after all tokens are counted.
    pub fn count(&mut self, token: &str) {
        let token = tokenize_for_symbols(token);
        if token.is_empty() || token.len() <= 1 {
            return;
        }
        // Skip tokens that already have a built-in opcode.
        if self.forward.contains_key(token) {
            return;
        }
        *self.frequency.entry(token.to_string()).or_insert(0) += 1;
    }

    /// Build Huffman codes from the accumulated frequency table.
    /// Tokens are sorted by descending frequency; the most frequent get the
    /// shortest codes. Tokens with the same frequency are sorted
    /// lexicographically for determinism.
    ///
    /// Codes that are already used by primitive opcodes are skipped.
    pub fn build_codes(&mut self) {
        let mut freq_vec: Vec<(String, usize)> = self
            .frequency
            .iter()
            .map(|(k, &v)| (k.clone(), v))
            .collect();

        // Sort by descending frequency, then ascending token name for
        // deterministic output.
        freq_vec.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

        let mut code_idx = 0;
        for (token, _count) in &freq_vec {
            // Skip codes already used by primitive opcodes.
            while code_idx < HUFFMAN_CODES.len()
                && self.reverse.contains_key(HUFFMAN_CODES[code_idx])
            {
                code_idx += 1;
            }
            if code_idx >= HUFFMAN_CODES.len() {
                break;
            }
            let code = HUFFMAN_CODES[code_idx];
            self.forward.insert(token.clone(), code.to_string());
            self.reverse.insert(code.to_string(), token.clone());
            code_idx += 1;
        }
    }

    /// Encode text by replacing known tokens with their Huffman codes.
    /// Operates identically to `SymbolDictionary::encode`.
    pub fn encode(&self, text: &str) -> String {
        let mut result = String::new();
        let mut word = String::new();

        for c in text.chars() {
            if c.is_alphanumeric() || c == '_' {
                word.push(c);
            } else {
                if !word.is_empty() {
                    let replacement = self.forward.get(&word).cloned().unwrap_or(word.clone());
                    result.push_str(&replacement);
                    word.clear();
                }
                result.push(c);
            }
        }

        if !word.is_empty() {
            let replacement = self.forward.get(&word).cloned().unwrap_or(word.clone());
            result.push_str(&replacement);
        }

        result
    }

    /// Format the Huffman symbol map footer.
    ///
    /// ```text
    /// §HUF
    ///   $a = Service (42)
    ///   $b = Observable (38)
    /// ```
    pub fn format_footer(&self) -> String {
        let mut custom: Vec<_> = self
            .reverse
            .iter()
            .filter(|(opcode, token)| {
                !is_primitive_opcode(opcode) && self.frequency.contains_key(token.as_str())
            })
            .collect();

        if custom.is_empty() {
            return String::new();
        }

        // Sort by code for deterministic output.
        custom.sort_by_key(|(code, _)| code.to_string());

        let mut footer = String::from("§HUF\n");
        for (opcode, token) in &custom {
            let count = self.frequency.get(token.as_str()).unwrap_or(&0);
            footer.push_str(&format!("  {} = {} ({})\n", opcode, token, count));
        }
        footer
    }

    /// Get a token's opcode if it exists.
    pub fn get_opcode(&self, token: &str) -> Option<&str> {
        self.forward.get(token).map(|s| s.as_str())
    }

    /// Get the total number of encoded tokens (sum of frequencies).
    pub fn total_frequency(&self) -> usize {
        self.frequency.values().sum()
    }

    /// Get the number of custom (non-primitive) symbols assigned.
    pub fn custom_symbol_count(&self) -> usize {
        self.frequency.len()
    }

    /// Decode a footer back into a token→opcode mapping.
    /// Returns a HashMap suitable for loading into a `SymbolDictionary`.
    pub fn parse_footer(footer: &str) -> Option<HashMap<String, String>> {
        let mut map = HashMap::new();
        let mut in_huf = false;

        for line in footer.lines() {
            let line = line.trim();
            if line == "§HUF" {
                in_huf = true;
                continue;
            }
            if !in_huf {
                continue;
            }
            if line.is_empty() || line.starts_with('§') {
                break;
            }
            // Parse "$a = Service (42)"
            let parts: Vec<&str> = line.splitn(2, " = ").collect();
            if parts.len() != 2 {
                continue;
            }
            let code = parts[0].trim();
            let rest = parts[1].trim();
            // Strip the count in parentheses if present
            let token = if let Some(paren_pos) = rest.rfind('(') {
                rest[..paren_pos].trim()
            } else {
                rest
            };
            map.insert(token.to_string(), code.to_string());
        }

        if map.is_empty() { None } else { Some(map) }
    }
}

impl Default for HuffmanSymbolDictionary {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "../tests/dictionary/huffman.rs"]
mod tests;
