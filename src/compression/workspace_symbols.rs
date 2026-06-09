// src/compression/workspace_symbols.rs
//
// Phase III (Idea #9 — Cross-File Symbol Deduplication).
//
// Workspace-aware symbol compression using a two-pass approach:
//   Pass 1: Collect token frequencies across all files
//   Pass 2: Encode each file using the globally-optimised dictionary
//
// This replaces per-file `$1=Service; $2=Observable` dictionaries with
// a single shared `§GSYM` dictionary at the workspace level. Per-file
// output only emits a compact `§GSYM 0,1,2` reference list.

use crate::compression::Fidelity;
use crate::dictionary::GlobalSymbolTable;

/// Build a workspace-level global symbol table from pre-compressed file bodies.
///
/// This is Pass 1 of the two-pass approach. Call this after all files have
/// been compressed through the normal pipeline (without symbol compression),
/// and before encoding with the global dictionary.
///
/// Returns a `GlobalSymbolTable` with codes built and ready for encoding.
pub fn build_global_symbol_table(
    file_bodies: &[(String, String)], // (file_path, body_content) pairs
) -> GlobalSymbolTable {
    let mut table = GlobalSymbolTable::new();

    // Pass 1: Count all token frequencies across all files
    for (_path, body) in file_bodies {
        table.count_tokens(body);
    }

    // Build the global Huffman codes
    table.build_codes();

    table
}

/// Encode a single file's body using the global symbol dictionary.
///
/// This is Pass 2. Call `begin_file()` before each file, then `encode_body()`,
/// then `format_file_refs()` to get the per-file footer.
pub fn encode_with_global_symbols(
    table: &mut GlobalSymbolTable,
    body: &str,
    fidelity: Fidelity,
) -> (String, String) {
    if fidelity != Fidelity::Low {
        return (body.to_string(), String::new());
    }

    table.begin_file();
    let encoded = table.encode_body(body);
    let footer = table.format_file_refs();
    (encoded, footer)
}

#[cfg(test)]
#[path = "../tests/compression/workspace_symbols.rs"]
mod tests;