//! Causal RED for LegacyEdit population-only adequacy.
//!
//! Supported production history under test:
//! provide Low -> populated WorkspaceIndex -> generic workspace edge query.
//! Low intentionally omits Spring method mappings, while the cold LegacyEdit
//! path compiles at Edit and can produce them.

use crate::mcp::tool_handlers::hydration::{
    TEST_PROJECT_HYDRATION_SERIALIZE, fail_test_publication_once, hydrate_workspace_index,
    invalidate_discovery_for_root,
};
use crate::mcp::tools::dispatch_tools_call;
use crate::tests::assert_valid_mcp_envelope;
use crate::workspace::index::SemanticFidelity;
use serde_json::json;

fn pop_response() -> serde_json::Value {
    let mut responses = crate::protocol::CAPTURED_RESPONSES
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    responses.pop().expect("handler response")
}

fn call(
    state: &crate::mcp::McpState,
    id: i64,
    tool: &str,
    arguments: serde_json::Value,
) -> serde_json::Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&json!(id), tool, &json!({ "arguments": arguments }), state);
    pop_response()
}

fn low_publish(state: &crate::mcp::McpState, root: &std::path::Path) {
    let response = call(
        state,
        1,
        "provide_code_context",
        json!({
            "filePath": "OrdersController.java",
            "fidelity": "low",
            "workspaceRoot": root.to_string_lossy()
        }),
    );
    assert!(
        response.get("result").is_some(),
        "supported Low publication must succeed: {response:?}"
    );
}

fn forward_edges(state: &crate::mcp::McpState, root: &std::path::Path) -> Vec<serde_json::Value> {
    let response = call(
        state,
        2,
        "workspace_query",
        json!({
            "type": "forward_edges",
            "domain": "spring",
            "entity_type": "Controller",
            "name": "OrdersController",
            "workspaceRoot": root.to_string_lossy()
        }),
    );
    let result = response["result"]
        .as_object()
        .unwrap_or_else(|| panic!("workspace query must succeed: {response:?}"));
    assert_valid_mcp_envelope(result);
    result["structuredContent"]["edges"]
        .as_array()
        .cloned()
        .expect("edge array")
}

fn has_orders_mapping(edges: &[serde_json::Value]) -> bool {
    edges.iter().any(|edge| {
        edge["relation"] == "EndpointMapsTo"
            && edge["subject"]["domain"] == "spring"
            && edge["subject"]["entity_type"] == "Controller"
            && edge["subject"]["name"] == "OrdersController"
            && edge["object"]["domain"] == "spring"
            && edge["object"]["entity_type"] == "Endpoint"
            && edge["object"]["name"] == "GET /orders"
    })
}

fn state() -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    crate::mcp::McpState::new(config)
}

fn fixture() -> (tempfile::TempDir, std::path::PathBuf) {
    let root = tempfile::TempDir::new().expect("workspace");
    let file = root.path().join("OrdersController.java");
    std::fs::write(
        &file,
        r#"import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RestController;

@RestController
public class OrdersController {
    @GetMapping("/orders")
    public String list() { return "ok"; }
}
"#,
    )
    .expect("Spring controller fixture");
    (root, file)
}

fn current_high_projection(state: &crate::mcp::McpState, file: &std::path::Path) -> bool {
    let canonical = state.semantic_owner_path(&file.to_string_lossy());
    let source = state.read_source(&canonical).expect("current source");
    let hash = state.cache_read().compute_hash(source.as_bytes());
    state
        .workspace_index_read()
        .has_current_semantic_projection(&canonical, SemanticFidelity::High, &hash)
}

