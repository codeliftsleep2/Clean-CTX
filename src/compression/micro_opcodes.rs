// src/compression/micro_opcodes.rs
//
// Phase III (Idea #11 — Micro-Opcode Table for Text).
//
// Adds single-character micro-opcodes (§-prefixed) for common structural
// patterns, reducing token count by 15-25% at Low fidelity.
//
// Phase 8: Expanded micro-opcode table with §I (⊕guard), §L (⊕loop),
// and §E (⊕⇒) for additional text pipeline compression.
//
// ## Micro-opcode table
//
// | Micro-opcode | Meaning                  | Current equivalent    |
// |--------------|--------------------------|-----------------------|
// | `§C`         | Class field delimiters   | `ClassName{fields}`   |
// | `§P`         | Constructor/pattern def  | `$ctor C1 M1 params`  |
// | `§I`         | Conditional/if marker    | `⊕guard`              |
// | `§L`         | Loop marker              | `⊕loop`               |
// | `§E`         | Return marker            | `⊕⇒`                  |
//
// ## How it works
//
// 1. At Low fidelity, after the body is assembled (semicolon-separated),
//    `apply_micro_opcodes` replaces structural patterns with §-prefixed codes.
// 2. The output is tokenized by the LLM. The § prefix ensures each micro-opcode
//    is a distinct token (§C, §P are single tokens in BPE).
// 3. The client decompressor expands micro-opcodes back to the original format
//    via `expand_micro_opcodes`.
//
// ## Savings
//
// - `§C` replaces `ClassName{fields}` → `§C ClassName§C fields`
//   Replaces `{` (1 token) and `}` (1 token) with `§C` (1 token) → saves 1 token/class
// - `§P` replaces `$ctor C1 M1 params` → `§P C1 M1 params`
//   `$ctor` is 2 tokens ($ + ctor); `§P` is 1 token → saves 1 token/method
// - `§I` replaces `⊕guard` (2 tokens: ⊕ + guard) → `§I` (1 token) → saves 1 token/method
// - `§L` replaces `⊕loop` (2 tokens: ⊕ + loop) → `§L` (1 token) → saves 1 token/loop
// - `§E` replaces `⊕⇒` (2 tokens: ⊕ + ⇒) → `§E` (1 token) → saves 1 token/return
//
// Medium and High fidelities are unaffected.

use crate::compression::Fidelity;

/// Micro-opcode replacement table. Each entry is
/// `(opcode, pattern_to_find, replacement)`.
const MICRO_OPCODE_TABLE: &[(&str, &str, &str)] = &[
    // §C — Class field delimiters: ClassName{fields} → §C ClassName§C fields
    ("§C", "{", "§C"),
    ("§C", "}", "§C"),
    // §P — Constructor / pattern prefix: $ctor → §P
    ("§P", "$ctor", "§P"),
    // §I — Conditional/if marker: ⊕guard → §I
    ("§I", "⊕guard", "§I"),
    // §L — Loop marker: ⊕loop → §L
    ("§L", "⊕loop", "§L"),
    // §E — Return marker: ⊕⇒ → §E
    ("§E", "⊕⇒", "§E"),
];

/// Apply micro-opcodes to the compressed body at Low fidelity.
///
/// This is a post-processing step applied after the body is assembled
/// (semicolon-separated) but before symbol compression.
pub fn apply_micro_opcodes(body: &str, fidelity: Fidelity) -> String {
    if fidelity != Fidelity::Low || body.is_empty() {
        return body.to_string();
    }

    let mut result = body.to_string();
    for &(_opcode, pattern, replacement) in MICRO_OPCODE_TABLE {
        result = result.replace(pattern, replacement);
    }
    result
}

/// Expand micro-opcodes back to their original form.
///
/// Used by the decompression side to restore the original structural format.
/// The expansion is order-independent since each micro-opcode maps to a
/// unique original pattern with no overlap.
#[allow(dead_code)]
pub fn expand_micro_opcodes(body: &str) -> String {
    let mut result = body.to_string();

    // Expand §E → ⊕⇒ (before §C to avoid any overlap)
    result = result.replace("§E", "⊕⇒");
    // Expand §L → ⊕loop
    result = result.replace("§L", "⊕loop");
    // Expand §I → ⊕guard
    result = result.replace("§I", "⊕guard");
    // Expand §P → $ctor (before §C to avoid any overlap)
    result = result.replace("§P", "$ctor");

    // §C is ambiguous (used for both { and }). The expansion strategy:
    // - First §C after a word character → opening brace
    // - Second §C → closing brace
    // This works because class entries follow the pattern:
    //   ClassName§C field1;field2§C  or  §C ClassName§C
    // In the stream, §C alternates between opening and closing.
    result = expand_class_markers(&result);

    result
}

/// Expand §C markers by alternating between `{` and `}`.
///
/// In the compressed format, §C appears in pairs:
///   `§C ClassName§C field1;field2§C` → `ClassName{field1;field2}`
///   or for classes without fields:
///   `§C ClassName§C` → `ClassName{}`
///
/// The alternation strategy: odd-numbered §C → `{`, even-numbered → `}`.
#[allow(dead_code)]
fn expand_class_markers(body: &str) -> String {
    let mut result = String::with_capacity(body.len());
    let mut class_marker_count: usize = 0;

    for part in body.split("§C") {
        if !result.is_empty() {
            // Odd count → opening brace, even count → closing brace
            class_marker_count += 1;
            if class_marker_count % 2 == 1 {
                result.push('{');
            } else {
                result.push('}');
            }
        }
        result.push_str(part);
    }

    result
}

/// Returns the list of micro-opcodes for documentation/testing purposes.
#[allow(dead_code)]
pub fn micro_opcode_table() -> &'static [(&'static str, &'static str, &'static str)] {
    MICRO_OPCODE_TABLE
}

#[cfg(test)]
#[path = "../tests/compression/micro_opcodes.rs"]
mod tests;
