//! Crash-recoverable, byte-exact source edit intents.

use super::SqliteStore;
use crate::compression::Fidelity;
use crate::layers::meta::semantic::SemanticEdge;
use crate::mcp::compatibility::identity::{
    CompatibilityIdentities, PersistedCompatibilityIdentities,
};
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum EditRecovery {
    None,
    PriorRestored,
    TargetCommitted,
}

struct StoredIntent {
    intent: EditIntent,
    edges_json: String,
    compatibility: PersistedCompatibilityIdentities,
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
    #[cfg(test)]
    pub(crate) fn establish_edit_intent(
        &mut self,
        intent: &EditIntent,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.insert_edit_intent(intent, None)
    }

    pub(crate) fn establish_compatible_edit_intent(
        &mut self,
        intent: &EditIntent,
        identities: &CompatibilityIdentities,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.insert_edit_intent(intent, Some(identities))
    }

    fn insert_edit_intent(
        &mut self,
        intent: &EditIntent,
        identities: Option<&CompatibilityIdentities>,
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
              target_edges_json, fidelity, stage_path, canonical_config_identity,
              canonical_producer_identity, semantic_config_identity,
              semantic_producer_identity)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12,
                     ?13, ?14, ?15, ?16)",
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
                identities
                    .map(|value| serde_json::to_string(&value.canonical_config))
                    .transpose()?,
                identities
                    .map(|value| serde_json::to_string(&value.canonical_producers))
                    .transpose()?,
                identities
                    .map(|value| serde_json::to_string(&value.semantic_config))
                    .transpose()?,
                identities
                    .map(|value| serde_json::to_string(&value.semantic_producers))
                    .transpose()?,
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
        if let Some(identities) = complete_identities(&stored.compatibility)? {
            self.save_context_with_compatibility(
                file_path,
                stored.intent.fidelity,
                "",
                &stored.intent.target_ir,
                &stored.intent.target_hash,
                stored.intent.target_version,
                &edges,
                0,
                0,
                &identities,
            )?;
        } else {
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
        }
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
                        target_edges_json, fidelity, stage_path,
                        canonical_config_identity, canonical_producer_identity,
                        semantic_config_identity, semantic_producer_identity
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
                        compatibility: PersistedCompatibilityIdentities {
                            canonical_config: decode_identity(row.get(11)?)?,
                            canonical_producers: decode_identity(row.get(12)?)?,
                            semantic_config: decode_identity(row.get(13)?)?,
                            semantic_producers: decode_identity(row.get(14)?)?,
                        },
                    })
                },
            )
            .optional()
            .map_err(Into::into)
    }
}

fn decode_identity<T: serde::de::DeserializeOwned>(
    value: Option<String>,
) -> Result<Option<T>, rusqlite::Error> {
    value
        .map(|encoded| {
            serde_json::from_str(&encoded).map_err(|error| {
                rusqlite::Error::FromSqlConversionFailure(
                    encoded.len(),
                    rusqlite::types::Type::Text,
                    Box::new(error),
                )
            })
        })
        .transpose()
}

fn complete_identities(
    value: &PersistedCompatibilityIdentities,
) -> Result<Option<CompatibilityIdentities>, Box<dyn std::error::Error>> {
    let present = [
        value.canonical_config.is_some(),
        value.canonical_producers.is_some(),
        value.semantic_config.is_some(),
        value.semantic_producers.is_some(),
    ]
    .into_iter()
    .filter(|present| *present)
    .count();
    match present {
        0 => Ok(None),
        4 => Ok(Some(CompatibilityIdentities {
            canonical_config: value.canonical_config.clone().expect("counted identity"),
            canonical_producers: value.canonical_producers.clone().expect("counted identity"),
            semantic_config: value.semantic_config.clone().expect("counted identity"),
            semantic_producers: value.semantic_producers.clone().expect("counted identity"),
        })),
        _ => Err("edit intent has a partial compatibility envelope".into()),
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

/// Remove an internally staged recovery artifact only after durable context
/// deletion commits. Never permit the source path itself to become the target.
pub(crate) fn remove_deleted_recovery_stage(file_path: &str, stage_path: &str) {
    if !stage_path.is_empty()
        && crate::dictionary::path::canonical_identity_key(stage_path)
            != crate::dictionary::path::canonical_identity_key(file_path)
    {
        remove_stage(stage_path);
    }
}
