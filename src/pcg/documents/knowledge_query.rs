use super::{knowledge::Request, knowledge_store, scan::Capture, store::sql, traversal};
use crate::pcg::{Error, Result, identity, require};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn metadata(c: &Connection, g: &str, id: &str) -> Result<Value> {
    let row: Option<(String,String,String,String,Option<String>)>=sql(c.query_row(
        "SELECT m.revision,r.path,r.source_sha,r.declaration,u.scope FROM document_member m JOIN document_revision r USING(revision) LEFT JOIN knowledge_unit u USING(revision) WHERE m.g=? AND m.id=?",
        params![g,id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))).optional())?;
    let (revision, path, source_sha, declaration, scope) =
        row.ok_or(Error("document_not_found"))?;
    let declaration: Value =
        serde_json::from_str(&declaration).map_err(|_| Error("document_corruption"))?;
    Ok(
        json!({"id":id,"revision":revision,"path":path,"source_sha256":source_sha,"scope":scope,
        "title":declaration["title"],"kind":declaration["kind"],"status":declaration["status"],"track":declaration["track"],
        "note":declaration["note"],"verdict":declaration["verdict"],"declared_commit":declaration["commit"],"declared_relationships":declaration["relationships"],"source_request":{"mode":"source","id":id,"sourceSha256":source_sha}}),
    )
}
fn describe(c: &Connection, mut unit: Value, q: &Request) -> Result<Value> {
    let revision = unit["revision"]
        .as_str()
        .ok_or(Error("knowledge_corruption"))?;
    let mut components = knowledge_store::components(c, revision)?;
    let selected = |v: &Value| {
        q.role
            .as_ref()
            .is_none_or(|r| v["role"].as_str() == Some(r))
            && q.label
                .as_ref()
                .is_none_or(|r| v["label"].as_str() == Some(r))
    };
    let has_requested = components.iter().any(selected);
    components.retain(|v| selected(v) || (has_requested && v["role"] == "qualification"));
    for component in &mut components {
        component["selected_by_predicate"] = selected(component).into();
        let fields = component
            .as_object_mut()
            .ok_or(Error("knowledge_corruption"))?;
        let source = fields
            .remove("source_sha")
            .ok_or(Error("knowledge_corruption"))?;
        let content = fields
            .remove("content_sha")
            .ok_or(Error("knowledge_corruption"))?;
        fields.insert("source_sha256".into(), source);
        fields.insert("content_sha256".into(), content);
    }
    let evidence: String = sql(c.query_row(
        "SELECT evidence FROM document_revision WHERE revision=?",
        [revision],
        |r| r.get(0),
    ))?;
    let mut evidence: Value =
        serde_json::from_str(&evidence).map_err(|_| Error("document_corruption"))?;
    for ev in evidence
        .as_array_mut()
        .ok_or(Error("document_corruption"))?
    {
        ev.as_object_mut()
            .ok_or(Error("document_corruption"))?
            .remove("text");
        ev["source_request"] =
            json!({"mode":"source","id":unit["id"],"path":ev["path"],"sourceSha256":ev["sha256"]});
    }
    unit["components"] = components.into();
    unit["evidence"] = evidence;
    unit["unit_complete"] = (q.role.is_none() && q.label.is_none()).into();
    unit["component_selection"] =
        json!({"role":q.role,"label":q.label,"includes_required_qualifications":true});
    Ok(unit)
}
fn missing(payload: &mut Value, ids: Vec<Value>) {
    payload["result_state"] = "incomplete".into();
    payload["complete"] = false.into();
    payload["units"] = json!([]);
    payload["relationships"] = json!([]);
    payload["paths"] = json!([]);
    payload["diagnostics"] = json!([{"code":"E_KNOWLEDGE_UNDECLARED","identities":ids}]);
}

fn search(c: &Connection, g: &str, q: &Request) -> Result<Vec<Value>> {
    let mut selected = BTreeMap::<String, Vec<Value>>::new();
    let statement = "SELECT m.id,k.ordinal,k.origin,k.label,x.path,x.source_sha,x.start,x.end,x.content_sha FROM knowledge_search k JOIN document_member m ON m.id=k.id AND m.revision=k.revision JOIN knowledge_unit u ON u.revision=m.revision LEFT JOIN knowledge_component x ON x.revision=k.revision AND x.ordinal=k.ordinal WHERE m.g=?1 AND (?2 IS NULL OR u.track=?2) AND (?3 IS NULL OR u.kind=?3) AND (?4 IS NULL OR u.status=?4) AND (?5 IS NULL OR k.origin=?5) AND (?6 IS NULL OR k.label=?6)";
    let query = if q.text.is_some() {
        format!("{statement} AND k.text MATCH ?7 ORDER BY m.id,k.ordinal")
    } else {
        format!("{statement} AND (?7 IS NULL) ORDER BY m.id,k.ordinal")
    };
    let mut s = sql(c.prepare(&query))?;
    let rows=sql(s.query_map(params![g,q.track,q.kind,q.status,q.role,q.label,q.text],|r|Ok((r.get::<_,String>(0)?,json!({
        "ordinal":r.get::<_,i64>(1)?,"origin":r.get::<_,String>(2)?,"label":r.get::<_,Option<String>>(3)?,
        "path":r.get::<_,Option<String>>(4)?,"source_sha256":r.get::<_,Option<String>>(5)?,
        "start":r.get::<_,Option<usize>>(6)?,"end":r.get::<_,Option<usize>>(7)?,"content_sha256":r.get::<_,Option<String>>(8)?
    })))))?;
    for row in rows {
        let (id, origin) = sql(row)?;
        selected.entry(id).or_default().push(origin);
    }
    let mut answer = Vec::new();
    for (id, mut matches) in selected {
        if matches.len() > 1 {
            matches.retain(|m| m["ordinal"] != -2);
        }
        let mut unit = metadata(c, g, &id)?;
        unit["matches"] = matches.into();
        unit["acquire_request"] = json!({"mode":"get","id":id});
        answer.push(unit);
    }
    Ok(answer)
}

