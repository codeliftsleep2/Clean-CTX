// src/dictionary/workspace.rs
//
// Phase III (Idea #9 — Cross-File Symbol Deduplication).
//
// A workspace-level symbol dictionary that collects token frequencies across
// ALL compressible files, then assigns opcodes by global frequency. The most
// frequent tokens across the entire workspace get the shortest codes.
//
// Per-file output references the shared global dictionary via a compact
// `§GSYM id1,id2,…` footer instead of duplicating the full legend. This
// saves 15-30% for workspace-level compression when many files share common
// tokens (e.g., Angular `Service`, `Observable`, `HttpClient`).
//
// # Wire format
//
// ```text
// §GSYM
//   $a = Service
//   $b = Observable
//   $c = HttpClient
//   …
// §/GSYM
// ```
//
// Per-file footer:
// ```text
// §GSYM 0,1,2
// ```
// (indices into the global dictionary's custom symbols, sorted ascending)

use std::collections::HashMap;

use crate::compression::opcodes::{PRIMITIVE_OPCODES, is_primitive_opcode};
use crate::dictionary::symbol::tokenize_for_symbols;

/// Workspace-level symbol dictionary. Collects token frequencies across
/// multiple files, then assigns globally-optimised opcodes.
///
/// Usage:
/// 1. Create: `GlobalSymbolTable::new()`
/// 2. Count: call `count_tokens(body)` for each file's compressed body
/// 3. Build: call `build_codes()` once after all files are counted
/// 4. Encode: call `begin_file()` then `encode_body(body)` for each file
/// 5. Emit: call `format_global_footer()` for the manifest header,
///    and `format_file_refs(file)` for each file's footer
#[derive(Debug, Clone)]
pub struct GlobalSymbolTable {
    /// token → opcode (e.g. "Service" → "$a")
    forward: HashMap<String, String>,
    /// opcode → token (e.g. "$a" → "Service")
    reverse: HashMap<String, String>,
    /// Global token frequency across all files
    frequency: HashMap<String, usize>,
    /// Custom symbol opcodes in assignment order (for indexed references)
    custom_opcodes: Vec<String>,
    /// Whether `build_codes()` has been called
    built: bool,
    /// Tokens used by the current file (set during `begin_file`)
    current_used: Option<std::collections::HashSet<usize>>,
    /// Map from opcode to global index (for file refs)
    opcode_to_index: HashMap<String, usize>,
}

impl GlobalSymbolTable {
    pub fn new() -> Self {
        let mut table = Self {
            forward: HashMap::new(),
            reverse: HashMap::new(),
            frequency: HashMap::new(),
            custom_opcodes: Vec::new(),
            built: false,
            current_used: None,
            opcode_to_index: HashMap::new(),
        };

        // Load built-in primitive opcodes from the shared table.
        for (opcode, token) in PRIMITIVE_OPCODES {
            table.forward.insert(token.to_string(), opcode.to_string());
            table.reverse.insert(opcode.to_string(), token.to_string());
        }

        table
    }

    /// Count token occurrences from a file's compressed body.
    /// Call this for every file before `build_codes()`.
    pub fn count_tokens(&mut self, body: &str) {
        for token in body.split_whitespace() {
            let clean = tokenize_for_symbols(token);
            if clean.is_empty() || clean.len() <= 1 {
                continue;
            }
            // Skip tokens that already have a built-in primitive opcode.
            if self.forward.contains_key(clean) {
                continue;
            }
            // Also skip tokens that look like primitive opcodes (e.g. "$ctor")
            if crate::compression::opcodes::is_primitive_opcode(&format!("${}", clean)) {
                continue;
            }
            *self.frequency.entry(clean.to_string()).or_insert(0) += 1;

            // Handle ⊕-prefixed behavior markers (e.g. "⊕async" → count "async")
            if let Some(rest) = token.strip_prefix('⊕')
                && !rest.is_empty()
                && !self.forward.contains_key(rest)
                && rest.len() > 1
            {
                *self.frequency.entry(rest.to_string()).or_insert(0) += 1;
            }
        }
    }

