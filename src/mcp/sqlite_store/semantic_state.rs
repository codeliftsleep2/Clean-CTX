//! Atomic persistence and checked loading of canonical IR plus semantic edges.

use super::SqliteStore;
use crate::compression::Fidelity;
use crate::ir::compiler::CompiledIR;
use crate::layers::meta::semantic::SemanticEdge;
use crate::mcp::context_store::ContextStore;
use crate::mcp::state::durable_semantics::DurableSemanticSnapshot;
use rusqlite::{OptionalExtension, params};

#[cfg(test)]
static TEST_FAIL_SEMANTIC_SAVE_FOR: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

#[cfg(test)]
pub(crate) fn fail_next_semantic_save(file_path: &str) {
    *TEST_FAIL_SEMANTIC_SAVE_FOR
        .lock()
        .expect("semantic save failure hook") = Some(file_path.to_string());
}

#[cfg(test)]
pub(super) fn should_fail_semantic_save(file_path: &str) -> bool {
    if let Ok(mut target) = TEST_FAIL_SEMANTIC_SAVE_FOR.lock()
        && target.as_deref() == Some(file_path)
    {
        target.take();
        return true;
    }
    false
}

pub(crate) struct RestoredDurableContext {
    pub ir: CompiledIR,
    pub semantic_edges: Vec<SemanticEdge>,
    pub source_hash: String,
    pub fidelity: Fidelity,
    pub compact_output: Option<String>,
}

impl SqliteStore {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn durable_state_matches(
        &self,
        file_path: &str,
        fidelity: Fidelity,
        compact_output: &str,
        ir_binary: &[u8],
        source_hash: &str,
        version: u64,
        semantic_edges: &[SemanticEdge],
    ) -> Result<bool, Box<dyn std::error::Error>> {
        if !self.checkpoint_matches(file_path, fidelity, compact_output, ir_binary, source_hash)? {
            return Ok(false);
        }
        let context_id = match self.current_context_id(file_path)? {
            Some(context_id) => context_id,
            None => return Ok(false),
        };
        let stored = self
            .conn
            .query_row(
                "SELECT edges_json FROM semantic_edge_snapshots
                 WHERE context_id = ?1 AND semantic_version = ?2",
                params![context_id, version as i64],
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        let expected = serde_json::to_string(&DurableSemanticSnapshot::new(
            file_path.to_string(),
            source_hash.to_string(),
            version,
            semantic_edges,
        ))?;
        Ok(stored.as_deref() == Some(expected.as_str()))
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn save_context_with_semantics(
        &mut self,
        file_path: &str,
        fidelity: Fidelity,
        compact_output: &str,
        ir_binary: &[u8],
        source_hash: &str,
        version: u64,
        semantic_edges: &[SemanticEdge],
        raw_tokens: u64,
        compressed_tokens: u64,
    ) -> Result<String, Box<dyn std::error::Error>> {
        #[cfg(test)]
        if should_fail_semantic_save(file_path) {
            return Err("injected semantic baseline persistence failure".into());
        }
        self.begin_transaction()?;
        let result = (|| {
            let context_id = ContextStore::save_context(
                self,
                file_path,
                fidelity,
                compact_output,
                Some(ir_binary),
                source_hash,
                raw_tokens,
                compressed_tokens,
            )?;
            let snapshot = DurableSemanticSnapshot::new(
                file_path.to_string(),
                source_hash.to_string(),
                version,
                semantic_edges,
            );
            self.write_semantic_snapshot(&context_id, &snapshot)?;
            self.commit()?;
            Ok(context_id)
        })();
        if result.is_err() {
            let _ = self.rollback();
        }
        result
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn append_delta_with_semantics(
        &mut self,
        context_id: &str,
        delta_payload: &[u8],
        edit_type: &str,
        compact_output: &str,
        snapshot: &DurableSemanticSnapshot,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.begin_transaction()?;
        let result = (|| {
            let persisted_file = self
                .context_file_path(context_id)?
                .ok_or_else(|| format!("unknown persisted context: {context_id}"))?;
            if persisted_file != snapshot.file_path {
                return Err(format!(
                    "semantic snapshot file mismatch: expected {persisted_file:?}, found {:?}",
                    snapshot.file_path
                )
                .into());
            }
            ContextStore::append_delta(self, context_id, delta_payload, Some(edit_type))?;
            self.conn.execute(
                "UPDATE contexts SET source_hash = ?1, pretty_text = ?2 WHERE id = ?3",
                params![snapshot.source_hash, compact_output, context_id],
            )?;
            self.write_semantic_snapshot(context_id, snapshot)?;
            self.commit()?;
            Ok(())
        })();
        if result.is_err() {
            let _ = self.rollback();
        }
        result
    }

    pub(crate) fn load_durable_context(
        &self,
        file_path: &str,
        target_sequence: Option<u32>,
    ) -> Result<Option<RestoredDurableContext>, Box<dyn std::error::Error>> {
        let metadata = match ContextStore::load_latest(self, file_path)? {
            Some(metadata) => metadata,
            None => return Ok(None),
        };
        let context_id = self
            .current_context_id(file_path)?
            .ok_or_else(|| format!("missing persisted owner for {file_path}"))?;
        let (ir, _) = self
            .load_context_with_deltas(file_path, target_sequence)?
            .ok_or_else(|| format!("missing canonical IR for {file_path}"))?;
        let snapshot_row = self
            .conn
            .query_row(
                "SELECT file_path, source_hash, semantic_version, edges_json
                 FROM semantic_edge_snapshots
                 WHERE context_id = ?1 AND semantic_version = ?2",
                params![context_id, ir.version as i64],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, String>(3)?,
                    ))
                },
            )
            .optional()?
            .ok_or_else(|| format!("missing semantic-edge snapshot for {file_path}"))?;
        let (row_file, row_hash, row_version, snapshot_json) = snapshot_row;
        let snapshot: DurableSemanticSnapshot = serde_json::from_str(&snapshot_json)
            .map_err(|error| format!("malformed semantic-edge snapshot: {error}"))?;
        if row_file != file_path || snapshot.file_path != file_path {
            return Err("semantic-edge snapshot file identity mismatch".into());
        }
        if row_hash != snapshot.source_hash {
            return Err("canonical and semantic-edge source hashes do not match".into());
        }
        if target_sequence.is_none() && snapshot.source_hash != metadata.source_hash {
            return Err("latest canonical and semantic-edge source hashes do not match".into());
        }
        if row_version < 0 || row_version as u64 != ir.version || snapshot.version != ir.version {
            return Err(format!(
                "canonical and semantic-edge versions do not match: IR v{}, edges v{}",
                ir.version, snapshot.version
            )
            .into());
        }
        let semantic_edges = snapshot.restore_edges()?;
        Ok(Some(RestoredDurableContext {
            ir,
            semantic_edges,
            source_hash: snapshot.source_hash,
            fidelity: metadata.fidelity,
            compact_output: if target_sequence.is_none() {
                self.latest_compact_output(file_path)?
            } else {
                None
            },
        }))
    }

    pub(super) fn write_semantic_snapshot(
        &self,
        context_id: &str,
        snapshot: &DurableSemanticSnapshot,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let edges_json = serde_json::to_string(snapshot)?;
        self.conn.execute(
            "INSERT OR REPLACE INTO semantic_edge_snapshots
             (context_id, file_path, source_hash, semantic_version, edges_json)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                context_id,
                snapshot.file_path,
                snapshot.source_hash,
                snapshot.version as i64,
                edges_json
            ],
        )?;
        Ok(())
    }
}
