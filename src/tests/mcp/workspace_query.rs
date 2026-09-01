// src/tests/mcp/workspace_query.rs
//
// Tests for the workspace_query MCP tool handler.
// Exercises the handler through the MCP dispatch path, verifying
// argument validation, dispatch to WorkspaceIndex methods, and
// response format.

use crate::mcp::tools::dispatch_tools_call;
use serde_json::json;

// ── Helpers ───────────────────────────────────────────────────────────

/// Helper: create a minimal WorkspaceIndex with semantic edges from
/// test fixture files. This simulates the state that would exist after
/// a file has been compiled through the production pipeline.
fn seed_workspace_index(state: &crate::mcp::McpState) {
    use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};
    let mut idx = state.workspace_index_lock();

    // Simulate a UserController that autowires UserService
    idx.add_edges(
        "src/UserController.java",
        vec![SemanticEdge {
            relation: SemanticRelation::Autowired,
            subject: EntityRef::new("spring", "Controller", "UserController")
                .with_file("src/UserController.java".into()),
            object: EntityRef::new("spring", "Service", "UserService")
                .with_file("src/UserController.java".into()),
            layer: "spring",
        }],
    );

    // Simulate a UserService that is a service
    // (no self-loop — avoid creating a cycle in the seed data)
    idx.add_edges(
        "src/UserService.java",
        vec![SemanticEdge {
            relation: SemanticRelation::EndpointMapsTo,
            subject: EntityRef::new("spring", "Service", "UserService")
                .with_file("src/UserService.java".into()),
            object: EntityRef::new("spring", "Endpoint", "/api/users")
                .with_file("src/UserService.java".into()),
            layer: "spring",
        }],
    );
}

fn pop_response() -> serde_json::Value {
    crate::protocol::captured_responses()
        .pop()
        .expect("handler must send response")
}

// ── Query type validation tests ───────────────────────────────────────

#[test]
fn workspace_query_rejects_unknown_type() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = json!(1);
    let params = json!({ "arguments": { "type": "invalid_query" } });
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&id, "workspace_query", &params, &state);
    let resp = pop_response();
    assert!(
        resp.get("error").is_some(),
        "unknown type should produce error"
    );
    let msg = resp["error"]["message"].as_str().unwrap_or("");
    assert!(
        msg.contains("invalid_query"),
        "error should mention the invalid type: {msg}"
    );
}

#[test]
fn workspace_query_rejects_missing_type() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = json!(1);
    let params = json!({ "arguments": {} });
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&id, "workspace_query", &params, &state);
    let resp = pop_response();
    assert!(
        resp.get("error").is_some(),
        "missing type should produce error"
    );
    let code = resp["error"]["code"].as_i64().unwrap_or(0);
    assert_eq!(
        code, -32602,
        "missing type should produce InvalidParams error"
    );
}

#[test]
fn workspace_query_find_entities_missing_name() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = json!(1);
    let params = json!({ "arguments": { "type": "find_entities" } });
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&id, "workspace_query", &params, &state);
    let resp = pop_response();
    assert!(
        resp.get("error").is_some(),
        "missing name should produce error"
    );
}

#[test]
fn workspace_query_forward_edges_missing_args() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = json!(1);
    let params = json!({ "arguments": { "type": "forward_edges" } });
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&id, "workspace_query", &params, &state);
    let resp = pop_response();
    assert!(
        resp.get("error").is_some(),
        "missing domain should produce error"
    );
}

#[test]
fn workspace_query_entities_in_file_missing_path() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = json!(1);
    let params = json!({ "arguments": { "type": "entities_in_file" } });
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&id, "workspace_query", &params, &state);
    let resp = pop_response();
    assert!(
        resp.get("error").is_some(),
        "missing file_path should produce error"
    );
}

#[test]
fn workspace_query_transitive_deps_missing_args() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = json!(1);
    let params = json!({ "arguments": { "type": "transitive_dependencies" } });
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&id, "workspace_query", &params, &state);
    let resp = pop_response();
    assert!(
        resp.get("error").is_some(),
        "missing domain should produce error"
    );
}

// ── Query dispatch tests ─────────────────────────────────────────────

