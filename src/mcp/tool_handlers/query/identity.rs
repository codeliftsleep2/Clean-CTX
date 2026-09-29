use super::outcome::QueryFailure;
use crate::workspace::index::WorkspaceIndex;
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

pub(super) struct IdentityRequest {
    domain: Option<String>,
    entity_type: Option<String>,
    name: String,
}

impl IdentityRequest {
    pub(super) fn new(args: &Value, name: &str) -> Self {
        Self {
            domain: args["domain"]
                .as_str()
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            entity_type: args["entity_type"]
                .as_str()
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            name: name.to_string(),
        }
    }

    pub(super) fn exact(&self) -> Option<ResolvedIdentity> {
        Some(ResolvedIdentity {
            domain: self.domain.clone()?,
            entity_type: self.entity_type.clone()?,
            name: self.name.clone(),
        })
    }

    pub(super) fn resolve(
        &self,
        index: &WorkspaceIndex,
        scope: Option<&WorkspaceScope>,
    ) -> Result<ResolvedIdentity, QueryFailure> {
        if let Some(identity) = self.exact() {
            return Ok(identity);
        }
        let occurrences = match scope {
            Some(scope) => index.find_entities_by_name_in_scope(&self.name, scope),
            None => index.find_entities_by_name(&self.name),
        };
        let identities = occurrences
            .into_iter()
            .filter(|entity| {
                self.domain
                    .as_deref()
                    .is_none_or(|filter| entity.domain == filter)
            })
            .filter(|entity| {
                self.entity_type
                    .as_deref()
                    .is_none_or(|filter| entity.entity_type == filter)
            })
            .map(|entity| ResolvedIdentity {
                domain: entity.domain.to_string(),
                entity_type: entity.entity_type.to_string(),
                name: entity.name.clone(),
            })
            .collect::<BTreeSet<_>>();
        if identities.len() == 1 {
            return Ok(identities.into_iter().next().expect("one identity"));
        }
        let candidates: Vec<_> = identities.into_iter().collect();
        let message = if candidates.is_empty() {
            format!(
                "No entity identity matches name '{}' and the supplied filters.",
                self.name
            )
        } else {
            format!(
                "Entity name '{}' is ambiguous; supply domain and entity_type.",
                self.name
            )
        };
        Err(QueryFailure::new(
            -32602,
            message,
            Some(serde_json::json!({ "candidates": candidates })),
        ))
    }
}
