//! Transactional replay for corrected positional sequence deltas.

use super::{ContextState, DeltaError, OccurrenceKey};
use crate::ir::delta::{SEQUENCE_DELTA_VERSION, SequenceDelta, SequenceEdit, occurrence_key_at};

impl ContextState {
    /// Apply the corrected positional protocol transactionally.
    pub fn apply_sequence(&mut self, delta: SequenceDelta) -> Result<u64, DeltaError> {
        if delta.delta_version != SEQUENCE_DELTA_VERSION {
            return Err(DeltaError::UnsupportedDeltaVersion(delta.delta_version));
        }
        let file = self
            .files
            .get_mut(&delta.file)
            .ok_or_else(|| DeltaError::UnknownFile(delta.file.clone()))?;
        if file.version != delta.from {
            return Err(DeltaError::VersionMismatch {
                expected: file.version,
                got: delta.from,
            });
        }
        if delta.to <= delta.from {
            return Err(DeltaError::NonMonotonicVersion {
                from: delta.from,
                to: delta.to,
            });
        }

        let mut instructions = file.instructions.clone();
        for edit in &delta.edits {
            match edit {
                SequenceEdit::Insert {
                    at,
                    value,
                    instruction,
                } => {
                    if *at > instructions.len() {
                        return Err(DeltaError::InvalidSequenceInstruction { position: *at });
                    }
                    instructions.insert(*at, instruction.clone());
                    ensure_occurrence(&instructions, *at, value)?;
                }
                SequenceEdit::Remove {
                    at,
                    target,
                    expected,
                } => {
                    ensure_expected(&instructions, *at, expected)?;
                    ensure_occurrence(&instructions, *at, target)?;
                    instructions.remove(*at);
                }
                SequenceEdit::Replace {
                    at,
                    target,
                    expected,
                    value,
                    replacement,
                } => {
                    ensure_expected(&instructions, *at, expected)?;
                    ensure_occurrence(&instructions, *at, target)?;
                    instructions[*at] = replacement.clone();
                    ensure_occurrence(&instructions, *at, value)?;
                }
            }
        }

        file.instructions = instructions;
        file.version = delta.to;
        file.rebuild_indexes();
        self.version = self.version.max(delta.to);
        Ok(delta.to)
    }
}

fn ensure_expected(
    instructions: &[Vec<String>],
    position: usize,
    expected: &[String],
) -> Result<(), DeltaError> {
    let actual = instructions.get(position);
    if actual.is_some_and(|tuple| tuple == expected) {
        Ok(())
    } else {
        Err(DeltaError::SequenceConflict {
            position,
            expected: expected.to_vec(),
            actual: actual.cloned(),
        })
    }
}

fn ensure_occurrence(
    instructions: &[Vec<String>],
    position: usize,
    expected: &OccurrenceKey,
) -> Result<(), DeltaError> {
    let actual = occurrence_key_at(instructions, position);
    if actual.as_ref() == Some(expected) {
        Ok(())
    } else {
        Err(DeltaError::OccurrenceConflict {
            position,
            expected: expected.clone(),
            actual,
        })
    }
}
