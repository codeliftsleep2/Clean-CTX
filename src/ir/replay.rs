// src/ir/replay.rs
//
// Phase D: State Replay — client-side state machine that applies delta ops
// to reconstruct IR state, with version-based catch-up support.
//
// The state machine maintains per-file instruction streams and indexes,
// enabling incremental updates via delta application. Clients can:
//   1. Load a full IR (first compression or catch-up)
//   2. Apply deltas to update state incrementally
//   3. Render human-readable output from current state
//   4. Validate version chains to prevent out-of-order application

use std::collections::HashMap;
use super::delta::IRDelta;
use super::compiler::CompiledIR;
use super::render::ir_to_text;
use super::wire::op_to_tuple;
use super::delta::{primary_key_from_tuple, key_tuple_from_tuple};
use crate::compression::Fidelity;

/// Errors during delta application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeltaError {
    /// The file is not tracked in the current context state
    UnknownFile(String),
    /// Version mismatch: baseline version doesn't match current state
    VersionMismatch { expected: u64, got: u64 },
    /// A symbol referenced in the delta was not found
    SymbolNotFound(String),
    /// Attempted to add a symbol that already exists
    DuplicateSymbol(String),
}

impl std::fmt::Display for DeltaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeltaError::UnknownFile(file) => write!(f, "unknown file: {}", file),
            DeltaError::VersionMismatch { expected, got } => {
                write!(f, "version mismatch: expected {}, got {}", expected, got)
            }
            DeltaError::SymbolNotFound(sym) => write!(f, "symbol not found: {}", sym),
            DeltaError::DuplicateSymbol(sym) => write!(f, "duplicate symbol: {}", sym),
        }
    }
}

impl std::error::Error for DeltaError {}

/// Per-file IR state with indexed instruction stream.
///
/// Maintains an ordered list of instruction tuples and a primary-key index
/// for efficient lookup during delta operations. The index maps primary keys
/// (e.g., "DEF_M:C1:M1") to instruction indices in the `instructions` vec.
#[derive(Debug, Clone)]
pub struct FileState {
    /// Ordered instruction stream (each instruction is a positional tuple)
    pub instructions: Vec<Vec<String>>,
    /// Index: primary_key → instruction index in `instructions`
    pub index: HashMap<String, usize>,
    /// Version when this file was last modified
    pub version: u64,
}

impl FileState {
    /// Create a new empty FileState at the given version.
    pub fn new(version: u64) -> Self {
        Self {
            instructions: Vec::new(),
            index: HashMap::new(),
            version,
        }
    }

    /// Build a FileState from a CompiledIR.
    ///
    /// Converts each CoreOp to its positional tuple representation and
    /// builds the primary-key index for efficient delta operations.
    pub fn from_compiled(ir: &CompiledIR) -> Self {
        let mut state = Self::new(ir.version);
        for op in &ir.instructions {
            let tuple = op_to_tuple(op);
            let key = primary_key_from_tuple(&tuple);
            state.index.insert(key, state.instructions.len());
            state.instructions.push(tuple);
        }
        state
    }

    /// Remove an instruction by its key tuple.
    ///
    /// The key tuple is a subset of the full instruction that uniquely
    /// identifies it (e.g., `["DEF_M", "C1", "M1"]`). Returns true if
    /// the instruction was found and removed, false otherwise.
    ///
    /// Uses `swap_remove` (O(1)) and updates the swapped element's index.
    /// Instruction order is NOT preserved — the last instruction moves to
    /// the removed position. The IR stream is positional and a re-render
    /// at any fidelity does not depend on order.
    pub fn remove_by_key(&mut self, key_tuple: &[String]) -> bool {
        let key = primary_key_from_tuple(key_tuple);
        if let Some(idx) = self.index.remove(&key) {
            // Use swap_remove (O(1)) — removes the instruction by swapping
            // it with the last element, then popping.
            self.instructions.swap_remove(idx);
            
            // If the removed element was not the last one, update the index
            // for the element that was swapped into position `idx`.
            if idx < self.instructions.len() {
                let swapped = &self.instructions[idx];
                let swapped_key = primary_key_from_tuple(swapped);
                self.index.insert(swapped_key, idx);
            }
            
            true
        } else {
            false
        }
    }

    /// Replace an existing instruction identified by its key tuple.
    ///
    /// Returns true if the instruction was found and replaced, false if
    /// no match was found. If the replacement changes the primary key,
    /// the index is updated accordingly.
    pub fn replace_by_key(&mut self, key_tuple: &[String], replacement: &[String]) -> bool {
        let key = primary_key_from_tuple(key_tuple);
        if let Some(&idx) = self.index.get(&key) {
            self.instructions[idx] = replacement.to_vec();
            // Update index if the key changed
            let new_key = primary_key_from_tuple(replacement);
            if key != new_key {
                self.index.remove(&key);
                self.index.insert(new_key, idx);
            }
            true
        } else {
            false
        }
    }

    /// Append a new instruction to the end of the instruction stream.
    ///
    /// Automatically computes the primary key and updates the index.
    ///
    /// # Errors
    ///
    /// Returns `Err(DeltaError::DuplicateSymbol)` if an instruction with
    /// the same primary key already exists in this file state (F-23).
    pub fn append(&mut self, instruction: Vec<String>) -> Result<(), DeltaError> {
        let key = primary_key_from_tuple(&instruction);
        if self.index.contains_key(&key) {
            return Err(DeltaError::DuplicateSymbol(key));
        }
        self.index.insert(key, self.instructions.len());
        self.instructions.push(instruction);
        Ok(())
    }

