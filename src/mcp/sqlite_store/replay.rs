//! Corrected binary-baseline loading and version-discriminated delta replay.

use super::SqliteStore;
use crate::ir::compiler::CompiledIR;
use crate::mcp::persistence_ir::PersistedDelta;
use rusqlite::{OptionalExtension, params};

impl SqliteStore {
    pub(crate) fn latest_compact_output(
        &self,
        file_path: &str,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        Ok(self
            .conn
            .query_row(
                "SELECT pretty_text FROM contexts WHERE file_path = ?1
                 ORDER BY updated_at DESC LIMIT 1",
                params![file_path],
                |row| row.get(0),
            )
            .optional()?
            .flatten())
    }

    pub(crate) fn checkpoint_matches(
        &self,
        file_path: &str,
        fidelity: crate::compression::Fidelity,
        compressed_output: &str,
        binary: &[u8],
        source_hash: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let row = self
            .conn
            .query_row(
                "SELECT fidelity, pretty_text, ir_binary, source_hash FROM contexts
                 WHERE file_path = ?1 ORDER BY updated_at DESC LIMIT 1",
                params![file_path],
                |row| {
                    Ok((
                        row.get::<_, i32>(0)?,
                        row.get::<_, Option<String>>(1)?,
                        row.get::<_, Vec<u8>>(2)?,
                        row.get::<_, String>(3)?,
                    ))
                },
            )
            .optional()?;
        Ok(row.is_some_and(
            |(stored_fidelity, stored_output, stored_binary, stored_hash)| {
                stored_fidelity == fidelity as i32
                    && stored_output.as_deref().unwrap_or_default() == compressed_output
                    && stored_binary == binary
                    && stored_hash == source_hash
            },
        ))
    }

    pub(crate) fn current_context_id(
        &self,
        file_path: &str,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        Ok(self
            .conn
            .query_row(
                "SELECT id FROM contexts WHERE file_path = ?1 ORDER BY updated_at DESC LIMIT 1",
                params![file_path],
                |row| row.get(0),
            )
            .optional()?)
    }

    pub(super) fn context_file_path(
        &self,
        context_id: &str,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        Ok(self
            .conn
            .query_row(
                "SELECT file_path FROM contexts WHERE id = ?1",
                params![context_id],
                |row| row.get(0),
            )
            .optional()?)
    }

    pub(crate) fn baseline_binary(
        &self,
        file_path: &str,
    ) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
        Ok(self
            .conn
            .query_row(
                "SELECT ir_binary FROM contexts WHERE file_path = ?1 ORDER BY updated_at DESC LIMIT 1",
                params![file_path],
                |row| row.get(0),
            )
            .optional()?)
    }

    pub fn load_context_with_deltas(
        &self,
        file_path: &str,
        target_sequence: Option<u32>,
    ) -> Result<Option<(CompiledIR, u32)>, Box<dyn std::error::Error>> {
        let Some(blob) = self.baseline_binary(file_path)? else {
            return Ok(None);
        };
        let ir = crate::ir::binary_wire::decode(&blob)?;
        if ir.file_id != file_path {
            return Err(format!(
                "persisted binary file identity mismatch: expected {file_path:?}, found {:?}",
                ir.file_id
            )
            .into());
        }

        let context_id = self
            .current_context_id(file_path)?
            .ok_or_else(|| format!("missing persisted owner for {file_path}"))?;
        let mut stmt = self.conn.prepare(
            "SELECT delta_payload FROM deltas
             WHERE context_id = ?1 AND edit_sequence <= ?2 ORDER BY edit_sequence",
        )?;
        let rows = stmt.query_map(
            params![context_id, target_sequence.unwrap_or(u32::MAX) as i64],
            |row| row.get::<_, Vec<u8>>(0),
        )?;

        let mut state = crate::ir::replay::ContextState::new();
        state.load_ir(ir, None);
        for row in rows {
            match PersistedDelta::from_json(&row?)? {
                PersistedDelta::Sequence(delta) => state.apply_sequence(delta)?,
                PersistedDelta::Legacy(delta) => state.apply(delta)?,
            };
        }

        let instructions = state.get_ir(file_path).cloned().unwrap_or_default();
        let version = state.file_version(file_path).unwrap_or(1);
        let instructions = instructions
            .iter()
            .enumerate()
            .map(|(position, tuple)| {
                crate::ir::wire::tuple_to_op(tuple).ok_or_else(|| {
                    format!("invalid replayed canonical tuple at position {position}: {tuple:?}")
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Some((
            CompiledIR {
                file_id: file_path.to_string(),
                instructions,
                version,
            },
            version as u32,
        )))
    }
}
