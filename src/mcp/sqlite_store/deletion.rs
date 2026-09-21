//! Transactional removal of one file-scoped durable semantic owner.

use super::{DeletedContext, SqliteStore};
use rusqlite::{OptionalExtension, params};

impl SqliteStore {
    /// Delete the baseline, history, semantic snapshot, and recovery intent as
    /// one durable operation. Recovery artifacts remain until the caller has
    /// observed this commit and may safely remove them.
    pub(crate) fn delete_context_transactionally(
        &mut self,
        file_path: &str,
    ) -> Result<DeletedContext, Box<dyn std::error::Error>> {
        let tx = self.conn.transaction()?;
        let context_count: i64 = tx.query_row(
            "SELECT COUNT(*) FROM contexts WHERE file_path = ?1",
            params![file_path],
            |row| row.get(0),
        )?;
        if context_count != 1 {
            tx.rollback()?;
            return Ok(DeletedContext {
                count: context_count as usize,
                recovery_stage: None,
            });
        }
        let recovery_stage = tx
            .query_row(
                "SELECT stage_path FROM edit_intents WHERE file_path = ?1",
                params![file_path],
                |row| row.get(0),
            )
            .optional()?;
        tx.execute(
            "DELETE FROM edit_intents WHERE file_path = ?1",
            params![file_path],
        )?;
        tx.execute(
            "DELETE FROM symbols WHERE context_id IN (SELECT id FROM contexts WHERE file_path = ?1)",
            params![file_path],
        )?;
        tx.execute(
            "DELETE FROM deltas WHERE context_id IN (SELECT id FROM contexts WHERE file_path = ?1)",
            params![file_path],
        )?;
        tx.execute(
            "DELETE FROM semantic_edge_snapshots WHERE context_id IN (SELECT id FROM contexts WHERE file_path = ?1)",
            params![file_path],
        )?;
        let deleted = tx.execute(
            "DELETE FROM contexts WHERE file_path = ?1",
            params![file_path],
        )?;
        tx.commit()?;
        Ok(DeletedContext {
            count: deleted,
            recovery_stage,
        })
    }
}
