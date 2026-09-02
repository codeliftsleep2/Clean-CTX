// src/tests/mcp/workspace_query.rs
//
// Tests for the workspace_query MCP tool handler.
// Exercises the handler through the MCP dispatch path, verifying
// argument validation, dispatch to WorkspaceIndex methods, and
// response format.

use crate::mcp::tools::dispatch_tools_call;
use crate::tests::assert_valid_mcp_envelope;
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
    let result = resp["result"].as_object().expect("result object");
    assert_valid_mcp_envelope(result);
    let sc = result["structuredContent"]
        .as_object()
        .expect("structuredContent");
    let entities = &sc["entities"];
    assert!(entities.is_array(), "entities should be an array");
    assert!(
        !entities.as_array().unwrap().is_empty(),
        "should find at least one entity"
    );
    let count = sc["count"].as_i64().unwrap_or(0);
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
    let result = resp["result"].as_object().expect("result object");
    assert_valid_mcp_envelope(result);
    let sc = result["structuredContent"]
        .as_object()
        .expect("structuredContent");
    let entities = &sc["entities"];
    assert!(entities.is_array(), "entities should be an array");
    assert_eq!(
        entities.as_array().unwrap().len(),
        0,
        "empty index should return empty array"
    );
    let count = sc["count"].as_i64().unwrap_or(-1);
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
    let result = resp["result"].as_object().expect("result object");
    assert_valid_mcp_envelope(result);
    let sc = result["structuredContent"]
        .as_object()
        .expect("structuredContent");
    let edges = &sc["edges"];
    assert!(edges.is_array(), "edges should be an array");
    let count = sc["count"].as_i64().unwrap_or(0);
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
    let result = resp["result"].as_object().expect("result object");
    assert_valid_mcp_envelope(result);
    let sc = result["structuredContent"]
        .as_object()
        .expect("structuredContent");
    let edges = &sc["edges"];
    assert!(edges.is_array(), "edges should be an array");
    let count = sc["count"].as_i64().unwrap_or(0);
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
    let result = resp["result"].as_object().expect("result object");
    assert_valid_mcp_envelope(result);
    let sc = result["structuredContent"]
        .as_object()
        .expect("structuredContent");
    let entities = &sc["entities"];
    assert!(entities.is_array(), "entities should be an array");
    let count = sc["count"].as_i64().unwrap_or(0);
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
    let result = resp["result"].as_object().expect("result object");
    assert_valid_mcp_envelope(result);
    let sc = result["structuredContent"]
        .as_object()
        .expect("structuredContent");
    let entities = &sc["entities"];
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
    let result = resp["result"].as_object().expect("result object");
    assert_valid_mcp_envelope(result);
    let sc = result["structuredContent"]
        .as_object()
        .expect("structuredContent");
    let deps = &sc["dependencies"];
    assert!(deps.is_array(), "dependencies should be an array");
    let count = sc["count"].as_i64().unwrap_or(-1);
    assert!(count >= 0, "count should be >= 0");
    let depth_used = sc["depth_used"].as_i64().unwrap_or(-1);
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
    let result = resp["result"].as_object().expect("result object");
    assert_valid_mcp_envelope(result);
    let sc = result["structuredContent"]
        .as_object()
        .expect("structuredContent");
    let depth_used = sc["depth_used"].as_i64().unwrap_or(-1);
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
    let result = resp["result"].as_object().expect("result object");
    assert_valid_mcp_envelope(result);
    let sc = result["structuredContent"]
        .as_object()
        .expect("structuredContent");
    let has_cycle = sc["has_cycle"].as_bool().unwrap_or(false);
    assert!(!has_cycle, "seeded data should not have cycles");
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
    let result = resp["result"].as_object().expect("result object");
    assert_valid_mcp_envelope(result);
    let sc = result["structuredContent"]
        .as_object()
        .expect("structuredContent");
    let has_cycle = sc["has_cycle"].as_bool().unwrap_or(true);
    assert!(!has_cycle, "empty index should not have cycles");
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
    let result = resp["result"].as_object().expect("result object");
    assert_valid_mcp_envelope(result);
    let sc = result["structuredContent"]
        .as_object()
        .expect("structuredContent");
    let edges = &sc["edges"];
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
    let result = resp["result"].as_object().expect("result object");
    assert_valid_mcp_envelope(result);
    let sc = result["structuredContent"]
        .as_object()
        .expect("structuredContent");
    let entities = &sc["entities"];
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
    let result = resp["result"].as_object().expect("result object");
    assert_valid_mcp_envelope(result);
    let sc = result["structuredContent"]
        .as_object()
        .expect("structuredContent");
    let count = sc["count"].as_i64().unwrap_or(-1);
    // RED → GREEN: fails before the fix (count == 0 because semantic
    // edges discarded in token-economics fallback), passes after the
    // fix (edges persisted to WorkspaceIndex before fallback).
    assert!(
        count > 0,
        "find_entities must find entity after token-economics fallback; count={}",
        count
    );

    // Verify the returned entity has the expected identity.
    let entities = sc["entities"].as_array().unwrap();
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