#[test]
fn workspace_query_find_entities_returns_results() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    seed_workspace_index(&state);
    let id = json!(1);
    let params = json!({ "arguments": { "type": "find_entities", "name": "UserService" } });
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&id, "workspace_query", &params, &state);
    let resp = pop_response();
    assert!(resp.get("result").is_some(), "should return result");
    let entities = &resp["result"]["entities"];
    assert!(entities.is_array(), "entities should be an array");
    assert!(
        !entities.as_array().unwrap().is_empty(),
        "should find at least one entity"
    );
    let count = resp["result"]["count"].as_i64().unwrap_or(0);
    assert!(count > 0, "count should be > 0");
}

#[test]
fn workspace_query_find_entities_empty_index() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = json!(1);
    let params = json!({ "arguments": { "type": "find_entities", "name": "NonExistent" } });
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&id, "workspace_query", &params, &state);
    let resp = pop_response();
    assert!(
        resp.get("result").is_some(),
        "empty index should return result, not error"
    );
    let entities = &resp["result"]["entities"];
    assert!(entities.is_array(), "entities should be an array");
    assert_eq!(
        entities.as_array().unwrap().len(),
        0,
        "empty index should return empty array"
    );
    let count = resp["result"]["count"].as_i64().unwrap_or(-1);
    assert_eq!(count, 0, "count should be 0 for empty result");
}

#[test]
fn workspace_query_forward_edges_returns_results() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    seed_workspace_index(&state);
    let id = json!(1);
    let params = json!({
        "arguments": {
            "type": "forward_edges",
            "domain": "spring",
            "entity_type": "Controller",
            "name": "UserController"
        }
    });
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&id, "workspace_query", &params, &state);
    let resp = pop_response();
    assert!(resp.get("result").is_some(), "should return result");
    let edges = &resp["result"]["edges"];
    assert!(edges.is_array(), "edges should be an array");
    let count = resp["result"]["count"].as_i64().unwrap_or(0);
    assert!(count > 0, "should have at least one forward edge");
    if let Some(edge) = edges.as_array().and_then(|a| a.first()) {
        let relation = edge["relation"].as_str().unwrap_or("");
        assert_eq!(relation, "Autowired", "forward edge should be Autowired");
        let obj_name = edge["object"]["name"].as_str().unwrap_or("");
        assert_eq!(
            obj_name, "UserService",
            "forward edge should point to UserService"
        );
    }
}

#[test]
fn workspace_query_reverse_edges_returns_results() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    seed_workspace_index(&state);
    let id = json!(1);
    let params = json!({
        "arguments": {
            "type": "reverse_edges",
            "domain": "spring",
            "entity_type": "Service",
            "name": "UserService"
        }
    });
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&id, "workspace_query", &params, &state);
    let resp = pop_response();
    assert!(resp.get("result").is_some(), "should return result");
    let edges = &resp["result"]["edges"];
    assert!(edges.is_array(), "edges should be an array");
    let count = resp["result"]["count"].as_i64().unwrap_or(0);
    assert!(count > 0, "should have at least one reverse edge");
    if let Some(edge) = edges.as_array().and_then(|a| a.first()) {
        let subj_name = edge["subject"]["name"].as_str().unwrap_or("");
        assert_eq!(
            subj_name, "UserController",
            "reverse edge should come from UserController"
        );
    }
}
#[test]
fn workspace_query_entities_in_file_returns_results() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    seed_workspace_index(&state);
    let id = json!(1);
    let params = json!({
        "arguments": {
            "type": "entities_in_file",
            "file_path": "src/UserController.java"
        }
    });
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&id, "workspace_query", &params, &state);
    let resp = pop_response();
    assert!(resp.get("result").is_some(), "should return result");
    let entities = &resp["result"]["entities"];
    assert!(entities.is_array(), "entities should be an array");
    let count = resp["result"]["count"].as_i64().unwrap_or(0);
    assert!(count > 0, "should have at least one entity in file");
}

