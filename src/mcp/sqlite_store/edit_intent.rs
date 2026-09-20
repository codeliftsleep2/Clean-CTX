//! Crash-recoverable, byte-exact source edit intents.

use super::SqliteStore;
use crate::compression::Fidelity;
use crate::layers::meta::semantic::SemanticEdge;
use crate::mcp::state::durable_semantics::DurableSemanticSnapshot;
use rusqlite::{Connection, OptionalExtension, params};
use sha2::{Digest, Sha256};

pub(crate) struct EditIntent {
    pub transition_id: String,
    pub file_path: String,
    pub prior_hash: String,
    pub target_hash: String,
    pub prior_version: u64,
    pub target_version: u64,
    pub prior_source: Vec<u8>,
    pub target_source: Vec<u8>,
    pub target_ir: Vec<u8>,
    pub target_edges: Vec<SemanticEdge>,
    pub fidelity: Fidelity,
    pub stage_path: String,
}

pub(crate) enum EditRecovery {
    None,
    PriorRestored,
    TargetCommitted,
}

struct StoredIntent {
    intent: EditIntent,
    edges_json: String,
}

pub(super) fn migrate(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS edit_intents (
            file_path TEXT PRIMARY KEY,
            transition_id TEXT NOT NULL UNIQUE,
            prior_hash TEXT NOT NULL,
            target_hash TEXT NOT NULL,
            prior_version INTEGER NOT NULL,
            target_version INTEGER NOT NULL,
            prior_source BLOB NOT NULL,
            target_source BLOB NOT NULL,
            target_ir BLOB NOT NULL,
            target_edges_json TEXT NOT NULL,
            fidelity INTEGER NOT NULL,
            stage_path TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        INSERT INTO _schema_version (version) VALUES (5);",
    )?;
    Ok(())
}

impl SqliteStore {
    pub(crate) fn establish_edit_intent(
        &mut self,
        intent: &EditIntent,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let snapshot = DurableSemanticSnapshot::new(
            intent.file_path.clone(),
            intent.target_hash.clone(),
            intent.target_version,
            &intent.target_edges,
        );
        self.conn.execute(
            "INSERT INTO edit_intents
             (file_path, transition_id, prior_hash, target_hash, prior_version,
              target_version, prior_source, target_source, target_ir,
              target_edges_json, fidelity, stage_path)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                intent.file_path,
                intent.transition_id,
                intent.prior_hash,
                intent.target_hash,
                intent.prior_version as i64,
                intent.target_version as i64,
                intent.prior_source,
                intent.target_source,
                intent.target_ir,
                serde_json::to_string(&snapshot)?,
                intent.fidelity as i32,
                intent.stage_path,
            ],
        )?;
        Ok(())
    }

    pub(crate) fn clear_edit_intent(
        &mut self,
        file_path: &str,
        transition_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let deleted = self.conn.execute(
            "DELETE FROM edit_intents WHERE file_path = ?1 AND transition_id = ?2",
            params![file_path, transition_id],
        )?;
        if deleted != 1 {
            return Err("edit intent ownership changed unexpectedly".into());
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn has_edit_intent(
        &self,
        file_path: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        Ok(self
            .conn
            .query_row(
                "SELECT 1 FROM edit_intents WHERE file_path = ?1",
                params![file_path],
                |_| Ok(()),
            )
            .optional()?
            .is_some())
    }

    pub(crate) fn recover_edit_intent(
        &mut self,
        file_path: &str,
    ) -> Result<EditRecovery, Box<dyn std::error::Error>> {
        let Some(stored) = self.load_edit_intent(file_path)? else {
            return Ok(EditRecovery::None);
        };
        let actual = std::fs::read(file_path)?;
        let actual_hash = hash(&actual);
        if actual_hash == stored.intent.prior_hash && actual == stored.intent.prior_source {
            self.clear_edit_intent(file_path, &stored.intent.transition_id)?;
            remove_stage(&stored.intent.stage_path);
            return Ok(EditRecovery::PriorRestored);
        }
        if actual_hash != stored.intent.target_hash || actual != stored.intent.target_source {
            return Err(format!(
                "irreconcilable edit recovery mismatch for {file_path}: prior={}, target={}, actual={actual_hash}",
                stored.intent.prior_hash, stored.intent.target_hash
            )
            .into());
        }

        let target_ir = crate::ir::binary_wire::decode(&stored.intent.target_ir)?;
        if target_ir.file_id != file_path || target_ir.version != stored.intent.target_version {
            return Err("edit intent target IR identity/version mismatch".into());
        }
        let snapshot: DurableSemanticSnapshot = serde_json::from_str(&stored.edges_json)?;
        if snapshot.file_path != file_path
            || snapshot.source_hash != stored.intent.target_hash
            || snapshot.version != stored.intent.target_version
        {
            return Err("edit intent semantic snapshot identity mismatch".into());
        }
        let edges = snapshot.restore_edges()?;
        self.save_context_with_semantics(
            file_path,
            stored.intent.fidelity,
            "",
            &stored.intent.target_ir,
            &stored.intent.target_hash,
            stored.intent.target_version,
            &edges,
            0,
            0,
        )?;
        self.clear_edit_intent(file_path, &stored.intent.transition_id)?;
        remove_stage(&stored.intent.stage_path);
        Ok(EditRecovery::TargetCommitted)
    }

    fn load_edit_intent(
        &self,
        file_path: &str,
    ) -> Result<Option<StoredIntent>, Box<dyn std::error::Error>> {
        self.conn
            .query_row(
                "SELECT transition_id, prior_hash, target_hash, prior_version,
                        target_version, prior_source, target_source, target_ir,
                        target_edges_json, fidelity, stage_path
                 FROM edit_intents WHERE file_path = ?1",
                params![file_path],
                |row| {
                    let fidelity = fidelity_from_i32(row.get(9)?)?;
                    Ok(StoredIntent {
                        intent: EditIntent {
                            file_path: file_path.to_string(),
                            transition_id: row.get(0)?,
                            prior_hash: row.get(1)?,
                            target_hash: row.get(2)?,
                            prior_version: row.get::<_, i64>(3)? as u64,
                            target_version: row.get::<_, i64>(4)? as u64,
                            prior_source: row.get(5)?,
                            target_source: row.get(6)?,
                            target_ir: row.get(7)?,
                            target_edges: Vec::new(),
                            fidelity,
                            stage_path: row.get(10)?,
                        },
                        edges_json: row.get(8)?,
                    })
                },
            )
            .optional()
            .map_err(Into::into)
    }
}

fn fidelity_from_i32(value: i32) -> Result<Fidelity, rusqlite::Error> {
    match value {
        0 => Ok(Fidelity::Low),
        1 => Ok(Fidelity::Medium),
        2 => Ok(Fidelity::High),
        3 => Ok(Fidelity::Edit),
        4 => Ok(Fidelity::Verbatim),
        _ => Err(rusqlite::Error::InvalidQuery),
    }
}

fn hash(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn remove_stage(path: &str) {
    if !path.is_empty() {
        let _ = std::fs::remove_file(path);
    }
}
