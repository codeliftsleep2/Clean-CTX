// src/compression/opcodes.rs
//
// SHARED primitive opcode table — the single source of truth for the 32
// ultra-frequent tokens that get encoded as 1–5 character opcodes.
//
// Before Phase 2, this table was duplicated in two places:
//   - `dictionary::symbol::SymbolDictionary::new`        (32 `insert_primitive` calls)
//   - `dictionary::symbol::SymbolDictionary::is_primitive_opcode` (literal list)
//   - `decompression::opcodes::PRIMITIVE_OPCODES`        (the actual table)
//
// Phase 2 collapses all three into this single module. The dictionary
// loads its built-in primitives by iterating `PRIMITIVE_OPCODES`, the
// decompressor expands its `BTreeMap` from `builtin_opcode_map()`, and
// `is_primitive_opcode` is now a `O(1)` lookup against the same constant.

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

/// Returns `true` if the given opcode is one of the built-in primitives
/// (as opposed to a runtime-assigned opcode like `$1`, `$2`, …).
///
/// This is `O(n)` in the number of primitives (32 today), which is fast
/// enough for the only caller (`SymbolDictionary::format_footer`) and
/// keeps the implementation simple. Promoting to a `HashSet` is a
/// trivially low-cost optimisation if profiling ever demands it.
pub fn is_primitive_opcode(opcode: &str) -> bool {
    PRIMITIVE_OPCODES.iter().any(|(op, _)| *op == opcode)
}

#[cfg(test)]
#[path = "../tests/compression/opcodes.rs"]
mod tests;
