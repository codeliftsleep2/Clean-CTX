//! Producer-recognized declarations must be reachable without relationship edges.
use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

fn dispatch(state: &crate::mcp::McpState, tool: &str, arguments: Value) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&json!(1), tool, &json!({"arguments":arguments}), state);
    let response = crate::protocol::captured_responses().pop().unwrap();
    assert!(response.get("error").is_none(), "{tool}: {response}");
    response["result"].clone()
}

fn isolated(source: &str, domain: &str, kind: &str, name: &str, marker: &str) {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("declaration.ts");
    let source = format!(
        "{source}\n{}",
        "// unchanged economics padding\n".repeat(100)
    );
    crate::edit::apply::verify_syntax(&source, "ts").unwrap();
    std::fs::write(&path, &source).unwrap();
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    let context = dispatch(
        &state,
        "provide_code_context",
        json!({
            "workspaceRoot":root.path(), "filePath":path, "fidelity":"high"
        }),
    );
    let text = context["content"][0]["text"].as_str().unwrap();
    assert!(text.contains(marker), "recognition evidence absent: {text}");
    if domain == "angular" {
        assert!(
            text.lines().any(|line| line == "C OwnerProbe"),
            "core owner absent: {text}"
        );
    }
    let result = dispatch(
        &state,
        "workspace_query",
        json!({
            "workspaceRoot":root.path(), "type":"entities_in_file", "file_path":path
        }),
    );
    let entities: Vec<_> = result["structuredContent"]["entities"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|entity| {
            entity["domain"] == domain && entity["entity_type"] == kind && entity["name"] == name
        })
        .collect();
    assert_eq!(
        entities.len(),
        1,
        "recognized edge-free declaration absent: {result}"
    );
    assert_eq!(entities[0]["file"], path.to_string_lossy().as_ref());
    let edges = dispatch(
        &state,
        "workspace_query",
        json!({
            "workspaceRoot":root.path(), "type":"forward_edges", "domain":domain,
            "entity_type":kind, "name":name
        }),
    );
    assert_eq!(
        edges["structuredContent"]["count"], 0,
        "registration became a relationship: {edges}"
    );
    assert_eq!(std::fs::read_to_string(path).unwrap(), source);
}

#[test]
fn framework_declaration_publication_isolated_service() {
    isolated(
        "import { Injectable } from '@angular/core';\n@Injectable()\nclass OwnerProbe {}",
        "angular",
        "Service",
        "OwnerProbe",
        "T @svc = OwnerProbe",
    );
}

#[test]
fn framework_declaration_publication_isolated_modern_action() {
    isolated(
        "import { createAction } from '@ngrx/store';\nexport const setAudit = createAction(\n '[Audit] Set Audit'\n);",
        "ngrx",
        "Action",
        "setAudit",
        "T @action = setAudit",
    );
}

fn declaration(domain: &str, name: &str) -> String {
    if domain == "angular" {
        format!(
            "import {{ Injectable }} from '@angular/core';\n@Injectable()\nexport class {name} {{}}"
        )
    } else {
        format!(
            "import {{ createAction }} from '@ngrx/store';\nexport const {name} = createAction('[Audit] {name}');"
        )
    }
}

fn state(root: &tempfile::TempDir) -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    crate::mcp::McpState::new(config)
}

fn entities(
    state: &crate::mcp::McpState,
    root: &tempfile::TempDir,
    path: &std::path::Path,
    domain: &str,
) -> Vec<Value> {
    let result = dispatch(
        state,
        "workspace_query",
        json!({
            "workspaceRoot":root.path(), "type":"entities_in_file", "file_path":path
        }),
    );
    result["structuredContent"]["entities"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|entity| {
            entity["domain"] == domain
                && entity["entity_type"]
                    == if domain == "angular" {
                        "Service"
                    } else {
                        "Action"
                    }
        })
        .cloned()
        .collect()
}

#[test]
fn framework_declaration_publication_same_file_multiple_declarations() {
    let _serial = crate::protocol::handler_response_serial();
    for domain in ["angular", "ngrx"] {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("multiple.ts");
        std::fs::write(
            &path,
            format!(
                "{}\n{}",
                declaration(domain, "First"),
                declaration(domain, "Second")
                    .lines()
                    .skip(1)
                    .collect::<Vec<_>>()
                    .join("\n")
            ),
        )
        .unwrap();
        let state = state(&root);
        let found = entities(&state, &root, &path, domain);
        assert_eq!(found.len(), 2, "{found:?}");
        for name in ["First", "Second"] {
            assert_eq!(
                found.iter().filter(|entity| entity["name"] == name).count(),
                1
            );
        }
        assert!(
            found
                .iter()
                .all(|entity| entity["file"] == path.to_string_lossy().as_ref())
        );
    }
}

