use super::outcome::QueryFailure;
use crate::mcp::McpState;
use crate::mcp::tool_handlers::hydration::{HydrationReport, hydrate_workspace_index};
use crate::workspace::scope::WorkspaceScope;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeSet;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub(super) struct ResolvedIdentity {
    pub(super) domain: String,
    pub(super) entity_type: String,
    pub(super) name: String,
}

pub(super) struct IdentitySelection {
    pub(super) identity: ResolvedIdentity,
    pub(super) hydration: Option<HydrationReport>,
}

pub(super) fn resolve_identity(
    args: &Value,
    state: &McpState,
    query_type: &str,
    name: &str,
    scope: Option<&WorkspaceScope>,
) -> Result<IdentitySelection, QueryFailure> {
    let domain = args["domain"].as_str().filter(|value| !value.is_empty());
    let entity_type = args["entity_type"]
        .as_str()
        .filter(|value| !value.is_empty());
    if let (Some(domain), Some(entity_type)) = (domain, entity_type) {
        return Ok(IdentitySelection {
            identity: ResolvedIdentity {
                domain: domain.to_string(),
                entity_type: entity_type.to_string(),
                name: name.to_string(),
            },
            hydration: None,
        });
    }

    let hydration =
        hydrate_workspace_index(state, query_type, name, args["workspaceRoot"].as_str())
            .map_err(QueryFailure::internal)?;
    let identities = {
        let index = state.workspace_index_read();
        let occurrences = match scope {
            Some(scope) => index.find_entities_by_name_in_scope(name, scope),
            None => index.find_entities_by_name(name),
        };
        occurrences
            .into_iter()
            .filter(|entity| domain.is_none_or(|filter| entity.domain == filter))
            .filter(|entity| entity_type.is_none_or(|filter| entity.entity_type == filter))
            .map(|entity| ResolvedIdentity {
                domain: entity.domain.to_string(),
                entity_type: entity.entity_type.to_string(),
                name: entity.name.clone(),
            })
            .collect::<BTreeSet<_>>()
    };

    if identities.len() != 1 {
        let candidates: Vec<_> = identities.into_iter().collect();
        let message = if candidates.is_empty() {
            format!("No entity identity matches name '{name}' and the supplied filters.")
        } else {
            format!("Entity name '{name}' is ambiguous; supply domain and entity_type.")
        };
        return Err(QueryFailure::new(
            -32602,
            message,
            Some(serde_json::json!({ "candidates": candidates })),
        ));
    }

    Ok(IdentitySelection {
        identity: identities.into_iter().next().expect("one identity"),
        hydration: Some(hydration),
    })
}
