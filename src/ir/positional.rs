// src/ir/positional.rs
//
// Phase H: Positional Encoding & Advanced Compression.
//
// Positional encoding is **not a separate format** — it's the same JSON
// tuple representation produced by `wire::op_to_tuple`, but with the
// opcode string **stripped** from the first position. The client knows
// the static schema (see docs/COMPILER_IR.md §14) and uses the index
// position to determine meaning.
//
// Layer 1 IR (named):   ["DEF_M","C1","M1","processComplexData"]
// Layer 4 IR (positional): the operand tuple only (no opcode):
//   ["C1","M1","processComplexData"]
//
// Two encodings are supported:
//   - **Stripped opcode** (default): the opcode is dropped, and the
//     remaining operands are encoded positionally per the schema.
//   - **Tagged opcode** (debug / mixed streams): the opcode is kept
//     at the front so heterogeneous streams can be decoded in any
//     order. The `tagged` flag controls this.
//
// This module is the "transport" side of Phase H. The compression side
// lives in `crate::ir::patterns` (the Layer 4 `CompressingPatternRecognizer`).

use serde_json::{json, Value};

use super::opcodes::{arity, opcode_name};
use super::opcodes::CoreOp;
use super::wire::{op_to_tuple, tuple_to_op};

/// Configuration for positional encoding.
///
/// `tagged: false`  → opcode string is stripped (maximum compression).
/// `tagged: true`   → opcode string is preserved at index 0 (mixed streams).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PositionalConfig {
    /// Whether to preserve the opcode in the output tuple.
    pub tagged: bool,
}

impl PositionalConfig {
    /// New stripped (default) config.
    pub fn stripped() -> Self {
        Self { tagged: false }
    }

    /// New tagged config (preserves the opcode in the output).
    pub fn tagged() -> Self {
        Self { tagged: true }
    }
}

/// Serialize a single `CoreOp` to its positional (key-stripped) representation.
///
/// With `tagged = false`: returns the operands only (no opcode).
/// With `tagged = true` : returns `[opcode, ...operands]` (same as `op_to_tuple`).
pub fn encode_op(op: &CoreOp, config: PositionalConfig) -> Vec<String> {
    let full = op_to_tuple(op);
    if config.tagged {
        full
    } else {
        // Strip the opcode (index 0) — the schema tells the client
        // which opcode to expect based on context.
        full.into_iter().skip(1).collect()
    }
}

/// Deserialize a positional (key-stripped) tuple back to a `CoreOp`.
///
/// The caller must supply the expected opcode name (clients consult the
/// static schema). Returns `None` if the opcode is unknown or the
/// arity doesn't match the tuple length.
pub fn decode_op(opcode: &str, operands: &[String]) -> Option<CoreOp> {
    let expected = arity(opcode)?;

    // Variadic opcodes (-1): accept any length >= 2 operands
    if expected < 0 {
        if operands.len() < 2 {
            return None;
        }
    } else {
        // Fixed-arity: must match exactly. Note: `op_to_tuple` produces
        // `arity - 1` operands (opcode is excluded) for fixed-arity ops.
        let expected_operands = (expected - 1) as usize;
        if operands.len() != expected_operands {
            return None;
        }
    }

    // Reconstruct the full tuple and delegate to the existing decoder.
    let mut full = Vec::with_capacity(operands.len() + 1);
    full.push(opcode.to_string());
    full.extend(operands.iter().cloned());
    tuple_to_op(&full)
}

/// Serialize a stream of `CoreOp`s to positional tuples, using the given config.
///
/// Returns a `Vec<Vec<String>>` that can be JSON-serialized directly. The
/// caller is responsible for grouping operands by opcode in a tagged
/// stream; the default (stripped) encoding does not require grouping
/// because the consumer knows the schema.
pub fn encode_stream(ops: &[CoreOp], config: PositionalConfig) -> Vec<Vec<String>> {
    ops.iter().map(|op| encode_op(op, config)).collect()
}

/// Serialize a `CompiledIR`'s instructions to the wire format, using
/// positional encoding.
///
/// The returned `serde_json::Value` shape:
///
/// ```json
/// {
///   "file": "<path_alias>",
///   "v": <version>,
///   "encoding": "positional" | "tagged",
///   "ir": [
///     ["C1", "M1", "processComplexData"],
///     ["C1", "M2", "doWork"],
///     ...
///   ]
/// }
/// ```
pub fn ir_to_positional_wire(
    file_id: &str,
    version: u64,
    ops: &[CoreOp],
    config: PositionalConfig,
) -> Value {
    let tuples = encode_stream(ops, config);
    json!({
        "file": file_id,
        "v": version,
        "encoding": if config.tagged { "tagged" } else { "positional" },
        "ir": tuples
    })
}

/// Estimate the token savings of positional vs. named encoding for a
/// stream of instructions. Returns `(named_chars, positional_chars)`.
///
/// Tokens are estimated as ceiling(chars / 4) — a common LLM rule of
/// thumb. Both counts include the JSON array brackets and quotes.
pub fn estimate_savings(ops: &[CoreOp]) -> (usize, usize) {
    let config = PositionalConfig::stripped();
    let named = ops.iter().map(|op| op_to_tuple(op).join(",").len() + 4).sum::<usize>();
    let positional = ops.iter()
        .map(|op| encode_op(op, config).join(",").len() + 4)
        .sum::<usize>();
    (named, positional)
}

/// Total character count of a positional stream, including the outer
/// `ir: [...]` envelope. Useful for printing compactness stats.
pub fn positional_char_count(ops: &[CoreOp], config: PositionalConfig) -> usize {
    let tuples = encode_stream(ops, config);
    let inner: usize = tuples.iter().map(|t| t.join(",").len() + 4).sum();
    inner + 12 // `{...,"ir":[...]}`
}

/// Verify that a positional stream decodes to the same ops as the
/// original. Returns the first index where they differ, or `None` if
/// they match. Each positional tuple must be tagged with its opcode
/// (so the verifier can decode it back).
pub fn verify_round_trip(ops: &[CoreOp], tagged: &[Vec<String>]) -> Option<usize> {
    if ops.len() != tagged.len() {
        return Some(ops.len().min(tagged.len()));
    }
    for (i, (op, tuple)) in ops.iter().zip(tagged.iter()).enumerate() {
        if tuple.is_empty() {
            return Some(i);
        }
        let opcode = &tuple[0];
        if opcode != opcode_name(op) {
            return Some(i);
        }
        let decoded = match tuple_to_op(tuple) {
            Some(d) => d,
            None => return Some(i),
        };
        if &decoded != op {
            return Some(i);
        }
    }
    None
}

#[cfg(test)]
#[path = "../tests/ir/positional.rs"]
mod tests;
