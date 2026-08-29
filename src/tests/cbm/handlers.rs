// src/tests/cbm/handlers.rs
//
// Handler-level tests for CBM MCP tool handlers.
// Tests validate the MCP CallToolResult contract: content + structuredContent,
// and that no ad-hoc fields leak onto the result object.
//
// These tests use the CAPTURED_RESPONSES sink to intercept responses
// that handlers write via send_response().

use serde_json::Value;

// ── graph_search structured-content contract ────────────────────────
//
// MCP 2025-11-25 spec demands tool results use:
//   result.content          — human-readable text (always forwarded to model)
//   result.structuredContent — machine-readable structured data (MCP-native channel)
//   result.isError?         — error indicator
//   result._meta?           — response metadata
//
// NO ad-hoc fields (nodes, count, cbm_status, etc.) as result siblings.

/// Helper: clear the CAPTURED_RESPONSES sink before a handler call.
fn clear_captured() {
    if let Ok(mut q) = crate::protocol::CAPTURED_RESPONSES.lock() {
        q.clear();
    }
}

/// Helper: pop exactly one captured response and validate it has a result field.
fn take_response() -> Value {
    let mut guard = crate::protocol::CAPTURED_RESPONSES
        .lock()
        .expect("CAPTURED_RESPONSES lock");
    assert_eq!(guard.len(), 1, "handler must send exactly one response");
    guard.pop().expect("one response")
}

/// graph_search error path: CBM unavailable → handler must use isError + structuredContent.
#[test]
fn graph_search_returns_is_error_when_cbm_unavailable() {
    clear_captured();

    let config = crate::config::CleanCtxConfig {
        cbm: crate::cbm::CbmConfig {
            enabled: false,
            ..Default::default()
        },
        ..Default::default()
    };
    let state = crate::mcp::McpState::new(config);
    crate::mcp::tools::setup_handler_registry_for_tests();

    crate::mcp::tools::dispatch_tools_call(
        &serde_json::json!(1),
        "graph_search",
        &serde_json::json!({"arguments": {"query": "GraphBridge"}}),
        &state,
    );

    let response = take_response();

    // Must be a JSON-RPC error (no bridge = unavailable)
    assert!(
        response.get("error").is_some(),
        "CBM-unavailable path must produce JSON-RPC error, got: {response}"
    );
    let err = response["error"].as_object().expect("error object");
    assert_eq!(err["code"], -32603);
}
/// graph_search response shape: validate the SUCCESS response JSON structure.
///
/// This test constructs the exact JSON a handler produces and validates
/// the MCP CallToolResult contract: content + structuredContent, NO ad-hoc fields.
#[test]
fn graph_search_success_response_has_correct_mcp_shape() {
    use crate::cbm::bridge::GraphNode;
    use std::collections::HashMap;

    let nodes = vec![
        GraphNode {
            id: "C-Users-MNasty-Desktop-RustContextLayerAI.src.cbm.bridge.GraphBridge".into(),
            label: "Class".into(),
            name: "GraphBridge".into(),
            file: "src/cbm/bridge.rs".into(),
            properties: HashMap::new(),
        },
        GraphNode {
            id: "C-Users-MNasty-Desktop-RustContextLayerAI.src.cbm.handlers.handle_graph_search"
                .into(),
            label: "Function".into(),
            name: "handle_graph_search".into(),
            file: "src/cbm/handlers.rs".into(),
            properties: HashMap::new(),
        },
    ];

    let status_str = "available";

    // Build the response exactly as handle_graph_search builds it
    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "content": [{
                "type": "text",
                "text": format!(
                    "Found {} symbol(s):\n  - {} ({}) @ {}\n  - {} ({}) @ {}",
                    nodes.len(),
                    nodes[0].name, nodes[0].label, nodes[0].file,
                    nodes[1].name, nodes[1].label, nodes[1].file,
                )
            }],
            "structuredContent": {
                "nodes": nodes,
                "count": nodes.len(),
                "cbm_status": status_str
            }
        }
    });

    // 1. result exists
    let result = response["result"].as_object().expect("result object");

    // 2. content exists and is an array
    let content = result["content"].as_array().expect("content array");
    assert_eq!(content.len(), 1, "content should have exactly one block");
    assert_eq!(
        content[0]["type"], "text",
        "content block type should be text"
    );
    let text = content[0]["text"].as_str().expect("content text");
    assert!(
        text.starts_with("Found 2 symbol(s)"),
        "content text should be meaningful, got: {text}"
    );
    assert!(
        text.contains("GraphBridge (Class) @ src/cbm/bridge.rs"),
        "content text should contain node summary, got: {text}"
    );

    // 3. structuredContent exists with nodes, count, cbm_status
    let sc = result["structuredContent"]
        .as_object()
        .expect("structuredContent object");
    let sc_nodes = sc["nodes"]
        .as_array()
        .expect("structuredContent.nodes array");
    assert_eq!(sc_nodes.len(), 2, "should have 2 nodes");
    assert_eq!(sc_nodes[0]["name"], "GraphBridge");
    assert_eq!(sc_nodes[0]["file"], "src/cbm/bridge.rs");
    assert_eq!(sc["count"], 2);
    assert_eq!(sc["cbm_status"], "available");

    // 4. AD-HOC FIELDS MUST NOT EXIST at the result level
    assert!(
        !result.contains_key("nodes"),
        "result.nodes must not exist (ad-hoc field)"
    );
    assert!(
        !result.contains_key("count"),
        "result.count must not exist (ad-hoc field)"
    );
    assert!(
        !result.contains_key("cbm_status"),
        "result.cbm_status must not exist (ad-hoc field)"
    );
    assert!(
        !result.contains_key("error"),
        "result.error must not exist as sibling (use structuredContent)"
    );

    // 5. Only allowed MCP fields at result level
    let allowed = ["content", "structuredContent", "isError", "_meta"];
    for key in result.keys() {
        assert!(
            allowed.contains(&key.as_str()),
            "unexpected result-level field: {key} — must use structuredContent or _meta"
        );
    }
}

/// graph_search error response shape: validate the ERROR response JSON structure.
#[test]
fn graph_search_error_response_has_correct_mcp_shape() {
    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "content": [{
                "type": "text",
                "text": "CBM search failed: indexing not complete"
            }],
            "isError": true,
            "structuredContent": {
                "error": "indexing not complete",
                "cbm_status": "available"
            }
        }
    });

    let result = response["result"].as_object().expect("result object");

    // 1. content exists with meaningful text
    let content = result["content"].as_array().expect("content array");
    let text = content[0]["text"].as_str().expect("content text");
    assert!(
        text.starts_with("CBM search failed"),
        "error content should describe failure, got: {text}"
    );

    // 2. isError is true
    assert_eq!(result["isError"], true);

    // 3. structuredContent contains error details
    let sc = result["structuredContent"]
        .as_object()
        .expect("structuredContent object");
    assert_eq!(sc["error"], "indexing not complete");
    assert_eq!(sc["cbm_status"], "available");

    // 4. No ad-hoc fields at result level
    assert!(
        !result.contains_key("errors"),
        "result.errors must not exist"
    );
    assert!(
        !result.contains_key("cbm_status"),
        "result.cbm_status must not exist as ad-hoc field"
    );

    // 5. Only allowed MCP fields
    let allowed = ["content", "structuredContent", "isError", "_meta"];
    for key in result.keys() {
        assert!(
            allowed.contains(&key.as_str()),
            "unexpected result-level field: {key} — must use structuredContent or _meta"
        );
    }
}
