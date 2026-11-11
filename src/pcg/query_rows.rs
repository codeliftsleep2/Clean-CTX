//! Indexed, generation-scoped reads and operation-local verification. No writer access.
use super::{
    Error, Result, identity,
    model::*,
    profile::Trusted,
    query::{Counts, Request},
    require, security,
};
use rusqlite::{Connection, OptionalExtension, params, params_from_iter, types::Value as Sql};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

#[derive(Clone)]
pub(super) struct RecordRow {
    pub data: Record,
    pub revision: String,
}
#[derive(Clone)]
pub(super) struct SourceRow {
    pub data: Source,
    pub revision: String,
}
#[derive(Clone)]
pub(super) struct EvidenceRow {
    pub id: String,
    pub revision: String,
    pub source: String,
    pub source_revision: String,
    pub start: u64,
    pub end: u64,
    pub hash: String,
}
#[derive(Clone)]
pub(super) struct RelationRow {
    pub id: String,
    pub revision: String,
    pub kind: String,
    pub from: String,
    pub from_revision: String,
    pub to: String,
    pub to_revision: String,
    pub evidence: String,
    pub gate: Option<Gate>,
}

mod objects;

type Row = Vec<Sql>;
fn sql<T>(v: rusqlite::Result<T>) -> Result<T> {
    v.map_err(|_| Error("projection_corrupt"))
}
fn text(row: &Row, index: usize) -> Result<String> {
    match row.get(index) {
        Some(Sql::Text(s)) => Ok(s.clone()),
        _ => Err(Error("projection_corrupt")),
    }
}
fn nullable(row: &Row, index: usize) -> Result<Option<String>> {
    match row.get(index) {
        Some(Sql::Null) => Ok(None),
        Some(Sql::Text(s)) => Ok(Some(s.clone())),
        _ => Err(Error("projection_corrupt")),
    }
}
fn integer(row: &Row, index: usize) -> Result<u64> {
    match row.get(index) {
        Some(Sql::Integer(n)) if *n >= 0 => Ok(*n as u64),
        _ => Err(Error("projection_corrupt")),
    }
}
fn boolean(row: &Row, index: usize) -> Result<bool> {
    let n = integer(row, index)?;
    require(n <= 1, "projection_corrupt")?;
    Ok(n == 1)
}
fn decode<T: serde::de::DeserializeOwned>(value: Value) -> Result<T> {
    serde_json::from_value(value).map_err(|_| Error("projection_corrupt"))
}

