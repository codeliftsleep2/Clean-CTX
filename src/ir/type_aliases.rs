// src/ir/type_aliases.rs
//
// R-02 Phase 3: Type-aware compression for the IR path.
//
// After IR compilation, this pass replaces type-named operands in
// `FieldType`, `Return`, and `Param` ops with alias tokens from the
// user's `.clean-ctx.json` config. For each *used* alias, a
// `CoreOp::TypeAlias(alias, original)` op is appended so the mapping
// is reversible.
//
// Invariants preserved:
//   - C1-C4: CBM never modifies Core IR — this pass is user-config
//     driven, not CBM. No conflict.
//   - B1-B5: behavioral enrichment never changes structural meaning —
//   this is a pure token rename (`User` → `$uid`), not a semantic
//   change. The IR structure is identical; only type-name strings change.
//
// The substitution reuses `compression::type_aliases::apply_type_aliases`
// for token-boundary matching, then parses the `§TA` footer to extract
// the used alias→original pairs for `CoreOp::TypeAlias` emission.

use std::collections::BTreeMap;

use super::opcodes::CoreOp;
use crate::compression::type_aliases::apply_type_aliases;

/// Apply configured type aliases to a compiled IR instruction stream.
///
/// Scans `FieldType`, `Return`, and `Param` ops for type names that
/// match configured aliases. Replaces whole-token occurrences with the
/// alias token and appends `CoreOp::TypeAlias(alias, original)` for
/// each *used* alias at the end of the instruction stream.
///
/// This is a post-compilation pass — call it after `IRCompiler::compile`
/// returns. The pass is additive: it never removes or reorders existing
/// instructions, only mutates type-name strings and appends `TypeAlias`
/// ops.
pub fn apply_type_aliases_to_ir(
    instructions: &mut Vec<CoreOp>,
    aliases: &BTreeMap<String, String>,
) {
    if aliases.is_empty() {
        return;
    }

    // Track which aliases were used across all type substitutions.
    // Key: alias token (e.g. "$uid"), Value: original type (e.g. "User").
    let mut used: BTreeMap<String, String> = BTreeMap::new();

    for op in instructions.iter_mut() {
        match op {
            CoreOp::FieldType(_, type_opcode) => {
                substitute_type_in_op(type_opcode, aliases, &mut used);
            }
            CoreOp::Return(_, type_opcode) => {
                substitute_type_in_op(type_opcode, aliases, &mut used);
            }
            CoreOp::Param(_, _, type_opcode, _) => {
                substitute_type_in_op(type_opcode, aliases, &mut used);
            }
            _ => {}
        }
    }

    // Append CoreOp::TypeAlias for each used alias. BTreeMap iteration
    // is deterministic (sorted by key), so the output is stable.
    for (alias, original) in &used {
        instructions.push(CoreOp::TypeAlias(alias.clone(), original.clone()));
    }
}

/// Substitute type aliases in a single type-opcode string.
///
/// Calls `apply_type_aliases` on the type string, updates the string
/// in place if substitution occurred, and records used aliases in the
/// `used` map.
fn substitute_type_in_op(
    type_opcode: &mut String,
    aliases: &BTreeMap<String, String>,
    used: &mut BTreeMap<String, String>,
) {
    let (substituted, footer) = apply_type_aliases(type_opcode, aliases);
    if !substituted.is_empty() && substituted != *type_opcode {
        // Parse the §TA footer to extract alias→original pairs.
        for pair in parse_ta_footer(&footer) {
            used.insert(pair.0, pair.1);
        }
        *type_opcode = substituted;
    }
}

/// Parse a `§TA` footer string into a list of `(alias, original)` pairs.
///
/// Footer format: `§TA $uid→UserId $jo→JsonObject`
/// Returns: `[("$uid", "UserId"), ("$jo", "JsonObject")]`
///
/// Returns an empty vec if the footer is empty or malformed.
fn parse_ta_footer(footer: &str) -> Vec<(String, String)> {
    let Some(rest) = footer.strip_prefix("§TA ") else {
        return Vec::new();
    };
    rest.split(' ')
        .filter_map(|entry| {
            entry.split_once('→').map(|(alias, original)| {
                (alias.to_string(), original.to_string())
            })
        })
        .collect()
}

#[cfg(test)]
#[path = "../tests/ir/type_aliases.rs"]
mod tests;