#[test]
fn workspace_query_transitive_dependencies_returns_results() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    seed_workspace_index(&state);
    let id = json!(1);
    let params = json!({
        "arguments": {
            "type": "transitive_dependencies",
            "domain": "spring",
            "entity_type": "Controller",
            "name": "UserController",
            "depth": 1
        }
    });
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&id, "workspace_query", &params, &state);
    let resp = pop_response();
    assert!(resp.get("result").is_some(), "should return result");
    let deps = &resp["result"]["dependencies"];
    assert!(deps.is_array(), "dependencies should be an array");
    let count = resp["result"]["count"].as_i64().unwrap_or(-1);
    assert!(count >= 0, "count should be >= 0");
    let depth_used = resp["result"]["depth_used"].as_i64().unwrap_or(-1);
    assert_eq!(depth_used, 1, "depth_used should match the requested depth");
}

#[test]
fn workspace_query_transitive_dependencies_default_depth() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    seed_workspace_index(&state);
    let id = json!(1);
    let params = json!({
        "arguments": {
            "type": "transitive_dependencies",
            "domain": "spring",
            "entity_type": "Controller",
            "name": "UserController"
        }
    });
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&id, "workspace_query", &params, &state);
    let resp = pop_response();
    let depth_used = resp["result"]["depth_used"].as_i64().unwrap_or(-1);
    assert_eq!(depth_used, 1, "default depth should be 1");
}

#[test]
fn workspace_query_has_cycle_returns_result() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    seed_workspace_index(&state);
    let id = json!(1);
    let params = json!({ "arguments": { "type": "has_cycle" } });
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&id, "workspace_query", &params, &state);
    let resp = pop_response();
    assert!(resp.get("result").is_some(), "should return result");
    let has_cycle = &resp["result"]["has_cycle"];
    assert_eq!(has_cycle, false, "seeded data should not have cycles");
}

#[test]
fn workspace_query_has_cycle_empty_index() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = json!(1);
    let params = json!({ "arguments": { "type": "has_cycle" } });
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&id, "workspace_query", &params, &state);
    let resp = pop_response();
    assert!(
        resp.get("result").is_some(),
        "empty index should return result"
    );
    let has_cycle = &resp["result"]["has_cycle"];
    assert_eq!(has_cycle, false, "empty index should not have cycles");
}
// ── Empty index tests ────────────────────────────────────────────────

#[test]
fn workspace_query_empty_index_forward_edges() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = json!(1);
    let params = json!({
        "arguments": {
            "type": "forward_edges",
            "domain": "spring",
            "entity_type": "Controller",
            "name": "NonExistent"
        }
    });
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&id, "workspace_query", &params, &state);
    let resp = pop_response();
    assert!(
        resp.get("result").is_some(),
        "empty index should return result"
    );
    let edges = &resp["result"]["edges"];
    assert!(edges.is_array(), "edges should be an array");
    assert_eq!(
        edges.as_array().unwrap().len(),
        0,
        "empty index should return empty array"
    );
}

#[test]
fn workspace_query_empty_index_entities_in_file() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = json!(1);
    let params = json!({
        "arguments": {
            "type": "entities_in_file",
            "file_path": "nonexistent.java"
        }
    });
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&id, "workspace_query", &params, &state);
    let resp = pop_response();
    assert!(
        resp.get("result").is_some(),
        "should return result even for missing file"
    );
    let entities = &resp["result"]["entities"];
    assert!(entities.is_array(), "entities should be an array");
    assert_eq!(
        entities.as_array().unwrap().len(),
        0,
        "missing file should return empty array"
    );
}

// ── Existing CBM tool verification ───────────────────────────────────

#[test]
fn workspace_query_does_not_affect_cbm_tools() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = json!(1);

    // Call workspace_query first
    let params = json!({ "arguments": { "type": "has_cycle" } });
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&id, "workspace_query", &params, &state);
    let resp = pop_response();
    assert!(
        resp.get("result").is_some(),
        "workspace_query should succeed"
    );

    // Verify dispatch_tools_call still routes to CBM for cbm_proxy
    let cbm_params = json!({ "arguments": { "cbm_tool": "search_graph", "parameters": {} } });
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&id, "cbm_proxy", &cbm_params, &state);
    let cbm_resp = pop_response();
    assert!(
        cbm_resp.get("error").is_some() || cbm_resp.get("result").is_some(),
        "cbm_proxy should still dispatch without panic"
    );
}
