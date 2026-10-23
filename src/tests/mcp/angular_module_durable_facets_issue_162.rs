use serde_json::{Value, json};
use std::path::Path;

fn call(state: &crate::mcp::McpState, root: &Path, tool: &str, arguments: Value) -> Value {
    let mut arguments = arguments;
    arguments["workspaceRoot"] = json!(root);
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(
        &json!(162),
        tool,
        &json!({"arguments":arguments}),
        state,
    );
    crate::protocol::captured_responses().pop().unwrap()
}

fn members(state: &crate::mcp::McpState, root: &Path) -> usize {
    let response = call(
        state,
        root,
        "workspace_query",
        json!({"type":"forward_edges",
        "domain":"angular","entity_type":"Module","name":"AppModule"}),
    );
    assert!(response.get("error").is_none(), "{response}");
    response["result"]["structuredContent"]["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|edge| edge["relation"] == "DeclaresInModule" && edge["object"]["name"] == "Leaf")
        .count()
}

fn fixture() -> (
    tempfile::TempDir,
    crate::config::CleanCtxConfig,
    String,
    Value,
) {
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    std::fs::write(p.join("tsconfig.json"), r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[]}}"#).unwrap();
    let package = p.join("node_modules/@angular/core");
    std::fs::create_dir_all(&package).unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{"name":"@angular/core","version":"20.0.0","types":"index.d.ts"}"#,
    )
    .unwrap();
    std::fs::write(package.join("index.d.ts"), "export declare function NgModule(metadata: unknown): any; export declare function Directive(metadata: unknown): any;").unwrap();
    std::fs::write(p.join("leaf.ts"), "import { Directive as Dir } from '@angular/core'; @Dir({selector:'[leaf]'}) export class Leaf {}").unwrap();
    let file = p.join("app.ts").to_string_lossy().into_owned();
    std::fs::write(&file, "import { NgModule as Mod } from '@angular/core'; import { Leaf } from './leaf'; @Mod({declarations:[Leaf]}) export class AppModule {}").unwrap();
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config.persistence.enabled = true;
    config.persistence.db_path = p.join("module-facets.db").to_string_lossy().into_owned();
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config.clone());
    let response = call(
        &state,
        p,
        "compress_code_context",
        json!({"filePath":file,"fidelity":"high"}),
    );
    assert!(response.get("error").is_none(), "{response}");
    state.flush_persistence();
    let alias = state.alias_for_path(&file).unwrap();
    let snapshot = state.semantic_snapshot(&alias).unwrap();
    let authority = snapshot.authority.typescript_authority.as_ref().unwrap();
    assert!(
        authority.declaration_facets.iter().any(|facet| facet
            .declaration
            .source
            .ends_with("leaf.ts")
            && !facet.decorators.is_empty()),
        "nonempty dependency facet control"
    );
    let expected = serde_json::to_value(authority).unwrap();
    let connection = rusqlite::Connection::open(&config.persistence.db_path).unwrap();
    let encoded: String = connection
        .query_row(
            "SELECT edges_json FROM semantic_edge_snapshots WHERE file_path = ?1",
            [&file],
            |row| row.get(0),
        )
        .unwrap();
    let durable: Value = serde_json::from_str(&encoded).unwrap();
    assert_eq!(
        durable["typescript_authority"], expected,
        "actual SQLite payload"
    );
    drop(connection);
    assert_eq!(members(&state, p), 1, "nonempty public membership control");
    drop(state);
    (root, config, file, expected)
}

#[test]
fn issue_162_module_durable_restart_retains_exact_facets_in_session_and_index() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config, file, expected) = fixture();
    let state = crate::mcp::McpState::new(config);
    let response = call(
        &state,
        root.path(),
        "restore_context",
        json!({"filePath":file}),
    );
    assert!(response.get("error").is_none(), "{response}");
    let alias = state.alias_for_path(&file).unwrap();
    let snapshot = state.semantic_snapshot(&alias).unwrap();
    assert_eq!(
        serde_json::to_value(snapshot.authority.typescript_authority.as_ref().unwrap()).unwrap(),
        expected
    );
    {
        let owner = state.semantic_owner_path(&file);
        let index = state.workspace_index_read();
        let authority = index
            .retained_typescript_authority(&owner)
            .expect("restore publication must retain typed dependency facets in the real index");
        assert_eq!(serde_json::to_value(authority).unwrap(), expected);
    }
    assert_eq!(members(&state, root.path()), 1);
}

#[test]
fn issue_162_module_durable_stale_dependency_refuses_restore() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config, file, _) = fixture();
    let source = std::fs::read_to_string(&file).unwrap();
    std::fs::write(root.path().join("leaf.ts"), "export interface Leaf {}").unwrap();
    let state = crate::mcp::McpState::new(config);
    let response = call(
        &state,
        root.path(),
        "restore_context",
        json!({"filePath":file}),
    );
    assert!(
        response.get("error").is_some(),
        "stale facets adopted: {response}"
    );
    assert!(state.alias_for_path(&file).is_none());
    assert_eq!(std::fs::read_to_string(&file).unwrap(), source);
}

