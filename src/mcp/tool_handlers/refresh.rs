//! Explicit reconciliation of externally edited workspace source.

use crate::mcp::McpState;
use crate::protocol::send_response;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

pub(crate) fn handle_refresh_workspace(id: &Value, params: &Value, state: &McpState) {
    let Some(raw_root) = params["arguments"]["workspaceRoot"]
        .as_str()
        .map(str::trim)
        .filter(|root| !root.is_empty())
    else {
        send_response(&crate::mcp::tool_helpers::jsonrpc_error(
            id.clone(),
            -32602,
            "Missing required argument: 'workspaceRoot'.",
            None,
        ));
        return;
    };

    let declared = Path::new(raw_root);
    let resolved = if declared.is_absolute() {
        declared.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(declared)
    };
    let canonical = match resolved.canonicalize() {
        Ok(root) if root.is_dir() => root,
        Ok(_) => {
            send_response(&crate::mcp::tool_helpers::jsonrpc_error(
                id.clone(),
                -32602,
                "'workspaceRoot' must identify a directory.",
                None,
            ));
            return;
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => resolved,
        Err(error) => {
            send_response(&crate::mcp::tool_helpers::jsonrpc_error(
                id.clone(),
                -32602,
                format!("Unable to resolve 'workspaceRoot': {error}"),
                None,
            ));
            return;
        }
    };
    let canonical = crate::dictionary::path::canonical_identity_key(&canonical.to_string_lossy());
    let roots = super::hydration::effective_refresh_roots(state, &canonical);
    let mut reconciliation = super::hydration::RefreshReconciliation::default();
    for root in &roots {
        reconciliation.merge(super::hydration::reconcile_external_refresh_for_root(
            state, root,
        ));
    }

    let roots_refreshed = roots.len();
    let structured = json!({
        "refreshed": true,
        "workspace_root": canonical,
        "workspace_roots": roots,
        "roots_refreshed": roots_refreshed,
        "indexed_owners_retracted": reconciliation.indexed_owners_retracted,
        "source_snapshots_invalidated": reconciliation.source_snapshots_invalidated,
        "pending_transitions_retired": reconciliation.pending_transitions_retired,
    });
    send_response(&json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "content": [{
                "type": "text",
                "text": format!(
                    "// REFRESH-WORKSPACE v1; structuredContent remains authoritative\n{}",
                    serde_json::to_string_pretty(&structured).unwrap_or_default()
                )
            }],
            "structuredContent": structured
        }
    }));
}
