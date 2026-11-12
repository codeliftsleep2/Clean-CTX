//! Recognized v1 -> v2 migration inside the caller's existing immediate transaction.
use super::{
    profile::Trusted,
    store::{self, sql},
};
use crate::pcg::{Result, require};
use rusqlite::Connection;

const CURRENT: &str = include_str!("schema.sql");
const LEGACY: &str = include_str!("schema_v1.sql");

pub fn check(c: &Connection, trusted: &Trusted) -> Result<Option<i64>> {
    let actual = store::metadata(c)?;
    if actual.is_empty() {
        return Ok(None);
    }
    let mut version = None;
    for (number, schema) in [(2, CURRENT), (1, LEGACY)] {
        let reference = sql(Connection::open_in_memory())?;
        sql(reference.execute_batch(schema))?;
        if actual == store::metadata(&reference)? {
            version = Some(number);
            break;
        }
    }
    require(version.is_some(), "document_schema")?;
    let expected = (
        version.expect("recognized schema"),
        trusted.scope_fingerprint.clone(),
    );
    let stored: (i64, String) = sql(c.query_row(
        "SELECT version,profile FROM document_schema",
        [],
        |r| Ok((r.get(0)?, r.get(1)?)),
    ))?;
    require(stored == expected, "document_profile_changed")?;
    Ok(version)
}

pub fn initialize(tx: &Connection, trusted: &Trusted) -> Result<()> {
    match check(tx, trusted)? {
        None => {
            sql(tx.execute_batch(CURRENT))?;
            sql(tx.execute(
                "INSERT INTO document_schema VALUES(2,?)",
                [&trusted.scope_fingerprint],
            ))?;
        }
        Some(1) => migrate(tx)?,
        Some(2) => {}
        _ => unreachable!("schema checked"),
    }
    Ok(())
}

fn migrate(tx: &Connection) -> Result<()> {
    // Deferred checking is transaction-local; FK enforcement is never disabled.
    sql(tx.execute_batch(
        "PRAGMA defer_foreign_keys=ON;
        CREATE TEMP TABLE legacy_search AS SELECT id,title,body,track FROM document_search;
        DROP TABLE document_search;
        DROP INDEX document_revision_identity;
        DROP INDEX document_edge_reverse;",
    ))?;
    for table in [
        "schema",
        "revision",
        "generation",
        "member",
        "head",
        "publication",
        "edge",
    ] {
        sql(tx.execute_batch(&format!(
            "ALTER TABLE document_{table} RENAME TO legacy_document_{table}"
        )))?;
    }
    sql(tx.execute_batch(CURRENT))?;
    sql(tx.execute_batch("INSERT INTO document_schema SELECT 2,profile FROM legacy_document_schema;
        INSERT INTO document_source SELECT DISTINCT path,source_sha,body FROM legacy_document_revision;
        INSERT INTO document_revision SELECT revision,id,path,source_sha,declaration,evidence,previous FROM legacy_document_revision;
        INSERT INTO document_generation SELECT * FROM legacy_document_generation;
        INSERT INTO document_member SELECT * FROM legacy_document_member;
        INSERT INTO document_head SELECT * FROM legacy_document_head;
        INSERT INTO document_publication SELECT * FROM legacy_document_publication;
        INSERT INTO document_edge SELECT * FROM legacy_document_edge;
        INSERT INTO document_search SELECT * FROM legacy_search;"))?;
    for table in [
        "edge",
        "head",
        "publication",
        "member",
        "revision",
        "generation",
        "schema",
    ] {
        sql(tx.execute_batch(&format!("DROP TABLE legacy_document_{table}")))?;
    }
    sql(tx.execute_batch("DROP TABLE legacy_search"))?;
    let mut statement = sql(tx.prepare("PRAGMA foreign_key_check"))?;
    require(
        sql(statement.query([]))?
            .next()
            .map_err(|_| crate::pcg::Error("document_database"))?
            .is_none(),
        "document_corruption",
    )?;
    let revisions = {
        let mut statement =
            sql(tx.prepare("SELECT revision FROM document_revision ORDER BY revision"))?;
        sql(statement.query_map([], |r| r.get::<_, String>(0)))?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|_| crate::pcg::Error("document_database"))?
    };
    for revision in revisions {
        store::revision(tx, &revision)?;
    }
    Ok(())
}