#[test]
fn issue_162_module_durable_post_restore_dependency_mutation_retracts_membership() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config, file, _) = fixture();
    let state = crate::mcp::McpState::new(config);
    let response = call(
        &state,
        root.path(),
        "restore_context",
        json!({"filePath":file}),
    );
    assert!(response.get("error").is_none(), "{response}");
    assert_eq!(members(&state, root.path()), 1);
    let source = std::fs::read_to_string(&file).unwrap();
    let leaf = root.path().join("leaf.ts");
    let original = std::fs::read(&leaf).unwrap();
    std::fs::write(&leaf, "import type { Directive as Dir } from '@angular/core'; @Dir({selector:'[leaf]'}) export class Leaf {}").unwrap();
    assert_eq!(
        members(&state, root.path()),
        0,
        "restored membership must consume current dependency proof"
    );
    assert_eq!(std::fs::read_to_string(&file).unwrap(), source);
    std::fs::write(&leaf, original).unwrap();
    assert_eq!(members(&state, root.path()), 1);
}

#[test]
fn issue_162_module_canonical_relations_prepublication_and_durable_restart() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config, file, _) = fixture();
    let p = root.path();
    let right = p.join("right.ts");
    let original = std::fs::read(p.join("leaf.ts")).unwrap();
    std::fs::write(&right, &original).unwrap();
    let source = "import { NgModule as Mod } from '@angular/core'; import { Leaf as Left } from './leaf'; import { Leaf as Right } from './right'; @Mod({declarations:[Left,Right],exports:[Left,Right]}) export class AppModule {}";
    std::fs::write(&file, source).unwrap();
    let state = crate::mcp::McpState::new(config.clone());
    let artifact = crate::mcp::tool_helpers::compile_source_ir_candidate(
        &file,
        source,
        crate::compression::Fidelity::High,
        &state,
        p.to_str(),
    )
    .unwrap();
    let (all, _, _) = artifact.published_semantics();
    let edges: Vec<_> = all
        .iter()
        .filter(|edge| {
            edge.relation == crate::layers::meta::semantic::SemanticRelation::DeclaresInModule
        })
        .cloned()
        .collect();
    assert_eq!(
        edges.len(),
        2,
        "both facts must exist before index publication"
    );
    assert_ne!(edges[0].object.canonical, edges[1].object.canonical);
    assert!(
        artifact
            .typescript_authority()
            .admits_relation_evidence(&edges)
    );
    assert!(
        !crate::mcp::semantic_compilation::retained::RetainedAuthority::default()
            .admits_relation_evidence(&edges)
    );
    let mut forged = edges.clone();
    forged[0]
        .object
        .canonical
        .as_mut()
        .unwrap()
        .declaration_mut()
        .unwrap()
        .source_hash = "unconsumed".to_string();
    assert!(
        !artifact
            .typescript_authority()
            .admits_relation_evidence(&forged)
    );

    assert!(
        edges
            .iter()
            .all(|edge| edge.subject.canonical.is_some() && edge.object.canonical.is_some())
    );
    let mut index = crate::workspace::index::WorkspaceIndex::new();
    index.add_edges(&file, edges.clone());
    index.add_edges(&file, edges.clone());
    assert_eq!(
        index
            .forward_edges_by_identity("angular", "Module", "AppModule")
            .len(),
        2,
        "same canonical facts deduplicate"
    );
    index.remove_file(&file);
    assert!(
        index
            .forward_edges_by_identity("angular", "Module", "AppModule")
            .is_empty()
    );
    index.add_edges(&file, vec![edges[0].clone()]);
    assert_eq!(
        index
            .forward_edges_by_identity("angular", "Module", "AppModule")
            .len(),
        1
    );
    let response = call(
        &state,
        p,
        "compress_code_context",
        json!({"filePath":file,"fidelity":"high"}),
    );
    assert!(response.get("error").is_none(), "{response}");
    assert_eq!(members(&state, p), 2);
    state.flush_persistence();
    let connection = rusqlite::Connection::open(&config.persistence.db_path).unwrap();
    let encoded: String = connection
        .query_row(
            "SELECT edges_json FROM semantic_edge_snapshots WHERE file_path = ?1",
            [&file],
            |row| row.get(0),
        )
        .unwrap();
    let durable: crate::mcp::state::durable_semantics::DurableSemanticSnapshot =
        serde_json::from_str(&encoded).unwrap();
    let restored = durable.restore_edges().unwrap();
    for edge in &edges {
        assert!(
            restored
                .iter()
                .any(|candidate| candidate.relation == edge.relation
                    && candidate.subject.canonical == edge.subject.canonical
                    && candidate.object.canonical == edge.object.canonical)
        );
    }
    // Public projections must not expose the new internal evidence.
    assert!(edges.iter().all(|edge| {
        serde_json::to_value(edge).unwrap()["object"]
            .get("canonical")
            .is_none()
    }));
    drop(connection);
    drop(state);
    let state = crate::mcp::McpState::new(config);
    let response = call(&state, p, "restore_context", json!({"filePath":file}));
    assert!(response.get("error").is_none(), "{response}");
    assert_eq!(
        members(&state, p),
        2,
        "restart preserves both canonical owners"
    );
    std::fs::write(&right, "export class Leaf {}").unwrap();
    assert_eq!(
        members(&state, p),
        1,
        "only the endpoint losing its role retires"
    );
    std::fs::write(&right, &original).unwrap();
    assert_eq!(members(&state, p), 2);
    std::fs::remove_file(&right).unwrap();
    assert_eq!(
        members(&state, p),
        1,
        "endpoint deletion preserves the unrelated fact"
    );
}