#[test]
fn red_legacy_edit_low_projection_cannot_suppress_required_reacquisition() {
    let _serial = TEST_PROJECT_HYDRATION_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let (root, file) = fixture();

    let cold = state();
    let cold_edges = forward_edges(&cold, root.path());
    assert!(
        has_orders_mapping(&cold_edges),
        "cold LegacyEdit acquisition must expose the method mapping: {cold_edges:?}"
    );

    let warm = state();
    low_publish(&warm, root.path());
    let canonical = warm.semantic_owner_path(&file.to_string_lossy());
    assert!(
        warm.workspace_index_read()
            .file_map()
            .contains_key(&canonical),
        "the supported Low publication must establish the populated state"
    );
    let warm_edges = forward_edges(&warm, root.path());
    assert!(
        has_orders_mapping(&warm_edges),
        "equivalent source must not lose query truth solely because Low publication populated file_map: cold={cold_edges:?}, warm={warm_edges:?}"
    );
}

#[test]
fn adequate_warm_edit_projection_suppresses_redundant_reacquisition() {
    let _serial = TEST_PROJECT_HYDRATION_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let (root, file) = fixture();
    let state = state();
    let first = hydrate_workspace_index(
        &state,
        "forward_edges",
        "OrdersController",
        Some(&root.path().to_string_lossy()),
    )
    .expect("first hydration");
    assert_eq!(first.candidates_compiled, 1);
    assert!(current_high_projection(&state, &file));

    invalidate_discovery_for_root(&state, &root.path().to_string_lossy());
    let warm = hydrate_workspace_index(
        &state,
        "forward_edges",
        "OrdersController",
        Some(&root.path().to_string_lossy()),
    )
    .expect("warm hydration");
    assert_eq!(warm.candidates_discovered, 1);
    assert_eq!(
        warm.candidates_compiled, 0,
        "source-current High evidence must suppress redundant Edit compilation"
    );
}

#[test]
fn inadequate_low_projection_forces_legacy_edit_reacquisition() {
    let _serial = TEST_PROJECT_HYDRATION_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let (root, file) = fixture();
    let state = state();
    low_publish(&state, root.path());
    assert!(!current_high_projection(&state, &file));

    let report = hydrate_workspace_index(
        &state,
        "forward_edges",
        "OrdersController",
        Some(&root.path().to_string_lossy()),
    )
    .expect("LegacyEdit hydration");
    assert_eq!(report.candidates_compiled, 1);
    assert!(current_high_projection(&state, &file));
}

#[test]
fn failed_required_reacquisition_cannot_authorize_completion() {
    let _serial = TEST_PROJECT_HYDRATION_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let (root, file) = fixture();
    let state = state();
    low_publish(&state, root.path());
    fail_test_publication_once(&file.to_string_lossy());

    let failed = hydrate_workspace_index(
        &state,
        "forward_edges",
        "OrdersController",
        Some(&root.path().to_string_lossy()),
    )
    .expect("failed hydration report");
    assert_eq!(failed.discovery_status, "partial");
    assert_eq!(failed.candidates_compiled, 0);
    assert!(!current_high_projection(&state, &file));

    let retried = hydrate_workspace_index(
        &state,
        "forward_edges",
        "OrdersController",
        Some(&root.path().to_string_lossy()),
    )
    .expect("retry hydration");
    assert_eq!(retried.discovery_status, "completed");
    assert_eq!(retried.candidates_compiled, 1);
    assert!(current_high_projection(&state, &file));
}

#[test]
fn reversed_query_history_preserves_semantic_truth() {
    let _serial = TEST_PROJECT_HYDRATION_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let (root, file) = fixture();
    let state = state();
    let before = forward_edges(&state, root.path());
    assert!(has_orders_mapping(&before));
    assert!(current_high_projection(&state, &file));

    low_publish(&state, root.path());
    assert!(!current_high_projection(&state, &file));
    let after = forward_edges(&state, root.path());
    assert!(
        has_orders_mapping(&after),
        "a later Low publication must revoke stale discovery authority and force adequate reacquisition: before={before:?}, after={after:?}"
    );
    assert!(current_high_projection(&state, &file));
}
