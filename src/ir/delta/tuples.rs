// src/ir/delta/tuples.rs
//
// Tuple-key helpers for external consumers (state replay, etc.): the
// `Vec<String>` wire-tuple equivalents of the `CoreOp`-based primary-key and
// key-tuple extraction.
//
// Split out of `src/ir/delta.rs` (active-file size policy): the module had
// exceeded the 615-line ceiling. This is a pure relocation -- the code below
// is byte-for-byte the previous implementation, and `delta` re-exports both
// helpers so `crate::ir::delta::{primary_key_from_tuple, key_tuple_from_tuple}`
// keeps resolving.

// ── Public helpers for external consumers (replay, etc.) ─────────

/// Extract the primary key from an instruction tuple (Vec<String>).
/// Used by state replay to match instructions by key.
pub fn primary_key_from_tuple(tuple: &[String]) -> String {
    if tuple.is_empty() {
        return String::new();
    }
    match tuple[0].as_str() {
        "DEF_C" => format!("DEF_C:{}", tuple.get(1).unwrap_or(&String::new())),
        "DEF_M" => format!(
            "DEF_M:{}:{}",
            tuple.get(1).unwrap_or(&String::new()),
            tuple.get(2).unwrap_or(&String::new())
        ),
        "DEF_F" => format!(
            "DEF_F:{}:{}",
            tuple.get(1).unwrap_or(&String::new()),
            tuple.get(2).unwrap_or(&String::new())
        ),
        "DEF_I" => format!("DEF_I:{}", tuple.get(1).unwrap_or(&String::new())),
        "DEF_IM" => format!(
            "DEF_IM:{}:{}",
            tuple.get(1).unwrap_or(&String::new()),
            tuple.get(2).unwrap_or(&String::new())
        ),
        "DEF_IF" => format!(
            "DEF_IF:{}:{}",
            tuple.get(1).unwrap_or(&String::new()),
            tuple.get(2).unwrap_or(&String::new())
        ),
        "SIG" => format!(
            "SIG:{}:{}",
            tuple.get(1).unwrap_or(&String::new()),
            tuple.get(2).unwrap_or(&String::new())
        ),
        "RET" => format!("RET:{}", tuple.get(1).unwrap_or(&String::new())),
        "FIELD_T" => format!("FIELD_T:{}", tuple.get(1).unwrap_or(&String::new())),
        "MOD_M" => format!("MOD_M:{}", tuple.get(1).unwrap_or(&String::new())),
        "MOD_C" => format!("MOD_C:{}", tuple.get(1).unwrap_or(&String::new())),
        "MOD_I" => format!("MOD_I:{}", tuple.get(1).unwrap_or(&String::new())),
        "CTRL_SUM" => format!("CTRL_SUM:{}", tuple.get(1).unwrap_or(&String::new())),
        "PAT_FACT" => format!("PAT_FACT:{}", tuple.get(1).unwrap_or(&String::new())),
        "FLAGS" => format!("FLAGS:{}", tuple.get(1).unwrap_or(&String::new())),
        "FLAGS_C" => format!("FLAGS_C:{}", tuple.get(1).unwrap_or(&String::new())),
        "EXT" => format!("EXT:{}", tuple.get(1).unwrap_or(&String::new())),
        "EXT_I" => format!("EXT_I:{}", tuple.get(1).unwrap_or(&String::new())),
        "IMPL" => format!(
            "IMPL:{}:{}",
            tuple.get(1).unwrap_or(&String::new()),
            tuple.get(2).unwrap_or(&String::new())
        ),
        "INJECTS" => format!("INJECTS:{}", tuple.get(1).unwrap_or(&String::new())),
        "IMP" => format!("IMP:{}", tuple.get(1).unwrap_or(&String::new())),
        "TYPE" => format!("TYPE:{}", tuple.get(1).unwrap_or(&String::new())),
        // Edit Mode: Verbatim Method Bodies
        "BODY" => format!("BODY:{}", tuple.get(1).unwrap_or(&String::new())),
        // R-43a: Execution Semantics
        "DATAFLOW" => format!("DATAFLOW:{}", tuple.get(1).unwrap_or(&String::new())),
        "CTRL" => format!("CTRL:{}", tuple.get(1).unwrap_or(&String::new())),
        "EFFECT" => format!("EFFECT:{}", tuple.get(1).unwrap_or(&String::new())),
        "CTX" => format!("CTX:{}", tuple.get(1).unwrap_or(&String::new())),
        // Structural invocations (native call graph): explicit arm so a
        // known opcode never reaches the unknown-opcode fallback (which
        // warns in debug builds). Dual shape, and it must agree EXACTLY with
        // `delta::primary_key`: the exact form keeps its established key, and
        // the spread form appends the qualifier so `foo(a)` and `foo(...args)`
        // can never share a delta identity.
        "CALL" => {
            let base = format!(
                "CALL:{}:{}:{}",
                tuple.get(1).unwrap_or(&String::new()),
                tuple.get(2).unwrap_or(&String::new()),
                tuple.get(3).unwrap_or(&String::new())
            );
            match tuple.get(4) {
                Some(qualifier) if !qualifier.is_empty() => format!("{base}:{qualifier}"),
                _ => base,
            }
        }
        _ => {
            // F-16: Unknown opcode — fallback produces a key from the full tuple.
            if cfg!(debug_assertions) {
                eprintln!(
                    "[warn] primary_key_from_tuple: unknown opcode '{}'",
                    tuple[0]
                );
            }
            tuple.join(":")
        }
    }
}

