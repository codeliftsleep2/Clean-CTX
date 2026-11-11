//! Lazy immutable object rows and their exact normalized revision checks.
use super::{EvidenceRow, Reader, RecordRow, SourceRow, boolean, decode, integer, text};
use crate::pcg::{
    Error, Result, identity,
    model::{EvidenceRef, Record, Source},
    require, security,
};
use rusqlite::types::Value as Sql;
use serde_json::json;
use std::rc::Rc;

impl Reader<'_> {
    pub fn source(&mut self, id: &str) -> Result<SourceRow> {
        if let Some(row) = self.sources.get(id) {
            return Ok(row.clone());
        }
        let row = self.one("source_revision", "sid", id)?;
        let data = Source {
            id: text(&row, 3)?,
            path: text(&row, 5)?,
            commit: text(&row, 6)?,
            blob: text(&row, 7)?,
            sha256: text(&row, 8)?,
            check: text(&row, 9)?,
        };
        require(
            data.id == id && ["current", "historical"].contains(&data.check.as_str()),
            "projection_corrupt",
        )?;
        security::relative(&data.path)?;
        let revision = text(&row, 4)?;
        self.verify(
            &revision,
            json!({"source_id":data.id,"path":data.path,"commit":data.commit,
            "blob":data.blob,"sha256":data.sha256,"check":data.check}),
        )?;
        let row = SourceRow { data, revision };
        self.sources.insert(id.into(), row.clone());
        Ok(row)
    }
    pub fn evidence(&mut self, id: &str) -> Result<EvidenceRow> {
        if let Some(row) = self.evidence.get(id) {
            return Ok(row.clone());
        }
        // Deliberately exclude quote_utf8 until required witness validation.
        let mut rows = self.scan("SELECT p,c,g,eid,erev,sid,srev,bstart,bend,quote_sha FROM evidence_revision WHERE p=?1 AND c=?2 AND g=?3 AND eid=?4",
            vec![Sql::Text(id.into())], 1)?;
        let row = rows.pop().ok_or(Error("projection_corrupt"))?;
        let source = self.source(&text(&row, 5)?)?;
        let result = EvidenceRow {
            id: text(&row, 3)?,
            revision: text(&row, 4)?,
            source: source.data.id,
            source_revision: text(&row, 6)?,
            start: integer(&row, 7)?,
            end: integer(&row, 8)?,
            hash: text(&row, 9)?,
        };
        require(
            result.id == id
                && result.source_revision == source.revision
                && result.end > result.start,
            "projection_corrupt",
        )?;
        self.verify(
            &result.revision,
            json!({"evidence_id":result.id,"source_revision":result.source_revision,
            "start":result.start,"end":result.end,"quote_sha256":result.hash}),
        )?;
        self.evidence.insert(id.into(), result.clone());
        Ok(result)
    }
    pub fn record(&mut self, id: &str) -> Result<Rc<RecordRow>> {
        if let Some(row) = self.records.get(id) {
            return Ok(Rc::clone(row));
        }
        self.unit("record", id, false)?;
        self.counts.records += 1;
        let row = self.one("record_revision", "rid", id)?;
        let primary = self.evidence(&text(&row, 7)?)?;
        require(primary.revision == text(&row, 8)?, "projection_corrupt")?;
        let refs = self.scan(
            "SELECT * FROM evidence_ref WHERE p=?1 AND c=?2 AND g=?3 AND rid=?4 ORDER BY eid",
            vec![Sql::Text(id.into())],
            self.request.limits.edges,
        )?;
        let revision = text(&row, 4)?;
        let mut evidence_refs = Vec::new();
        let mut hashes = Vec::new();
        for reference in refs {
            let eid = text(&reference, 5)?;
            self.unit("ref", &format!("{id}:{eid}"), true)?;
            self.counts.refs += 1;
            let target = self.evidence(&eid)?;
            require(
                text(&reference, 4)? == revision && text(&reference, 6)? == target.revision,
                "projection_corrupt",
            )?;
            let required = boolean(&reference, 7)?;
            hashes.push(json!({"evidence_revision":target.revision,"required":required}));
            evidence_refs.push(EvidenceRef { id: eid, required });
        }
        let data: Record = decode(
            json!({"id":text(&row,3)?,"role":text(&row,5)?,"required":boolean(&row,6)?,
            "primary":primary.id,"authority":{"class":text(&row,9)?,"attributed_to":text(&row,10)?},
            "verification":{"outcome":text(&row,11)?,"subject":text(&row,12)?,"execution":text(&row,13)?},
            "lifecycle":text(&row,14)?,"subject":text(&row,15)?,"disposition":text(&row,16)?,"evidence_refs":evidence_refs}),
        )?;
        require(data.id == id, "projection_corrupt")?;
        self.verify(&revision, json!({"record_id":data.id,"role":data.role,"required":data.required,
            "primary_evidence_revision":primary.revision,"authority":data.authority,"verification":data.verification,
            "lifecycle":data.lifecycle,"subject":data.subject,"disposition":data.disposition,"sorted_evidence_refs":identity::set(hashes)}))?;
        let row = Rc::new(RecordRow { data, revision });
        self.records.insert(id.into(), Rc::clone(&row));
        Ok(row)
    }
}