    /// Check if this state contains an instruction with the given key tuple.
    pub fn contains_key(&self, key_tuple: &[String]) -> bool {
        let key = primary_key_from_tuple(key_tuple);
        self.index.contains_key(&key)
    }
}

/// Top-level context state — tracks all files and their IR states.
///
/// This is the main state machine that clients use to:
/// - Load initial IR state for new files
/// - Apply incremental deltas to update existing state
/// - Render human-readable output at any point
/// - Track the global monotonic version
#[derive(Debug, Clone)]
pub struct ContextState {
    /// Per-file IR state
    files: HashMap<String, FileState>,
    /// Current global version (monotonic, across all files)
    version: u64,
}

impl ContextState {
    /// Create a new empty ContextState.
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
            version: 0,
        }
    }

    /// Load a full IR into state.
    ///
    /// This is used for the first compression of a file, or for catch-up
    /// after the client falls behind. If the file already exists in state,
    /// it is overwritten with the new IR.
    ///
    /// The global version is updated to max(current, ir.version).
    pub fn load_ir(&mut self, ir: CompiledIR) {
        let file_id = ir.file_id.clone();
        let version = ir.version;
        self.files.insert(file_id, FileState::from_compiled(&ir));
        self.version = self.version.max(version);
    }

    /// Apply a delta to update state for a specific file.
    ///
    /// The apply order is: deletions → modifications → additions.
    /// This ensures that:
    ///   1. Removed instructions are gone before modifications check keys
    ///   2. Existing instructions are updated before new ones are added
    ///   3. New instructions don't collide with old indices
    ///
    /// # Errors
    ///
    /// Returns `DeltaError::UnknownFile` if the file is not tracked.
    /// Returns `DeltaError::VersionMismatch` if the delta's `from`
    /// doesn't match the file's current version.
    /// Returns `DeltaError::SymbolNotFound` if a deletion or modification
    /// references a key that doesn't exist.
    ///
    /// # Returns
    ///
    /// `Ok(to_version)` on success, where `to_version` is the delta's target version.
    pub fn apply(&mut self, delta: IRDelta) -> Result<u64, DeltaError> {
        let file = self.files.get_mut(&delta.file)
            .ok_or_else(|| DeltaError::UnknownFile(delta.file.clone()))?;

        // Validate version chain
        if file.version != delta.from {
            return Err(DeltaError::VersionMismatch {
                expected: file.version,
                got: delta.from,
            });
        }

        // Phase 1: Deletions (process first so modifications don't find stale keys)
        for del in &delta.ops.dels {
            let key_tuple = key_tuple_from_tuple(del);
            if !file.remove_by_key(&key_tuple) {
                // Check if the key_tuple was derived from a full instruction tuple
                // Primary key only uses opcode + id(s), which is what key_tuple gives us
                let key = primary_key_from_tuple(&key_tuple);
                return Err(DeltaError::SymbolNotFound(key));
            }
        }

        // Phase 2: Modifications
        for mod_op in &delta.ops.mods {
            if !file.replace_by_key(&mod_op.key, &mod_op.replace) {
                let key = primary_key_from_tuple(&mod_op.key);
                return Err(DeltaError::SymbolNotFound(key));
            }
        }

        // Phase 3: Additions
        for add in &delta.ops.adds {
            file.append(add.clone())?;
        }

        // Update version tracking
        file.version = delta.to;
        self.version = self.version.max(delta.to);

        Ok(delta.to)
    }

    /// Render human-readable output from current state for a given file.
    ///
    /// Returns `None` if the file is not tracked in state.
    pub fn render_pretty(&self, file_id: &str, fidelity: Fidelity) -> Option<String> {
        let file = self.files.get(file_id)?;
        Some(ir_to_text(&file.instructions, fidelity))
    }

    /// Get the raw instruction tuples for a file.
    ///
    /// Returns `None` if the file is not tracked.
    pub fn get_ir(&self, file_id: &str) -> Option<&Vec<Vec<String>>> {
        self.files.get(file_id).map(|f| &f.instructions)
    }

    /// Get the current global version.
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Check if a file is tracked in state.
    pub fn has_file(&self, file_id: &str) -> bool {
        self.files.contains_key(file_id)
    }

    /// Get the version of a specific file.
    ///
    /// Returns `None` if the file is not tracked.
    pub fn file_version(&self, file_id: &str) -> Option<u64> {
        self.files.get(file_id).map(|f| f.version)
    }

    /// Get the number of instructions for a file.
    ///
    /// Returns `None` if the file is not tracked.
    pub fn instruction_count(&self, file_id: &str) -> Option<usize> {
        self.files.get(file_id).map(|f| f.instructions.len())
    }

    /// Remove a file from state entirely.
    ///
    /// Returns true if the file was tracked and removed.
    pub fn remove_file(&mut self, file_id: &str) -> bool {
        self.files.remove(file_id).is_some()
    }

    /// List all tracked file IDs.
    pub fn file_ids(&self) -> Vec<String> {
        self.files.keys().cloned().collect()
    }
}

impl Default for ContextState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "../tests/ir/replay.rs"]
mod tests;