pub(super) struct Reader<'a> {
    pub db: &'a Connection,
    pub trusted: &'a Trusted,
    pub request: &'a Request,
    pub counts: Counts,
    record_units: BTreeSet<String>,
    edge_units: BTreeSet<String>,
    records: BTreeMap<String, Rc<RecordRow>>,
    sources: BTreeMap<String, SourceRow>,
    evidence: BTreeMap<String, EvidenceRow>,
    selectors: BTreeMap<String, (Selector, String)>,
    relations: BTreeMap<String, RelationRow>,
    bindings: BTreeMap<String, RelationRow>,
}
impl<'a> Reader<'a> {
    pub fn new(db: &'a Connection, trusted: &'a Trusted, request: &'a Request) -> Self {
        Self {
            db,
            trusted,
            request,
            counts: Counts::default(),
            record_units: BTreeSet::new(),
            edge_units: BTreeSet::new(),
            records: BTreeMap::new(),
            sources: BTreeMap::new(),
            evidence: BTreeMap::new(),
            selectors: BTreeMap::new(),
            relations: BTreeMap::new(),
            bindings: BTreeMap::new(),
        }
    }
    fn prefix(&self) -> Vec<Sql> {
        [
            &self.trusted.profile.partition_id,
            &self.request.corpus,
            &self.request.generation,
        ]
        .into_iter()
        .map(|s| Sql::Text(s.clone()))
        .collect()
    }
    fn unit(&mut self, kind: &str, id: &str, edge: bool) -> Result<()> {
        let key = format!("{kind}:{id}");
        let (seen, limit) = if edge {
            (&mut self.edge_units, self.request.limits.edges)
        } else {
            (&mut self.record_units, self.request.limits.records)
        };
        if !seen.contains(&key) {
            require((seen.len() as u64) < limit, "work_limit")?;
            seen.insert(key);
        }
        Ok(())
    }
    fn scan(&self, query: &str, extra: Vec<Sql>, ceiling: u64) -> Result<Vec<Row>> {
        let mut values = self.prefix();
        values.extend(extra);
        let mut statement = sql(self.db.prepare(query))?;
        let columns = statement.column_count();
        let mut cursor = sql(statement.query(params_from_iter(values)))?;
        let mut output = Vec::new();
        while let Some(row) = sql(cursor.next())? {
            require((output.len() as u64) < ceiling, "work_limit")?;
            let mut bytes = 0u64;
            for i in 0..columns {
                bytes += match sql(row.get_ref(i))? {
                    rusqlite::types::ValueRef::Text(s) | rusqlite::types::ValueRef::Blob(s) => {
                        s.len() as u64
                    }
                    _ => 8,
                };
            }
            require(
                bytes <= self.trusted.profile.limits.evidence_bytes,
                "projection_corrupt",
            )?;
            output.push(sql((0..columns)
                .map(|i| row.get(i))
                .collect::<rusqlite::Result<Row>>())?);
        }
        Ok(output)
    }
    fn one(&self, table: &str, key: &str, id: &str) -> Result<Row> {
        let rows = self.scan(
            &format!("SELECT * FROM {table} WHERE p=?1 AND c=?2 AND g=?3 AND {key}=?4"),
            vec![Sql::Text(id.into())],
            1,
        )?;
        rows.into_iter().next().ok_or(Error("projection_corrupt"))
    }
    fn verify(&mut self, revision: &str, value: Value) -> Result<()> {
        security::json_nonsecret(&value)?;
        let bytes = identity::canonical(&value);
        self.counts.metadata_bytes += bytes.len() as u64;
        require(
            identity::sha(bytes.as_bytes()) == revision,
            "projection_corrupt",
        )
    }
    pub fn enrollment_complete(&self) -> Result<bool> {
        let value: i64 = sql(self.db.query_row(
            "SELECT enrollment_complete FROM generation WHERE p=? AND c=? AND g=?",
            params![
                self.trusted.profile.partition_id,
                self.request.corpus,
                self.request.generation
            ],
            |r| r.get(0),
        ))?;
        require(value == 0 || value == 1, "projection_corrupt")?;
        Ok(value == 1)
    }
    pub fn exists(&self, id: &str) -> Result<bool> {
        let found: Option<String> = sql(self
            .db
            .query_row(
                "SELECT rid FROM record_revision WHERE p=? AND c=? AND g=? AND rid=?",
                params![
                    self.trusted.profile.partition_id,
                    self.request.corpus,
                    self.request.generation,
                    id
                ],
                |r| r.get(0),
            )
            .optional())?;
        Ok(found.is_some())
    }
    pub fn candidates(&mut self) -> Result<Vec<Selector>> {
        let mut output = Vec::new();
        for purpose in ["coverage", "applicability"] {
            let option = |s: Option<&str>| s.map(|s| Sql::Text(s.into())).unwrap_or(Sql::Null);
            let rows = self.scan("SELECT * FROM selector WHERE p=?1 AND c=?2 AND g=?3 AND purpose=?4 AND workspace=?5 AND (file_path IS NULL OR ?6 IS NULL OR file_path=?6) AND (operation IS NULL OR ?7 IS NULL OR operation=?7) AND (task IS NULL OR ?8 IS NULL OR task=?8) ORDER BY xid",
                vec![Sql::Text(purpose.into()), Sql::Text(self.request.subject.workspace.clone()), option(self.request.subject.file()),
                    option(self.request.subject.operation.as_deref()), option(self.request.subject.task.as_deref())], self.request.limits.records)?;
            for row in rows {
                let id = text(&row, 3)?;
                self.unit("selector", &id, false)?;
                self.counts.selectors += 1;
                let normalized = identity::parse(text(&row, 13)?.as_bytes())?;
                let selector: Selector = decode(normalized.clone())?;
                require(
                    ["coverage", "applicability"].contains(&selector.purpose.as_str()),
                    "projection_corrupt",
                )?;
                for instant in [&selector.from, &selector.until].into_iter().flatten() {
                    super::selectors::instant(instant)?;
                }
                require(
                    !matches!((&selector.from,&selector.until),(Some(from),Some(until)) if from >= until),
                    "projection_corrupt",
                )?;
                let entity = selector
                    .entity
                    .as_ref()
                    .map(|e| identity::canonical(&json!(e)));
                require(
                    selector.id == id
                        && selector.purpose == text(&row, 5)?
                        && selector.workspace == text(&row, 6)?
                        && selector.file == nullable(&row, 7)?
                        && entity == nullable(&row, 8)?
                        && selector.operation == nullable(&row, 9)?
                        && selector.task == nullable(&row, 10)?
                        && selector.from == nullable(&row, 11)?
                        && selector.until == nullable(&row, 12)?,
                    "projection_corrupt",
                )?;
                let revision = text(&row, 4)?;
                self.verify(&revision, normalized)?;
                self.selectors.insert(id, (selector.clone(), revision));
                output.push(selector);
            }
        }
        Ok(output)
    }
    pub fn bindings(&mut self, selector: &str) -> Result<Vec<RelationRow>> {
        let rows = self.scan(
            "SELECT * FROM binding WHERE p=?1 AND c=?2 AND g=?3 AND xid=?4 ORDER BY bid",
            vec![Sql::Text(selector.into())],
            self.request.limits.edges,
        )?;
        let mut output = Vec::new();
        for row in rows {
            let id = text(&row, 3)?;
            if let Some(edge) = self.bindings.get(&id) {
                output.push(edge.clone());
                continue;
            }
            self.unit("binding", &id, false)?;
            self.unit("binding", &id, true)?;
            self.counts.bindings += 1;
            let from = self.record(&text(&row, 5)?)?;
            let evidence = self.evidence(&text(&row, 9)?)?;
            let xrev = self
                .selectors
                .get(selector)
                .ok_or(Error("projection_corrupt"))?
                .1
                .clone();
            require(
                text(&row, 6)? == from.revision
                    && text(&row, 7)? == selector
                    && text(&row, 8)? == xrev
                    && text(&row, 10)? == evidence.revision
                    && ["decision", "constraint"].contains(&from.data.role.as_str()),
                "projection_corrupt",
            )?;
            let revision = text(&row, 4)?;
            self.verify(
                &revision,
                json!({"edge_id":id,"kind":"CONSTRAINS","from_record_revision":from.revision,
                "selector_revision":xrev,"provenance_evidence_revision":evidence.revision}),
            )?;
            let edge = RelationRow {
                id: id.clone(),
                revision,
                kind: "CONSTRAINS".into(),
                from: from.data.id.clone(),
                from_revision: from.revision.clone(),
                to: selector.into(),
                to_revision: xrev,
                evidence: evidence.id,
                gate: None,
            };
            self.bindings.insert(id, edge.clone());
            output.push(edge);
        }
        Ok(output)
    }
    pub fn relations(
        &mut self,
        id: &str,
        incoming: bool,
        kinds: &[&str],
    ) -> Result<Vec<RelationRow>> {
        let direction = if incoming { "dst" } else { "src" };
        let placeholders = (5..5 + kinds.len())
            .map(|n| format!("?{n}"))
            .collect::<Vec<_>>()
            .join(",");
        let query = format!(
            "SELECT * FROM relation WHERE p=?1 AND c=?2 AND g=?3 AND {direction}=?4 AND kind IN ({placeholders}) ORDER BY kind,src,dst,aid"
        );
        let mut args = vec![Sql::Text(id.into())];
        args.extend(kinds.iter().map(|s| Sql::Text((*s).into())));
        let rows = self.scan(&query, args, self.request.limits.edges)?;
        let mut output = Vec::new();
        for row in rows {
            let aid = text(&row, 3)?;
            if let Some(edge) = self.relations.get(&aid) {
                output.push(edge.clone());
                continue;
            }
            self.unit("relation", &aid, true)?;
            self.counts.relations += 1;
            let from = self.record(&text(&row, 6)?)?;
            let to = self.record(&text(&row, 8)?)?;
            let evidence = self.evidence(&text(&row, 10)?)?;
            require(
                text(&row, 7)? == from.revision
                    && text(&row, 9)? == to.revision
                    && text(&row, 11)? == evidence.revision,
                "projection_corrupt",
            )?;
            let gate: Option<Gate> = nullable(&row, 12)?
                .map(|s| identity::parse(s.as_bytes()).and_then(decode))
                .transpose()?;
            let revision = text(&row, 4)?;
            let kind = text(&row, 5)?;
            require(
                (kind == "BLOCKED_BY") == gate.is_some(),
                "projection_corrupt",
            )?;
            self.verify(&revision, json!({"edge_id":aid,"kind":kind,"from_record_revision":from.revision,
                "to_record_revision":to.revision,"gate":gate,"provenance_evidence_revision":evidence.revision}))?;
            let edge = RelationRow {
                id: aid.clone(),
                revision,
                kind,
                from: from.data.id.clone(),
                from_revision: from.revision.clone(),
                to: to.data.id.clone(),
                to_revision: to.revision.clone(),
                evidence: evidence.id,
                gate,
            };
            self.relations.insert(aid, edge.clone());
            output.push(edge);
        }
        Ok(output)
    }
    pub fn peers(&mut self, record: &Record) -> Result<Vec<String>> {
        let rows = self.scan("SELECT rid FROM record_revision WHERE p=?1 AND c=?2 AND g=?3 AND subject=?4 AND role=?5 AND lifecycle='active' ORDER BY rid",
            vec![Sql::Text(record.subject.clone()), Sql::Text(record.role.clone())], self.request.limits.records)?;
        rows.iter().map(|r| text(r, 0)).collect()
    }
    pub fn current_sources(&mut self) -> Result<Vec<Source>> {
        let rows = self.scan("SELECT sid FROM source_revision WHERE p=?1 AND c=?2 AND g=?3 AND check_mode='current' ORDER BY sid", vec![], self.trusted.profile.limits.records)?;
        let commit: String = sql(self.db.query_row(
            "SELECT source_commit FROM generation WHERE p=? AND c=? AND g=?",
            params![
                self.trusted.profile.partition_id,
                self.request.corpus,
                self.request.generation
            ],
            |r| r.get(0),
        ))?;
        let mut output = Vec::new();
        for row in rows {
            let source = self.source(&text(&row, 0)?)?;
            require(
                source.data.commit == commit && source.data.check == "current",
                "projection_corrupt",
            )?;
            output.push(source.data);
        }
        Ok(output)
    }
    pub fn quote(
        &self,
        evidence: &EvidenceRow,
        work: &mut super::profile::Work,
    ) -> Result<Vec<u8>> {
        let length: i64 = sql(self.db.query_row(
            "SELECT length(quote_utf8) FROM evidence_revision WHERE p=? AND c=? AND g=? AND eid=?",
            params![
                self.trusted.profile.partition_id,
                self.request.corpus,
                self.request.generation,
                evidence.id
            ],
            |r| r.get(0),
        ))?;
        require(
            length >= 0 && length as u64 == evidence.end - evidence.start,
            "projection_corrupt",
        )?;
        require(
            length as u64 <= work.ceiling.saturating_sub(work.bytes),
            "work_limit",
        )?;
        work.debit(length as u64)?;
        sql(self.db.query_row(
            "SELECT quote_utf8 FROM evidence_revision WHERE p=? AND c=? AND g=? AND eid=?",
            params![
                self.trusted.profile.partition_id,
                self.request.corpus,
                self.request.generation,
                evidence.id
            ],
            |r| r.get(0),
        ))
    }
}
