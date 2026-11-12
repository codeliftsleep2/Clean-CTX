//! Bounded pre-authoring context. Reachability never promotes descriptive edges to premises.
use super::store;
use crate::pcg::{Error, Result};
use rusqlite::{Connection, params};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub struct Context {
    pub complete: bool,
    pub documents: Vec<Value>,
    pub relationships: Vec<Value>,
    pub paths: Vec<Value>,
}
pub struct Membership {
    pub complete: bool,
    pub revisions: Vec<(String, String)>,
    pub relationships: Vec<Value>,
    pub paths: Vec<Value>,
}
pub(super) fn meaning(kind: &str) -> &'static str {
    match kind {
        "DEPENDS_ON" => "necessary_premise",
        "BLOCKED_BY" => "blocker_satisfied_only_by_reclosed_or_banked",
        "SUPERSEDES" => "controlling_interpretation_not_falsification",
        "FALSIFIES" => "counterevidence_not_erasure",
        "IMPLEMENTS" => "realization_not_validation",
        "USES_EVIDENCE" => "informative_not_prerequisite",
        "PRESERVES" => "respects_truth_not_prerequisite",
        "VALIDATES" => "support_not_reclosure",
        "RECLOSES" => "reclosure_not_banking",
        "BANKS" => "source_declared_banking_not_authenticated_approval",
        "DEEPENS" => "extension_not_replacement",
        "PRODUCES_AUTHORITY" => "source_declared_authority_establishment",
        "CONSUMES_AUTHORITY" => "authority_read_not_ownership",
        _ => "unknown",
    }
}
pub fn collect(
    c: &Connection,
    g: &str,
    seed: &str,
    max_nodes: usize,
    max_edges: usize,
) -> Result<Context> {
    let membership = collect_membership(c, g, seed, max_nodes, max_edges)?;
    let mut documents = Vec::new();
    for (_, revision) in membership.revisions {
        let mut value = store::revision(c, &revision)?;
        value["freshness"] = "current_observed_boundaries".into();
        documents.push(value);
    }
    Ok(Context {
        complete: membership.complete,
        documents,
        relationships: membership.relationships,
        paths: membership.paths,
    })
}

pub fn collect_membership(
    c: &Connection,
    g: &str,
    seed: &str,
    max_nodes: usize,
    max_edges: usize,
) -> Result<Membership> {
    let mut frontier = VecDeque::from([seed.to_string()]);
    let mut paths = BTreeMap::<String, Vec<Value>>::from([(seed.into(), Vec::new())]);
    let mut seen = BTreeSet::new();
    let mut documents = Vec::new();
    let mut edges = BTreeMap::new();
    let mut complete = true;
    while let Some(id) = frontier.pop_front() {
        if !seen.insert(id.clone()) {
            continue;
        }
        if seen.len() > max_nodes {
            complete = false;
            break;
        }
        let rev: Option<String> = store::sql(
            c.query_row(
                "SELECT revision FROM document_member WHERE g=? AND id=?",
                params![g, id],
                |r| r.get(0),
            )
            .optional(),
        )?;
        let Some(rev) = rev else {
            return Err(Error("document_not_found"));
        };
        documents.push((id.clone(), rev));
        let mut s=store::sql(c.prepare("SELECT source,kind,target,evidence FROM document_edge WHERE g=?1 AND (source=?2 OR (target=?2 AND kind IN ('SUPERSEDES','FALSIFIES','VALIDATES','RECLOSES','BANKS'))) ORDER BY kind,source,target"))?;
        let rows = store::sql(s.query_map(params![g, id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
            ))
        }))?;
        for row in rows {
            let (source, kind, target, provenance) = store::sql(row)?;
            let key = (source.clone(), kind.clone(), target.clone());
            if !edges.contains_key(&key) && edges.len() >= max_edges {
                complete = false;
                break;
            }
            let provenance: Value =
                serde_json::from_str(&provenance).map_err(|_| Error("document_corruption"))?;
            let edge = json!({"source":source,"kind":kind,"target":target,"meaning":meaning(&kind),"provenance":provenance});
            edges.insert(key, edge);
            let next = if source == id { &target } else { &source };
            if !paths.contains_key(next) {
                if paths.len() >= max_nodes {
                    complete = false;
                    break;
                }
                let mut path = paths[&id].clone();
                path.push(json!({"source":source,"kind":kind,"target":target,"direction":if source==id {"outgoing"}else{"incoming"}}));
                paths.insert(next.clone(), path);
                frontier.push_back(next.clone());
            }
        }
        if !complete {
            break;
        }
    }
    if !complete {
        return Ok(Membership {
            complete: false,
            revisions: Vec::new(),
            relationships: Vec::new(),
            paths: Vec::new(),
        });
    }
    documents.sort();
    Ok(Membership {
        complete,
        revisions: documents,
        relationships: edges.into_values().collect(),
        paths: paths
            .into_iter()
            .map(|(node, steps)| json!({"node":node,"steps":steps}))
            .collect(),
    })
}

use rusqlite::OptionalExtension;
