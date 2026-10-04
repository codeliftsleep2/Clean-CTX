//! Atomic persistence and checked loading of canonical IR plus semantic edges.

use super::SqliteStore;
use crate::compression::Fidelity;
use crate::layers::meta::semantic::SemanticEdge;
use crate::mcp::compatibility::identity::{CompatibilityIdentities, PersistedCompatibilityIdentities};
use crate::mcp::compatibility::validator::{CompatibilityFailure, UntrustedDurableContext};
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

impl SqliteStore {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn save_context_with_compatibility(
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
        identities: &CompatibilityIdentities,
    ) -> Result<String, Box<dyn std::error::Error>> {
        self.save_context_transaction(
            file_path,
            fidelity,
            compact_output,
            ir_binary,
            source_hash,
            version,
            semantic_edges,
            raw_tokens,
            compressed_tokens,
            Some(identities),
        )
    }

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
        identities: &CompatibilityIdentities,
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
                "SELECT s.edges_json, c.canonical_config_identity,
                        c.canonical_producer_identity, s.semantic_config_identity,
                        s.semantic_producer_identity
                 FROM semantic_edge_snapshots AS s
                 JOIN contexts AS c ON c.id = s.context_id
                 WHERE s.context_id = ?1 AND s.semantic_version = ?2",
                params![context_id, version as i64],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, Option<String>>(1)?,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, Option<String>>(4)?,
                    ))
                },
            )
            .optional()?;
        let expected = serde_json::to_string(&DurableSemanticSnapshot::new(
            file_path.to_string(),
            source_hash.to_string(),
            version,
            semantic_edges,
        ))?;
        let expected_identities = (
            Some(serde_json::to_string(&identities.canonical_config)?),
            Some(serde_json::to_string(&identities.canonical_producers)?),
            Some(serde_json::to_string(&identities.semantic_config)?),
            Some(serde_json::to_string(&identities.semantic_producers)?),
        );
        Ok(stored.is_some_and(|stored| {
            stored.0 == expected
                && (stored.1, stored.2, stored.3, stored.4) == expected_identities
        }))
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
        self.save_context_transaction(
            file_path,
            fidelity,
            compact_output,
            ir_binary,
            source_hash,
            version,
            semantic_edges,
            raw_tokens,
            compressed_tokens,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn save_context_transaction(
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
        identities: Option<&CompatibilityIdentities>,
    ) -> Result<String, Box<dyn std::error::Error>> {
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
            if let Some(identities) = identities {
                self.conn.execute(
                    "UPDATE contexts
                     SET canonical_config_identity = ?1,
                         canonical_producer_identity = ?2
                     WHERE id = ?3",
                    params![
                        serde_json::to_string(&identities.canonical_config)?,
                        serde_json::to_string(&identities.canonical_producers)?,
                        context_id,
                    ],
                )?;
            }
            #[cfg(test)]
            if should_fail_semantic_save(file_path) {
                return Err("injected semantic baseline persistence failure".into());
            }
            self.write_semantic_snapshot(&context_id, &snapshot, identities)?;
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
        identities: &CompatibilityIdentities,
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
            self.write_semantic_snapshot(context_id, snapshot, Some(identities))?;
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
    ) -> Result<Option<UntrustedDurableContext>, CompatibilityFailure> {
        let metadata = match ContextStore::load_latest(self, file_path).map_err(physical_error)? {
            Some(metadata) => metadata,
            None => return Ok(None),
        };
        let context_id = self
            .current_context_id(file_path)
            .map_err(physical_error)?
            .ok_or_else(|| structural_error(format!("missing persisted owner for {file_path}")))?;
        let (ir, _) = self
            .load_context_with_deltas(file_path, target_sequence)
            .map_err(physical_error)?
            .ok_or_else(|| structural_error(format!("missing canonical IR for {file_path}")))?;
        let snapshot_row = self
            .conn
            .query_row(
            "SELECT s.file_path, s.source_hash, s.semantic_version, s.edges_json,
                    c.canonical_config_identity, c.canonical_producer_identity,
                    s.semantic_config_identity, s.semantic_producer_identity
                 FROM semantic_edge_snapshots
                 AS s JOIN contexts AS c ON c.id = s.context_id
                 WHERE s.context_id = ?1 AND s.semantic_version = ?2",
                params![context_id, ir.version as i64],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, Option<String>>(4)?,
                        row.get::<_, Option<String>>(5)?,
                        row.get::<_, Option<String>>(6)?,
                        row.get::<_, Option<String>>(7)?,
                    ))
                },
            )
            .optional()
            .map_err(physical_error)?
            .ok_or_else(|| {
                structural_error(format!("missing semantic-edge snapshot for {file_path}"))
            })?;
        let (
            row_file,
            row_hash,
            row_version,
            snapshot_json,
            canonical_config,
            canonical_producers,
            semantic_config,
            semantic_producers,
        ) = snapshot_row;
        let snapshot: DurableSemanticSnapshot = serde_json::from_str(&snapshot_json).map_err(
            |error| structural_error(format!("malformed semantic-edge snapshot: {error}")),
        )?;
        if row_file != file_path || snapshot.file_path != file_path {
            return Err(structural_error(
                "semantic-edge snapshot file identity mismatch",
            ));
        }
        if row_hash != snapshot.source_hash {
            return Err(structural_error(
                "canonical and semantic-edge source hashes do not match",
            ));
        }
        if target_sequence.is_none() && snapshot.source_hash != metadata.source_hash {
            return Err(structural_error(
                "latest canonical and semantic-edge source hashes do not match",
            ));
        }
        if row_version < 0 || row_version as u64 != ir.version || snapshot.version != ir.version {
            return Err(structural_error(format!(
                "canonical and semantic-edge versions do not match: IR v{}, edges v{}",
                ir.version, snapshot.version
            )));
        }
        let semantic_edges = snapshot.restore_edges().map_err(structural_error)?;
        Ok(Some(UntrustedDurableContext {
            ir,
            semantic_edges,
            source_hash: snapshot.source_hash,
            fidelity: metadata.fidelity,
            compatibility: PersistedCompatibilityIdentities {
                canonical_config: decode_optional_identity(canonical_config)?,
                canonical_producers: decode_optional_identity(canonical_producers)?,
                semantic_config: decode_optional_identity(semantic_config)?,
                semantic_producers: decode_optional_identity(semantic_producers)?,
            },
        }))
    }

    pub(super) fn write_semantic_snapshot(
        &self,
        context_id: &str,
        snapshot: &DurableSemanticSnapshot,
        identities: Option<&CompatibilityIdentities>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let edges_json = serde_json::to_string(snapshot)?;
        let semantic_config = identities
            .map(|value| serde_json::to_string(&value.semantic_config))
            .transpose()?;
        let semantic_producers = identities
            .map(|value| serde_json::to_string(&value.semantic_producers))
            .transpose()?;
        self.conn.execute(
            "INSERT OR REPLACE INTO semantic_edge_snapshots
             (context_id, file_path, source_hash, semantic_version, edges_json,
              semantic_config_identity, semantic_producer_identity)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                context_id,
                snapshot.file_path,
                snapshot.source_hash,
                snapshot.version as i64,
                edges_json,
                semantic_config,
                semantic_producers,
            ],
        )?;
        Ok(())
    }
}

fn decode_optional_identity<T: serde::de::DeserializeOwned>(
    encoded: Option<String>,
) -> Result<Option<T>, CompatibilityFailure> {
    encoded
        .map(|value| serde_json::from_str(&value))
        .transpose()
        .map_err(physical_error)
}

fn physical_error(error: impl std::fmt::Display) -> CompatibilityFailure {
    CompatibilityFailure::PhysicalSchemaIncompatibility {
        detail: error.to_string(),
    }
}

fn structural_error(detail: impl Into<String>) -> CompatibilityFailure {
    CompatibilityFailure::StructuralSnapshotIncoherence {
        detail: detail.into(),
    }
}
