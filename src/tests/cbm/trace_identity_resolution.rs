use crate::cbm::bridge::test_helpers::new_mock_empty;
use crate::cbm::bridge::{CachedGraphData, GraphEdge, GraphNode};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::time::{Duration, Instant};

const BARE: &str = "duplicate_probe";
const ALPHA: &str = "probe.src.alpha.duplicate_probe";
const BETA: &str = "probe.src.beta.duplicate_probe";

fn node(id: &str, file: &str) -> GraphNode {
    GraphNode {
        id: id.to_string(),
        label: "Function".to_string(),
        name: BARE.to_string(),
        file: file.to_string(),
        properties: HashMap::new(),
    }
}

fn edge(from: &str, to: &str) -> GraphEdge {
    GraphEdge {
        from: from.to_string(),
        to: to.to_string(),
        label: "calls".to_string(),
        properties: HashMap::new(),
    }
}

fn state() -> crate::mcp::McpState {
    let bridge = new_mock_empty();
    let expires_at = Instant::now() + Duration::from_secs(600);
    bridge.cache.insert(
        format!("search:{BARE}"),
        CachedGraphData {
            data: serde_json::to_value([node(ALPHA, "src/alpha.rs"), node(BETA, "src/beta.rs")])
                .expect("candidate cache"),
            expires_at,
        },
    );
    bridge.cache.insert(
        format!("trace:{BARE}:alpha_leaf"),
        CachedGraphData {
            data: serde_json::to_value([edge(BARE, "probe.src.alpha.alpha_leaf")])
                .expect("bare trace cache"),
            expires_at,
        },
    );
    bridge.cache.insert(
        format!("trace:{ALPHA}:alpha_leaf"),
        CachedGraphData {
            data: serde_json::to_value([edge(ALPHA, "probe.src.alpha.alpha_leaf")])
                .expect("canonical trace cache"),
            expires_at,
        },
    );

    let state = crate::mcp::McpState::new(crate::tests::test_config());
    *state.graph_bridge_lock() = Some(bridge);
    crate::mcp::tools::setup_handler_registry_for_tests();
    state
}

fn dispatch(tool: &str, arguments: Value) -> Value {
    let _serial = crate::protocol::handler_response_serial();
    let state = state();
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(
        &json!(1),
        tool,
        &json!({ "arguments": arguments }),
        &state,
    );
    crate::protocol::captured_responses()
        .pop()
        .expect("registered handler response")
}

fn assert_ambiguity(response: &Value) {
    assert_eq!(response["error"]["code"], -32602, "{response}");
    assert!(
        response["error"]["message"]
            .as_str()
            .is_some_and(|message| message.contains("ambiguous")),
        "{response}"
    );
    assert_eq!(
        response["error"]["data"]["candidates"],
        json!([
            {
                "id": ALPHA,
                "label": "Function",
                "name": BARE,
                "file": "src/alpha.rs"
            },
            {
                "id": BETA,
                "label": "Function",
                "name": BARE,
                "file": "src/beta.rs"
            }
        ]),
        "{response}"
    );
}

#[test]
fn graph_trace_rejects_an_ambiguous_bare_source_before_tracing() {
    let response = dispatch("graph_trace", json!({ "from": BARE, "to": "alpha_leaf" }));

    assert_ambiguity(&response);
}

#[test]
fn proxy_trace_path_rejects_the_same_ambiguous_bare_source() {
    let response = dispatch(
        "cbm_proxy",
        json!({
            "cbm_tool": "trace_path",
            "parameters": {
                "function_name": BARE,
                "direction": "outbound",
                "depth": 3,
                "project": "test-project"
            }
        }),
    );

    assert_ambiguity(&response);
}

#[test]
fn graph_trace_preserves_the_canonical_identity_fast_path() {
    let response = dispatch("graph_trace", json!({ "from": ALPHA, "to": "alpha_leaf" }));

    assert!(response.get("error").is_none(), "{response}");
    assert_eq!(response["result"]["structuredContent"]["count"], 1);
    assert_eq!(
        response["result"]["structuredContent"]["edges"][0]["from"],
        ALPHA
    );
}
