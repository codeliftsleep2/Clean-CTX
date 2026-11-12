use super::{
    Request,
    model::{Candidate, Diagnostic},
    profile::Trusted,
};
use crate::pcg::{Error, Result, identity, require, security};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::{Value, json};
use std::{collections::BTreeMap, time::Duration};

pub(super) fn sql<T>(r: rusqlite::Result<T>) -> Result<T> {
    r.map_err(|_| Error("document_database"))
}
pub(super) fn metadata(c: &Connection) -> Result<Vec<(String, String)>> {
    let mut s =
        sql(c.prepare("SELECT name,sql FROM sqlite_master WHERE sql IS NOT NULL ORDER BY name"))?;
    let rows = sql(s.query_map([], |r| Ok((r.get(0)?, r.get(1)?))))?;
    rows.map(|r| r.map_err(|_| Error("document_database")))
        .collect()
}
pub fn open(trusted: &Trusted) -> Result<Connection> {
    let p = &trusted.profile.database_path;
    security::no_links(p, true)?;
    let c = sql(Connection::open(p))?;
    sql(c.busy_timeout(Duration::ZERO))?;
    super::schema::check(&c, trusted)?;
    sql(c.execute_batch(
        "PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL;",
    ))?;
    Ok(c)
}
pub fn initialize(tx: &Connection, trusted: &Trusted) -> Result<()> {
    super::schema::initialize(tx, trusted)
}
pub fn head(c: &Connection) -> Result<Option<String>> {
    sql(c
        .query_row("SELECT g FROM document_head WHERE slot=1", [], |r| r.get(0))
        .optional())
}
pub fn missing(c: &Connection, nodes: &BTreeMap<String, Candidate>) -> Result<Vec<Diagnostic>> {
    let mut errors = Vec::new();
    if let Some(g) = head(c)? {
        let mut s = sql(c.prepare("SELECT m.id,r.path FROM document_member m JOIN document_revision r ON r.revision=m.revision WHERE m.g=? ORDER BY m.id"))?;
        let rows = sql(s.query_map([g], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        }))?;
        for row in rows {
            let (id, path) = sql(row)?;
            if !nodes.contains_key(&id) {
                errors.push(Diagnostic::new("E_DECLARATION_REMOVED", &path, Some(&id), "Restore the source declaration or explicitly represent supersession; disappearance is not a lifecycle transition"));
            }
        }
    }
    Ok(errors)
}
pub fn publish(
    tx: &Connection,
    trusted: &Trusted,
    nodes: &BTreeMap<String, Candidate>,
) -> Result<String> {
    reconcile(tx, trusted, nodes, true)
}
pub(super) fn publish_knowledge(
    tx: &Connection,
    trusted: &Trusted,
    nodes: &BTreeMap<String, Candidate>,
) -> Result<String> {
    reconcile(tx, trusted, nodes, false)
}
fn reconcile(
    tx: &Connection,
    trusted: &Trusted,
    nodes: &BTreeMap<String, Candidate>,
    legacy_search: bool,
) -> Result<String> {
    let generation = identity::hash(
        &json!({"profile":trusted.fingerprint,"revisions":nodes.values().map(|n|&n.revision).collect::<Vec<_>>()}),
    );
    let old_head = head(tx)?;
    if old_head.as_ref() == Some(&generation) {
        verify(tx, &generation, nodes, legacy_search)?;
        super::knowledge_store::publish(tx, &generation, nodes, false)?;
        return Ok(generation);
    }
    sql(tx.execute(
        "INSERT OR IGNORE INTO document_generation VALUES(?)",
        [&generation],
    ))?;
    let mut source_keys = std::collections::BTreeSet::new();
    for (id, n) in nodes {
        let previous: Option<String> = sql(tx.query_row("SELECT revision FROM document_member WHERE g=(SELECT g FROM document_head WHERE slot=1) AND id=?", [id], |r|r.get(0)).optional())?;
        let declaration =
            serde_json::to_string(&n.declaration).map_err(|_| Error("document_encoding"))?;
        let evidence =
            serde_json::to_string(&n.evidence).map_err(|_| Error("document_encoding"))?;
        if source_keys.insert((&n.path, &n.source_sha)) {
            sql(tx.execute(
                "INSERT OR IGNORE INTO document_source VALUES(?,?,?)",
                params![n.path, n.source_sha, n.bytes.as_slice()],
            ))?;
        }
        sql(tx.execute(
            "INSERT OR IGNORE INTO document_revision VALUES(?,?,?,?,?,?,?)",
            params![
                n.revision,
                id,
                n.path,
                n.source_sha,
                declaration,
                evidence,
                previous
            ],
        ))?;
        sql(tx.execute(
            "INSERT OR IGNORE INTO document_member VALUES(?,?,?)",
            params![generation, id, n.revision],
        ))?;
    }
    for (id, n) in nodes {
        for e in &n.declaration.relationships {
            let evidence =
                serde_json::to_string(&json!({"path":n.path,"source_sha256":n.source_sha}))
                    .map_err(|_| Error("document_encoding"))?;
            sql(tx.execute(
                "INSERT OR IGNORE INTO document_edge VALUES(?,?,?,?,?)",
                params![generation, id, e.kind, e.target, evidence],
            ))?;
        }
    }
    sql(tx.execute("DELETE FROM document_search", []))?;
    for (id, n) in nodes {
        sql(tx.execute(
            "INSERT INTO document_search(id,title,body,track) VALUES(?,?,?,?)",
            params![
                id,
                n.declaration.title,
                std::str::from_utf8(&n.bytes).map_err(|_| Error("invalid_utf8"))?,
                n.declaration.track.clone()
            ],
        ))?;
    }
    verify(tx, &generation, nodes, true)?;
    super::knowledge_store::publish(tx, &generation, nodes, true)?;
    sql(tx.execute(
        "INSERT INTO document_head VALUES(1,?) ON CONFLICT(slot) DO UPDATE SET g=excluded.g",
        [&generation],
    ))?;
    sql(tx.execute(
        "INSERT INTO document_publication(g,previous_g) VALUES(?,?)",
        params![generation, old_head],
    ))?;
    Ok(generation)
}