#[test]
fn framework_declaration_publication_same_name_distinct_files() {
    let _serial = crate::protocol::handler_response_serial();
    for domain in ["angular", "ngrx"] {
        let root = tempfile::tempdir().unwrap();
        let state = state(&root);
        for file in ["a.ts", "b.ts"] {
            let path = root.path().join(file);
            std::fs::write(&path, declaration(domain, "Shared")).unwrap();
            let found = entities(&state, &root, &path, domain);
            assert_eq!(found.len(), 1);
            assert_eq!(found[0]["name"], "Shared");
            assert_eq!(found[0]["file"], path.to_string_lossy().as_ref());
        }
        let found = dispatch(
            &state,
            "workspace_query",
            json!({
                "workspaceRoot":root.path(), "type":"find_entities", "name":"Shared"
            }),
        );
        let occurrences: Vec<_> = found["structuredContent"]["entities"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|entity| entity["domain"] == domain)
            .collect();
        assert_eq!(occurrences.len(), 2, "{found}");
        assert_ne!(occurrences[0]["file"], occurrences[1]["file"]);
    }
}

#[test]
fn framework_declaration_publication_refresh_rename_remove_delete() {
    let _serial = crate::protocol::handler_response_serial();
    for domain in ["angular", "ngrx"] {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("lifecycle.ts");
        let state = state(&root);
        std::fs::write(&path, declaration(domain, "Old")).unwrap();
        assert_eq!(entities(&state, &root, &path, domain)[0]["name"], "Old");
        std::fs::write(&path, declaration(domain, "Renamed")).unwrap();
        dispatch(
            &state,
            "refresh_workspace",
            json!({"workspaceRoot":root.path()}),
        );
        let renamed = entities(&state, &root, &path, domain);
        assert_eq!(renamed.len(), 1);
        assert_eq!(renamed[0]["name"], "Renamed");
        let old = dispatch(
            &state,
            "workspace_query",
            json!({"workspaceRoot":root.path(), "type":"find_entities", "name":"Old"}),
        );
        assert!(
            old["structuredContent"]["entities"]
                .as_array()
                .unwrap()
                .iter()
                .all(|entity| entity["domain"] != domain)
        );
        std::fs::write(&path, "export const unchanged = 1;").unwrap();
        dispatch(
            &state,
            "refresh_workspace",
            json!({"workspaceRoot":root.path()}),
        );
        assert!(entities(&state, &root, &path, domain).is_empty());
        std::fs::remove_file(&path).unwrap();
        dispatch(
            &state,
            "refresh_workspace",
            json!({"workspaceRoot":root.path()}),
        );
        crate::protocol::captured_responses().clear();
        dispatch_tools_call(
            &json!(1),
            "workspace_query",
            &json!({"arguments":{
                "workspaceRoot":root.path(), "type":"entities_in_file", "file_path":path
            }}),
            &state,
        );
        let missing = crate::protocol::captured_responses().pop().unwrap();
        assert_eq!(missing["error"]["data"]["reason"], "file_not_found");

        let renamed = dispatch(
            &state,
            "workspace_query",
            json!({"workspaceRoot":root.path(), "type":"find_entities", "name":"Renamed"}),
        );
        assert!(
            renamed["structuredContent"]["entities"]
                .as_array()
                .unwrap()
                .iter()
                .all(|entity| entity["domain"] != domain)
        );
    }
}

#[test]
fn framework_declaration_publication_relationships_do_not_duplicate() {
    let _serial = crate::protocol::handler_response_serial();
    for (domain, relation, source) in [
        (
            "angular",
            "Injects",
            "import { Injectable, Component } from '@angular/core';\n@Injectable() class OwnerProbe {}\n@Component({selector:'x-consumer'}) class Consumer { constructor(private owner: OwnerProbe) {} }",
        ),
        (
            "ngrx",
            "Dispatches",
            "import { Component } from '@angular/core';\nimport { createAction, Store } from '@ngrx/store';\nexport const setAudit = createAction('[Audit] Set Audit');\n@Component({selector:'x-consumer'}) class Consumer { constructor(private store: Store) {} run() { this.store.dispatch(setAudit()); this.store.dispatch(setAudit()); } }",
        ),
    ] {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("consumer.ts");
        std::fs::write(&path, source).unwrap();
        let state = state(&root);
        let found = entities(&state, &root, &path, domain);
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(
            found[0]["name"],
            if domain == "angular" {
                "OwnerProbe"
            } else {
                "setAudit"
            }
        );
        let edges = dispatch(
            &state,
            "workspace_query",
            json!({
                "workspaceRoot":root.path(), "type":"forward_edges", "domain":"angular", "entity_type":"Component", "name":"Consumer"
            }),
        );
        let relationships = edges["structuredContent"]["edges"].as_array().unwrap();
        assert_eq!(
            relationships
                .iter()
                .filter(|edge| edge["relation"] == relation)
                .count(),
            1,
            "{edges}"
        );
        assert!(
            relationships
                .iter()
                .all(|edge| edge["relation"] != "Defines")
        );
    }
}

