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

use super::compiler::CompiledIR;
use super::delta::{
    DeltaIdentity, IRDelta, OccurrenceKey, key_tuple_from_tuple, primary_key_from_tuple,
};
use super::render::ir_to_text;
use super::wire::op_to_tuple;
use crate::compression::Fidelity;
use std::collections::HashMap;

mod sequence;

/// Errors during delta application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeltaError {
    /// The file is not tracked in the current context state
    UnknownFile(String),
    /// Version mismatch: baseline version doesn't match current state
    VersionMismatch {
        expected: u64,
        got: u64,
    },
    /// A symbol referenced in the delta was not found
    SymbolNotFound(String),
    /// Attempted to add a symbol that already exists
    DuplicateSymbol(String),
    /// Delta to version is not greater than from version (non-monotonic)
    NonMonotonicVersion {
        from: u64,
        to: u64,
    },
    UnsupportedDeltaVersion(u8),
    InvalidSequenceInstruction {
        position: usize,
    },
    SequenceConflict {
        position: usize,
        expected: Vec<String>,
        actual: Option<Vec<String>>,
    },
    OccurrenceConflict {
        position: usize,
        expected: OccurrenceKey,
        actual: Option<OccurrenceKey>,
    },
    AmbiguousLegacyTarget(String),
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
            DeltaError::NonMonotonicVersion { from, to } => {
                write!(
                    f,
                    "non-monotonic version: delta from {} to {} must be strictly increasing",
                    from, to
                )
            }
            DeltaError::UnsupportedDeltaVersion(version) => {
                write!(f, "unsupported delta protocol version: {version}")
            }
            DeltaError::InvalidSequenceInstruction { position } => {
                write!(f, "invalid sequence instruction at position {position}")
            }
            DeltaError::SequenceConflict {
                position,
                expected,
                actual,
            } => write!(
                f,
                "sequence conflict at position {position}: expected {expected:?}, found {actual:?}"
            ),
            DeltaError::OccurrenceConflict {
                position,
                expected,
                actual,
            } => write!(
                f,
                "occurrence conflict at position {position}: expected {expected:?}, found {actual:?}"
            ),
            DeltaError::AmbiguousLegacyTarget(key) => {
                write!(f, "legacy delta target is ambiguous: {key}")
            }
        }
    }
}

impl std::error::Error for DeltaError {}

/// Per-file IR state with indexed instruction stream.
///
/// Maintains the ordered instruction tuples, a legacy coarse-key index, and
/// the occurrence-aware semantic index used by corrected sequence replay.
#[derive(Debug, Clone)]
pub struct FileState {
    /// Ordered instruction stream (each instruction is a positional tuple)
    pub instructions: Vec<Vec<String>>,
    /// Compatibility index for legacy `+ / ~ / -` deltas.
    pub index: HashMap<String, usize>,
    /// Corrected occurrence-aware index used by sequence replay.
    occurrence_index: HashMap<DeltaIdentity, Vec<usize>>,
    /// Version when this file was last modified
    pub version: u64,
}

impl FileState {
    /// Create a new empty FileState at the given version.
    pub fn new(version: u64) -> Self {
        Self {
            instructions: Vec::new(),
            index: HashMap::new(),
            occurrence_index: HashMap::new(),
            version,
        }
    }

    /// Build a FileState from a CompiledIR.
    ///
    /// Converts each CoreOp to its positional tuple representation and
    /// builds the primary-key index for efficient delta operations.
    pub fn from_compiled(ir: &CompiledIR) -> Self {
        let mut state = Self::new(ir.version);
        state.instructions = ir.instructions.iter().map(op_to_tuple).collect();
        state.rebuild_indexes();
        state
    }

    fn rebuild_indexes(&mut self) {
        self.index.clear();
        self.occurrence_index.clear();
        for (position, tuple) in self.instructions.iter().enumerate() {
            self.index.insert(primary_key_from_tuple(tuple), position);
            if let Some(identity) = DeltaIdentity::from_tuple(tuple) {
                self.occurrence_index
                    .entry(identity)
                    .or_default()
                    .push(position);
            }
        }
    }

    fn legacy_target_count(&self, key_tuple: &[String]) -> usize {
        let key = primary_key_from_tuple(key_tuple);
        self.instructions
            .iter()
            .filter(|tuple| primary_key_from_tuple(tuple) == key)
            .count()
    }

    /// Remove an instruction by its key tuple.
    ///
    /// The key tuple is a subset of the full instruction that uniquely
    /// identifies it (e.g., `["DEF_M", "C1", "M1"]`). Returns true if
    /// the instruction was found and removed, false otherwise.
    ///
    /// Legacy compatibility helper. It retains the historical `swap_remove`
    /// behavior and therefore does not preserve instruction order. Corrected
    /// sequence replay never calls this helper; it applies explicit positional
    /// edits through `apply_sequence`.
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