/// A full candidate is already acquired; do not certify absence against missing stored membership.
fn verify(
    c: &Connection,
    g: &str,
    nodes: &BTreeMap<String, Candidate>,
    search: bool,
) -> Result<()> {
    let mut statement =
        sql(c.prepare("SELECT id,revision FROM document_member WHERE g=? ORDER BY id"))?;
    let rows = sql(statement.query_map([g], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
    }))?;
    let actual = rows
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| Error("document_database"))?;
    let expected: Vec<_> = nodes
        .iter()
        .map(|(id, n)| (id.clone(), n.revision.clone()))
        .collect();
    require(actual == expected, "document_corruption")?;
    // Validate exact rows without constructing legacy full-source response Values.
    // A declaring source is checked once for this complete operation.
    let mut checked_sources = std::collections::BTreeSet::new();
    for (id, n) in nodes {
        let row: (String,String,String,String,String)=sql(c.query_row(
            "SELECT id,path,source_sha,declaration,evidence FROM document_revision WHERE revision=?",[&n.revision],
            |r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))))?;
        let declaration: Value =
            serde_json::from_str(&row.3).map_err(|_| Error("document_corruption"))?;
        let evidence: Value =
            serde_json::from_str(&row.4).map_err(|_| Error("document_corruption"))?;
        require(
            row.0 == *id
                && row.1 == n.path
                && row.2 == n.source_sha
                && declaration
                    == serde_json::to_value(&n.declaration)
                        .map_err(|_| Error("document_encoding"))?
                && evidence == json!(n.evidence),
            "document_corruption",
        )?;
        if checked_sources.insert((&n.path, &n.source_sha)) {
            let body: Vec<u8> = sql(c.query_row(
                "SELECT body FROM document_source WHERE path=? AND sha=?",
                params![n.path, n.source_sha],
                |r| r.get(0),
            ))?;
            require(body.as_slice() == n.bytes.as_slice(), "document_corruption")?;
        }
    }
    let mut statement = sql(c.prepare("SELECT source,kind,target,evidence FROM document_edge WHERE g=? ORDER BY source,kind,target"))?;
    let rows = sql(statement.query_map([g], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
        ))
    }))?;
    let actual = rows
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| Error("document_database"))?;
    let mut expected = Vec::new();
    for (id, n) in nodes {
        for e in &n.declaration.relationships {
            expected.push((
                id.clone(),
                e.kind.clone(),
                e.target.clone(),
                serde_json::to_string(&json!({"path":n.path,"source_sha256":n.source_sha}))
                    .map_err(|_| Error("document_encoding"))?,
            ));
        }
    }
    expected.sort();
    expected.dedup();
    require(actual == expected, "document_corruption")?;
    if search {
        let mut statement =
            sql(c.prepare("SELECT id,title,body,track FROM document_search ORDER BY id"))?;
        let rows = sql(statement.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
            ))
        }))?;
        let actual = rows
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|_| Error("document_database"))?;
        let expected: Vec<_> = nodes
            .iter()
            .map(|(id, n)| {
                Ok((
                    id.clone(),
                    n.declaration.title.clone(),
                    std::str::from_utf8(&n.bytes)
                        .map_err(|_| Error("invalid_utf8"))?
                        .to_owned(),
                    n.declaration.track.clone(),
                ))
            })
            .collect::<Result<_>>()?;
        require(actual == expected, "document_corruption")?;
    }
    Ok(())
}
pub(super) fn revision(c: &Connection, rev: &str) -> Result<Value> {
    let row: (String,String,String,Vec<u8>,String,Option<String>) = sql(c.query_row(
        "SELECT r.path,r.source_sha,r.declaration,s.body,r.evidence,r.previous FROM document_revision r JOIN document_source s ON s.path=r.path AND s.sha=r.source_sha WHERE revision=?", [rev],
        |r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?))))?;
    require(identity::sha(&row.3) == row.1, "document_corruption")?;
    let d: Value = serde_json::from_str(&row.2).map_err(|_| Error("document_corruption"))?;
    let e: Value = serde_json::from_str(&row.4).map_err(|_| Error("document_corruption"))?;
    require(
        identity::hash(&json!({"path":row.0,"source_sha256":row.1,"declaration":d,"evidence":e}))
            == rev,
        "document_corruption",
    )?;
    Ok(
        json!({"revision":rev,"path":row.0,"source_sha256":row.1,"declaration":d,"evidence":e,
        "source_text":String::from_utf8(row.3).map_err(|_|Error("document_corruption"))?,"previous_revision":row.5,
        "authority_assurance":"source_attributed","freshness":"historical_snapshot"}),
    )
}
pub fn query(c: &Connection, g: &str, q: &Request) -> Result<Vec<Value>> {
    let mut selected = std::collections::BTreeSet::new();
    if let Some(text) = &q.text {
        let mut s = sql(
            c.prepare("SELECT id FROM document_search WHERE document_search MATCH ? ORDER BY id")
        )?;
        for row in sql(s.query_map([text], |r| r.get::<_, String>(0)))? {
            selected.insert(sql(row)?);
        }
    }
    if q.mode == "related" {
        let mut s = sql(c.prepare("SELECT target FROM document_edge WHERE g=?1 AND source=?2 UNION SELECT source FROM document_edge WHERE g=?1 AND target=?2 ORDER BY 1"))?;
        for row in sql(s.query_map(params![g, q.id], |r| r.get::<_, String>(0)))? {
            selected.insert(sql(row)?);
        }
    }
    let mut s = sql(c.prepare("SELECT id,revision FROM document_member WHERE g=? ORDER BY id"))?;
    let rows = sql(s.query_map([g], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
    }))?;
    let mut answer = Vec::new();
    for row in rows {
        let (id, rev) = sql(row)?;
        if q.mode == "get" || q.mode == "history" {
            if q.id.as_ref() != Some(&id) {
                continue;
            }
        } else if (q.text.is_some() || q.mode == "related") && !selected.contains(&id) {
            continue;
        }
        let value = revision(c, &rev)?;
        if q.track
            .as_ref()
            .is_some_and(|track| value["declaration"]["track"].as_str() != Some(track))
        {
            continue;
        }
        if q.mode == "history" {
            let mut history = sql(c.prepare("SELECT p.sequence,p.g,m.revision FROM document_publication p JOIN document_member m ON m.g=p.g WHERE m.id=? ORDER BY p.sequence"))?;
            for old in sql(history.query_map([id], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                ))
            }))? {
                let (sequence, generation, rev) = sql(old)?;
                let mut value = revision(c, &rev)?;
                value["publication_sequence"] = sequence.into();
                value["published_generation"] = generation.into();
                answer.push(value);
            }
        } else {
            let mut value = value;
            value["freshness"] = "current_observed_boundaries".into();
            answer.push(value);
        }
    }
    Ok(answer)
}
