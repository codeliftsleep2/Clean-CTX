//! Source-owned document admission. SQLite owns admitted graph snapshots/history.
mod model;
mod parse;
mod profile;
mod response;
mod scan;
mod store;
mod traversal;
mod validate;

use crate::pcg::{Error, Result, identity, projection, require};
use crate::tokenizer::{TokenizerKind, create_tokenizer};
use profile::Trusted;
use rusqlite::TransactionBehavior;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::path::PathBuf;

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Request {
    pub workspace_root: PathBuf,
    pub mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    pub tokenizer: String,
    /// Legacy request hint retained for compatibility, never an admission limit.
    #[serde(default)]
    pub token_budget: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_nodes: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_edges: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<String>,
}
pub struct Session {
    trusted: Result<Trusted>,
}
pub struct Guard<'a> {
    trusted: &'a Trusted,
    capture: scan::Capture,
    connection: rusqlite::Connection,
}
impl Guard<'_> {
    pub fn complete(mut self) -> Result<()> {
        self.capture.completion(self.trusted)?;
        store::sql(self.connection.execute_batch("COMMIT"))
    }
}
impl Session {
    pub fn new(path: PathBuf) -> Self {
        Self {
            trusted: Trusted::load(&path),
        }
    }

    /// Guard the other PCG MCP read surface when it addresses this same workspace.
    /// The operation-local capture is reused at COMPLETION; no second graph scan.
    pub fn guard(&self, root: &std::path::Path) -> Result<Option<Guard<'_>>> {
        let trusted = self
            .trusted
            .as_ref()
            .map_err(|_| Error("document_profile_unavailable"))?;
        trusted.recheck()?;
        crate::pcg::security::no_links(root, false)?;
        if root.canonicalize().map_err(|_| Error("scope_refused"))?
            != trusted.profile.workspace_root
        {
            return Ok(None);
        }
        let connection = store::open(trusted)?;
        store::sql(connection.execute_batch("BEGIN IMMEDIATE"))?;
        store::initialize(&connection, trusted)?;
        let mut capture = scan::scan(trusted)?;
        capture
            .diagnostics
            .extend(validate::validate(&capture.nodes));
        capture
            .diagnostics
            .extend(store::missing(&connection, &capture.nodes)?);
        require(capture.diagnostics.is_empty(), "document_invalid")?;
        store::publish(&connection, trusted, &capture.nodes)?;
        Ok(Some(Guard {
            trusted,
            capture,
            connection,
        }))
    }

    pub fn query(&self, q: &Request) -> Result<projection::Response> {
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
        require(
            ["search", "get", "history", "related", "context"].contains(&q.mode.as_str()),
            "invalid_request",
        )?;
        let source_refs = match q.response_format.as_deref() {
            None | Some("inline_v1") => false,
            Some("source_refs_v1") if q.mode == "context" => true,
            _ => return Err(Error("invalid_request")),
        };
        if q.mode == "search" {
            require(q.id.is_none(), "invalid_request")?;
        } else {
            identity::id(q.id.as_deref().ok_or(Error("invalid_request"))?)?;
            require(q.track.is_none() && q.text.is_none(), "invalid_request")?;
        }
        if let Some(track) = &q.track {
            require(!track.trim().is_empty(), "invalid_request")?;
        }
        let max_nodes = q.max_nodes.unwrap_or(trusted.profile.max_nodes);
        let max_edges = q.max_edges.unwrap_or(trusted.profile.max_edges);
        require(
            max_nodes > 0
                && max_nodes <= trusted.profile.max_nodes
                && max_edges > 0
                && max_edges <= trusted.profile.max_edges,
            "request_limits",
        )?;
        require(
            q.mode == "context" || (q.max_nodes.is_none() && q.max_edges.is_none()),
            "invalid_request",
        )?;
        if let Some(text) = &q.text {
            require(!text.trim().is_empty(), "invalid_request")?;
        }
        let kind = match q.tokenizer.as_str() {
            "cl100k" => TokenizerKind::Cl100k,
            "o200k" => TokenizerKind::O200k,
            _ => return Err(Error("unsupported_tokenizer")),
        };
        let tokenizer = create_tokenizer(kind).map_err(|_| Error("tokenizer_unavailable"))?;
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
            Some(store::publish(&tx, trusted, &capture.nodes)?)
        } else {
            store::head(&tx)?
        };
        let mut context = if valid && q.mode == "context" {
            Some(traversal::collect(
                &tx,
                generation.as_deref().ok_or(Error("document_database"))?,
                q.id.as_deref().ok_or(Error("invalid_request"))?,
                max_nodes,
                max_edges,
            )?)
        } else {
            None
        };
        let context_complete = context.as_ref().is_none_or(|c| c.complete);
        if !context_complete {
            capture.diagnostics.push(model::Diagnostic::new("E_CONTEXT_LIMIT","",q.id.as_deref(),
                "Context exceeds explicit node/edge limits; no partial authoritative context is returned"));
        }
        let mut documents = if let Some(context) = &mut context {
            std::mem::take(&mut context.documents)
        } else if valid || q.mode == "history" {
            store::query(&tx, generation.as_deref().unwrap_or(""), q)?
        } else {
            Vec::new()
        };
        let historical_documents = if q.mode == "history" {
            std::mem::take(&mut documents)
        } else {
            Vec::new()
        };
        let context=context.map(|c|json!({"complete":c.complete,"relationships":c.relationships,"paths":c.paths,
            "path_semantics":"one_deterministic_witness_per_node_not_all_paths","authorizes_execution":false}));
        let mut payload = json!({"version":1,"result_state":if !valid {"blocked"}else if context_complete {"ready"}else{"incomplete"},
            "generation":generation,"catalog_current":valid,"documents":documents,"diagnostics":capture.diagnostics,
            "historical_documents":historical_documents,
            "context":context,"verification_execution":"not_executed_by_parser",
            "authority_assurance":"source_attributed","revision_authorization":"not_established",
            "freshness":"observed_boundaries_only","continuous_currentness":false,"atomic_filesystem_snapshot":false,
            "scope":{"workspaceRoot":trusted.profile.workspace_root,"documentRoots":trusted.profile.document_roots},
            "budget":{"limit":null,"enforced":false,"tokenizer":q.tokenizer,"accounting":"serialized_mcp_result_v1"}});
        if source_refs {
            response::source_refs(&mut payload)?;
        }
        let encode = |value: &Value| -> Result<String> {
            let text = identity::canonical(value);
            Ok(identity::canonical(
                &json!({"content":[{"type":"text","text":text}],"isError":false}),
            ))
        };
        let serialized = encode(&payload)?;
        let tokens = tokenizer.count_tokens(&serialized);
        // The counted draft precedes COMPLETION; no draft escapes if sources changed.
        capture.completion(trusted)?;
        if valid {
            store::sql(tx.commit())?;
        }
        // Invalid candidate transaction rolls back on drop, retaining the prior admitted graph.
        Ok(projection::Response {
            serialized_result: serialized,
            used_tokens: tokens,
            counts: projection::Counts::default(),
        })
    }
}

#[cfg(test)]
#[path = "../../tests/pcg/documents.rs"]
mod tests;