#[test]
fn framework_declaration_publication_respects_endpoint_authority() {
    let _serial = crate::protocol::handler_response_serial();
    for (domain, source, name) in [
        (
            "angular",
            "import { Component } from '@angular/core';\nimport { OwnerProbe } from './declaration';\n@Component({selector:'x-consumer'}) class Consumer { constructor(private owner: OwnerProbe) {} }",
            "OwnerProbe",
        ),
        (
            "ngrx",
            "import { Component } from '@angular/core';\nimport { Store } from '@ngrx/store';\nimport { setAudit } from './declaration';\n@Component({selector:'x-consumer'}) class Consumer { constructor(private store: Store) {} run() { this.store.dispatch(setAudit()); } }",
            "setAudit",
        ),
    ] {
        let root = tempfile::tempdir().unwrap();
        let declared = root.path().join("declaration.ts");
        let used = root.path().join("consumer.ts");
        std::fs::write(&declared, declaration(domain, name)).unwrap();
        std::fs::write(&used, source).unwrap();
        let state = state(&root);
        let reference = entities(&state, &root, &used, domain);
        let declaration = entities(&state, &root, &declared, domain);
        assert_eq!(declaration.len(), 1);
        let expected_references = usize::from(domain == "angular");
        assert_eq!(reference.len(), expected_references);
        assert_eq!(declaration[0]["file"], declared.to_string_lossy().as_ref());
        assert_eq!(declaration[0]["name"], name);
        if domain == "angular" {
            assert_eq!(reference[0]["file"], used.to_string_lossy().as_ref());
            assert_eq!(reference[0]["name"], name);
        }
    }
}

#[test]
fn framework_declaration_publication_legacy_class_now_has_independent_authority() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("legacy.ts");
    std::fs::write(&path, "import { Action } from '@ngrx/store';\nexport class LegacyAudit implements Action { readonly type = '[Audit] Legacy'; }").unwrap();
    let state = state(&root);
    // #140 explicitly supersedes only the earlier unsupported-class guard.
    // The source is unchanged; modern publication assertions remain frozen.
    let found = entities(&state, &root, &path, "ngrx");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0]["entity_type"], "Action");
    assert_eq!(found[0]["name"], "LegacyAudit");
    assert_eq!(found[0]["file"], path.to_string_lossy().as_ref());
}

#[test]
fn framework_declaration_publication_removal_respects_endpoint_authority() {
    let _serial = crate::protocol::handler_response_serial();
    for (domain, name, source) in [
        (
            "angular",
            "RetainedService",
            "import { Component } from '@angular/core';\nimport { RetainedService } from './declaration';\n@Component({selector:'x-reference'}) class Consumer { constructor(private owner: RetainedService) {} }",
        ),
        (
            "ngrx",
            "retainedAudit",
            "import { Component } from '@angular/core';\nimport { Store } from '@ngrx/store';\nimport { retainedAudit } from './declaration';\n@Component({selector:'x-reference'}) class Consumer { constructor(private store: Store) {} run() { this.store.dispatch(retainedAudit()); } }",
        ),
    ] {
        let root = tempfile::tempdir().unwrap();
        let declared = root.path().join("declaration.ts");
        let used = root.path().join("consumer.ts");
        std::fs::write(&declared, declaration(domain, name)).unwrap();
        std::fs::write(&used, source).unwrap();
        let state = state(&root);
        assert_eq!(entities(&state, &root, &declared, domain).len(), 1);
        let expected_references = usize::from(domain == "angular");
        assert_eq!(
            entities(&state, &root, &used, domain).len(),
            expected_references
        );
        std::fs::write(&declared, "export const noDeclaration = 1;").unwrap();
        dispatch(
            &state,
            "refresh_workspace",
            json!({"workspaceRoot":root.path()}),
        );
        assert!(entities(&state, &root, &declared, domain).is_empty());
        let reference = entities(&state, &root, &used, domain);
        assert_eq!(reference.len(), expected_references);
        if domain == "angular" {
            assert_eq!(reference[0]["file"], used.to_string_lossy().as_ref());
        }
        let found = dispatch(
            &state,
            "workspace_query",
            json!({
                "workspaceRoot":root.path(), "type":"find_entities", "name":name
            }),
        );
        let occurrences: Vec<_> = found["structuredContent"]["entities"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|entity| entity["domain"] == domain)
            .collect();
        assert_eq!(
            occurrences.len(),
            expected_references,
            "only independently authoritative reference occurrences survive: {found}"
        );
        if domain == "angular" {
            assert_eq!(occurrences[0]["file"], used.to_string_lossy().as_ref());
        }
    }
}
