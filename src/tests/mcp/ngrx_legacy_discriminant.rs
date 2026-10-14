use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

const VALID: &str = r#"
import { Action } from '@ngrx/store';
export const SET_AUDIT = '[Audit] Set';
export const APP_TITLE = 'Audit';
export function auditReducer(state: any, action: Action) {
  switch (action.type) {
    case SET_AUDIT: return state;
    default: return state;
  }
}
"#;

fn call(state: &crate::mcp::McpState, tool: &str, arguments: Value) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&json!(140), tool, &json!({"arguments": arguments}), state);
    let response = crate::protocol::captured_responses()
        .pop()
        .expect("MCP response");
    assert!(response.get("error").is_none(), "{response}");
    assert_ne!(response["result"]["isError"], true, "{response}");
    response["result"]["structuredContent"].clone()
}

fn publish(state: &crate::mcp::McpState, root: &std::path::Path, path: &std::path::Path) {
    call(
        state,
        "provide_code_context",
        json!({"workspaceRoot": root, "filePath": path, "fidelity": "high"}),
    );
}

fn declarations(
    state: &crate::mcp::McpState,
    root: &std::path::Path,
    path: &std::path::Path,
) -> Vec<Value> {
    call(
        state,
        "workspace_query",
        json!({"workspaceRoot": root, "type": "declarations_in_file", "file_path": path}),
    )["entities"]
        .as_array()
        .expect("declarations")
        .iter()
        .filter(|entity| entity["domain"] == "ngrx" && entity["entity_type"] == "Action")
        .cloned()
        .collect()
}

fn transitions(state: &crate::mcp::McpState, root: &std::path::Path) -> Vec<Value> {
    call(
        state,
        "workspace_query",
        json!({
            "workspaceRoot": root, "type": "forward_edges",
            "domain": "ngrx", "entity_type": "Action", "name": "SET_AUDIT"
        }),
    )["edges"]
        .as_array()
        .expect("edges")
        .iter()
        .filter(|edge| edge["relation"] == "TriggersReducer")
        .cloned()
        .collect()
}

fn state(root: &std::path::Path) -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(root.to_string_lossy().into_owned());
    crate::mcp::McpState::new(config)
}

#[test]
fn public_declaration_and_transition_require_the_exact_authority_chain() {
    let _serial = crate::protocol::handler_response_serial();
    for (name, source, expected) in [
        ("valid", VALID.to_owned(), true),
        (
            "foreign",
            VALID.replace("from '@ngrx/store'", "from './fake-action'"),
            false,
        ),
        (
            "local",
            VALID.replace(
                "import { Action } from '@ngrx/store';",
                "interface Action { type: string; }",
            ),
            false,
        ),
        (
            "unrelated",
            VALID
                .replace("action: Action", "value: any")
                .replace("action.type", "value.type"),
            false,
        ),
    ] {
        let root = tempfile::tempdir().expect("workspace");
        let path = root.path().join(format!("{name}.ts"));
        std::fs::write(&path, source).expect("fixture");
        let state = state(root.path());
        publish(&state, root.path(), &path);
        let found = declarations(&state, root.path(), &path);
        let edges = transitions(&state, root.path());
        if expected {
            assert_eq!(found.len(), 1, "{found:?}");
            assert_eq!(found[0]["name"], "SET_AUDIT");
            assert_eq!(found[0]["file"], path.to_string_lossy().as_ref());
            assert_eq!(edges.len(), 1, "{edges:?}");
            assert_eq!(edges[0]["object"]["name"], "auditReducer");
        } else {
            assert!(found.is_empty(), "{name}: {found:?}");
            assert!(edges.is_empty(), "{name}: {edges:?}");
        }
    }
}

#[test]
fn public_authority_tracks_refresh_removal_restore_and_delete() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let path = root.path().join("lifecycle.ts");
    let state = state(root.path());
    for (source, expected) in [
        (Some(VALID.to_owned()), true),
        (
            Some(VALID.replace("case SET_AUDIT: return state;", "")),
            false,
        ),
        (Some(VALID.to_owned()), true),
        (
            Some(VALID.replace("from '@ngrx/store'", "from './fake-action'")),
            false,
        ),
        (Some(VALID.to_owned()), true),
        (None, false),
    ] {
        if let Some(source) = source {
            std::fs::write(&path, source).expect("fixture");
        } else {
            std::fs::remove_file(&path).expect("delete fixture");
        }
        call(
            &state,
            "refresh_workspace",
            json!({"workspaceRoot": root.path()}),
        );
        if path.exists() {
            publish(&state, root.path(), &path);
        }
        assert_eq!(
            !declarations(&state, root.path(), &path).is_empty(),
            expected
        );
        assert_eq!(!transitions(&state, root.path()).is_empty(), expected);
    }
}

#[test]
fn discriminant_authority_advances_only_angular_semantics() {
    assert_eq!(
        crate::layers::meta::ANGULAR_SEMANTIC_PRODUCER_GENERATION,
        14
    );
    assert_eq!(crate::layers::meta::ANGULAR_MARKER_PRODUCER_GENERATION, 5);
    assert_eq!(crate::ir::pipeline::CANONICAL_PIPELINE_GENERATION, 1);
}
