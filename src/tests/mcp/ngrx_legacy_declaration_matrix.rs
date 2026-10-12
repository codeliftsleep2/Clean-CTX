use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

fn call(state: &crate::mcp::McpState, tool: &str, args: Value) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&json!(140), tool, &json!({"arguments":args}), state);
    let response = crate::protocol::captured_responses().pop().unwrap();
    assert!(response.get("error").is_none(), "{response}");
    response["result"].clone()
}
fn state(root: &tempfile::TempDir, disabled: bool) -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    if disabled {
        config
            .meta_layers
            .entry("angular".into())
            .or_default()
            .ngrx
            .enabled = false;
    }
    crate::mcp::McpState::new(config)
}
fn entities(
    state: &crate::mcp::McpState,
    root: &std::path::Path,
    path: &std::path::Path,
) -> Vec<Value> {
    let result = call(
        state,
        "workspace_query",
        json!({"workspaceRoot":root,"type":"entities_in_file","file_path":path}),
    );
    result["structuredContent"]["entities"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|entity| entity["domain"] == "ngrx" && entity["entity_type"] == "Action")
        .cloned()
        .collect()
}
fn check(source: &str, expected: &[&str], disabled: bool) {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("actions.ts");
    let source = format!(
        "{source}\n{}",
        "// unchanged economics padding\n".repeat(100)
    );
    crate::edit::apply::verify_syntax(&source, "ts").unwrap();
    std::fs::write(&path, &source).unwrap();
    let state = state(&root, disabled);
    for fidelity in ["high", "medium", "low", "high"] {
        let context = call(
            &state,
            "provide_code_context",
            json!({"workspaceRoot":root.path(),"filePath":path,"fidelity":fidelity}),
        );
        let text = context["content"][0]["text"].as_str().unwrap();
        for name in expected {
            assert!(
                text.contains(&format!("T @action-class = {name}")),
                "missing truthful class marker: {text}"
            );
        }
        if expected.is_empty() {
            assert!(
                !text.contains("T @action-class"),
                "unproven class marker: {text}"
            );
        }
        let found = entities(&state, root.path(), &path);
        let mut names: Vec<_> = found
            .iter()
            .map(|entity| {
                assert_eq!(entity["file"], path.to_string_lossy().as_ref());
                entity["name"].as_str().unwrap()
            })
            .collect();
        names.sort();
        let mut wanted = expected.to_vec();
        wanted.sort();
        assert_eq!(names, wanted);
        for name in expected {
            let edges = call(
                &state,
                "workspace_query",
                json!({"workspaceRoot":root.path(),"type":"forward_edges","domain":"ngrx","entity_type":"Action","name":name}),
            );
            assert!(
                edges["structuredContent"]["edges"]
                    .as_array()
                    .unwrap()
                    .is_empty()
            );
        }
    }
    assert_eq!(std::fs::read_to_string(path).unwrap(), source);
}

#[test]
fn ngrx_legacy_declaration_supported_class_forms() {
    for declaration in [
        "export class Record implements Action {readonly type = 'record';}",
        "class Record implements Action {readonly type = 'record';}",
        "export abstract class Record implements Action {abstract type: string;}",
        "export class Record implements Other, Action {readonly type = 'record';}",
        "export class Record extends Base implements Action {readonly type = 'record';}",
        "export class Record\n implements\n Action {readonly type = 'record';}",
        "export class Record<T> implements Action {readonly type = 'record'; constructor(public payload: T) {}}",
    ] {
        check(
            &format!("import {{ Action }} from '@ngrx/store';\n{declaration}"),
            &["Record"],
            false,
        );
    }
    check(
        "import type { Action } from '@ngrx/store'; class Record implements Action {type = 'record';}",
        &["Record"],
        false,
    );
    check(
        "import { Action } from \"@ngrx/store\"; class Record implements Action {type = 'record';}",
        &["Record"],
        false,
    );
}

#[test]
fn ngrx_legacy_declaration_negative_and_disabled_controls() {
    for source in [
        "import { Action } from '@ngrx/store'; class PretendAction {type = '[Audit] Fake';}",
        "import { Store } from '@ngrx/store'; interface Action {} class Record implements Action {}",
        "import { Store } from '@ngrx/store'; import { Action } from './other'; class Record implements Action {}",
        "import { Action as StoreAction } from '@ngrx/store'; class Record implements StoreAction {}",
        "import * as Store from '@ngrx/store'; class Record implements Store.Action {}",
        "import { Action } from '@ngrx/store'; class Record<Action> implements Action {}",
        "import { Action } from '@ngrx/store'; function scope() {class Record implements Action {}}",
        "import { Store } from '@ngrx/store'; export const SET_AUDIT = '[Audit] Set Audit';",
    ] {
        check(source, &[], false);
    }
    check(
        "import { Action } from '@ngrx/store'; class Record implements Action {readonly type = 'record';}",
        &[],
        true,
    );
}

#[test]
fn ngrx_legacy_declaration_same_name_source_occurrences_are_distinct_not_resolved_links() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let state = state(&root, false);
    for file in ["a.ts", "b.ts"] {
        let path = root.path().join(file);
        std::fs::write(&path, "import { Action } from '@ngrx/store'; export class SharedRecord implements Action {readonly type = 'shared';}").unwrap();
        let found = entities(&state, root.path(), &path);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0]["name"], "SharedRecord");
        assert_eq!(found[0]["file"], path.to_string_lossy().as_ref());
    }
}

#[test]
fn ngrx_legacy_declaration_rename_remove_delete_lifecycle() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("actions.ts");
    let state = state(&root, false);
    for name in [Some("OriginalRecord"), Some("RenamedRecord"), None] {
        let source = name.map(|name| format!("import {{ Action }} from '@ngrx/store'; export class {name} implements Action {{readonly type = 'record';}}"))
            .unwrap_or_else(|| "import { Action } from '@ngrx/store'; export const ordinary = '[Audit] Ordinary';".into());
        std::fs::write(&path, source).unwrap();
        let refreshed = call(
            &state,
            "refresh_workspace",
            json!({"workspaceRoot":root.path()}),
        );
        assert_eq!(refreshed["structuredContent"]["refreshed"], true);
        let found = entities(&state, root.path(), &path);
        assert_eq!(found.len(), usize::from(name.is_some()));
        if let Some(name) = name {
            assert_eq!(found[0]["name"], name);
        }
    }
    std::fs::remove_file(&path).unwrap();
    call(
        &state,
        "refresh_workspace",
        json!({"workspaceRoot":root.path()}),
    );
    assert!(entities(&state, root.path(), &path).is_empty());
}
