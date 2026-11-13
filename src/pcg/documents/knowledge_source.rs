//! Explicit retained whole-artifact acquisition; never substitutes current bytes.
use super::{knowledge::Request, store::sql};
use crate::pcg::{Error, Result, identity, require};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::{Value, json};

pub(super) fn historical(c: &Connection, q: &Request, payload: &mut Value) -> Result<()> {
    let id = q.id.as_deref().ok_or(Error("invalid_request"))?;
    let revision = q.revision.as_deref().ok_or(Error("invalid_request"))?;
    let row: Option<(String,String,String)> = sql(c.query_row(
        "SELECT r.path,r.source_sha,r.evidence FROM document_revision r WHERE r.revision=?1 AND EXISTS(SELECT 1 FROM document_member m WHERE m.id=?2 AND m.revision=r.revision)",
        params![revision,id],|row|Ok((row.get(0)?,row.get(1)?,row.get(2)?)),
    ).optional())?;
    let (declaring_path, declaring_sha, evidence) = row.ok_or(Error("source_reference_invalid"))?;
    let evidence: Vec<Value> =
        serde_json::from_str(&evidence).map_err(|_| Error("document_corruption"))?;
    let path = q.path.as_deref().unwrap_or(&declaring_path);
    let association = evidence
        .iter()
        .find(|e| e["path"].as_str() == Some(path))
        .ok_or(Error("scope_refused"))?;
    let sha = association["sha256"]
        .as_str()
        .ok_or(Error("document_corruption"))?;
    require(
        q.source_sha256.as_ref().is_none_or(|pin| pin == sha),
        "source_revision_changed",
    )?;
    let stored: Option<Vec<u8>> = sql(c
        .query_row(
            "SELECT body FROM document_source WHERE path=? AND sha=?",
            params![path, sha],
            |row| row.get(0),
        )
        .optional())?;
    let bytes = stored
        .or_else(|| {
            association["text"]
                .as_str()
                .map(|text| text.as_bytes().to_vec())
        })
        .ok_or(Error("document_source_unavailable"))?;
    require(identity::sha(&bytes) == sha, "document_corruption")?;
    crate::pcg::security::nonsecret(&String::from_utf8_lossy(&bytes))?;
    let source = match std::str::from_utf8(&bytes) {
        Ok(text) => json!({"path":path,"source_sha256":sha,"encoding":"utf8","text":text}),
        Err(_) => json!({"path":path,"source_sha256":sha,"encoding":"byte_array","bytes":bytes}),
    };
    payload["units"] = json!([{"id":id,"revision":revision,"path":declaring_path,"source_sha256":declaring_sha,"freshness":"historical_snapshot"}]);
    payload["sources"] = json!([source]);
    payload["freshness"] = "historical_snapshot".into();
    payload["complete"] = true.into();
    payload["result_state"] = "ready".into();
    Ok(())
}
