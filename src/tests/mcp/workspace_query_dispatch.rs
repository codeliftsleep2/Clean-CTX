// Sibling test module for a `#[path]`-loaded test file that exceeded the
// 615-line active-file size ceiling.
//
// Declared from the parent test file with `#[path = "<file>.rs"] mod <name>;`
// -- the same nested-`#[path]` idiom already used by `src/tests/cbm/e2e.rs` --
// so this module is a DESCENDANT of that test module and `use super::*`
// inherits its entire scope (imports, helpers, fixtures). Nothing needed to be
// widened or re-imported.
//
// Pure relocation: the tests below are byte-for-byte the previously inlined
// implementations.

use super::*;

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
            call_evidence: None,
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
            call_evidence: None,
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
                call_evidence: None,
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