// ── BuiltinMetaLayer: REAL production-path tests ───────────────────────
//
// The tests above (mostly) seed WorkspaceIndex directly with synthetic
// `add_edges` calls. These tests exercise the REAL production path for plain
// (non-framework) files that no framework meta layer claims:
//
//   real source file
//   → provide_code_context (MCP dispatch)
//   → resolve_file_path_checked
//   → compile_file_ir_focused
//   → MetaLayerPass → BuiltinMetaLayer.extract_semantic_edges
//   → WorkspaceIndex.add_edges
//   → workspace_query
//
// Before the BuiltinMetaLayer, plain files produced zero semantic edges and
// therefore zero indexed entities (the root cause).

/// Compile a real temp source file through the production MCP dispatch and
/// assert the compilation succeeded (result present, no error).
fn compile_via_provide_code_context(
    dir: &tempfile::TempDir,
    state: &crate::mcp::McpState,
    rel_path: &str,
    source: &str,
) {
    let abs = dir.path().join(rel_path);
    std::fs::write(&abs, source).unwrap();
    let params = json!({
        "arguments": {
            "filePath": rel_path,
            "fidelity": "edit",
            "workspaceRoot": dir.path().to_string_lossy().to_string()
        }
    });
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&json!(1), "provide_code_context", &params, state);
    let resp = pop_response();
    assert!(
        resp.get("result").is_some(),
        "provide_code_context should succeed for {}: {:?}",
        rel_path,
        resp
    );
}

/// Query `find_entities` through the MCP dispatch and return the entity array.
fn find_entities(state: &crate::mcp::McpState, name: &str) -> serde_json::Value {
    let params = json!({
        "arguments": { "type": "find_entities", "name": name }
    });
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&json!(1), "workspace_query", &params, state);
    let resp = pop_response();
    assert!(
        resp.get("result").is_some(),
        "find_entities should succeed for {:?}",
        name
    );
    let result = resp["result"].as_object().expect("result object");
    assert_valid_mcp_envelope(result);
    let sc = result["structuredContent"]
        .as_object()
        .expect("structuredContent");
    sc["entities"].clone()
}

/// Collect entities matching `builtin` domain + exact (entity_type, name).
fn assert_builtin_entity(
    entities: &serde_json::Value,
    entity_type: &str,
    name: &str,
) -> Vec<serde_json::Value> {
    let arr = entities
        .as_array()
        .unwrap_or_else(|| panic!("expected entities array, got {}", entities));
    arr.iter()
        .filter(|e| {
            e["domain"].as_str() == Some("builtin")
                && e["entity_type"].as_str() == Some(entity_type)
                && e["name"].as_str() == Some(name)
        })
        .cloned()
        .collect()
}