/// Extract the key tuple from an instruction tuple (Vec<String>).
/// Returns the opcode + identifying operands.
pub fn key_tuple_from_tuple(tuple: &[String]) -> Vec<String> {
    if tuple.is_empty() {
        return Vec::new();
    }
    match tuple[0].as_str() {
        "DEF_C" => vec![tuple[0].clone(), tuple.get(1).cloned().unwrap_or_default()],
        "DEF_M" => vec![
            tuple[0].clone(),
            tuple.get(1).cloned().unwrap_or_default(),
            tuple.get(2).cloned().unwrap_or_default(),
        ],
        "DEF_F" => vec![
            tuple[0].clone(),
            tuple.get(1).cloned().unwrap_or_default(),
            tuple.get(2).cloned().unwrap_or_default(),
        ],
        "DEF_I" => vec![tuple[0].clone(), tuple.get(1).cloned().unwrap_or_default()],
        "DEF_IM" | "DEF_IF" => vec![
            tuple[0].clone(),
            tuple.get(1).cloned().unwrap_or_default(),
            tuple.get(2).cloned().unwrap_or_default(),
        ],
        "SIG" => vec![
            tuple[0].clone(),
            tuple.get(1).cloned().unwrap_or_default(),
            tuple.get(2).cloned().unwrap_or_default(),
        ],
        "RET" => vec![tuple[0].clone(), tuple.get(1).cloned().unwrap_or_default()],
        "FIELD_T" => vec![tuple[0].clone(), tuple.get(1).cloned().unwrap_or_default()],
        "MOD_M" => vec![tuple[0].clone(), tuple.get(1).cloned().unwrap_or_default()],
        "MOD_C" => vec![tuple[0].clone(), tuple.get(1).cloned().unwrap_or_default()],
        "MOD_I" => vec![tuple[0].clone(), tuple.get(1).cloned().unwrap_or_default()],
        "CTRL_SUM" => vec![tuple[0].clone(), tuple.get(1).cloned().unwrap_or_default()],
        "PAT_FACT" => vec![tuple[0].clone(), tuple.get(1).cloned().unwrap_or_default()],
        "FLAGS" => vec![tuple[0].clone(), tuple.get(1).cloned().unwrap_or_default()],
        "FLAGS_C" => vec![tuple[0].clone(), tuple.get(1).cloned().unwrap_or_default()],
        "EXT" => vec![tuple[0].clone(), tuple.get(1).cloned().unwrap_or_default()],
        "EXT_I" => vec![tuple[0].clone(), tuple.get(1).cloned().unwrap_or_default()],
        "IMPL" => vec![
            tuple[0].clone(),
            tuple.get(1).cloned().unwrap_or_default(),
            tuple.get(2).cloned().unwrap_or_default(),
        ],
        "INJECTS" => vec![tuple[0].clone(), tuple.get(1).cloned().unwrap_or_default()],
        "IMP" => vec![tuple[0].clone(), tuple.get(1).cloned().unwrap_or_default()],
        "TYPE" => vec![tuple[0].clone(), tuple.get(1).cloned().unwrap_or_default()],
        // Edit Mode: Verbatim Method Bodies
        "BODY" => vec![tuple[0].clone(), tuple.get(1).cloned().unwrap_or_default()],
        // R-43a: Execution Semantics
        "DATAFLOW" => vec![tuple[0].clone(), tuple.get(1).cloned().unwrap_or_default()],
        "CTRL" => vec![tuple[0].clone(), tuple.get(1).cloned().unwrap_or_default()],
        "EFFECT" => vec![tuple[0].clone(), tuple.get(1).cloned().unwrap_or_default()],
        "CTX" => vec![tuple[0].clone(), tuple.get(1).cloned().unwrap_or_default()],
        // Structural invocations (native call graph): explicit arm so a
        // known opcode never reaches the unknown-opcode fallback. The key
        // keeps every identifying operand — caller, callee, written count, and
        // the spread qualifier when present (matching `delta::key_tuple`).
        "CALL" => {
            let mut key = vec![
                tuple[0].clone(),
                tuple.get(1).cloned().unwrap_or_default(),
                tuple.get(2).cloned().unwrap_or_default(),
                tuple.get(3).cloned().unwrap_or_default(),
            ];
            if let Some(qualifier) = tuple.get(4) {
                key.push(qualifier.clone());
            }
            key
        }
        _ => {
            // F-17: Unknown opcode — fallback returns the full instruction body.
            if cfg!(debug_assertions) {
                eprintln!("[warn] key_tuple_from_tuple: unknown opcode '{}'", tuple[0]);
            }
            tuple.to_vec()
        }
    }
}
