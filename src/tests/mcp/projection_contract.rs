// Production MCP mapping for checked hierarchical projection failures.

use super::common::projection_error_response;
use crate::ir::hierarchical::{HierarchicalProjectionError, ProjectionIdentityKind};

#[test]
fn projection_identity_failure_uses_existing_ir_error_contract() {
    let response = projection_error_response(
        &serde_json::json!(17),
        &HierarchicalProjectionError::KindMismatch {
            operation: "RET",
            id: "C1".into(),
            expected: ProjectionIdentityKind::Method,
            actual: ProjectionIdentityKind::Class,
            instruction: 4,
        },
    );

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 17);
    assert_eq!(response["error"]["code"], -32602);
    assert_eq!(response["error"]["data"]["retryable"], false);
    assert_eq!(
        response["error"]["data"]["projection_code"],
        "ir_projection_kind_mismatch"
    );
    assert!(response.get("result").is_none());
    assert!(
        response["error"]["message"]
            .as_str()
            .is_some_and(|message| message.contains("hierarchical projection failed"))
    );
}
