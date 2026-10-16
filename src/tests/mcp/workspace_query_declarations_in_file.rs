//! Public declaration queries project WorkspaceIndex authority, never endpoints.
use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

fn response(
    state: &crate::mcp::McpState,
    query_type: &str,
    root: &std::path::Path,
    path: &std::path::Path,
) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(140),
        "workspace_query",
        &json!({"arguments":{"workspaceRoot":root,"type":query_type,"file_path":path}}),
        state,
    );
    crate::protocol::captured_responses().pop().unwrap()
}

fn entities(response: &Value) -> &[Value] {
    response["result"]["structuredContent"]["entities"]
        .as_array()
        .unwrap()
}

fn ngrx_selectors(response: &Value) -> Vec<&Value> {
    entities(response)
        .iter()
        .filter(|entity| entity["domain"] == "ngrx" && entity["entity_type"] == "Selector")
        .collect()
}

fn state(root: &tempfile::TempDir, ngrx_disabled: bool) -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    if ngrx_disabled {
        config
            .meta_layers
            .entry("angular".into())
            .or_default()
            .ngrx
            .enabled = false;
    }
    crate::mcp::McpState::new(config)
}

#[test]
fn declarations_in_file_exposes_authority_without_promoting_use_endpoints() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let declaration_path = root.path().join("audit.selectors.ts");
    let use_path = root.path().join("audit.component.ts");
    std::fs::write(
        &declaration_path,
        "import { createSelector } from '@ngrx/store';\nexport const selectAudit = createSelector(selectAuditState, state => state.audit);",
    )
    .unwrap();
    std::fs::write(
        &use_path,
        "import { Store } from '@ngrx/store';\ndeclare const store: Store<unknown>;\nstore.select(selectAudit);",
    )
    .unwrap();
    let state = state(&root, false);

    for path in [&declaration_path, &use_path] {
        let entities = response(&state, "entities_in_file", root.path(), path);
        assert!(entities.get("error").is_none(), "{entities}");
    }
    {
        let index = state.workspace_index_read();
        assert_eq!(
            index
                .declarations_in_file(&declaration_path.to_string_lossy())
                .len(),
            1
        );
        assert!(
            index
                .declarations_in_file(&use_path.to_string_lossy())
                .is_empty()
        );
    }

    let declaration = response(
        &state,
        "declarations_in_file",
        root.path(),
        &declaration_path,
    );
    assert!(declaration.get("error").is_none(), "{declaration}");
    let declared = &declaration["result"]["structuredContent"]["entities"];
    assert_eq!(declared.as_array().unwrap().len(), 1);
    assert_eq!(declared[0]["domain"], "ngrx");
    assert_eq!(declared[0]["entity_type"], "Selector");
    assert_eq!(declared[0]["name"], "selectAudit");
    assert_eq!(
        declared[0]["file"],
        declaration_path.to_string_lossy().as_ref()
    );

    let use_only = response(&state, "declarations_in_file", root.path(), &use_path);
    assert!(use_only.get("error").is_none(), "{use_only}");
    assert!(
        use_only["result"]["structuredContent"]["entities"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn declarations_in_file_selector_and_action_matrix() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let cases = [
        (
            "declared.ts",
            "import { createSelector } from '@ngrx/store'; export const selectAudit = createSelector(selectState, state => state.audit);",
            Some(("Selector", "selectAudit")),
        ),
        (
            "declared-and-used.ts",
            "import { createSelector, Store } from '@ngrx/store'; declare const store: Store<unknown>; export const selectAudit = createSelector(selectState, state => state.audit); store.select(selectAudit);",
            Some(("Selector", "selectAudit")),
        ),
        (
            "use-only.ts",
            "import { Store } from '@ngrx/store'; declare const store: Store<unknown>; store.select(selectAudit);",
            None,
        ),
        (
            "quoted-use.ts",
            "import { Store } from '@ngrx/store'; declare const store: Store<unknown>; store.select('selectAudit');",
            None,
        ),
        (
            "ordinary.ts",
            "export const selectAudit = 'ordinary';",
            None,
        ),
        (
            "action.ts",
            "import { createAction } from '@ngrx/store'; export const setAudit = createAction('[Audit] Set');",
            Some(("Action", "setAudit")),
        ),
    ];
    let state = state(&root, false);
    for (file, source, expected) in cases {
        let path = root.path().join(file);
        std::fs::write(&path, source).unwrap();
        let occurrences = response(&state, "entities_in_file", root.path(), &path);
        assert!(occurrences.get("error").is_none(), "{occurrences}");
        let declarations = response(&state, "declarations_in_file", root.path(), &path);
        assert!(declarations.get("error").is_none(), "{declarations}");
        match expected {
            Some((kind, name)) => {
                assert_eq!(entities(&declarations).len(), 1, "{file}: {declarations}");
                assert_eq!(entities(&declarations)[0]["domain"], "ngrx");
                assert_eq!(entities(&declarations)[0]["entity_type"], kind);
                assert_eq!(entities(&declarations)[0]["name"], name);
                assert_eq!(
                    entities(&declarations)[0]["file"],
                    path.to_string_lossy().as_ref()
                );
            }
            None => assert!(entities(&declarations).is_empty(), "{file}: {declarations}"),
        }
    }
}

#[test]
fn declarations_in_file_preserves_file_provenance_and_disabled_refusal() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let source = "import { createSelector } from '@ngrx/store'; export const selectAudit = createSelector(selectState, state => state.audit);";
    let enabled = state(&root, false);
    for file in ["a.ts", "b.ts"] {
        let path = root.path().join(file);
        std::fs::write(&path, source).unwrap();
        let result = response(&enabled, "declarations_in_file", root.path(), &path);
        assert!(result.get("error").is_none(), "{result}");
        assert_eq!(entities(&result).len(), 1);
        assert_eq!(
            entities(&result)[0]["file"],
            path.to_string_lossy().as_ref()
        );
    }

    let disabled_path = root.path().join("disabled.ts");
    std::fs::write(&disabled_path, source).unwrap();
    let disabled = state(&root, true);
    let result = response(
        &disabled,
        "declarations_in_file",
        root.path(),
        &disabled_path,
    );
    assert!(result.get("error").is_none(), "{result}");
    assert!(entities(&result).is_empty());
}