// Test 1 — plain class end-to-end.
#[test]
fn builtin_plain_class_end_to_end() {
    use tempfile::TempDir;

    let dir = TempDir::new().unwrap();
    let state = crate::mcp::McpState::new(crate::tests::test_config());

    compile_via_provide_code_context(
        &dir,
        &state,
        "plain_service.ts",
        "export class UserService {\n    getUser(id: number) { return id; }\n}\n",
    );

    let entities = find_entities(&state, "UserService");
    let matches = assert_builtin_entity(&entities, "Class", "UserService");
    assert_eq!(
        matches.len(),
        1,
        "a plain builtin declaration must produce exactly one occurrence: {}",
        entities
    );
    assert_eq!(
        entities.as_array().map(Vec::len).unwrap_or_default(),
        1,
        "no other domain may claim an ordinary declaration: {}",
        entities
    );

    // B1: the self-Defines registration record must never reach the
    // relationship graph — a workspace of ordinary compiled files has no cycle.
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(1),
        "workspace_query",
        &json!({ "arguments": { "type": "has_cycle" } }),
        &state,
    );
    let cycle_resp = pop_response();
    let cycle_result = cycle_resp["result"].as_object().expect("result object");
    assert_valid_mcp_envelope(cycle_result);
    let cycle_sc = cycle_result["structuredContent"]
        .as_object()
        .expect("structuredContent");
    assert_eq!(
        cycle_sc["has_cycle"].as_bool(),
        Some(false),
        "ordinary builtin registration must not be reported as a cycle: {:?}",
        cycle_resp
    );
}

// Test 2 — entities_in_file end-to-end (relative path + workspaceRoot).
#[test]
fn builtin_entities_in_file_end_to_end() {
    use tempfile::TempDir;

    let dir = TempDir::new().unwrap();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let rel = "plain_repo.ts";
    compile_via_provide_code_context(
        &dir,
        &state,
        rel,
        "export class UserService {\n    getUser(id: number) { return id; }\n}\n",
    );

    let params = json!({
        "arguments": {
            "type": "entities_in_file",
            "file_path": rel,
            "workspaceRoot": dir.path().to_string_lossy().to_string()
        }
    });
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&json!(1), "workspace_query", &params, &state);
    let resp = pop_response();
    assert!(
        resp.get("result").is_some(),
        "entities_in_file should succeed: {:?}",
        resp
    );
    let result = resp["result"].as_object().expect("result object");
    assert_valid_mcp_envelope(result);
    let sc = result["structuredContent"]
        .as_object()
        .expect("structuredContent");
    let matches = assert_builtin_entity(&sc["entities"], "Class", "UserService");
    assert_eq!(
        matches.len(),
        1,
        "entities_in_file must report exactly one occurrence per declaration: {}",
        sc["entities"]
    );
    assert_eq!(
        sc["count"].as_i64(),
        Some(1),
        "file bookkeeping must not duplicate registration records: {}",
        resp["result"]
    );
}

// Test 3 — framework + builtin coexistence.
#[test]
#[cfg(any(feature = "csharp", feature = "dotnet"))]
fn builtin_framework_coexistence() {
    use tempfile::TempDir;

    let dir = TempDir::new().unwrap();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let cs = r#"
using Microsoft.AspNetCore.Mvc;

[ApiController]
[Route("api/test")]
public class TestController : ControllerBase
{
    [HttpGet]
    public IActionResult Get() => Ok("hello");
}
"#;
    compile_via_provide_code_context(&dir, &state, "TestController.cs", cs);

    let entities = find_entities(&state, "TestController");
    let arr = entities.as_array().unwrap();
    let dotnet = arr.iter().any(|e| {
        e["domain"].as_str() == Some("dotnet")
            && e["entity_type"].as_str() == Some("Controller")
            && e["name"].as_str() == Some("TestController")
    });
    let builtin_matches = assert_builtin_entity(&entities, "Class", "TestController");
    assert!(
        dotnet,
        "framework dotnet Controller must remain alongside builtin: {}",
        entities
    );
    assert_eq!(
        builtin_matches.len(),
        1,
        "builtin Class must coexist alongside the framework entity, exactly once: {}",
        entities
    );
}

