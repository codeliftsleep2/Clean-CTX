// src/dictionary/symbol.rs
//
// Maps frequently repeated tokens to ultra-short opcodes (e.g., $1, $2).
// This is the "extended lookup dictionary graph" — a learnable symbol table
// that compresses repeated tokens into tiny references.
//
// Phase 2: the built-in primitive opcodes are loaded from the shared
// `crate::compression::opcodes::PRIMITIVE_OPCODES` table so the
// dictionary and the decompressor both read from the same single source
// of truth (previously the 32 entries were duplicated inline here and
// in `decompression/opcodes.rs`).

use std::collections::HashMap;

use crate::compression::opcodes::{PRIMITIVE_OPCODES, is_primitive_opcode};

/// Trim a token string to the canonical form used for symbol registration
/// and lookup. Strips surrounding punctuation that is part of the syntax
/// rather than the token itself.
///
/// F-37: centralises the trim logic that was previously duplicated in
/// `SymbolDictionary::register` and `apply_symbol_compression`.
pub fn tokenize_for_symbols(s: &str) -> &str {
    s.trim_matches(|c: char| {
        matches!(
            c,
            '(' | ')' | '[' | ']' | '{' | '}' | '<' | '>' | ':' | ';' | ',' | '.'
        )
    })
}

/// Opcode symbol type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    Keyword,
    Type,
    Method,
    Field,
    Marker,
    Literal,
}

pub struct SymbolDictionary {
    /// token -> opcode (e.g. "async" -> "$1")
    forward: HashMap<String, String>,
    /// opcode -> token (e.g. "$1" -> "async")
    reverse: HashMap<String, String>,
    /// Running counter for new opcodes
    next_id: usize,
    /// Frequency tracking — tokens seen multiple times get opcodes
    frequency: HashMap<String, usize>,
}

impl SymbolDictionary {
    pub fn new() -> Self {
        let mut dict = Self {
            forward: HashMap::new(),
            reverse: HashMap::new(),
            next_id: 1,
            frequency: HashMap::new(),
        };

        // Built-in primitives: load from the SHARED opcode table so the
        // dictionary and the decompressor can never drift apart.
        for (opcode, token) in PRIMITIVE_OPCODES {
            dict.insert_primitive(token, opcode);
        }

        dict
    }

    fn insert_primitive(&mut self, token: &str, opcode: &str) {
        self.forward.insert(token.to_string(), opcode.to_string());
        self.reverse.insert(opcode.to_string(), token.to_string());
    }

    /// Load custom type aliases from config into the dictionary
    pub fn load_custom_aliases(&mut self, aliases: &std::collections::HashMap<String, String>) {
        for (opcode, token) in aliases {
            // Only insert if the token doesn't already have an opcode
            if !self.forward.contains_key(token) {
                self.forward.insert(token.clone(), opcode.clone());
                self.reverse.insert(opcode.clone(), token.clone());
            }
        }
    }

    /// Register a token — if seen >= 2 times, assign it an opcode.
    /// F-37: uses `tokenize_for_symbols` for consistent trimming.
    pub fn register(&mut self, token: &str) {
        let token = tokenize_for_symbols(token);
        if token.is_empty() || token.len() <= 1 {
            return;
        }

        // Skip if already in the dictionary
        if self.forward.contains_key(token) {
            return;
        }

        let count = self.frequency.entry(token.to_string()).or_insert(0);
        *count += 1;

        // Assign an opcode on the 2nd occurrence
        if *count >= 2 {
            let opcode = format!("${}", self.next_id);
            self.next_id += 1;
            let opcode_str = opcode.clone();
            self.forward.insert(token.to_string(), opcode);
            self.reverse.insert(opcode_str, token.to_string());
        }
    }

    /// Encode a string using the dictionary — replaces known tokens with opcodes.
    /// Splits on whitespace, replaces whole alphanumeric words that match the dictionary,
    /// then reassembles. This is simple, safe, and handles Unicode correctly.
    pub fn encode(&self, text: &str) -> String {
        let mut result = String::new();
        let mut word = String::new();

        for c in text.chars() {
            if c.is_alphanumeric() || c == '_' {
                word.push(c);
            } else {
                // Process the accumulated word
                if !word.is_empty() {
                    let replacement = self.forward.get(&word).cloned().unwrap_or(word.clone());
                    result.push_str(&replacement);
                    word.clear();
                }
                result.push(c);
            }
        }

        // Process the last word if any
        if !word.is_empty() {
            let replacement = self.forward.get(&word).cloned().unwrap_or(word.clone());
            result.push_str(&replacement);
        }

        result
    }

    /// Get a token's opcode if it exists
    pub fn get_opcode(&self, token: &str) -> Option<&str> {
        self.forward.get(token).map(|s| s.as_str())
    }

    /// Format the symbol map footer for the output
    pub fn format_footer(&self) -> String {
        if self.reverse.len() <= 32 {
            // Only show opcodes that aren't built-in primitives (IDs >= 1 after primitives)
            let custom: Vec<_> = self
                .reverse
                .iter()
                .filter(|(opcode, _)| !is_primitive_opcode(opcode))
                .collect();

            if custom.is_empty() {
                return String::new();
            }

            let mut footer = String::from("§SYM\n");
            for (opcode, token) in &custom {
                footer.push_str(&format!("  {} = {}\n", opcode, token));
            }
            footer
        } else {
            String::new()
        }
    }
}

impl Default for SymbolDictionary {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "../tests/dictionary/symbol.rs"]
mod tests;
