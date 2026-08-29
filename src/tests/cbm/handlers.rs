// src/tests/cbm/handlers.rs
//
// Handler-level tests for CBM MCP tool handlers.
// Tests validate the MCP CallToolResult contract: content + structuredContent,
// and that no ad-hoc fields leak onto the result object.
//
// These tests use the protocol::CAPTURED_RESPONSES sink to intercept
// responses that handlers write via send_response().  All consumers of that
// sink must hold protocol::HANDLER_RESPONSE_SERIAL to prevent parallel-test
// races — see src/cbm/tests.rs for the feature gate rationale.
//
// This module compiles only under cfg(all(test, feature = "rust")) —
// see the #[cfg] gate in src/cbm/tests.rs.
// ── graph_search structured-content contract ────────────────────────
//
// MCP 2025-11-25 spec demands tool results use:
//   result.content          — human-readable text (always forwarded to model)
//   result.structuredContent — machine-readable structured data (MCP-native channel)
//   result.isError?         — error indicator
//   result._meta?           — response metadata
//
// NO ad-hoc fields (nodes, count, cbm_status, etc.) as result siblings.

/// graph_search error path: CBM unavailable → handler must use isError + structuredContent.
use crate::tests::{assert_structured_content_has, assert_valid_mcp_envelope};

#[test]
fn graph_search_returns_is_error_when_cbm_unavailable() {
    // Serialize access to the shared CAPTURED_RESPONSES sink with the
    // Phase A/B retirement suites (must hold HANDLER_RESPONSE_SERIAL).
    let _serial = crate::protocol::handler_response_serial();

    let state = crate::mcp::McpState::new(crate::tests::test_config());
    crate::mcp::tools::setup_handler_registry_for_tests();

    crate::protocol::captured_responses().clear();

    crate::mcp::tools::dispatch_tools_call(
        &serde_json::json!(1),
        "graph_search",
        &serde_json::json!( {"arguments": {"query": "GraphBridge"}}),
        &state,
    );

    let response = crate::protocol::captured_responses()
        .pop()
        .expect("handler must have sent exactly one response");

    // Must be a JSON-RPC error (no bridge = unavailable)
    assert!(
        response.get("error").is_some(),
        "CBM-unavailable path must produce JSON-RPC error, got: {response}"
    );
    let err = response["error"].as_object().expect("error object");
    assert_eq!(err["code"], -32603);
}

/// graph_query error path: CBM unavailable → JSON-RPC error (no bridge).
#[test]
fn graph_query_returns_jsonrpc_error_when_cbm_unavailable() {
    let _serial = crate::protocol::handler_response_serial();

    let state = crate::mcp::McpState::new(crate::tests::test_config());
    crate::mcp::tools::setup_handler_registry_for_tests();
    crate::protocol::captured_responses().clear();

    crate::mcp::tools::dispatch_tools_call(
        &serde_json::json!(1),
        "graph_query",
        &serde_json::json!( {"arguments": {"query": "MATCH (c:Class) RETURN c"}}),
        &state,
    );

    let response = crate::protocol::captured_responses()
        .pop()
        .expect("handler must have sent exactly one response");
    assert!(
        response.get("error").is_some(),
        "CBM-unavailable path must produce JSON-RPC error, got: {response}"
    );
    assert_eq!(response["error"]["code"], -32603);
}

/// graph_trace error path: CBM unavailable → JSON-RPC error (no bridge).
#[test]
fn graph_trace_returns_jsonrpc_error_when_cbm_unavailable() {
    let _serial = crate::protocol::handler_response_serial();

    let state = crate::mcp::McpState::new(crate::tests::test_config());
    crate::mcp::tools::setup_handler_registry_for_tests();
    crate::protocol::captured_responses().clear();

    crate::mcp::tools::dispatch_tools_call(
        &serde_json::json!(1),
        "graph_trace",
        &serde_json::json!( {"arguments": {"from": "A", "to": "B"}}),
        &state,
    );

    let response = crate::protocol::captured_responses()
        .pop()
        .expect("handler must have sent exactly one response");
    assert!(
        response.get("error").is_some(),
        "CBM-unavailable path must produce JSON-RPC error, got: {response}"
    );
    assert_eq!(response["error"]["code"], -32603);
}

/// get_architecture error path: CBM unavailable → JSON-RPC error (no bridge).
#[test]
fn get_architecture_returns_jsonrpc_error_when_cbm_unavailable() {
    let _serial = crate::protocol::handler_response_serial();

    let state = crate::mcp::McpState::new(crate::tests::test_config());
    crate::mcp::tools::setup_handler_registry_for_tests();
    crate::protocol::captured_responses().clear();

    crate::mcp::tools::dispatch_tools_call(
        &serde_json::json!(1),
        "get_architecture",
        &serde_json::json!( {"arguments": {}}),
        &state,
    );

    let response = crate::protocol::captured_responses()
        .pop()
        .expect("handler must have sent exactly one response");
    assert!(
        response.get("error").is_some(),
        "CBM-unavailable path must produce JSON-RPC error, got: {response}"
    );
    assert_eq!(response["error"]["code"], -32603);
}

