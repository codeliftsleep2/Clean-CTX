//! Query-local merging for derived semantic edges.

use crate::layers::meta::semantic::SemanticEdge;
use serde_json::Value;

pub(super) fn append_distinct_json_edges(edges: &mut Vec<Value>, additions: Vec<SemanticEdge>) {
    for edge in serde_json::to_value(additions)
        .ok()
        .and_then(|value| value.as_array().cloned())
        .unwrap_or_default()
    {
        if !edges
            .iter()
            .any(|existing| same_semantic_identity(existing, &edge))
        {
            edges.push(edge);
        }
    }
}

fn same_semantic_identity(left: &Value, right: &Value) -> bool {
    left["relation"] == right["relation"]
        && left["subject"]["domain"] == right["subject"]["domain"]
        && left["subject"]["entity_type"] == right["subject"]["entity_type"]
        && left["subject"]["name"] == right["subject"]["name"]
        && left["object"]["domain"] == right["object"]["domain"]
        && left["object"]["entity_type"] == right["object"]["entity_type"]
        && left["object"]["name"] == right["object"]["name"]
}
