//! SQLite schema ownership for durable authority compatibility evidence.

use rusqlite::Connection;

#[cfg(test)]
use super::SqliteStore;

pub(super) fn migrate(
    connection: &Connection,
    current_version: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    if current_version < 6 {
        for (table, column) in [
            ("contexts", "canonical_config_identity"),
            ("contexts", "canonical_producer_identity"),
            ("semantic_edge_snapshots", "semantic_config_identity"),
            ("semantic_edge_snapshots", "semantic_producer_identity"),
            ("edit_intents", "canonical_config_identity"),
            ("edit_intents", "canonical_producer_identity"),
            ("edit_intents", "semantic_config_identity"),
            ("edit_intents", "semantic_producer_identity"),
        ] {
            if !has_column(connection, table, column)? {
                connection.execute(
                    &format!("ALTER TABLE {table} ADD COLUMN {column} TEXT"),
                    [],
                )?;
            }
        }
        connection.execute(
            "INSERT OR IGNORE INTO _schema_version (version) VALUES (6)",
            [],
        )?;
    }
    Ok(())
}

fn has_column(
    connection: &Connection,
    table: &str,
    expected: &str,
) -> Result<bool, rusqlite::Error> {
    let mut statement = connection.prepare(&format!("PRAGMA table_info({table})"))?;
    let mut rows = statement.query([])?;
    while let Some(row) = rows.next()? {
        if row.get::<_, String>(1)? == expected {
            return Ok(true);
        }
    }
    Ok(false)
}

#[cfg(test)]
impl SqliteStore {
    pub(crate) fn stored_compatibility_json(
        &self,
        file_path: &str,
    ) -> Result<[Option<String>; 4], Box<dyn std::error::Error>> {
        self.conn
            .query_row(
                "SELECT c.canonical_config_identity,
                        c.canonical_producer_identity,
                        s.semantic_config_identity,
                        s.semantic_producer_identity
                 FROM contexts c
                 JOIN semantic_edge_snapshots s ON s.context_id = c.id
                 WHERE c.file_path = ?1",
                [file_path],
                |row| Ok([row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?]),
            )
            .map_err(Into::into)
    }

    pub(crate) fn stored_edit_intent_compatibility_json(
        &self,
        file_path: &str,
    ) -> Result<[Option<String>; 4], Box<dyn std::error::Error>> {
        self.conn
            .query_row(
                "SELECT canonical_config_identity, canonical_producer_identity,
                        semantic_config_identity, semantic_producer_identity
                 FROM edit_intents WHERE file_path = ?1",
                [file_path],
                |row| Ok([row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?]),
            )
            .map_err(Into::into)
    }
}
