//! Isolated derived SQLite owner. Published generation rows are never updated.
use super::{Error, Result, model::Generation, profile::Trusted, require, security};
use rusqlite::{
    Connection, OpenFlags, OptionalExtension, Transaction, params, params_from_iter, types::Value,
};
use std::{path::Path, time::Duration};

const DDL: &str = include_str!("schema.sql");
const INSERT_ORDER: &[&str] = &[
    "generation",
    "source_revision",
    "evidence_revision",
    "record_revision",
    "evidence_ref",
    "selector",
    "binding",
    "relation",
];

fn sql<T>(value: rusqlite::Result<T>) -> Result<T> {
    value.map_err(|_| Error("projection_unavailable"))
}

fn metadata(connection: &Connection) -> Result<Vec<(String, String, String)>> {
    let mut statement = sql(connection.prepare(
        "SELECT type,name,sql FROM sqlite_master WHERE sql IS NOT NULL ORDER BY type,name",
    ))?;
    let rows = sql(statement.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))))?;
    rows.map(|r| r.map_err(|_| Error("projection_unavailable")))
        .collect()
}

/// Bounded schema/header verification only. Never scan generation constituents here.
pub(super) fn schema(connection: &Connection) -> Result<()> {
    let reference = sql(Connection::open_in_memory())?;
    sql(reference.execute_batch(DDL))?;
    require(
        metadata(connection)? == metadata(&reference)?,
        "projection_schema",
    )?;
    let version: Vec<i64> = {
        let mut statement = sql(connection.prepare("SELECT version FROM pcg_schema LIMIT 2"))?;
        let rows = sql(statement.query_map([], |r| r.get(0)))?;
        rows.collect::<rusqlite::Result<_>>()
            .map_err(|_| Error("projection_schema"))?
    };
    require(version == [1], "projection_schema")
}

pub(super) fn head(
    connection: &Connection,
    trusted: &Trusted,
    corpus: &str,
) -> Result<Option<String>> {
    sql(connection
        .query_row(
            "SELECT g FROM active_generation WHERE p=? AND c=?",
            params![trusted.profile.partition_id, corpus],
            |r| r.get(0),
        )
        .optional())
}

pub(super) fn reader(trusted: &Trusted, corpus: &str) -> Result<Connection> {
    security::no_links(Path::new(&trusted.profile.database_path), false)?;
    let connection = sql(Connection::open_with_flags(
        &trusted.profile.database_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    ))?;
    sql(connection.busy_timeout(Duration::ZERO))?;
    sql(connection.execute_batch("PRAGMA query_only=ON; PRAGMA foreign_keys=ON; BEGIN"))?;
    schema(&connection)?;
    let g = head(&connection, trusted, corpus)?.ok_or(Error("projection_unavailable"))?;
    let header: (String, String, String, String) = sql(connection.query_row(
        "SELECT manifest_sha,policy,parser,fingerprint FROM generation WHERE p=? AND c=? AND g=?",
        params![trusted.profile.partition_id, corpus, g],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
    ))?;
    require(
        header
            == (
                trusted.profile.approved_manifest_sha256.clone(),
                trusted.profile.policy_revision.clone(),
                "pcg-json-v1".into(),
                g,
            ),
        "generation_mismatch",
    )?;
    Ok(connection)
}

pub(super) fn expected_head(trusted: &Trusted, corpus: &str) -> Result<Option<String>> {
    if !Path::new(&trusted.profile.database_path).exists() {
        return Ok(None);
    }
    // A replacement manifest may not match the old header. This read obtains only
    // the old CAS identity; it does not certify it as eligible for retrieval.
    let connection = sql(Connection::open_with_flags(
        &trusted.profile.database_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY,
    ))?;
    if metadata(&connection)?.is_empty() {
        return Ok(None);
    }
    schema(&connection)?;
    head(&connection, trusted, corpus)
}

pub(super) fn writer(trusted: &Trusted) -> Result<(Connection, bool)> {
    let path = Path::new(&trusted.profile.database_path);
    security::no_links(path, true)?;
    let absent = !path.exists();
    let connection = sql(Connection::open(path))?;
    sql(connection.busy_timeout(Duration::ZERO))?;
    // Explicit initial admission may initialize a truly empty SQLite file left
    // by an interrupted first transaction. Never repair a nonempty/unknown schema.
    let new = absent || metadata(&connection)?.is_empty();
    if !new {
        schema(&connection)?;
    }
    let mode: String = sql(connection.query_row("PRAGMA journal_mode=WAL", [], |r| r.get(0)))?;
    require(mode == "wal", "projection_configuration")?;
    sql(connection.execute_batch("PRAGMA foreign_keys=ON; PRAGMA synchronous=FULL"))?;
    Ok((connection, new))
}

pub(super) fn initialize(transaction: &Transaction<'_>, new: bool) -> Result<()> {
    if new && metadata(transaction)?.is_empty() {
        sql(transaction.execute_batch(DDL))?;
    }
    schema(transaction)
}

pub(super) fn existing(
    transaction: &Transaction<'_>,
    trusted: &Trusted,
    g: &Generation,
) -> Result<Option<String>> {
    sql(transaction.query_row("SELECT g FROM generation WHERE p=? AND c=? AND manifest_sha=? AND policy=? AND parser=?",
        params![trusted.profile.partition_id, g.manifest.corpus_id, trusted.profile.approved_manifest_sha256,
            trusted.profile.policy_revision, g.manifest.parser_version], |r| r.get(0)).optional())
}

