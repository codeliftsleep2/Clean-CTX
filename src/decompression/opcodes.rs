// src/decompression/opcodes.rs
//
// The 32-entry primitive opcode table. In Phase 2 this becomes the single
// source of truth that both `dictionary::symbol::SymbolDictionary` and
// `decompression::Decompressor` load from.
//
// In Phase 1 the table is duplicated between this file and
// `dictionary::symbol` — the consolidation happens in Phase 2.

/// Built-in primitive opcodes: (opcode, token) pairs that compress
/// ultra-frequent tokens to 1–5 character codes.
pub const PRIMITIVE_OPCODES: &[(&str, &str)] = &[
    ("$e", "export"),
    ("$c", "class"),
    ("$a", "async"),
    ("$P", "Promise"),
    ("$b", "boolean"),
    ("$s", "string"),
    ("$n", "number"),
    ("$v", "void"),
    ("$T", "true"),
    ("$F", "false"),
    ("$pu", "public"),
    ("$pv", "private"),
    ("$pd", "protected"),
    ("$st", "static"),
    ("$nw", "new"),
    ("$r", "return"),
    ("$t", "throw"),
    ("$E", "Error"),
    ("$k", "const"),
    ("$l", "let"),
    ("$i", "if"),
    ("$fr", "for"),
    ("$w", "while"),
    ("$h", "this"),
    ("$x", "extends"),
    ("$m", "implements"),
    ("$im", "import"),
    ("$fm", "from"),
    ("$if", "interface"),
    ("$ty", "type"),
    ("$fn", "function"),
    ("$ctor", "constructor"),
    ("$ud", "undefined"),
    ("$nl", "null"),
];

/// Build a BTreeMap from opcode → token for fast lookup during expansion.
pub(crate) fn builtin_opcode_map() -> std::collections::BTreeMap<&'static str, &'static str> {
    PRIMITIVE_OPCODES.iter().copied().collect()
}
