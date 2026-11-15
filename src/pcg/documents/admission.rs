//! Rejection references resolve only against the accepted publication, never candidate policy.
use super::{knowledge_store, model::Diagnostic, store};
use crate::pcg::Result;
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::json;
use std::path::Path;

pub(super) fn attach_authorities(
    connection: &Connection,
    workspace: &Path,
    diagnostics: &mut [Diagnostic],
) -> Result<(usize, usize)> {
    let head = store::head(connection)?;
    let mut references = std::collections::BTreeMap::new();
    let mut revision_lookups = 0;
    let mut component_loads = 0;
    for diagnostic in diagnostics {
        let authority = match diagnostic.code.as_str() {
            "E_INVALID_STATUS"
            | "E_INVALID_KIND"
            | "E_INVALID_NODE_ID"
            | "E_INVALID_EDGE_TYPE"
            | "E_UNKNOWN_NODE"
            | "E_SELF_EDGE"
            | "E_DEPENDS_ON_INVALID_PROPOSITION"
            | "E_INVALID_EDGE_TARGET_KIND"
            | "E_RECLOSURE_TARGET_NOT_CLOSED"
            | "E_SUPERSESSION_STATUS"
            | "E_BLOCKED_WITHOUT_BLOCKER"
            | "E_STALE_BLOCK"
            | "E_UNSATISFIED_PREREQUISITE"
            | "E_SUPERSEDED_WITHOUT_SUCCESSOR" => "AUTH.PCG_GLS",
            _ => "AUTH.PCG_DOCUMENTS",
        };
        if !references.contains_key(authority) {
            let revision: Option<String> = if let Some(head) = &head {
                revision_lookups += 1;
                store::sql(
                    connection
                        .query_row(
                            "SELECT revision FROM document_member WHERE g=? AND id=?",
                            params![head, authority],
                            |row| row.get(0),
                        )
                        .optional(),
                )?
            } else {
                None
            };
            let mut governing = json!({"id":authority,"standing":"accepted_reference_unavailable"});
            if let Some(revision) = revision {
                component_loads += 1;
                let components = knowledge_store::components(connection, &revision)?;
                // A source-owned rule may be independently represented outside
                // the summary. Retain its controlling components and qualifications,
                // without presenting observations or superseded history as rules.
                let selected: Vec<_> = components.iter().filter(|component|
                    matches!(component["role"].as_str(), Some("statement" | "constraint"
                        | "scope" | "verification_obligation" | "qualification"
                        | "rationale" | "rejected_alternative" | "rejection_reason"
                        | "consequence"))).map(|component| json!({
                    "id":authority,"revision":revision,"ordinal":component["ordinal"],
                    "role":component["role"],"label":component["label"],"path":component["path"],
                    "sourceSha256":component["source_sha"],"start":component["start"],"end":component["end"],
                    "contentSha256":component["content_sha"],"selector":component["selector"],"historical":true
                })).collect();
                governing["revision"] = json!(revision);
                if !selected.is_empty() {
                    governing["standing"] = json!("previously_accepted_snapshot");
                    governing["acquire_request"] =
                        json!({"workspaceRoot":workspace,"mode":"acquire","references":selected});
                }
            }
            references.insert(authority, governing);
        }
        diagnostic.governing_authority = references.get(authority).cloned();
        diagnostic.admission_disposition = Some("rejected");
        diagnostic.accepted_publication = Some(false);
    }
    Ok((revision_lookups, component_loads))
}
