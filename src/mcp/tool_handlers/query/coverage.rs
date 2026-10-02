//! Truthfulness metadata for exact-identity graph answers.

use super::identity::ResolvedIdentity;
use crate::mcp::tool_handlers::hydration::HydrationReport;
use crate::workspace::index::WorkspaceIndex;
use crate::workspace::scope::WorkspaceScope;
use serde_json::{Value, json};

#[derive(Clone, Copy)]
pub(super) enum CapabilityDirection {
    Forward,
    Reverse,
}

pub(super) fn exact_identity_coverage(
    index: &WorkspaceIndex,
    identity: &ResolvedIdentity,
    scope: Option<&WorkspaceScope>,
    direction: CapabilityDirection,
    hydration: &HydrationReport,
) -> Value {
    let identity_indexed = index.has_identity_in_scope(
        &identity.domain,
        &identity.entity_type,
        &identity.name,
        scope,
    );
    let capability_established = match direction {
        CapabilityDirection::Forward => {
            index.has_forward_capability_evidence(&identity.domain, &identity.entity_type, scope)
        }
        CapabilityDirection::Reverse => {
            index.has_reverse_capability_evidence(&identity.domain, &identity.entity_type, scope)
        }
    };
    let status = if !identity_indexed {
        "identity_not_indexed"
    } else if hydration.hydration_attempted && hydration.discovery_status != "completed" {
        "indexed_evidence_only"
    } else if capability_established {
        "established_indexed_capability"
    } else {
        "capability_not_established"
    };
    json!({
        "status": status,
        "identity_indexed": identity_indexed,
        "capability_established": capability_established,
        "source_complete": false,
    })
}

#[cfg(test)]
#[path = "../../../tests/mcp/workspace_query_coverage.rs"]
mod tests;
