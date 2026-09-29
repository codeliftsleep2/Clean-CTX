use crate::cbm::{GraphBridge, GraphNode};
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Serialize)]
pub(crate) struct TraceCandidate {
    id: String,
    label: String,
    name: String,
    file: String,
}

pub(crate) enum TraceIdentityError {
    NotFound(String),
    Ambiguous {
        source: String,
        candidates: Vec<TraceCandidate>,
    },
    Search(String),
}

impl TraceIdentityError {
    pub(crate) fn response(&self, id: &Value) -> Value {
        match self {
            Self::NotFound(source) => serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "error": {
                    "code": -32602,
                    "message": format!(
                        "Trace source '{source}' was not found in the selected CBM project."
                    )
                }
            }),
            Self::Ambiguous { source, candidates } => serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "error": {
                    "code": -32602,
                    "message": format!(
                        "Trace source '{source}' is ambiguous; supply a canonical identity."
                    ),
                    "data": { "candidates": candidates }
                }
            }),
            Self::Search(message) => serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "error": { "code": -32603, "message": message }
            }),
        }
    }
}

pub(crate) fn resolve_trace_source(
    bridge: &mut GraphBridge,
    source: &str,
    project: Option<&str>,
) -> Result<String, TraceIdentityError> {
    // CBM canonical identities are path-qualified. They remain a zero-search
    // fast path, preserving the efficient and unambiguous existing contract.
    if source.contains('.') {
        return Ok(source.to_string());
    }

    let matches = match project {
        Some(project) => bridge.search_scoped(source, project),
        None => bridge.search(source),
    };
    if let Some(error) = bridge.take_last_error() {
        return Err(TraceIdentityError::Search(format!(
            "CBM trace-source resolution failed: {error}"
        )));
    }

    let exact: BTreeMap<String, GraphNode> = matches
        .into_iter()
        .filter(|node| node.name == source)
        .map(|node| (node.id.clone(), node))
        .collect();

    match exact.len() {
        0 => Err(TraceIdentityError::NotFound(source.to_string())),
        1 => Ok(exact
            .into_keys()
            .next()
            .expect("one exact trace identity must have one key")),
        _ => Err(TraceIdentityError::Ambiguous {
            source: source.to_string(),
            candidates: exact.into_values().map(TraceCandidate::from).collect(),
        }),
    }
}

impl From<GraphNode> for TraceCandidate {
    fn from(node: GraphNode) -> Self {
        Self {
            id: node.id,
            label: node.label,
            name: node.name,
            file: node.file,
        }
    }
}
