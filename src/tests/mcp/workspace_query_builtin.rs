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

/// Query `forward_edges` through the MCP dispatch and return the edges array.
fn query_forward_edges(
    state: &crate::mcp::McpState,
    domain: &str,
    entity_type: &str,
    name: &str,
) -> serde_json::Value {
    let params = json!({
        "arguments": {
            "type": "forward_edges",
            "domain": domain,
            "entity_type": entity_type,
            "name": name
        }
    });
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&json!(1), "workspace_query", &params, state);
    let resp = pop_response();
    assert!(
        resp.get("result").is_some(),
        "forward_edges should succeed for {domain}/{name}: {resp:?}"
    );
    let result = resp["result"].as_object().expect("result object");
    assert_valid_mcp_envelope(result);
    let sc = result["structuredContent"]
        .as_object()
        .expect("structuredContent");
    sc["edges"].clone()
}

/// Query `entities_in_file` through the MCP dispatch and return the entity
/// array.
fn query_entities_in_file(
    state: &crate::mcp::McpState,
    file_path: &str,
    workspace_root: &std::path::Path,
) -> serde_json::Value {
    let params = json!({
        "arguments": {
            "type": "entities_in_file",
            "file_path": file_path,
            "workspaceRoot": workspace_root.to_string_lossy().to_string()
        }
    });
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&json!(1), "workspace_query", &params, state);
    let resp = pop_response();
    assert!(
        resp.get("result").is_some(),
        "entities_in_file should succeed for {file_path}: {resp:?}"
    );
    let result = resp["result"].as_object().expect("result object");
    assert_valid_mcp_envelope(result);
    let sc = result["structuredContent"]
        .as_object()
        .expect("structuredContent");
    sc["entities"].clone()
}

/// Does the edges array contain an edge with the given relation whose object
/// is the exact (domain, entity_type, name) entity?
fn edges_contain_to(
    edges: &serde_json::Value,
    relation: &str,
    domain: &str,
    entity_type: &str,
    name: &str,
) -> bool {
    edges
        .as_array()
        .unwrap_or_else(|| panic!("expected edges array, got {edges}"))
        .iter()
        .any(|e| {
            e["relation"].as_str() == Some(relation)
                && e["object"]["domain"].as_str() == Some(domain)
                && e["object"]["entity_type"].as_str() == Some(entity_type)
                && e["object"]["name"].as_str() == Some(name)
        })
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

// Test 6 — decorated TS class + corrected NgRx semantic names through the
// real production path. The builtin layer must register the decorated
// declaration's real name (`ShellComponent`, not the `@Component(...)`
// decorator head), and the NgRx edges must carry the corrected semantic
// names (`panelState`, `TOGGLE_PANEL`) while preserving the action-creator
// form (`someAction`).
#[test]
fn builtin_decorated_class_and_ngrx_semantic_names_production_path() {
    use tempfile::TempDir;

    let dir = TempDir::new().unwrap();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let rel = "widget-shell.component.ts";
    let source = r#"
import { Component } from '@angular/core';
import { Store } from '@ngrx/store';
import { TOGGLE_PANEL, someAction } from '../store/actions';

@Component({ selector: 'widget-shell' })
export class ShellComponent {
  constructor(private store: Store) {}

  ngOnInit() {
    this.store.pipe(select('panelState'));
    this.store.dispatch({ type: TOGGLE_PANEL });
    this.store.dispatch(someAction());
  }
}
"#;
    compile_via_provide_code_context(&dir, &state, rel, source);

    // The builtin Class for the decorated TS declaration must carry the real
    // class name — never the `@Component(...)` decorator head.
    let entities = find_entities(&state, "ShellComponent");
    let matches = assert_builtin_entity(&entities, "Class", "ShellComponent");
    assert_eq!(
        matches.len(),
        1,
        "decorated TS class must register a builtin Class named ShellComponent: {}",
        entities
    );

    // No decorator-truncated builtin name may be registered for this file.
    let in_file = query_entities_in_file(&state, rel, dir.path());
    let garbage = in_file
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| {
            e["domain"].as_str() == Some("builtin")
                && e["name"]
                    .as_str()
                    .map(|n| n.starts_with('@') || n.starts_with('['))
                    .unwrap_or(false)
        })
        .count();
    assert_eq!(
        garbage, 0,
        "no decorator-truncated builtin entity may be registered: {}",
        in_file
    );

    // The corrected NgRx semantic names must survive into the workspace
    // index under the component subject.
    let edges = query_forward_edges(&state, "angular", "Component", "ShellComponent");
    assert!(
        edges_contain_to(&edges, "Selects", "ngrx", "Selector", "panelState"),
        "select('panelState') must index a Selects edge to panelState: {}",
        edges
    );
    assert!(
        edges_contain_to(&edges, "Dispatches", "ngrx", "Action", "TOGGLE_PANEL"),
        "dispatch({{ type: TOGGLE_PANEL }}) must index a Dispatches edge to TOGGLE_PANEL: {}",
        edges
    );
    assert!(
        edges_contain_to(&edges, "Dispatches", "ngrx", "Action", "someAction"),
        "dispatch(someAction()) must keep indexing an edge to someAction: {}",
        edges
    );
    assert!(
        !edges_contain_to(&edges, "Selects", "ngrx", "Selector", "'panelState'"),
        "the raw quoted selector slice must never reach the index: {}",
        edges
    );
    assert!(
        !edges_contain_to(&edges, "Dispatches", "ngrx", "Action", "{"),
        "the object-literal opening brace must never reach the index: {}",
        edges
    );
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
