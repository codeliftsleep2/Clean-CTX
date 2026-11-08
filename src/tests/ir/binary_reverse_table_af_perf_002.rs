use serde_json::{Value, json};
use std::collections::HashMap;

use crate::ir::opcodes::CoreOp;
use crate::ir::wire::op_to_tuple;

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

impl Default for StringTable {
    fn default() -> Self {
        Self::new()
    }
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
        super::observe(|w| {
            w.intern_calls += 1;
            w.intern_key_bytes += s.len();
        });
        if let Some(&idx) = self.indices.get(s) {
            return idx;
        }
        super::observe(|w| {
            w.new_strings += 1;
            w.new_string_bytes += s.len();
        });
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
    pub(super) fn index_of(&self, s: &str) -> Option<usize> {
        self.indices.get(s).copied()
    }
    pub(super) fn capacities(&self) -> (usize, usize) {
        (self.strings.capacity(), self.indices.capacity())
    }
}