#[test]
fn declarations_in_file_selector_factory_qualification_and_multiple_declarations() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let state = state(&root, false);
    let positive_path = root.path().join("multiple.ts");
    std::fs::write(
        &positive_path,
        "import { createSelector } from '@ngrx/store';\nexport const selectAudit = createSelector(selectState, state => state.audit);\nexport const neighboring = 'ordinary';\nexport function selectHelper() { return 'ordinary'; }\nexport const selectOther = createSelector(selectState, state => state.other);",
    )
    .unwrap();
    let positive = response(&state, "declarations_in_file", root.path(), &positive_path);
    assert!(positive.get("error").is_none(), "{positive}");
    let selectors = ngrx_selectors(&positive);
    let mut names: Vec<_> = selectors
        .iter()
        .map(|entity| entity["name"].as_str().unwrap())
        .collect();
    names.sort();
    assert_eq!(names, ["selectAudit", "selectOther"]);
    assert!(selectors.iter().all(|entity| {
        entity["domain"] == "ngrx"
            && entity["entity_type"] == "Selector"
            && entity["file"] == positive_path.to_string_lossy().as_ref()
    }));
    assert!(entities(&positive).iter().any(|entity| {
        entity["name"] == "selectHelper"
            && !(entity["domain"] == "ngrx" && entity["entity_type"] == "Selector")
    }));

    for (file, source) in [
        (
            "no-import.ts",
            "export const selectAudit = createSelector(selectState, state => state.audit);",
        ),
        (
            "foreign-import.ts",
            "import { createSelector } from './fake'; export const selectAudit = createSelector(selectState, state => state.audit);",
        ),
        (
            "aliased-import.ts",
            "import { createSelector as makeSelector } from '@ngrx/store'; export const selectAudit = makeSelector(selectState, state => state.audit);",
        ),
        (
            "namespace-import.ts",
            "import * as Store from '@ngrx/store'; export const selectAudit = Store.createSelector(selectState, state => state.audit);",
        ),
        (
            "ordinary-function.ts",
            "export function selectAudit() { return 'not a selector'; }",
        ),
    ] {
        let path = root.path().join(file);
        std::fs::write(&path, source).unwrap();
        let result = response(&state, "declarations_in_file", root.path(), &path);
        assert!(result.get("error").is_none(), "{file}: {result}");
        assert!(
            ngrx_selectors(&result).is_empty(),
            "unsupported factory/import form gained Selector authority in {file}: {result}"
        );
    }
}

#[test]
fn declarations_in_file_refreshes_rename_removal_and_delete() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("lifecycle.ts");
    let state = state(&root, false);
    for expected in [Some("selectAudit"), None, Some("selectRenamed")] {
        let source = expected.map_or_else(
            || "import { Store } from '@ngrx/store'; declare const store: Store<unknown>; store.select(selectAudit);".to_string(),
            |name| format!("import {{ createSelector }} from '@ngrx/store'; export const {name} = createSelector(selectState, state => state.audit);"),
        );
        std::fs::write(&path, source).unwrap();
        let refreshed = response(&state, "declarations_in_file", root.path(), &path);
        assert!(refreshed.get("error").is_none(), "{refreshed}");
        match expected {
            Some(name) => {
                assert_eq!(entities(&refreshed).len(), 1);
                assert_eq!(entities(&refreshed)[0]["name"], name);
            }
            None => {
                assert!(entities(&refreshed).is_empty());
                let occurrences = response(&state, "entities_in_file", root.path(), &path);
                assert!(
                    entities(&occurrences).iter().all(|entity| {
                        entity["domain"] != "ngrx" || entity["entity_type"] != "Selector"
                    }),
                    "an unbound selector spelling must not fabricate an occurrence: {occurrences}"
                );
            }
        }
    }
    std::fs::remove_file(&path).unwrap();
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(141),
        "refresh_workspace",
        &json!({"arguments":{"workspaceRoot":root.path()}}),
        &state,
    );
    let refresh = crate::protocol::captured_responses().pop().unwrap();
    assert!(refresh.get("error").is_none(), "{refresh}");
    let deleted = response(&state, "declarations_in_file", root.path(), &path);
    assert_eq!(deleted["error"]["data"]["reason"], "file_not_found");
    assert!(
        state
            .workspace_index_read()
            .declarations_by_identity("ngrx", "Selector", "selectRenamed")
            .is_empty(),
        "deleting the declaring file must retract declaration authority"
    );
}
