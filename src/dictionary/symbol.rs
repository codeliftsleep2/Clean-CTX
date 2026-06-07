// src/dictionary/symbol.rs
//
// Maps frequently repeated tokens to ultra-short opcodes (e.g., $1, $2).
// This is the "extended lookup dictionary graph" — a learnable symbol table
// that compresses repeated tokens into tiny references.

use std::collections::BTreeMap;

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
    forward: BTreeMap<String, String>,
    /// opcode -> token (e.g. "$1" -> "async")
    reverse: BTreeMap<String, String>,
    /// Running counter for new opcodes
    next_id: usize,
    /// Frequency tracking — tokens seen multiple times get opcodes
    frequency: BTreeMap<String, usize>,
}

impl SymbolDictionary {
    pub fn new() -> Self {
        // Pre-populate with very common token "primitives" that appear frequently
        let mut dict = Self {
            forward: BTreeMap::new(),
            reverse: BTreeMap::new(),
            next_id: 1,
            frequency: BTreeMap::new(),
        };

        // Built-in primitives: one-char opcodes for ultra-common tokens
        dict.insert_primitive("export", "$e");
        dict.insert_primitive("class", "$c");
        dict.insert_primitive("async", "$a");
        dict.insert_primitive("Promise", "$P");
        dict.insert_primitive("boolean", "$b");
        dict.insert_primitive("string", "$s");
        dict.insert_primitive("number", "$n");
        dict.insert_primitive("void", "$v");
        dict.insert_primitive("true", "$T");
        dict.insert_primitive("false", "$F");
        dict.insert_primitive("public", "$pu");
        dict.insert_primitive("private", "$pv");
        dict.insert_primitive("protected", "$pd");
        dict.insert_primitive("static", "$st");
        dict.insert_primitive("new", "$nw");
        dict.insert_primitive("return", "$r");
        dict.insert_primitive("throw", "$t");
        dict.insert_primitive("Error", "$E");
        dict.insert_primitive("const", "$k");
        dict.insert_primitive("let", "$l");
        dict.insert_primitive("if", "$i");
        dict.insert_primitive("for", "$fr");
        dict.insert_primitive("while", "$w");
        dict.insert_primitive("this", "$h");
        dict.insert_primitive("extends", "$x");
        dict.insert_primitive("implements", "$m");
        dict.insert_primitive("import", "$im");
        dict.insert_primitive("from", "$fm");
        dict.insert_primitive("interface", "$if");
        dict.insert_primitive("type", "$ty");
        dict.insert_primitive("function", "$fn");
        dict.insert_primitive("constructor", "$ctor");
        dict.insert_primitive("undefined", "$ud");
        dict.insert_primitive("null", "$nl");

        dict
    }

    fn insert_primitive(&mut self, token: &str, opcode: &str) {
        self.forward.insert(token.to_string(), opcode.to_string());
        self.reverse.insert(opcode.to_string(), token.to_string());
    }

    /// Load custom type aliases from config into the dictionary
    pub fn load_custom_aliases(&mut self, aliases: &std::collections::BTreeMap<String, String>) {
        for (opcode, token) in aliases {
            // Only insert if the token doesn't already have an opcode
            if !self.forward.contains_key(token) {
                self.forward.insert(token.clone(), opcode.clone());
                self.reverse.insert(opcode.clone(), token.clone());
            }
        }
    }

    /// Register a token — if seen >= 2 times, assign it an opcode
    pub fn register(&mut self, token: &str) {
        // Skip whitespace-only and empty tokens
        let token = token.trim();
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
            let custom: Vec<_> = self.reverse.iter()
                .filter(|(opcode, _)| {
                    // Filter out the built-in primitives by checking if opcode matches $[a-z]+ pattern
                    !Self::is_primitive_opcode(opcode)
                })
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

    fn is_primitive_opcode(opcode: &str) -> bool {
        // Primitives are $ followed by 1-3 lowercase letters, or $im, $fm, $if, $ty, $fn, $ctor
        let primitives = [
            "$e", "$c", "$a", "$P", "$b", "$s", "$n", "$v", "$T", "$F",
            "$pu", "$pv", "$pd", "$st", "$nw", "$r", "$t", "$E", "$k", "$l",
            "$i", "$fr", "$w", "$h", "$x", "$m", "$im", "$fm", "$if", "$ty",
            "$fn", "$ctor", "$ud", "$nl",
        ];
        primitives.contains(&opcode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_dictionary_basic() {
        let mut sd = SymbolDictionary::new();

        // Built-in primitives
        assert_eq!(sd.get_opcode("async"), Some("$a"));
        assert_eq!(sd.get_opcode("class"), Some("$c"));

        // Auto-register on second occurrence
        sd.register("CustomType");
        assert_eq!(sd.get_opcode("CustomType"), None);
        sd.register("CustomType");
        assert_eq!(sd.get_opcode("CustomType"), Some("$1"));
    }

    #[test]
    fn test_encode() {
        let mut sd = SymbolDictionary::new();
        sd.register("CustomType");
        sd.register("CustomType");

        // `async` and `function` are both built-in primitives; only
        // `CustomType` gets auto-registered on its second occurrence.
        let encoded = sd.encode("async function CustomType");
        assert_eq!(encoded, "$a $fn $1");
    }
}
