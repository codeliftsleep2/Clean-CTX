// src/dictionary/mod.rs
//
// Two unrelated dictionaries share this module because they're both keyed
// lookup tables. They're split into separate files for clarity:
//   - `path`   : maps absolute file paths to short aliases (α1, α2, …)
//   - `symbol` : maps frequently repeated tokens to ultra-short opcodes

pub(crate) mod path;
pub(crate) mod symbol;

pub use path::PathDictionary;
pub use symbol::{SymbolDictionary, SymbolKind};