// Test 4 — recompilation deduplication.
#[test]
fn builtin_recompile_deduplicates() {
    use tempfile::TempDir;

    let dir = TempDir::new().unwrap();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let rel = "plain_dup.ts";
    compile_via_provide_code_context(&dir, &state, rel, "export class DuplicateService { }\n");
    let after_first = assert_builtin_entity(
        &find_entities(&state, "DuplicateService"),
        "Class",
        "DuplicateService",
    )
    .len();
    assert_eq!(
        after_first, 1,
        "exactly one occurrence after the first compile (registration-record boundary)"
    );

    // Recompile the SAME file. remove_file → add_edges must reset the
    // file's occurrences, so the count must NOT grow (no accumulation
    // across recompilations). The registration-record boundary (B1) keeps
    // the baseline at exactly one occurrence per (identity, file).
    compile_via_provide_code_context(&dir, &state, rel, "export class DuplicateService { }\n");
    let after_second = assert_builtin_entity(
        &find_entities(&state, "DuplicateService"),
        "Class",
        "DuplicateService",
    )
    .len();
    assert_eq!(
        after_second, after_first,
        "recompiling the same file must not accumulate builtin entities"
    );
}

// Test 5 — declaration coverage (class/interface/struct/enum/trait/record).
#[test]
#[cfg(all(feature = "rust", feature = "csharp"))]
fn builtin_declaration_coverage() {
    use tempfile::TempDir;

    let dir = TempDir::new().unwrap();
    let state = crate::mcp::McpState::new(crate::tests::test_config());

    let fixtures: Vec<(&str, &str, &str, &str)> = vec![
        (
            "csharp_class.cs",
            "public class UserClass { }\n",
            "Class",
            "UserClass",
        ),
        (
            "csharp_interface.cs",
            "public interface UserInterface { }\n",
            "Interface",
            "UserInterface",
        ),
        (
            "csharp_struct.cs",
            "public struct UserStruct { }\n",
            "Struct",
            "UserStruct",
        ),
        (
            "csharp_enum.cs",
            "public enum UserStatus { Active }\n",
            "Enum",
            "UserStatus",
        ),
        (
            "rust_trait.rs",
            "pub trait UserTrait { fn get(&self); }\n",
            "Trait",
            "UserTrait",
        ),
        (
            "csharp_record.cs",
            "public record UserRecord { }\n",
            "Record",
            "UserRecord",
        ),
    ];

        for (rel, source, entity_type, name) in fixtures {
        compile_via_provide_code_context(&dir, &state, rel, source);
        let entities = find_entities(&state, name);
        let matches = assert_builtin_entity(&entities, entity_type, name);
        assert_eq!(
            matches.len(),
            1,
            "builtin {entity_type} '{name}' must produce exactly one occurrence after compile of {rel}: {}",
            entities
        );
    }
}

// ── outputSchema contract ─────────────────────────────────────────────
//
// Verifies that tools/list exposes outputSchema for workspace_query,
// matching the convention established by apply_edit and CBM tools.

#[test]
fn workspace_query_tool_declares_output_schema() {
    let tools = crate::mcp::tools::tool_list();
    let wq = tools
        .iter()
        .find(|t| t["name"] == "workspace_query")
        .expect("workspace_query must be in tool_list");
    let schema = wq["outputSchema"]
        .as_object()
        .expect("workspace_query must declare outputSchema");
    assert_eq!(schema["type"], "object");
    let props = schema["properties"]
        .as_object()
        .expect("outputSchema.properties must be an object");
    // All six query result shapes must be represented.
    for key in [
        "entities",
        "edges",
        "dependencies",
        "count",
        "has_cycle",
        "depth_used",
    ] {
        assert!(
            props.contains_key(key),
            "outputSchema.properties must contain '{key}'"
        );
    }
}
