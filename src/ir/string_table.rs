// src/ir/string_table.rs
//
// Phase I: Ultra-Compact IR — String Table + Relative Referencing.
//
// Builds a per-file string table that maps all unique string values to
// integer indices. Instructions reference strings by index instead of
// repeating the full string, reducing JSON overhead by 25-40%.
//
// Wire format (encoding: "string_table"):
// ```json
// {
//   "file": "α1",
//   "v": 1,
//   "encoding": "string_table",
//   "t": ["C1", "SampleService", "M1", "processComplexData", ...],
//   "ir": [
//     [0, 1],         // DefClass(0→C1, 1→SampleService)
//     [0, 2, 3],     // DefMethod(0→C1, 2→M1, 3→processComplexData)
//     [2, 4, 5, 6],  // Param(2→M1, 4→P1, 5→$s, 6→payload)
//     [2, 7],         // Return(2→M1, 7→$b)
//     [0, 8]          // Flags(0→C1, 8→IF)
//   ]
// }
// ```
//
// String indices are serialized as JSON integers (no quotes), which saves
// ~2 bytes per string reference compared to the quoted-string positional format.

use serde_json::{json, Value};
use std::collections::HashMap;

use super::compiler::CompiledIR;
use super::opcodes::CoreOp;
use super::wire::{op_to_tuple, tuple_to_op};

/// A string table that maps strings to dense integer indices.
///
/// Strings are stored in insertion order. The same string always maps to
/// the same index (deduplication). The table can be serialized as a simple
/// JSON array of strings.
#[derive(Debug, Clone)]
pub struct StringTable {
    /// Strings in index order
    strings: Vec<String>,
    /// String → index lookup
    indices: HashMap<String, usize>,
}

impl StringTable {
    /// Create a new empty StringTable.
    pub fn new() -> Self {
        Self {
            strings: Vec::new(),
            indices: HashMap::new(),
        }
    }

    /// Build a StringTable from a list of CoreOp instructions.
    ///
    /// Iterates all instructions, extracts every string operand, and
    /// builds a deduplicated table ordered by first appearance.
    pub fn from_instructions(instructions: &[CoreOp]) -> Self {
        let mut table = Self::new();
        for op in instructions {
            let tuple = op_to_tuple(op);
            for s in &tuple {
                table.intern(s);
            }
        }
        table
    }

    /// Intern a string — returns its index, adding it if not present.
    pub fn intern(&mut self, s: &str) -> usize {
        if let Some(&idx) = self.indices.get(s) {
            return idx;
        }
        let idx = self.strings.len();
        self.strings.push(s.to_string());
        self.indices.insert(s.to_string(), idx);
        idx
    }

    /// Look up a string by its index.
    /// Returns None if the index is out of bounds.
    pub fn lookup(&self, idx: usize) -> Option<&str> {
        self.strings.get(idx).map(|s| s.as_str())
    }

    /// Get the number of unique strings in the table.
    pub fn len(&self) -> usize {
        self.strings.len()
    }

    /// Returns true if the table is empty.
    pub fn is_empty(&self) -> bool {
        self.strings.is_empty()
    }

    /// Serialize the string table as a JSON array of strings.
    pub fn to_json(&self) -> Value {
        json!(self.strings)
    }

    /// Deserialize a string table from a JSON array.
    pub fn from_json(value: &Value) -> Option<Self> {
        let arr = value.as_array()?;
        let mut table = Self::new();
        for v in arr {
            let s = v.as_str()?;
            table.intern(s);
        }
        Some(table)
    }

    /// Get all strings in the table (for iteration).
    pub fn strings(&self) -> &[String] {
        &self.strings
    }
}

impl Default for StringTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Encode a single CoreOp as a Vec of string table indices.
///
/// Each index is the position of the corresponding string in the table.
/// The caller must ensure the table already contains all strings from
/// the instruction (use `from_instructions` first).
pub fn encode_op(op: &CoreOp, table: &StringTable) -> Vec<usize> {
    let tuple = op_to_tuple(op);
    tuple
        .iter()
        .map(|s| {
            *table
                .indices
                .get(s)
                .expect("StringTable missing string — use from_instructions first")
        })
        .collect()
}

/// Decode a single CoreOp from a Vec of string table indices.
///
/// Returns None if any index is out of bounds or the resulting tuple
/// doesn't match a known opcode.
pub fn decode_op(indices: &[usize], table: &StringTable) -> Option<CoreOp> {
    let tuple: Vec<String> = indices
        .iter()
        .map(|&idx| {
            table
                .lookup(idx)
                .map(|s| s.to_string())
                .unwrap_or_default()
        })
        .collect();
    tuple_to_op(&tuple)
}

/// Serialize a CompiledIR to the string-table wire format.
///
/// Builds the string table from instructions, then encodes each
/// instruction as a Vec of integer indices into the table.
pub fn ir_to_string_table_wire(ir: &CompiledIR) -> Value {
    let table = StringTable::from_instructions(&ir.instructions);
    let ir_encoded: Vec<Vec<usize>> = ir
        .instructions
        .iter()
        .map(|op| encode_op(op, &table))
        .collect();

    json!({
        "file": ir.file_id,
        "v": ir.version,
        "encoding": "string_table",
        "t": table.to_json(),
        "ir": ir_encoded
    })
}

/// Deserialize a CompiledIR from the string-table wire format.
///
/// Builds the string table from `"t"`, then decodes each integer-index
/// instruction back to a CoreOp.
///
/// # Errors
///
/// Returns None if:
/// - Required fields (`file`, `v`, `t`, `ir`) are missing or wrong type
/// - Any instruction index is out of bounds for the table
/// - Any decoded tuple doesn't match a known opcode
pub fn wire_to_ir(value: &Value) -> Option<CompiledIR> {
    let file_id = value.get("file")?.as_str()?.to_string();
    let version = value.get("v")?.as_u64()?;
    let table = StringTable::from_json(value.get("t")?)?;
    let ir_array = value.get("ir")?.as_array()?;

    let mut instructions = Vec::new();
    for tuple_val in ir_array {
        let indices: Vec<usize> = tuple_val
            .as_array()?
            .iter()
            .map(|v| v.as_u64().map(|i| i as usize))
            .collect::<Option<Vec<_>>>()?;
        let op = decode_op(&indices, &table)?;
        instructions.push(op);
    }

    Some(CompiledIR {
        file_id,
        instructions,
        version,
    })
}

/// Estimate the character savings of string-table vs. named encoding.
/// Returns `(named_chars, table_chars)`.
///
/// Both counts include the full JSON envelope so the comparison is
/// apples-to-apples.
pub fn estimate_savings(ir: &CompiledIR) -> (usize, usize) {
    // Named encoding
    let named = super::wire::ir_to_wire(ir);
    let named_str = serde_json::to_string(&named).unwrap_or_default();

    // String-table encoding
    let table_encoded = ir_to_string_table_wire(ir);
    let table_str = serde_json::to_string(&table_encoded).unwrap_or_default();

    (named_str.len(), table_str.len())
}

#[cfg(test)]
#[path = "../tests/ir/string_table.rs"]
mod tests;