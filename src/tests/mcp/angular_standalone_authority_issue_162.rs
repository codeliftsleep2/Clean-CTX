use serde_json::{Value, json};
use std::path::Path;

fn call(state: &crate::mcp::McpState, root: &Path, arguments: Value) -> Value {
    tool(state, root, "workspace_query", arguments)["result"]["structuredContent"].clone()
}

fn tool(state: &crate::mcp::McpState, root: &Path, name: &str, mut arguments: Value) -> Value {
    arguments["workspaceRoot"] = json!(root);
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(
        &json!(162),
        name,
        &json!({"arguments":arguments}),
        state,
    );
    let response = crate::protocol::captured_responses().pop().unwrap();
    assert!(response.get("error").is_none(), "{response}");
    assert_ne!(response["result"]["isError"], true, "{response}");
    response
}

fn fixture() -> (tempfile::TempDir, crate::mcp::McpState) {
    let root = tempfile::tempdir().unwrap();
    crate::tests::angular_authority_fixture::install(root.path());
    std::fs::write(root.path().join("node_modules/@angular/core/index.d.ts"), "export declare function Component(metadata: unknown): any; export declare function Directive(metadata: unknown): any;").unwrap();
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    (root, crate::mcp::McpState::new(config))
}

fn owner(root: &Path, flag: &str) {
    std::fs::write(root.join("app.ts"), format!("import {{ Component as View }} from '@angular/core'; import {{ Leaf as Selected }} from './leaf'; const SHARED = [Selected]; @View({{{flag}selector:'x-app',imports:[...SHARED]}}) export class Dashboard {{}}" )).unwrap();
}

fn leaf(root: &Path, flag: &str) {
    std::fs::write(root.join("leaf.ts"), format!("import {{ Directive as Dir }} from '@angular/core'; @Dir({{{flag}selector:'[leaf]'}}) export class Leaf {{}}" )).unwrap();
}

fn imports(state: &crate::mcp::McpState, root: &Path) -> Vec<Value> {
    call(
        state,
        root,
        json!({"type":"entities_in_file","file_path":root.join("app.ts"),"fidelity":"high"}),
    );
    let response = call(
        state,
        root,
        json!({"type":"forward_edges","domain":"angular","entity_type":"Component","name":"Dashboard"}),
    );
    response["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|edge| edge["relation"] == "ImportsModule")
        .cloned()
        .collect()
}

fn version(root: &Path, version: &str) {
    std::fs::write(
        root.join("node_modules/@angular/core/package.json"),
        json!({"name":"@angular/core","version":version,"types":"index.d.ts"}).to_string(),
    )
    .unwrap();
}

#[test]
fn issue_162_standalone_authority_version_mutation_retracts_only_default_eligibility() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state) = fixture();
    let p = root.path();
    owner(p, "");
    leaf(p, "");
    version(p, "19.0.0");
    let source = std::fs::read(p.join("app.ts")).unwrap();
    let result = imports(&state, p);
    assert_eq!(
        result.len(),
        1,
        "default standalone owner and target: {result:?}"
    );
    assert_eq!(result[0]["object"]["name"], "Leaf");
    assert_eq!(
        result[0]["object"]["file"],
        json!(crate::dictionary::path::canonical_identity_key(
            &p.join("leaf.ts").to_string_lossy()
        ))
    );
    version(p, "18.2.0");
    assert!(
        imports(&state, p).is_empty(),
        "package version is a consumed default input"
    );
    assert_eq!(std::fs::read(p.join("app.ts")).unwrap(), source);
    version(p, "19.0.0");
    assert_eq!(imports(&state, p).len(), 1);
    owner(p, "standalone:true,");
    leaf(p, "standalone:true,");
    version(p, "18.2.0");
    assert_eq!(
        imports(&state, p).len(),
        1,
        "explicit flags do not depend on the default"
    );
}

