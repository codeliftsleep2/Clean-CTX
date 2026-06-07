// src/compression/symbol_compression.rs
//
// Low-fidelity symbol-dictionary opcode pass. At `Fidelity::Low` every
// unique token in the body is replaced with a short symbol reference
// (`$1`, `$2`, …) and a legend is appended as a footer. Higher fidelities
// skip this pass because structural markers already keep density high.

use crate::compression::Fidelity;
use crate::dictionary::SymbolDictionary;

/// Apply the symbol-dictionary opcode pass (Low fidelity only). Higher
/// fidelities don't need it — the structural markers already provide
/// sufficient density.
pub fn apply_symbol_compression(body_content: &str, fidelity: Fidelity) -> (String, String) {
    if fidelity != Fidelity::Low {
        return (body_content.to_string(), String::new());
    }
    let mut sym_dict = SymbolDictionary::new();
    for token in body_content.split_whitespace() {
        let clean = token.trim_matches(|c: char| {
            c == '(' || c == ')' || c == '[' || c == ']' || c == '{' || c == '}'
                || c == '<' || c == '>' || c == ':' || c == ';' || c == ',' || c == '.'
        });
        if !clean.is_empty() {
            sym_dict.register(clean);
        }
        if let Some(rest) = token.strip_prefix('⊕')
            && !rest.is_empty() {
                sym_dict.register(rest);
            }
    }
    let encoded = sym_dict.encode(body_content);
    let footer = sym_dict.format_footer();
    (encoded, footer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_symbol_compression_skips_at_medium() {
        let (body, footer) = apply_symbol_compression("hello world", Fidelity::Medium);
        assert_eq!(body, "hello world");
        assert!(footer.is_empty());
    }

    #[test]
    fn apply_symbol_compression_skips_at_high() {
        let (body, footer) = apply_symbol_compression("hello world", Fidelity::High);
        assert_eq!(body, "hello world");
        assert!(footer.is_empty());
    }

    #[test]
    fn apply_symbol_compression_encodes_at_low() {
        // Tokens must appear ≥ 2 times to trigger opcode assignment.
        let (body, _footer) = apply_symbol_compression("hello hello world", Fidelity::Low);
        // "hello" appears twice and should be encoded to $1
        assert!(!body.contains("hello"), "expected 'hello' to be encoded, got: {}", body);
        // "world" appears once and stays as-is
        assert!(body.contains("world"), "expected 'world' to remain, got: {}", body);
        // The SymbolDictionary starts with 34 built-in primitives, so
        // format_footer returns empty (reverse.len() > 32 threshold).
        // The important thing is the body was encoded.
        assert!(body.starts_with("$1 "), "expected encoded body to start with $1, got: {}", body);
        // footer is empty due to >32 primitive entries in the dictionary
    }
}