pub fn execute(
    c: &Connection,
    g: &str,
    q: &Request,
    capture: &Capture,
    max_nodes: usize,
    max_edges: usize,
    payload: &mut Value,
) -> Result<()> {
    if q.mode == "search" {
        let mut units = Vec::new();
        for unit in search(c, g, q)? {
            units.push(describe(c, unit, q)?);
        }
        payload["units"] = units.into();
        return Ok(());
    }
    if q.mode == "source" && q.historical {
        return super::knowledge_source::historical(c, q, payload);
    }
    let id = q.id.as_deref().ok_or(Error("invalid_request"))?;
    let unit = metadata(c, g, id)?;
    if q.mode == "source" {
        require(
            q.revision
                .as_ref()
                .is_none_or(|revision| unit["revision"].as_str() == Some(revision.as_str())),
            "source_revision_changed",
        )?;
        let node = capture.nodes.get(id).ok_or(Error("document_not_found"))?;
        let path = q.path.as_deref().unwrap_or(&node.path);
        require(
            node.evidence
                .iter()
                .any(|e| e["path"].as_str() == Some(path)),
            "scope_refused",
        )?;
        let bytes = capture.source(path)?;
        // Detection view only: returned original bytes are never normalized.
        crate::pcg::security::nonsecret(&String::from_utf8_lossy(bytes))?;
        let hash = identity::sha(bytes);
        require(
            q.source_sha256
                .as_ref()
                .is_none_or(|expected| expected == &hash),
            "source_revision_changed",
        )?;
        let source = match std::str::from_utf8(bytes) {
            Ok(text) => {
                json!({"path":path,"source_sha256":hash,"encoding":"utf8","text":text})
            }
            Err(_) => {
                json!({"path":path,"source_sha256":hash,"encoding":"byte_array","bytes":bytes})
            }
        };
        payload["units"] = json!([unit]);
        payload["sources"] = json!([source]);
        return Ok(());
    }
    if unit["scope"].is_null() {
        missing(payload, vec![id.into()]);
        return Ok(());
    }
    match q.mode.as_str() {
        "get" => {
            payload["relationships"]=unit["declared_relationships"].as_array().ok_or(Error("knowledge_corruption"))?.iter().map(|edge| {
                json!({"identity":[id,edge["kind"],edge["target"]],"source":id,"kind":edge["kind"],"target":edge["target"],
                    "direction":"outgoing","meaning":traversal::meaning(edge["kind"].as_str().unwrap_or("")),
                    "provenance":{"path":unit["path"],"source_sha256":unit["source_sha256"]}})
            }).collect::<Vec<_>>().into();
            payload["units"] = json!([describe(c, unit, q)?]);
        }
        "related" => {
            let direction = q.direction.as_deref().unwrap_or("both");
            let mut s=sql(c.prepare("SELECT source,kind,target,evidence FROM document_edge WHERE g=?1 AND (?3 IS NULL OR kind=?3) AND ((source=?2 AND ?4 IN ('outgoing','both')) OR (target=?2 AND ?4 IN ('incoming','both'))) ORDER BY source,kind,target"))?;
            let rows = sql(s.query_map(params![g, id, q.edge_kind, direction], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                ))
            }))?;
            let mut endpoints = BTreeMap::from([(id.to_string(), unit)]);
            let mut edges = Vec::new();
            for row in rows {
                let (a, kind, b, provenance) = sql(row)?;
                for endpoint in [&a, &b] {
                    if !endpoints.contains_key(endpoint) {
                        endpoints.insert(endpoint.clone(), metadata(c, g, endpoint)?);
                    }
                }
                edges.push(json!({"identity":[a,kind,b],"source":a,"kind":kind,"target":b,"direction":if a==id {"outgoing"}else{"incoming"},
                    "meaning":traversal::meaning(&kind),"provenance":serde_json::from_str::<Value>(&provenance).map_err(|_|Error("document_corruption"))?}));
            }
            let missing_ids: Vec<_> = endpoints
                .values()
                .filter(|u| u["scope"].is_null())
                .map(|u| u["id"].clone())
                .collect();
            if !missing_ids.is_empty() {
                missing(payload, missing_ids);
                return Ok(());
            }
            payload["units"] = endpoints
                .into_values()
                .map(|unit| describe(c, unit, q))
                .collect::<Result<Vec<_>>>()?
                .into();
            payload["relationships"] = edges.into();
        }
        "context" => {
            let closure = traversal::collect_membership(c, g, id, max_nodes, max_edges)?;
            if !closure.complete {
                payload["complete"] = false.into();
                payload["result_state"] = "incomplete".into();
                payload["diagnostics"] = json!([{"code":"E_CONTEXT_LIMIT","message":"No partial authoritative context returned"}]);
                return Ok(());
            }
            let mut units = Vec::new();
            let mut missing_ids = Vec::new();
            for (identity, _) in closure.revisions {
                let unit = metadata(c, g, &identity)?;
                if unit["scope"].is_null() {
                    missing_ids.push(identity.into());
                } else {
                    units.push(describe(c, unit, q)?);
                }
            }
            if !missing_ids.is_empty() {
                missing(payload, missing_ids);
                return Ok(());
            }
            payload["units"] = units.into();
            payload["relationships"] = closure.relationships.into();
            payload["paths"] = closure.paths.into();
        }
        _ => return Err(Error("invalid_request")),
    }
    Ok(())
}
