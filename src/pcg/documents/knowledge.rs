//! First-class exact knowledge interface. Legacy whole-source operations stay separate.
pub(crate) use super::content::ROLES;
pub(crate) use super::model::{KINDS, RELATIONS, STATUSES};
use super::{Session, knowledge_query, model, scan, store, validate};
use crate::pcg::{Error, Result, identity, require};
use rusqlite::TransactionBehavior;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::path::PathBuf;

#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Request {
    pub workspace_root: PathBuf,
    pub mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edge_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_nodes: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_edges: Option<usize>,
}
impl Request {
    fn completeness_scope(&self) -> &'static str {
        match self.mode.as_str() {
            "search" => "discovery_over_all_declared_units",
            "get" if self.role.is_some() => "requested_components_with_required_qualifications",
            "get" => "complete_declared_knowledge_unit",
            "related" => "requested_typed_incident_relationships_and_endpoint_metadata",
            "context" => "complete_declared_transitive_authority",
            "source" => "complete_original_source_artifact",
            _ => unreachable!("validated operation"),
        }
    }
    fn validate(&self) -> Result<()> {
        require(
            ["search", "get", "related", "context", "source"].contains(&self.mode.as_str()),
            "invalid_request",
        )?;
        if self.mode == "search" {
            require(self.id.is_none(), "invalid_request")?;
        } else {
            identity::id(self.id.as_deref().ok_or(Error("invalid_request"))?)?;
        }
        for value in [
            &self.text,
            &self.track,
            &self.kind,
            &self.status,
            &self.role,
            &self.label,
            &self.edge_kind,
            &self.direction,
            &self.path,
        ]
        .into_iter()
        .flatten()
        {
            require(!value.trim().is_empty(), "invalid_request")?;
        }
        require(
            self.mode == "search"
                || (self.text.is_none()
                    && self.track.is_none()
                    && self.kind.is_none()
                    && self.status.is_none()),
            "invalid_request",
        )?;
        require(
            ["search", "get"].contains(&self.mode.as_str())
                || (self.role.is_none() && self.label.is_none()),
            "invalid_request",
        )?;
        require(
            self.label.is_none() || self.role.is_some(),
            "invalid_request",
        )?;
        require(
            self.mode == "related" || (self.edge_kind.is_none() && self.direction.is_none()),
            "invalid_request",
        )?;
        require(
            self.mode == "source" || (self.path.is_none() && self.source_sha256.is_none()),
            "invalid_request",
        )?;
        require(
            self.source_sha256.as_ref().is_none_or(|s| {
                s.len() == 64
                    && s.bytes()
                        .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
            }),
            "invalid_request",
        )?;
        require(
            self.mode == "context" || (self.max_nodes.is_none() && self.max_edges.is_none()),
            "invalid_request",
        )?;
        require(
            self.role
                .as_ref()
                .is_none_or(|r| super::content::ROLES.contains(&r.as_str())),
            "invalid_request",
        )?;
        require(
            self.edge_kind
                .as_ref()
                .is_none_or(|r| model::RELATIONS.contains(&r.as_str())),
            "invalid_request",
        )?;
        require(
            self.direction
                .as_ref()
                .is_none_or(|r| ["incoming", "outgoing", "both"].contains(&r.as_str())),
            "invalid_request",
        )?;
        require(
            self.kind
                .as_ref()
                .is_none_or(|r| model::KINDS.contains(&r.as_str())),
            "invalid_request",
        )?;
        require(
            self.status
                .as_ref()
                .is_none_or(|r| model::STATUSES.contains(&r.as_str())),
            "invalid_request",
        )
    }
}
impl Session {
    pub fn knowledge(&self, q: &Request) -> Result<Value> {
        q.validate()?;
        let trusted = self
            .trusted
            .as_ref()
            .map_err(|_| Error("document_profile_unavailable"))?;
        trusted.recheck()?;
        crate::pcg::security::no_links(&q.workspace_root, false)?;
        require(
            q.workspace_root
                .canonicalize()
                .map_err(|_| Error("scope_refused"))?
                == trusted.profile.workspace_root,
            "scope_refused",
        )?;
        let max_nodes = q.max_nodes.unwrap_or(trusted.profile.max_nodes);
        let max_edges = q.max_edges.unwrap_or(trusted.profile.max_edges);
        require(
            max_nodes > 0
                && max_nodes <= trusted.profile.max_nodes
                && max_edges > 0
                && max_edges <= trusted.profile.max_edges,
            "request_limits",
        )?;
        let mut connection = store::open(trusted)?;
        let tx = store::sql(connection.transaction_with_behavior(TransactionBehavior::Immediate))?;
        store::initialize(&tx, trusted)?;
        let mut capture = scan::scan(trusted)?;
        capture
            .diagnostics
            .extend(validate::validate(&capture.nodes));
        capture
            .diagnostics
            .extend(store::missing(&tx, &capture.nodes)?);
        let valid = capture.diagnostics.is_empty();
        let generation = if valid {
            Some(store::publish_knowledge(&tx, trusted, &capture.nodes)?)
        } else {
            store::head(&tx)?
        };
        let mut payload = json!({"contract":"pcg_knowledge_v1","version":1,"operation":q.mode,"workspaceRoot":q.workspace_root,
            "generation":generation,"result_state":if valid {"ready"}else{"blocked"},"catalog_current":valid,
            "complete":valid,"units":[],"relationships":[],"paths":[],"sources":[],"diagnostics":capture.diagnostics,
            "completeness_scope":q.completeness_scope(),"inferred_applicability":false,
            "authority_assurance":"source_attributed","verification_execution":"not_executed_by_parser",
            "freshness":"observed_boundaries_only","atomic_filesystem_snapshot":false,"continuous_currentness":false,
            "evidence_delivery":"exact_current_artifact_references_retrievable_by_source_operation",
            "token_budget_enforced":false});
        if valid {
            let unrepresented: Vec<_> = capture
                .nodes
                .iter()
                .filter(|(_, n)| n.knowledge.is_none())
                .map(|(id, _)| id.clone())
                .collect();
            payload["catalog_coverage"] = json!({"identities":capture.nodes.len(),"represented":capture.nodes.len()-unrepresented.len(),"unrepresented":unrepresented});
            if q.mode == "search" && !unrepresented.is_empty() {
                payload["complete"] = false.into();
                payload["result_state"] = "incomplete".into();
                payload["diagnostics"] = json!([{"code":"E_KNOWLEDGE_UNDECLARED","message":"Discovery cannot claim completeness over unrepresented identities"}]);
            } else {
                knowledge_query::execute(
                    &tx,
                    generation.as_deref().ok_or(Error("document_database"))?,
                    q,
                    &capture,
                    max_nodes,
                    max_edges,
                    &mut payload,
                )?;
            }
        }
        for unit in payload["units"]
            .as_array_mut()
            .ok_or(Error("knowledge_corruption"))?
        {
            for field in ["source_request", "acquire_request"] {
                if let Some(reference) = unit.get_mut(field) {
                    reference["workspaceRoot"] = json!(q.workspace_root);
                }
            }
            let id = unit["id"].clone();
            if let Some(components) = unit.get_mut("components").and_then(Value::as_array_mut) {
                for component in components {
                    component["source_request"] = json!({"workspaceRoot":q.workspace_root,"mode":"source","id":id,"path":component["path"],"sourceSha256":component["source_sha256"]});
                }
            }
            if let Some(evidence) = unit.get_mut("evidence").and_then(Value::as_array_mut) {
                for association in evidence {
                    association["source_request"]["workspaceRoot"] = json!(q.workspace_root);
                }
            }
        }
        capture.completion(trusted)?;
        if valid {
            store::sql(tx.commit())?;
        }
        Ok(payload)
    }
}