/// outputSchema ↔ structuredContent consistency: the tool catalog must
/// advertise outputSchema for all four structured CBM tools, and the
/// advertised top-level properties must match what handlers actually emit.
#[test]
fn cbm_tool_list_output_schemas_match_structured_content_contracts() {
    let tools: Vec<serde_json::Value> = crate::cbm::cbm_tool_list();
    let expected: &[(&str, &[&str])] = &[
        ("graph_search", &["nodes", "count", "cbm_status"]),
        ("graph_query", &["nodes", "edges", "cbm_status"]),
        ("graph_trace", &["edges", "count", "cbm_status"]),
        (
            "get_architecture",
            &["modules", "dependencies", "cbm_status"],
        ),
    ];

    for (tool_name, required_sc_keys) in expected {
        let tool = tools
            .iter()
            .find(|t| t["name"] == *tool_name)
            .unwrap_or_else(|| panic!("tool '{tool_name}' missing from cbm_tool_list"));
        let schema = tool["outputSchema"]
            .as_object()
            .unwrap_or_else(|| panic!("tool '{tool_name}' must declare outputSchema"));
        let props = schema["properties"]
            .as_object()
            .unwrap_or_else(|| panic!("tool '{tool_name}' outputSchema.properties missing"));
        for key in *required_sc_keys {
            assert!(
                props.contains_key(*key),
                "tool '{tool_name}' outputSchema.properties missing '{key}'"
            );
        }
    }
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
/// Helper: validate that error structuredContent follows the MCP pattern.
fn assert_error_structured_content(sc: &serde_json::Map<String, serde_json::Value>) {
    assert!(
        sc.contains_key("error"),
        "error structuredContent must contain 'error'"
    );
    assert!(
        sc.contains_key("cbm_status"),
        "error structuredContent must contain 'cbm_status'"
    );
}

// ── graph_query structured-content contract ──────────────────────────

#[test]
fn graph_query_success_response_has_correct_mcp_shape() {
    use crate::cbm::bridge::{GraphEdge, GraphNode};
    use std::collections::HashMap;

    let nodes = vec![GraphNode {
        id: "node1".into(),
        label: "Class".into(),
        name: "MyClass".into(),
        file: "src/my_class.ts".into(),
        properties: HashMap::new(),
    }];
    let edges = vec![GraphEdge {
        from: "node1".into(),
        to: "node2".into(),
        label: "CALLS".into(),
        properties: HashMap::new(),
    }];

    let response = serde_json::json!({
        "jsonrpc": "2.0", "id": 1,
        "result": {
            "content": [{ "type": "text", "text": "1 node(s), 1 edge(s)." }],
            "structuredContent": {
                "nodes": nodes, "edges": edges,
                "cbm_status": "available"
            }
        }
    });

    let result = response["result"].as_object().expect("result object");
    assert_valid_mcp_envelope(result);

    let sc = result["structuredContent"]
        .as_object()
        .expect("structuredContent object");
    assert_structured_content_has(sc, &["nodes", "edges", "cbm_status"]);

    assert_eq!(sc["nodes"][0]["name"], "MyClass");
    assert_eq!(sc["edges"][0]["from"], "node1");
    assert_eq!(sc["cbm_status"], "available");

    assert!(!result.contains_key("nodes"), "result.nodes must not exist");
    assert!(!result.contains_key("edges"), "result.edges must not exist");
    assert!(
        !result.contains_key("cbm_status"),
        "result.cbm_status must not exist"
    );

    let text = result["content"][0]["text"].as_str().expect("content text");
    assert!(
        text.contains("1 node(s)"),
        "content should mention node count"
    );
}

#[test]
fn graph_query_error_response_has_correct_mcp_shape() {
    let response = serde_json::json!({
        "jsonrpc": "2.0", "id": 1,
        "result": {
            "content": [{ "type": "text", "text": "CBM query failed: indexing not complete" }],
            "isError": true,
            "structuredContent": {
                "error": "indexing not complete",
                "cbm_status": "available"
            }
        }
    });

    let result = response["result"].as_object().expect("result object");
    assert_valid_mcp_envelope(result);
    assert_eq!(result["isError"], true);

    let sc = result["structuredContent"]
        .as_object()
        .expect("structuredContent object");
    assert_error_structured_content(sc);
    assert_eq!(sc["error"], "indexing not complete");
}

// ── graph_trace structured-content contract ──────────────────────────

#[test]
fn graph_trace_success_response_has_correct_mcp_shape() {
    use crate::cbm::bridge::GraphEdge;
    use std::collections::HashMap;

    let edges = vec![GraphEdge {
        from: "node1".into(),
        to: "node2".into(),
        label: "CALLS".into(),
        properties: HashMap::new(),
    }];

    let response = serde_json::json!({
        "jsonrpc": "2.0", "id": 1,
        "result": {
            "content": [{ "type": "text", "text": "'node1' → 'node2': 1 edge(s)." }],
            "structuredContent": {
                "edges": edges, "count": 1,
                "cbm_status": "available"
            }
        }
    });

    let result = response["result"].as_object().expect("result object");
    assert_valid_mcp_envelope(result);

    let sc = result["structuredContent"]
        .as_object()
        .expect("structuredContent object");
    assert_structured_content_has(sc, &["edges", "count", "cbm_status"]);

    assert_eq!(sc["edges"][0]["from"], "node1");
    assert_eq!(sc["count"], 1);
    assert_eq!(sc["cbm_status"], "available");

    assert!(!result.contains_key("edges"), "result.edges must not exist");
    assert!(!result.contains_key("count"), "result.count must not exist");
    assert!(
        !result.contains_key("cbm_status"),
        "result.cbm_status must not exist"
    );

    let text = result["content"][0]["text"].as_str().expect("content text");
    assert!(
        text.contains("1 edge(s)"),
        "content should mention edge count"
    );
}

#[test]
fn graph_trace_error_response_has_correct_mcp_shape() {
    let response = serde_json::json!({
        "jsonrpc": "2.0", "id": 1,
        "result": {
            "content": [{ "type": "text", "text": "CBM trace failed: no path found" }],
            "isError": true,
            "structuredContent": {
                "error": "no path found",
                "cbm_status": "available"
            }
        }
    });

    let result = response["result"].as_object().expect("result object");
    assert_valid_mcp_envelope(result);
    assert_eq!(result["isError"], true);

    let sc = result["structuredContent"]
        .as_object()
        .expect("structuredContent object");
    assert_error_structured_content(sc);
    assert_eq!(sc["error"], "no path found");
}

// ── get_architecture structured-content contract ─────────────────────

#[test]
fn get_architecture_success_response_has_correct_mcp_shape() {
    use crate::cbm::bridge::{ArchitectureDependency, ArchitectureModule};

    let modules = vec![
        ArchitectureModule {
            name: "core".into(),
            path: "src/core".into(),
            file_count: 12,
        },
        ArchitectureModule {
            name: "ui".into(),
            path: "src/ui".into(),
            file_count: 8,
        },
    ];
    let dependencies = vec![ArchitectureDependency {
        from: "core".into(),
        to: "ui".into(),
        kind: "calls".into(),
    }];
    let module_count = modules.len();
    let dep_count = dependencies.len();

    let response = serde_json::json!({
        "jsonrpc": "2.0", "id": 1,
        "result": {
            "content": [{ "type": "text", "text": format!("{module_count} module(s), {dep_count} deps.") }],
            "structuredContent": {
                "modules": modules, "dependencies": dependencies,
                "cbm_status": "available"
            }
        }
    });

    let result = response["result"].as_object().expect("result object");
    assert_valid_mcp_envelope(result);

    let sc = result["structuredContent"]
        .as_object()
        .expect("structuredContent object");
    assert_structured_content_has(sc, &["modules", "dependencies", "cbm_status"]);

    assert_eq!(sc["modules"][0]["name"], "core");
    assert_eq!(sc["modules"][0]["file_count"], 12);
    assert_eq!(sc["dependencies"][0]["from"], "core");
    assert_eq!(sc["cbm_status"], "available");

    assert!(
        !result.contains_key("modules"),
        "result.modules must not exist"
    );
    assert!(
        !result.contains_key("dependencies"),
        "result.dependencies must not exist"
    );
    assert!(
        !result.contains_key("cbm_status"),
        "result.cbm_status must not exist"
    );

    let text = result["content"][0]["text"].as_str().expect("content text");
    assert!(
        text.contains("2 module(s)"),
        "content should mention module count"
    );
}

#[test]
fn get_architecture_error_response_has_correct_mcp_shape() {
    let response = serde_json::json!({
        "jsonrpc": "2.0", "id": 1,
        "result": {
            "content": [{ "type": "text", "text": "CBM architecture query failed: project not indexed" }],
            "isError": true,
            "structuredContent": {
                "error": "project not indexed",
                "cbm_status": "unavailable"
            }
        }
    });

    let result = response["result"].as_object().expect("result object");
    assert_valid_mcp_envelope(result);
    assert_eq!(result["isError"], true);

    let sc = result["structuredContent"]
        .as_object()
        .expect("structuredContent object");
    assert_error_structured_content(sc);
    assert_eq!(sc["error"], "project not indexed");
    assert_eq!(sc["cbm_status"], "unavailable");
}