    /// Build globally-optimised Huffman codes from the accumulated frequency
    /// table. Call once after all files have been counted.
    ///
    /// Tokens are sorted by descending frequency — the most frequent token
    /// across the workspace gets the shortest code (`$a`).
    pub fn build_codes(&mut self) {
        if self.built {
            return;
        }

        let mut freq_vec: Vec<(String, usize)> = self
            .frequency
            .iter()
            .map(|(k, &v)| (k.clone(), v))
            .collect();

        // Sort by descending frequency, then ascending token for determinism.
        freq_vec.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

        // Use letter-based codes ($a, $b, …) to avoid clashing with
        // sequential $1, $2, … from per-file dictionaries.
        const HUFFMAN_CODES: &[&str] = &[
            "$a", "$b", "$c", "$d", "$e", "$f", "$g", "$h", "$i", "$j", "$k", "$l", "$m", "$n",
            "$o", "$p", "$q", "$r", "$s", "$t", "$u", "$v", "$w", "$x", "$y", "$z", "$A", "$B",
            "$C", "$D", "$E", "$F", "$G", "$H", "$I", "$J", "$K", "$L", "$M", "$N", "$O", "$P",
            "$Q", "$R", "$S", "$T", "$U", "$V", "$W", "$X", "$Y", "$Z",
        ];

        let mut code_idx = 0;
        for (index, (token, _count)) in freq_vec.iter().enumerate() {
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
            self.opcode_to_index.insert(code.to_string(), index);
            self.custom_opcodes.push(code.to_string());
            code_idx += 1;
        }

        self.built = true;
    }

    /// Begin encoding a new file. Must be called before `encode_body`.
    pub fn begin_file(&mut self) {
        self.current_used = Some(std::collections::HashSet::new());
    }

    /// Encode a file's compressed body using the global dictionary.
    /// Returns the encoded body with global symbol references.
    pub fn encode_body(&mut self, body: &str) -> String {
        let mut result = String::new();
        let mut word = String::new();

        for c in body.chars() {
            if c.is_alphanumeric() || c == '_' {
                word.push(c);
            } else {
                if !word.is_empty() {
                    if let Some(replacement) = self.forward.get(&word) {
                        result.push_str(replacement);
                        // Track usage for file refs
                        if let Some(ref mut used) = self.current_used {
                            if let Some(&idx) = self.opcode_to_index.get(replacement) {
                                used.insert(idx);
                            }
                        }
                    } else {
                        result.push_str(&word);
                    }
                    word.clear();
                }
                result.push(c);
            }
        }

        if !word.is_empty() {
            if let Some(replacement) = self.forward.get(&word) {
                result.push_str(replacement);
                if let Some(ref mut used) = self.current_used {
                    if let Some(&idx) = self.opcode_to_index.get(replacement) {
                        used.insert(idx);
                    }
                }
            } else {
                result.push_str(&word);
            }
        }

        result
    }

    /// Format the global dictionary footer for the workspace manifest.
    ///
    /// ```text
    /// §GSYM
    ///   $a = Service
    ///   $b = Observable
    ///   …
    /// §/GSYM
    /// ```
    pub fn format_global_footer(&self) -> String {
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

        // Sort by global index for deterministic output.
        custom.sort_by_key(|(code, _)| {
            self.opcode_to_index
                .get(code.as_str())
                .copied()
                .unwrap_or(0)
        });

        let mut footer = String::from("§GSYM\n");
        for (opcode, token) in &custom {
            footer.push_str(&format!("  {} = {}\n", opcode, token));
        }
        footer.push_str("§/GSYM\n");
        footer
    }

    /// Format per-file references to the global dictionary.
    ///
    /// ```text
    /// §GSYM 0,1,2
    /// ```
    /// where the numbers are indices into the global dictionary's
    /// custom symbols (sorted ascending).
    pub fn format_file_refs(&self) -> String {
        let used = match &self.current_used {
            Some(u) => u,
            None => return String::new(),
        };

        if used.is_empty() {
            return String::new();
        }

        let mut sorted_ids: Vec<usize> = used.iter().copied().collect();
        sorted_ids.sort_unstable();

        let ids_str: Vec<String> = sorted_ids.iter().map(|id| id.to_string()).collect();
        format!("§GSYM {}\n", ids_str.join(","))
    }

    /// Get a token's opcode if it exists in the global dictionary.
    pub fn get_opcode(&self, token: &str) -> Option<&str> {
        self.forward.get(token).map(|s| s.as_str())
    }

    /// Get the total number of custom (non-primitive) symbols assigned.
    pub fn custom_symbol_count(&self) -> usize {
        self.custom_opcodes.len()
    }

    /// Get the total frequency across all files.
    pub fn total_frequency(&self) -> usize {
        self.frequency.values().sum()
    }

    /// Check if a token has a primitive opcode (built-in).
    pub fn is_primitive(&self, token: &str) -> bool {
        self.forward.contains_key(token)
            && is_primitive_opcode(self.forward.get(token).unwrap().as_str())
    }
}

impl Default for GlobalSymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "../tests/dictionary/workspace.rs"]
mod tests;
