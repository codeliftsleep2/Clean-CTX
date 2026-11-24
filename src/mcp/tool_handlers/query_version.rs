//! Early version routing, before any legacy preparation or acquisition.
use crate::mcp::McpState;
use serde_json::{Value, json};
pub(super) fn intercept(id: &Value, args: &Value, state: &McpState) -> bool {
    let items = args["queries"].as_array();
    let unsupported = |v: &Value| {
        v.get("contract_version")
            .is_some_and(|n| !matches!(n.as_u64(), Some(1 | 2)))
    };
    let has_v2 = args["contract_version"] == 2
        || items.is_some_and(|a| a.iter().any(|v| v["contract_version"] == 2));
    let invalid = unsupported(args) || items.is_some_and(|a| a.iter().any(unsupported));
    if invalid {
        error(
            id,
            "Unsupported contract_version; expected omitted, 1 or explicitly enabled 2.",
        );
        return true;
    }
    if !has_v2 {
        return false;
    }
    if items.is_some_and(|a| {
        a.iter()
            .any(|v| v["contract_version"] != 2 || v["type"] != "resolver_registrations")
    }) {
        error(
            id,
            "Mixed legacy/v2 batches are refused before workspace acquisition.",
        );
        return true;
    }
    #[cfg(feature = "resolver_evidence_v2")]
    crate::mcp::resolver_evidence::handle(id, args, state);
    #[cfg(not(feature = "resolver_evidence_v2"))]
    {
        let _ = state;
        error(id, "Resolver evidence v2 is not compiled into this server.");
    }
    true
}
fn error(id: &Value, message: &str) {
    crate::protocol::send_response(
        &json!({"jsonrpc":"2.0","id":id,"error":{"code":-32602,"message":message}}),
    );
}
