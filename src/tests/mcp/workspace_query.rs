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
            call_evidence: None,
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
            call_evidence: None,
        }],
    );
}

fn pop_response() -> serde_json::Value {
    let mut guard = match crate::protocol::CAPTURED_RESPONSES.lock() {
        Ok(g) => g,
        Err(_) => panic!("handler must send response (sink poisoned)"),
    };
    let result = guard.pop();
    drop(guard);
    match result {
        Some(v) => v,
        None => panic!("handler must send response"),
    }
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

// ── Token-economics regression (Issue #2) ──────────────────────────
//
// Verifies that CONTROL-FULL publication and the WorkspaceIndex share the
// same semantic-edge authority.
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
fn find_entities_after_control_full_publication() {
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

    assert!(
        pcc_resp["result"]["content"][0]["text"]
            .as_str()
            .is_some_and(|text| text.starts_with("// CONTROL-FULL v1"))
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
    // The same authoritative edge stream feeds both model-visible content and
    // the WorkspaceIndex.
    assert!(
        count > 0,
        "find_entities must find entity after CONTROL-FULL publication; count={}",
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

#[path = "workspace_query_dispatch.rs"]
mod workspace_query_dispatch;

#[path = "workspace_query_builtin.rs"]
mod workspace_query_builtin;

// Workspace-scope regressions: a query issued FOR a workspace answers with the
// evidence asserted from inside that workspace only (occurrence provenance, never
// semantic identity). `workspace_query_scope` owns the fixtures and RED-SCOPE1–8;
// `workspace_query_scope_provenance` continues with RED-SCOPE9–10 and the
// provenance/lifecycle controls.
#[path = "workspace_query_scope.rs"]
mod workspace_query_scope;

#[path = "workspace_query_scope_provenance.rs"]
mod workspace_query_scope_provenance;

#[path = "workspace_query_scope_entities.rs"]
mod workspace_query_scope_entities;

#[path = "workspace_query_scope_traversal.rs"]
mod workspace_query_scope_traversal;

// `withinPath` provenance narrowing — the OPTIONAL second scope layer over an
// already authorized workspace (`WorkspaceScope ∩ withinPath`). The entity/edge
// surfaces and the graph surfaces have their own files, and the TypeScript
// property-arrow case is feature-gated with the arrow producer.
#[path = "workspace_query_within_path.rs"]
mod workspace_query_within_path;

#[path = "workspace_query_within_path_edges.rs"]
mod workspace_query_within_path_edges;

#[path = "workspace_query_within_path_traversal.rs"]
mod workspace_query_within_path_traversal;

#[cfg(feature = "typescript")]
#[path = "workspace_query_within_path_arrows.rs"]
mod workspace_query_within_path_arrows;