            self.rebuild_indexes();

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
            self.rebuild_indexes();
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
        let repeatable = DeltaIdentity::from_tuple(&instruction)
            .is_some_and(|identity| identity.is_repeatable());
        if self.index.contains_key(&key) && !repeatable {
            return Err(DeltaError::DuplicateSymbol(key));
        }
        self.index.insert(key, self.instructions.len());
        self.instructions.push(instruction);
        self.rebuild_indexes();
        Ok(())
    }

    /// Check if this state contains an instruction with the given key tuple.
    pub fn contains_key(&self, key_tuple: &[String]) -> bool {
        let key = primary_key_from_tuple(key_tuple);
        self.index.contains_key(&key)
    }

    fn remove_legacy_tuple(&mut self, expected: &[String]) -> Result<(), DeltaError> {
        let key_tuple = key_tuple_from_tuple(expected);
        let key_only = key_tuple == expected;
        let coarse_key = primary_key_from_tuple(&key_tuple);
        let matches = self
            .instructions
            .iter()
            .enumerate()
            .filter_map(|(position, tuple)| {
                let matches = if key_only {
                    primary_key_from_tuple(tuple) == coarse_key
                } else {
                    tuple == expected
                };
                matches.then_some(position)
            })
            .collect::<Vec<_>>();
        match matches.as_slice() {
            [position] => {
                self.instructions.remove(*position);
                self.rebuild_indexes();
                Ok(())
            }
            [] => Err(DeltaError::SymbolNotFound(primary_key_from_tuple(expected))),
            _ => Err(DeltaError::AmbiguousLegacyTarget(primary_key_from_tuple(
                expected,
            ))),
        }
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
    /// A-08: Source hashes for detecting unchanged files and avoiding
    /// unnecessary recompilation. Key is path alias, value is SHA-256
    /// hash of the source content at the time of last compilation.
    source_hashes: HashMap<String, String>,
}

impl ContextState {
    /// Create a new empty ContextState.
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
            version: 0,
            source_hashes: HashMap::new(),
        }
    }

    /// Load a full IR into state.
    ///
    /// This is used for the first compression of a file, or for catch-up
    /// after the client falls behind. If the file already exists in state,
    /// it is overwritten with the new IR.
    ///
    /// The global version is updated to max(current, ir.version).
    ///
    /// A-08: Accepts an optional source hash to track whether the file
    /// has changed since last compilation.
    pub fn load_ir(&mut self, ir: CompiledIR, source_hash: Option<String>) {
        let file_id = ir.file_id.clone();
        let version = ir.version;
        self.files.insert(file_id, FileState::from_compiled(&ir));
        self.version = self.version.max(version);

        // A-08: Store source hash if provided
        if let Some(hash) = source_hash {
            self.source_hashes.insert(ir.file_id, hash);
        }
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
        let file = self
            .files
            .get_mut(&delta.file)
            .ok_or_else(|| DeltaError::UnknownFile(delta.file.clone()))?;

        // Validate version chain
        if file.version != delta.from {
            return Err(DeltaError::VersionMismatch {
                expected: file.version,
                got: delta.from,
            });
        }

        // NF-09: Validate monotonic version — to must be strictly greater than from.
        // This prevents delta replay from rolling backwards or staying the same.
        if delta.to <= delta.from {
            return Err(DeltaError::NonMonotonicVersion {
                from: delta.from,
                to: delta.to,
            });
        }

        let mut candidate = file.clone();

        // Phase 1: Deletions (process first so modifications don't find stale keys)
        for del in &delta.ops.dels {
            candidate.remove_legacy_tuple(del)?;
        }

        // Phase 2: Modifications — supports both full replacement and field-patch formats
        for mod_op in &delta.ops.mods {
            if candidate.legacy_target_count(&mod_op.key) > 1 {
                return Err(DeltaError::AmbiguousLegacyTarget(primary_key_from_tuple(
                    &mod_op.key,
                )));
            }
            if let Some(replacement) = &mod_op.replace {
                // Full replacement format
                if !candidate.replace_by_key(&mod_op.key, replacement) {
                    let key = primary_key_from_tuple(&mod_op.key);
                    return Err(DeltaError::SymbolNotFound(key));
                }
            } else if let Some(patches) = &mod_op.patches {
                // Field-patch format (Idea #3) — apply patches to the existing instruction
                let key = primary_key_from_tuple(&mod_op.key);
                let idx = *candidate
                    .index
                    .get(&key)
                    .ok_or_else(|| DeltaError::SymbolNotFound(key.clone()))?;
                let instruction = &mut candidate.instructions[idx];
                for patch in patches {
                    if patch.field_index < instruction.len() {
                        instruction[patch.field_index] = patch.new_value.clone();
                    }
                }
                // Re-index if the key changed (e.g., a rename patch)
                let new_key = primary_key_from_tuple(instruction);
                if key != new_key {
                    candidate.index.remove(&key);
                    candidate.index.insert(new_key, idx);
                }
            }
        }

        // Phase 3: Additions
        for add in &delta.ops.adds {
            candidate.append(add.clone())?;
        }

        // Update version tracking
        candidate.version = delta.to;
        candidate.rebuild_indexes();
        *file = candidate;
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

    /// A-08: Check if the source for a file has changed since last compilation.
    ///
    /// Returns `true` if the file is not tracked (no baseline to compare against),
    /// or if the provided source hash matches the stored hash (file unchanged).
    /// Returns `false` if the file is tracked but the hash doesn't match (file changed).
    pub fn is_source_unchanged(&self, file_id: &str, source_hash: &str) -> bool {
        match self.source_hashes.get(file_id) {
            None => true, // No baseline hash - treat as unchanged (first compile)
            Some(stored_hash) => stored_hash == source_hash, // Compare hashes
        }
    }

    /// A-08: Get the stored source hash for a file.
    ///
    /// Returns `None` if the file is not tracked or no hash was stored.
    pub fn get_source_hash(&self, file_id: &str) -> Option<&String> {
        self.source_hashes.get(file_id)
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
