// src/ir/layers/patterns.rs
//
// Phase F: Layered Encoding — Pattern Recognizer (Layer 4)
// Identifies common code patterns and compresses them to single ops.
//
// Patterns detected:
//   - Constructor injection (DEF_M + SIG for injectable params + INJECTS)
//   - Observable stream (DEF_M + RET(Promise) + FLAGS(ASYNC))
//   - Getter/Setter pattern (DEF_M("get/set X"))
//   - Override pattern (DEF_M + FLAGS(OVERRIDE))

use crate::ir::opcodes::CoreOp;
use super::PatternRecognizer;

/// Pattern recognizer (Layer 4).
/// Analyzes the instruction stream and compresses recognized patterns.
pub struct CodePatternRecognizer;

impl CodePatternRecognizer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CodePatternRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

impl PatternRecognizer for CodePatternRecognizer {
    fn recognize(&self, instructions: &[CoreOp]) -> Vec<CoreOp> {
        let mut output = Vec::new();
        let mut i = 0;

        while i < instructions.len() {
            // Try to match patterns starting at position i
            if let Some((pat, consumed)) = try_recognize_pattern(&instructions[i..]) {
                output.push(pat);
                if consumed == 0 {
                    // Additive pattern: keep the original instructions too,
                    // advance by 1 (the triggering instruction)
                    output.push(instructions[i].clone());
                    i += 1;
                } else {
                    // Consumptive pattern: skip consumed instructions
                    i += consumed;
                }
            } else {
                output.push(instructions[i].clone());
                i += 1;
            }
        }

        output
    }
}

/// Try to recognize a pattern at the current position.
/// Returns (compressed_op, instructions_consumed) if pattern matches.
fn try_recognize_pattern(slice: &[CoreOp]) -> Option<(CoreOp, usize)> {
    if slice.is_empty() {
        return None;
    }

    // Pattern: Constructor injection
    // DEF_M where name suggests constructor + following Param instructions + INJECTS
    if let Some(result) = try_ctor_pattern(slice) {
        return Some(result);
    }

    // Pattern: Observable/async method
    // DEF_M + RET($P|$s) + FLAGS(ASYNC)
    if let Some(result) = try_observable_pattern(slice) {
        return Some(result);
    }

    // Pattern: Getter/Setter
    // DEF_M where name starts with "get " or "set "
    if let Some(result) = try_accessor_pattern(slice) {
        return Some(result);
    }

    None
}

/// Pattern: Constructor injection
/// Matches: DEF_M with name "constructor" or "new" — emits a CTOR flag
/// but does NOT consume the instructions (they still need to be emitted).
fn try_ctor_pattern(slice: &[CoreOp]) -> Option<(CoreOp, usize)> {
    if slice.is_empty() {
        return None;
    }

    // First instruction must be DefMethod with "constructor" or class name
    let (_, method_id, _name) = match &slice[0] {
        CoreOp::DefMethod(cid, mid, name) => {
            if name == "constructor" || name == "new" {
                (cid, mid, name)
            } else {
                return None;
            }
        }
        _ => return None,
    };

    // Emit a CTOR flag but do NOT consume instructions — the original
    // DefMethod, Param, and Return instructions must all be preserved.
    Some((
        CoreOp::Flags(method_id.clone(), vec!["CTOR".to_string()]),
        0, // consumed = 0 means no instructions are consumed
    ))
}

/// Pattern: Observable/async method
/// Matches: DEF_M + RET(Promise/Observable) + FLAGS(ASYNC)
/// Emits an OBSERVABLE flag but does NOT consume instructions.
fn try_observable_pattern(slice: &[CoreOp]) -> Option<(CoreOp, usize)> {
    if slice.is_empty() {
        return None;
    }

    let (_class_id, method_id, _method_name) = match &slice[0] {
        CoreOp::DefMethod(cid, mid, name) => (cid.clone(), mid.clone(), name.clone()),
        _ => return None,
    };

    // Look ahead for RET with Promise/Observable type
    let mut has_observable_return = false;
    let mut has_async_flag = false;

    for i in 1..slice.len().min(6) {
        match &slice[i] {
            CoreOp::Return(_, ty) => {
                if ty == "$P" || ty.contains("Promise") || ty.contains("Observable") {
                    has_observable_return = true;
                }
            }
            CoreOp::Flags(tid, flags) => {
                if tid == &method_id && flags.contains(&"ASYNC".to_string()) {
                    has_async_flag = true;
                }
            }
            _ => {}
        }
    }

    if has_observable_return || has_async_flag {
        Some((
            CoreOp::Flags(method_id, vec!["OBSERVABLE".to_string()]),
            0, // additive — do not consume any instructions
        ))
    } else {
        None
    }
}

/// Pattern: Getter/Setter accessor
/// Matches: DEF_M where name starts with "get " or "set "
fn try_accessor_pattern(slice: &[CoreOp]) -> Option<(CoreOp, usize)> {
    if slice.is_empty() {
        return None;
    }

    match &slice[0] {
        CoreOp::DefMethod(_, method_id, name) => {
            let name_lower = name.to_lowercase();
            if name_lower.starts_with("get ") {
                let property = name[4..].trim().to_string();
                Some((
                    CoreOp::Flags(method_id.clone(), vec!["GETTER".to_string(), property]),
                    1,
                ))
            } else if name_lower.starts_with("set ") {
                let property = name[4..].trim().to_string();
                Some((
                    CoreOp::Flags(method_id.clone(), vec!["SETTER".to_string(), property]),
                    1,
                ))
            } else {
                None
            }
        }
        _ => None,
    }
}

#[cfg(test)]
#[path = "../../tests/ir/layers/patterns.rs"]
mod tests;
