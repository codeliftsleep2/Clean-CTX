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
    use tempfile::TempDir;

    let _dir = TempDir::new().unwrap();
    let controller_path = _dir.path().join("UserController.java");
    let service_path = _dir.path().join("UserService.java");
    std::fs::write(&controller_path, "public class UserController {}").unwrap();
    std::fs::write(&service_path, "public class UserService {}").unwrap();
    let controller_str = controller_path.to_string_lossy().to_string();
    let service_str = service_path.to_string_lossy().to_string();

    let mut idx = state.workspace_index_lock();

    // Simulate a UserController that autowires UserService
    idx.add_edges(
        &controller_str,
        vec![SemanticEdge {
            relation: SemanticRelation::Autowired,
            subject: EntityRef::new("spring", "Controller", "UserController")
                .with_file(controller_str.clone()),
            object: EntityRef::new("spring", "Service", "UserService")
                .with_file(controller_str.clone()),
            layer: "spring",
        }],
    );

    // Simulate a UserService that is a service
    // (no self-loop — avoid creating a cycle in the seed data)
    idx.add_edges(
        &service_str,
        vec![SemanticEdge {
            relation: SemanticRelation::EndpointMapsTo,
            subject: EntityRef::new("spring", "Service", "UserService")
                .with_file(service_str.clone()),
            object: EntityRef::new("spring", "Endpoint", "/api/users")
                .with_file(service_str.clone()),
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
    use tempfile::TempDir;
    let dir = TempDir::new().unwrap();
    let controller_path = dir.path().join("UserController.java");
    std::fs::write(&controller_path, "public class UserController {}").unwrap();
    let controller_str = controller_path.to_string_lossy().to_string();

    let service_path = dir.path().join("UserService.java");
    std::fs::write(&service_path, "public class UserService {}").unwrap();
    let service_str = service_path.to_string_lossy().to_string();

    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let mut idx = state.workspace_index_lock();
    use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};

    idx.add_edges(
        &controller_str,
        vec![SemanticEdge {
            relation: SemanticRelation::Autowired,
            subject: EntityRef::new("spring", "Controller", "UserController")
                .with_file(controller_str.clone()),
            object: EntityRef::new("spring", "Service", "UserService")
                .with_file(controller_str.clone()),
            layer: "spring",
        }],
    );

    idx.add_edges(
        &service_str,
        vec![SemanticEdge {
            relation: SemanticRelation::EndpointMapsTo,
            subject: EntityRef::new("spring", "Service", "UserService")
                .with_file(service_str.clone()),
            object: EntityRef::new("spring", "Endpoint", "/api/users")
                .with_file(service_str.clone()),
            layer: "spring",
        }],
    );
    drop(idx);

    let id = json!(1);
    let params = json!({
        "arguments": {
            "type": "entities_in_file",
            "file_path": controller_str,
            "workspaceRoot": dir.path().to_string_lossy().to_string()
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
fn entities_in_file_production_path_canonicalization() {
    use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};
    use crate::mcp::tool_helpers::resolve_file_path_checked;
    use tempfile::TempDir;

    let dir = TempDir::new().unwrap();
    let cs_path = dir.path().join("TestController.cs");
    std::fs::write(&cs_path, "public class TestController {}").unwrap();

    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = json!(1);

    // Populate the index using the production-style path:
    // resolve_file_path_checked() → canonical_identity_key().
    // This is identical to how handle_provide_code_context
    // stores paths.
    let relative_path = "TestController.cs";
    let resolved = resolve_file_path_checked(
        relative_path,
        Some(dir.path().to_string_lossy().as_ref()),
        &[],
    )
    .unwrap();
    let canonical_path = crate::dictionary::path::canonical_identity_key(&resolved);

    // The resolved canonical path differs from the raw relative path.
    assert_ne!(
        canonical_path, relative_path,
        "resolved+cannonical path must differ from raw relative path"
    );

    {
        let mut idx = state.workspace_index_lock();
        idx.add_edges(
            &canonical_path,
            vec![SemanticEdge {
                relation: SemanticRelation::Defines,
                subject: EntityRef::new("test", "Module", "TestQueryModule")
                    .with_file(canonical_path.clone()),
                object: EntityRef::new("test", "Function", "test_query_fn")
                    .with_file(canonical_path.clone()),
                layer: "test",
            }],
        );
    }

    // Query with the same relative path and workspaceRoot.
    // This reproduces the production defect: the handler must
    // resolve the path (relative → absolute via workspaceRoot)
    // BEFORE canonicalizing, matching the write-side pipeline.
    let params = json!({
        "arguments": {
            "type": "entities_in_file",
            "file_path": relative_path,
            "workspaceRoot": dir.path().to_string_lossy().to_string()
        }
    });
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&id, "workspace_query", &params, &state);
    let resp = pop_response();

    assert!(
        resp.get("result").is_some(),
        "entities_in_file should succeed"
    );
    let entities = &resp["result"]["entities"];
    assert!(entities.is_array(), "entities should be an array");
    // RED → GREEN: would fail with only canonical_identity_key()
    // (the previous fix) because raw relative path "TestController.cs"
    // does not exist from CWD and resolves to a different key.
    // Passes after adding resolve_file_path_checked so the handler
    // uses the same workspaceRoot-aware resolution as the write path.
    assert!(
        !entities.as_array().unwrap().is_empty(),
        "entities_in_file must find entities stored under resolved+cannonical path"
    );
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

// ── Token-economics regression (Issue #2) ──────────────────────────
//
// Verifies that compile_file_ir_focused semantic edges survive the
// token-economics unfavorable-prediction fallback path.
//
// The write path (handle_provide_code_context) compiles a small file
// at Edit fidelity. Token-economics predicts the full render is not
// economical and returns raw passthrough. BEFORE the fix, the extracted
// semantic edges were bound to `_` and discarded, leaving the
// WorkspaceIndex empty. AFTER the fix, they are persisted to the index.
//
// Uses a .cs file with [ApiController] so DotNetMetaLayer produces
// semantic entities. The file is intentionally small (<~50 raw tokens)
// to guarantee the token-economics unfavorable prediction for Edit
// fidelity on .cs files (threshold ~612 tokens).

#[test]
#[cfg(any(feature = "csharp", feature = "dotnet"))]
fn find_entities_after_token_economics_fallback() {
    use crate::mcp::tools::dispatch_tools_call;
    use serde_json::json;
    use tempfile::TempDir;

    let dir = TempDir::new().unwrap();
    let cs_path = dir.path().join("TestController.cs");
    let cs_content = r#"
using Microsoft.AspNetCore.Mvc;

[ApiController]
[Route("api/test")]
public class TestController : ControllerBase
{
    [HttpGet]
    public IActionResult Get() => Ok("hello");
}
"#;
    std::fs::write(&cs_path, cs_content).unwrap();
    let file_str = cs_path.to_string_lossy().to_string();

    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = json!(1);

    // Step 1: Call provide_code_context with Edit fidelity.
    // The file is small so token-economics should predict
    // unfavorable and return raw passthrough.
    let pcc_params = json!({
        "arguments": {
            "filePath": file_str,
            "fidelity": "edit",
            "workspaceRoot": dir.path().to_string_lossy().to_string()
        }
    });
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&id, "provide_code_context", &pcc_params, &state);
    let pcc_resp = pop_response();
    assert!(
        pcc_resp.get("result").is_some(),
        "provide_code_context should succeed for small file"
    );

    // Confirm the response uses the economics fallback (raw passthrough)
    let is_fallback = pcc_resp["result"]["_meta"]["content_kind"]
        .as_str()
        .map(|k| k == "raw_passthrough")
        .unwrap_or(false);
    assert!(
        is_fallback,
        "small file at Edit fidelity must trigger token-economics fallback: {:?}",
        pcc_resp["result"]["_meta"]["content_kind"].as_str()
    );

    // Step 2: Query find_entities for the expected entity name.
    // DotNetMetaLayer produces: Controller entity named "TestController"
    let wq_params = json!({
        "arguments": {
            "type": "find_entities",
            "name": "TestController"
        }
    });
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&id, "workspace_query", &wq_params, &state);
    let resp = pop_response();

    assert!(
        resp.get("result").is_some(),
        "workspace_query.find_entities should succeed"
    );
    let count = resp["result"]["count"].as_i64().unwrap_or(-1);
    // RED → GREEN: fails before the fix (count == 0 because semantic
    // edges discarded in token-economics fallback), passes after the
    // fix (edges persisted to WorkspaceIndex before fallback).
    assert!(
        count > 0,
        "find_entities must find entity after token-economics fallback; count={}",
        count
    );

    // Verify the returned entity has the expected identity.
    let entities = resp["result"]["entities"].as_array().unwrap();
    let controller_entity = entities.iter().find(|e| {
        e["domain"].as_str() == Some("dotnet")
            && e["entity_type"].as_str() == Some("Controller")
            && e["name"].as_str() == Some("TestController")
    });
    assert!(
        controller_entity.is_some(),
        "dotnet Controller 'TestController' must be in find_entities results"
    );
}
