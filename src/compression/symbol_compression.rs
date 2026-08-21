// src/compression/symbol_compression.rs
//
// Low-fidelity symbol-dictionary opcode pass. At `Fidelity::Low` every
// unique token in the body is replaced with a short symbol reference
// (`$1`, `$2`, …) and a legend is appended as a footer. Higher fidelities
// skip this pass because structural markers already keep density high.

use crate::compression::Fidelity;
use crate::dictionary::SymbolDictionary;
use crate::dictionary::symbol::tokenize_for_symbols;

/// Apply the symbol-dictionary opcode pass (Low fidelity only). Higher
/// fidelities don't need it — the structural markers already provide
/// sufficient density.
pub fn apply_symbol_compression(body_content: &str, fidelity: Fidelity) -> (String, String) {
    if fidelity != Fidelity::Low {
        return (body_content.to_string(), String::new());
    }
    let mut sym_dict = SymbolDictionary::new();
    for token in body_content.split_whitespace() {
        let clean = tokenize_for_symbols(token);
        if !clean.is_empty() {
            sym_dict.register(clean);
        }
        if let Some(rest) = token.strip_prefix('⊕')
            && !rest.is_empty()
        {
            sym_dict.register(rest);
        }
    }
    let encoded = sym_dict.encode(body_content);
    let footer = sym_dict.format_footer();
    (encoded, footer)
}

#[cfg(test)]
#[path = "../tests/compression/symbol_compression.rs"]
mod tests;
