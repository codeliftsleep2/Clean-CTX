use serde_json::{Value, json};
use std::path::Path;

fn query(state: &crate::mcp::McpState, root: &Path, mut arguments: Value) -> Value {
    arguments["workspaceRoot"] = json!(root);
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(
        &json!(162),
        "workspace_query",
        &json!({"arguments":arguments}),
        state,
    );
    let response = crate::protocol::captured_responses().pop().unwrap();
    assert!(response.get("error").is_none(), "{response}");
    response["result"]["structuredContent"].clone()
}

fn fixture() -> (tempfile::TempDir, crate::mcp::McpState) {
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install(p);
    std::fs::write(
        p.join("node_modules/@angular/core/index.d.ts"),
        "export declare function NgModule(metadata: unknown): any; export declare function Directive(metadata: unknown): any;",
    )
    .unwrap();
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    (root, crate::mcp::McpState::new(config))
}

fn edges(state: &crate::mcp::McpState, root: &Path, relation: &str) -> Vec<Value> {
    query(
        state,
        root,
        json!({"type":"entities_in_file","file_path":root.join("app.ts"),"fidelity":"high"}),
    );
    let result = query(
        state,
        root,
        json!({"type":"forward_edges","domain":"angular","entity_type":"Module","name":"AppModule"}),
    );
    result["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|edge| edge["relation"] == relation)
        .cloned()
        .collect()
}

#[test]
fn issue_162_module_relations_aliased_imported_module_and_type_erasure() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state) = fixture();
    let p = root.path();
    let target =
        "import { NgModule as Mod } from '@angular/core'; @Mod({}) export class FeatureModule {}";
    std::fs::write(p.join("feature.ts"), target).unwrap();
    std::fs::write(
        p.join("barrel.ts"),
        "export { FeatureModule as PublicModule } from './feature';",
    )
    .unwrap();
    std::fs::write(p.join("app.ts"), "import { NgModule as Mod } from '@angular/core'; import { PublicModule as Feature } from './barrel'; @Mod({imports:[Feature]}) export class AppModule {}").unwrap();
    for _ in 0..2 {
        let imports = edges(&state, p, "ImportsModule");
        assert_eq!(imports.len(), 1, "{imports:?}");
        assert_eq!(imports[0]["object"]["name"], "FeatureModule");
        assert_eq!(imports[0]["object"]["entity_type"], "Module");
        assert_eq!(
            imports[0]["object"]["file"],
            json!(crate::dictionary::path::canonical_identity_key(
                &p.join("feature.ts").to_string_lossy()
            ))
        );
    }
    std::fs::write(
        p.join("feature.ts"),
        target.replace("import {", "import type {"),
    )
    .unwrap();
    assert!(edges(&state, p, "ImportsModule").is_empty());
    std::fs::write(p.join("feature.ts"), target).unwrap();
    assert_eq!(edges(&state, p, "ImportsModule").len(), 1);
}

#[test]
fn issue_162_module_relations_same_named_exports_keep_distinct_owners() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state) = fixture();
    let p = root.path();
    let leaf = "import { Directive as Dir } from '@angular/core'; @Dir({selector:'[leaf]'}) export class Leaf {}";
    for file in ["left.ts", "right.ts"] {
        std::fs::write(p.join(file), leaf).unwrap();
    }
    std::fs::write(p.join("app.ts"), "import { NgModule as Mod } from '@angular/core'; import { Leaf as Left } from './left'; import { Leaf as Right } from './right'; const ALL = [Left, Right]; @Mod({declarations:[...ALL],exports:[...ALL]}) export class AppModule {}").unwrap();
    for relation in ["DeclaresInModule", "ExportsFromModule"] {
        let result = edges(&state, p, relation);
        assert_eq!(
            result.len(),
            2,
            "distinct owners for {relation}: {result:?}"
        );
        let mut files: Vec<_> = result
            .iter()
            .map(|edge| edge["object"]["file"].as_str().unwrap().to_owned())
            .collect();
        files.sort();
        let mut expected: Vec<_> = ["left.ts", "right.ts"]
            .iter()
            .map(|file| {
                crate::dictionary::path::canonical_identity_key(&p.join(file).to_string_lossy())
            })
            .collect();
        expected.sort();
        assert_eq!(files, expected);
        assert!(result.iter().all(|edge| edge["object"]["name"] == "Leaf"));
    }
    std::fs::write(p.join("right.ts"), "export class Leaf {}").unwrap();
    let result = edges(&state, p, "ExportsFromModule");
    assert_eq!(
        result.len(),
        1,
        "one owner losing its role must not remove the other: {result:?}"
    );
    assert_eq!(
        result[0]["object"]["file"],
        json!(crate::dictionary::path::canonical_identity_key(
            &p.join("left.ts").to_string_lossy()
        ))
    );
    std::fs::write(p.join("right.ts"), leaf).unwrap();
    assert_eq!(edges(&state, p, "ExportsFromModule").len(), 2);
}