pub(super) fn insert(
    transaction: &Transaction<'_>,
    trusted: &Trusted,
    g: &Generation,
) -> Result<()> {
    let namespace: Option<String> = sql(transaction
        .query_row(
            "SELECT namespace FROM corpus WHERE p=? AND c=?",
            params![trusted.profile.partition_id, g.manifest.corpus_id],
            |r| r.get(0),
        )
        .optional())?;
    if let Some(namespace) = namespace {
        require(
            namespace == g.manifest.source_namespace,
            "source_namespace_changed",
        )?;
    } else {
        sql(transaction.execute(
            "INSERT INTO corpus VALUES(?,?,?)",
            params![
                trusted.profile.partition_id,
                g.manifest.corpus_id,
                g.manifest.source_namespace
            ],
        ))?;
    }
    for table in INSERT_ORDER {
        for row in &g.rows.0[table] {
            let placeholders = vec!["?"; row.len()].join(",");
            let query = format!("INSERT INTO {table} VALUES({placeholders})");
            sql(transaction.execute(&query, params_from_iter(row)))?;
        }
    }
    compare(transaction, trusted, g)
}

pub(super) fn compare(connection: &Connection, trusted: &Trusted, g: &Generation) -> Result<()> {
    let namespace: String = sql(connection.query_row(
        "SELECT namespace FROM corpus WHERE p=? AND c=?",
        params![trusted.profile.partition_id, g.manifest.corpus_id],
        |r| r.get(0),
    ))?;
    require(
        namespace == g.manifest.source_namespace,
        "projection_corrupt",
    )?;
    for (table, expected) in &g.rows.0 {
        let query = format!("SELECT * FROM {table} WHERE p=? AND c=? AND g=?");
        let mut statement = sql(connection.prepare(&query))?;
        let count = statement.column_count();
        let mut cursor = sql(statement.query(params![
            trusted.profile.partition_id,
            g.manifest.corpus_id,
            g.id
        ]))?;
        let mut actual = Vec::new();
        while let Some(row) = sql(cursor.next())? {
            require(actual.len() < expected.len(), "projection_corrupt")?;
            let values = (0..count)
                .map(|i| row.get::<_, Value>(i))
                .collect::<rusqlite::Result<Vec<_>>>();
            actual.push(sql(values)?);
        }
        actual.sort_by_cached_key(|r| format!("{r:?}"));
        require(&actual == expected, "projection_corrupt")?;
    }
    Ok(())
}

pub(super) fn clear(transaction: &Transaction<'_>, trusted: &Trusted, corpus: &str) -> Result<()> {
    let p = &trusted.profile.partition_id;
    sql(transaction.execute(
        "DELETE FROM active_generation WHERE p=? AND c=?",
        params![p, corpus],
    ))?;
    for table in [
        "relation",
        "binding",
        "evidence_ref",
        "record_revision",
        "selector",
        "evidence_revision",
        "source_revision",
        "generation",
    ] {
        sql(transaction.execute(
            &format!("DELETE FROM {table} WHERE p=? AND c=?"),
            params![p, corpus],
        ))?;
    }
    Ok(())
}

pub(super) fn publish(
    transaction: &Transaction<'_>,
    trusted: &Trusted,
    g: &Generation,
) -> Result<()> {
    sql(transaction.execute(
        "INSERT INTO active_generation VALUES(?,?,?) ON CONFLICT(p,c) DO UPDATE SET g=excluded.g",
        params![trusted.profile.partition_id, g.manifest.corpus_id, g.id],
    ))?;
    Ok(())
}

/// Same-ID witness refresh cannot silently replace a controlling conclusion.
/// Reviewed source prose remains owner-attributed; no textual equivalence is inferred.
pub(super) fn preserve_decisions(
    connection: &Connection,
    trusted: &Trusted,
    g: &Generation,
) -> Result<()> {
    let Some(previous) = head(connection, trusted, &g.manifest.corpus_id)? else {
        return Ok(());
    };
    let mut statement = sql(connection.prepare(
        "SELECT r.rid,r.role,r.authority,r.attribution,r.verification,r.verification_subject,r.execution,r.subject,r.disposition
         FROM record_revision r
         WHERE r.p=? AND r.c=? AND r.g=? AND r.role IN ('decision','constraint')"))?;
    let mut cursor = sql(statement.query(params![
        trusted.profile.partition_id,
        g.manifest.corpus_id,
        previous
    ]))?;
    let new_records: std::collections::BTreeMap<_, _> = g.rows.0["record_revision"]
        .iter()
        .filter_map(|r| {
            if let Value::Text(id) = &r[3] {
                Some((id, r))
            } else {
                None
            }
        })
        .collect();
    while let Some(row) = sql(cursor.next())? {
        let id: String = sql(row.get(0))?;
        if let Some(new) = new_records.get(&id) {
            let columns = [5, 9, 10, 11, 12, 13, 15, 16];
            for (old, index) in columns.iter().enumerate() {
                require(
                    sql(row.get::<_, Value>(old + 1))? == new[*index],
                    "decision_replacement_requires_successor",
                )?;
            }
        }
    }
    Ok(())
}

pub(super) fn begin(connection: &mut Connection) -> Result<Transaction<'_>> {
    connection
        .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
        .map_err(|_| Error("import_busy"))
}

pub(super) fn commit(transaction: Transaction<'_>) -> Result<()> {
    sql(transaction.commit())
}
