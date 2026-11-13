//! Explicit revision-pinned component acquisition; never graph traversal.
use super::{knowledge::Reference, knowledge_store, store::sql};
use crate::pcg::{Error, Result, identity, require};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Clone, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ComponentReference {
    pub id: String,
    pub revision: String,
    pub ordinal: usize,
    pub role: String,
    pub label: Option<String>,
    pub path: String,
    pub source_sha256: String,
    pub start: usize,
    pub end: usize,
    pub content_sha256: String,
    pub selector: String,
    #[serde(default)]
    pub historical: bool,
}

pub(super) fn reference(id: &Value, revision: &Value, component: &Value) -> Value {
    json!({"id":id,"revision":revision,"ordinal":component["ordinal"],
        "role":component["role"],"label":component["label"],"path":component["path"],
        "sourceSha256":component["source_sha256"],"start":component["start"],"end":component["end"],
        "contentSha256":component["content_sha256"],"selector":component["selector"],"historical":false})
}

#[derive(Default)]
struct Acquisition {
    revisions: BTreeMap<(String, String, bool), Vec<Value>>,
    sources: BTreeMap<(String, String), Vec<u8>>,
    components: BTreeMap<(String, usize), Value>,
    intervals: BTreeMap<(String, String, usize, usize), (String, String)>,
}

fn resolve(
    c: &Connection,
    generation: &str,
    r: &Reference,
    work: &mut Acquisition,
) -> Result<Value> {
    identity::id(&r.id)?;
    let revision_key = (r.id.clone(), r.revision.clone(), r.historical);
    if !work.revisions.contains_key(&revision_key) {
        // Revision membership is checked independently from caller-provided descriptors.
        let exists: bool = sql(c.query_row(
            "SELECT EXISTS(SELECT 1 FROM document_member WHERE id=? AND revision=?)",
            params![r.id, r.revision],
            |row| row.get(0),
        ))?;
        require(exists, "source_reference_invalid")?;
        if !r.historical {
            let current: Option<String> = sql(c
                .query_row(
                    "SELECT revision FROM document_member WHERE g=? AND id=?",
                    params![generation, r.id],
                    |row| row.get(0),
                )
                .optional())?;
            require(
                current.as_deref() == Some(r.revision.as_str()),
                "source_revision_changed",
            )?;
        }
        work.revisions.insert(
            revision_key.clone(),
            knowledge_store::components(c, &r.revision)?,
        );
    }
    let components = work
        .revisions
        .get(&revision_key)
        .ok_or(Error("knowledge_corruption"))?;
    let selected = components
        .get(r.ordinal)
        .ok_or(Error("source_reference_invalid"))?;
    let expected = ComponentReference {
        id: r.id.clone(),
        revision: r.revision.clone(),
        ordinal: r.ordinal,
        role: selected["role"]
            .as_str()
            .ok_or(Error("knowledge_corruption"))?
            .into(),
        label: selected["label"].as_str().map(str::to_owned),
        path: selected["path"]
            .as_str()
            .ok_or(Error("knowledge_corruption"))?
            .into(),
        source_sha256: selected["source_sha"]
            .as_str()
            .ok_or(Error("knowledge_corruption"))?
            .into(),
        start: selected["start"]
            .as_u64()
            .ok_or(Error("knowledge_corruption"))? as usize,
        end: selected["end"]
            .as_u64()
            .ok_or(Error("knowledge_corruption"))? as usize,
        content_sha256: selected["content_sha"]
            .as_str()
            .ok_or(Error("knowledge_corruption"))?
            .into(),
        selector: selected["selector"]
            .as_str()
            .ok_or(Error("knowledge_corruption"))?
            .into(),
        historical: r.historical,
    };
    require(r == &expected, "source_reference_invalid")?;
    let mut acquired = Vec::new();
    for component in components
        .iter()
        .filter(|v| v["ordinal"] == r.ordinal || v["role"] == "qualification")
    {
        let component_key = (
            r.revision.clone(),
            component["ordinal"]
                .as_u64()
                .ok_or(Error("knowledge_corruption"))? as usize,
        );
        if let Some(value) = work.components.get(&component_key) {
            let mut value = value.clone();
            value["selected_by_reference"] = (component["ordinal"] == r.ordinal).into();
            acquired.push(value);
            continue;
        }
        let source_key = (
            component["path"]
                .as_str()
                .ok_or(Error("knowledge_corruption"))?
                .to_owned(),
            component["source_sha"]
                .as_str()
                .ok_or(Error("knowledge_corruption"))?
                .to_owned(),
        );
        if !work.sources.contains_key(&source_key) {
            let body: Vec<u8> = sql(c.query_row(
                "SELECT body FROM document_source WHERE path=? AND sha=?",
                params![component["path"].as_str(), component["source_sha"].as_str()],
                |row| row.get(0),
            ))?;
            require(identity::sha(&body) == source_key.1, "knowledge_corruption")?;
            work.sources.insert(source_key.clone(), body);
        }
        let body = work
            .sources
            .get(&source_key)
            .ok_or(Error("knowledge_corruption"))?;
        let start = component["start"]
            .as_u64()
            .ok_or(Error("knowledge_corruption"))? as usize;
        let end = component["end"]
            .as_u64()
            .ok_or(Error("knowledge_corruption"))? as usize;
        let interval_key = (source_key.0.clone(), source_key.1.clone(), start, end);
        if !work.intervals.contains_key(&interval_key) {
            let bytes = body.get(start..end).ok_or(Error("knowledge_corruption"))?;
            let hash = identity::sha(bytes);
            require(
                hash == component["content_sha"]
                    .as_str()
                    .ok_or(Error("knowledge_corruption"))?,
                "knowledge_corruption",
            )?;
            let text = std::str::from_utf8(bytes).map_err(|_| Error("knowledge_corruption"))?;
            crate::pcg::security::nonsecret(text)?;
            work.intervals
                .insert(interval_key.clone(), (hash, text.to_owned()));
        }
        let (hash, text) = work
            .intervals
            .get(&interval_key)
            .ok_or(Error("knowledge_corruption"))?;
        require(
            Some(hash.as_str()) == component["content_sha"].as_str(),
            "knowledge_corruption",
        )?;
        let mut value = component.clone();
        let fields = value.as_object_mut().ok_or(Error("knowledge_corruption"))?;
        let source = fields
            .remove("source_sha")
            .ok_or(Error("knowledge_corruption"))?;
        let content = fields
            .remove("content_sha")
            .ok_or(Error("knowledge_corruption"))?;
        fields.insert("source_sha256".into(), source);
        fields.insert("content_sha256".into(), content);
        value["text"] = text.as_str().into();
        value["selected_by_reference"] = (component["ordinal"] == r.ordinal).into();
        work.components.insert(component_key, value.clone());
        acquired.push(value);
    }
    Ok(
        json!({"reference":r,"result_state":"ready","complete":true,"freshness":if r.historical {"historical_snapshot"}else{"observed_boundaries_only"},"components":acquired}),
    )
}