#[test]
fn issue_162_standalone_authority_explicit_false_and_erased_targets_refuse_imports() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state) = fixture();
    let p = root.path();
    owner(p, "standalone:true,");
    leaf(p, "standalone:true,");
    assert_eq!(imports(&state, p).len(), 1);
    leaf(p, "standalone:false,");
    assert!(
        imports(&state, p).is_empty(),
        "nonstandalone declaration cannot be imported directly"
    );
    leaf(p, "standalone:true,");
    owner(p, "standalone:false,");
    assert!(
        imports(&state, p).is_empty(),
        "nonstandalone owner has no standalone imports"
    );
    owner(p, "standalone:true,");
    let original = std::fs::read_to_string(p.join("leaf.ts")).unwrap();
    std::fs::write(
        p.join("leaf.ts"),
        original.replace("import {", "import type {"),
    )
    .unwrap();
    assert!(
        imports(&state, p).is_empty(),
        "erased decorator has no runtime declaration role"
    );
    std::fs::write(p.join("leaf.ts"), original).unwrap();
    assert_eq!(imports(&state, p).len(), 1);
}

#[test]
fn issue_162_standalone_authority_durable_restart_and_dependency_retraction() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, initial) = fixture();
    drop(initial);
    let p = root.path();
    owner(p, "standalone:true,");
    leaf(p, "standalone:true,");
    let file = p.join("app.ts").to_string_lossy().into_owned();
    let original_source = std::fs::read(&file).unwrap();
    let mut config = crate::tests::test_config();
    config.persistence.enabled = true;
    config.persistence.db_path = p.join("standalone.db").to_string_lossy().into_owned();
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config.clone());
    tool(
        &state,
        p,
        "compress_code_context",
        json!({"filePath":file,"fidelity":"high"}),
    );
    assert_eq!(imports(&state, p).len(), 1);
    state.flush_persistence();
    let alias = state.alias_for_path(&file).unwrap();
    let before = state.semantic_snapshot(&alias).unwrap();
    let expected =
        serde_json::to_value(before.authority.typescript_authority.as_ref().unwrap()).unwrap();
    let relation = before
        .edges
        .iter()
        .find(|edge| {
            edge.relation == crate::layers::meta::semantic::SemanticRelation::ImportsModule
        })
        .unwrap();
    let canonical_subject = relation.subject.canonical.clone();
    let canonical_object = relation.object.canonical.clone();
    assert!(canonical_subject.is_some() && canonical_object.is_some());
    let connection = rusqlite::Connection::open(&config.persistence.db_path).unwrap();
    let encoded: String = connection
        .query_row(
            "SELECT edges_json FROM semantic_edge_snapshots WHERE file_path = ?1",
            [&file],
            |row| row.get(0),
        )
        .unwrap();
    let persisted: Value = serde_json::from_str(&encoded).unwrap();
    assert_eq!(persisted["typescript_authority"], expected);
    assert!(
        encoded.contains("\"Boolean\":true"),
        "retained boolean facet must survive SQLite"
    );
    drop(connection);
    drop(state);
    let state = crate::mcp::McpState::new(config);
    tool(&state, p, "restore_context", json!({"filePath":file}));
    let alias = state.alias_for_path(&file).unwrap();
    let restored = state.semantic_snapshot(&alias).unwrap();
    assert_eq!(
        serde_json::to_value(restored.authority.typescript_authority.as_ref().unwrap()).unwrap(),
        expected
    );
    let owner = state.semantic_owner_path(&file);
    {
        let index = state.workspace_index_read();
        assert!(
            index.retained_typescript_authority(&owner).is_some(),
            "restore must publish proof before any query"
        );
        let relations = index.forward_edges_by_identity("angular", "Component", "Dashboard");
        let relation = relations
            .iter()
            .find(|edge| {
                edge.relation == crate::layers::meta::semantic::SemanticRelation::ImportsModule
            })
            .unwrap();
        assert_eq!(relation.subject.canonical, canonical_subject);
        assert_eq!(relation.object.canonical, canonical_object);
    }
    leaf(p, "standalone:false,");
    assert!(
        state
            .workspace_index_read()
            .retained_typescript_authority(&owner)
            .is_none(),
        "stale eligibility proof cannot remain current"
    );
    assert!(imports(&state, p).is_empty());
    assert_eq!(std::fs::read(&file).unwrap(), original_source);
    leaf(p, "standalone:true,");
    assert_eq!(imports(&state, p).len(), 1);
}