pub(super) fn execute(
    c: &Connection,
    generation: &str,
    references: &[Reference],
    payload: &mut Value,
) -> Result<()> {
    let mut resolved = BTreeMap::<String, Value>::new();
    let mut results = Vec::new();
    let mut complete = true;
    let mut work = Acquisition::default();
    for r in references {
        let key = serde_json::to_string(r).map_err(|_| Error("invalid_request"))?;
        if let Some(result) = resolved.get(&key) {
            let mut result = result.clone();
            result["duplicate_of"] = results
                .iter()
                .position(|value: &Value| value["reference"] == result["reference"])
                .into();
            results.push(result);
            continue;
        }
        let value = match resolve(c, generation, r, &mut work) {
            Ok(value) => value,
            Err(error) => {
                // Database/security failures remain operation failures, never hidden partial data.
                if ![
                    "source_reference_invalid",
                    "source_revision_changed",
                    "invalid_id",
                ]
                .contains(&error.0)
                {
                    return Err(error);
                }
                complete = false;
                json!({"reference":r,"result_state":"refused","complete":false,"components":[],"diagnostics":[{"code":error.0}]})
            }
        };
        resolved.insert(key, value.clone());
        results.push(value);
    }
    payload["acquisitions"] = results.into();
    #[cfg(test)]
    {
        payload["acquisition_work"] = json!({"revision_sets":work.revisions.len(),"sources":work.sources.len(),"components":work.components.len(),"intervals":work.intervals.len()});
    }
    payload["complete"] = complete.into();
    payload["result_state"] = if complete { "ready" } else { "incomplete" }.into();
    Ok(())
}
