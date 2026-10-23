use serde_json::{Value, json};
use std::path::Path;

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
    response["result"].clone()
}

fn fixture(persistent: bool) -> (tempfile::TempDir, crate::config::CleanCtxConfig) {
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install(p);
    std::fs::write(p.join("node_modules/@angular/core/index.d.ts"), "export declare function Injectable(metadata?: unknown): any; export declare function inject(token: unknown): unknown;").unwrap();
    for name in ["left.ts", "right.ts"] {
        std::fs::write(p.join(name), "export class Dependency {}").unwrap();
    }
    let mut config = crate::tests::test_config();
    config.persistence.enabled = persistent;
    config.persistence.db_path = p.join("inject.db").to_string_lossy().into_owned();
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    (root, config)
}

fn dependencies(state: &crate::mcp::McpState, root: &Path) -> Vec<Value> {
    tool(
        state,
        root,
        "workspace_query",
        json!({"type":"entities_in_file","file_path":root.join("app.ts"),"fidelity":"high"}),
    );
    let response = tool(
        state,
        root,
        "workspace_query",
        json!({"type":"forward_edges","domain":"angular","entity_type":"Service","name":"Owner"}),
    );
    response["structuredContent"]["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|edge| edge["relation"] == "Injects")
        .cloned()
        .collect()
}

#[test]
fn issue_162_inject_lifecycle_same_named_tokens_restart_and_independent_retraction() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = fixture(true);
    let p = root.path();
    let file = p.join("app.ts").to_string_lossy().into_owned();
    let source = "import { Injectable, inject as obtain } from '@angular/core'; import { Dependency as Left } from './left'; import { Dependency as Right } from './right'; @Injectable() export class Owner { first = obtain(Left); repeat = obtain(Left); second = obtain(Right); }";
    std::fs::write(&file, source).unwrap();
    let state = crate::mcp::McpState::new(config.clone());
    tool(
        &state,
        p,
        "compress_code_context",
        json!({"filePath":file,"fidelity":"high"}),
    );
    let edges = dependencies(&state, p);
    assert_eq!(
        edges.len(),
        2,
        "same token deduplicates; distinct canonical owners survive: {edges:?}"
    );
    let mut files: Vec<_> = edges
        .iter()
        .map(|edge| edge["object"]["file"].as_str().unwrap().to_owned())
        .collect();
    files.sort();
    let mut expected: Vec<_> = ["left.ts", "right.ts"]
        .iter()
        .map(|name| {
            crate::dictionary::path::canonical_identity_key(&p.join(name).to_string_lossy())
        })
        .collect();
    expected.sort();
    assert_eq!(files, expected);
    assert!(
        edges
            .iter()
            .all(|edge| edge["object"]["name"] == "Dependency")
    );
    state.flush_persistence();
    let alias = state.alias_for_path(&file).unwrap();
    let before = state.semantic_snapshot(&alias).unwrap();
    let expected_authority =
        serde_json::to_value(before.authority.typescript_authority.as_ref().unwrap()).unwrap();
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
    for edge in before
        .edges
        .iter()
        .filter(|edge| edge.relation == crate::layers::meta::semantic::SemanticRelation::Injects)
    {
        assert!(
            durable
                .restore_edges()
                .unwrap()
                .iter()
                .any(
                    |restored| restored.subject.canonical == edge.subject.canonical
                        && restored.object.canonical == edge.object.canonical
                )
        );
    }
    drop(connection);
    drop(state);
    let state = crate::mcp::McpState::new(config);
    tool(&state, p, "restore_context", json!({"filePath":file}));
    let alias = state.alias_for_path(&file).unwrap();
    let snapshot = state.semantic_snapshot(&alias).unwrap();
    assert_eq!(
        serde_json::to_value(snapshot.authority.typescript_authority.as_ref().unwrap()).unwrap(),
        expected_authority
    );
    {
        let index = state.workspace_index_read();
        let edges = index.forward_edges_by_identity("angular", "Service", "Owner");
        assert_eq!(
            edges
                .iter()
                .filter(|edge| edge.relation
                    == crate::layers::meta::semantic::SemanticRelation::Injects)
                .count(),
            2,
            "restore must publish both before query acquisition"
        );
    }
    std::fs::write(p.join("right.ts"), "export interface Dependency {}").unwrap();
    let edges = dependencies(&state, p);
    assert_eq!(
        edges.len(),
        1,
        "erased value endpoint retracts independently: {edges:?}"
    );
    assert_eq!(
        edges[0]["object"]["file"],
        json!(crate::dictionary::path::canonical_identity_key(
            &p.join("left.ts").to_string_lossy()
        ))
    );
    assert_eq!(std::fs::read_to_string(&file).unwrap(), source);
    std::fs::write(p.join("right.ts"), "export class Dependency {}").unwrap();
    assert_eq!(dependencies(&state, p).len(), 2);
    std::fs::remove_file(p.join("right.ts")).unwrap();
    assert_eq!(dependencies(&state, p).len(), 1);
}

#[test]
fn issue_162_inject_lifecycle_shadow_optional_and_nested_calls_cannot_borrow_api() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = fixture(false);
    let p = root.path();
    let state = crate::mcp::McpState::new(config);
    let prefix = "import { Injectable, inject as obtain } from '@angular/core'; import { Dependency } from './left';";
    std::fs::write(
        p.join("app.ts"),
        format!("{prefix} @Injectable() export class Owner {{ dep = obtain(Dependency); }}"),
    )
    .unwrap();
    assert_eq!(dependencies(&state, p).len(), 1);
    for initializer in [
        "obtain?.(Dependency)",
        "(obtain(Dependency))",
        "wrap(obtain(Dependency))",
    ] {
        std::fs::write(p.join("app.ts"), format!("{prefix} function wrap(value:unknown) {{return value;}} @Injectable() export class Owner {{ dep = {initializer}; }}")).unwrap();
        assert!(
            dependencies(&state, p).is_empty(),
            "not a supported direct invocation: {initializer}"
        );
    }
    std::fs::write(p.join("app.ts"), format!("{prefix} export function build(obtain:(token:unknown)=>unknown) {{ @Injectable() class Owner {{ dep = obtain(Dependency); }} return Owner; }}")).unwrap();
    assert!(
        dependencies(&state, p).is_empty(),
        "lexical parameter must not borrow the imported API"
    );
    let source = std::fs::read_to_string(p.join("app.ts")).unwrap();
    let at = source.rfind("obtain(Dependency)").unwrap();
    let file = p.join("app.ts");
    let artifact = crate::mcp::tool_helpers::compile_source_ir_candidate(
        file.to_str().unwrap(),
        &source,
        crate::compression::Fidelity::High,
        &state,
        p.to_str(),
    )
    .unwrap();
    assert!(artifact.typescript_authority().reference_sites.iter().any(|(start, _, resolution)| {
        *start == at && matches!(resolution, crate::ir::typescript_resolution::Resolution::Resolved(id)
            if id.source.ends_with("app.ts") && id.name == "obtain" && id.kind == "required_parameter")
    }), "the shadow's exact parameter owner must be retained, not merely omitted");
}

#[test]
fn issue_162_inject_symbolic_restart_and_global_binding_mutations() {
    use crate::layers::meta::semantic::SemanticRelation;
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = fixture(true);
    let p = root.path();
    let file = p.join("app.ts").to_string_lossy().into_owned();
    let globals = p.join("globals.d.ts");
    std::fs::write(
        &globals,
        "declare interface UnrelatedNoise { text: string }",
    )
    .unwrap();
    let source = r#"
import { Injectable as Service, inject as obtain } from '@angular/core';
import { Dependency as Left } from './left';
import type { Dependency as Erased } from './right';
function impostor(value:unknown) { return value; }
const prose = 'inject(MissingService)';
@Service() export class Owner {
    real = obtain(Left);
    symbolic = obtain(MissingService);
    erased = obtain(Erased);
    foreign = impostor(Erased);
    method(obtain:(token:unknown)=>unknown) { return obtain(Erased); }
}
"#;
    std::fs::write(&file, source).unwrap();
    let state = crate::mcp::McpState::new(config.clone());
    tool(
        &state,
        p,
        "compress_code_context",
        json!({"filePath":file,"fidelity":"high"}),
    );
    assert_eq!(dependencies(&state, p).len(), 2);
    state.flush_persistence();
    let alias = state.alias_for_path(&file).unwrap();
    let snapshot = state.semantic_snapshot(&alias).unwrap();
    let before =
        serde_json::to_value(snapshot.authority.typescript_authority.as_ref().unwrap()).unwrap();
    assert!(
        before["unbound_value_sites"]
            .as_array()
            .unwrap()
            .iter()
            .any(|site| site[2] == "MissingService")
    );
    drop(state);
    let state = crate::mcp::McpState::new(config);
    tool(&state, p, "restore_context", json!({"filePath":file}));
    let alias = state.alias_for_path(&file).unwrap();
    let restored = state.semantic_snapshot(&alias).unwrap();
    assert_eq!(
        serde_json::to_value(restored.authority.typescript_authority.as_ref().unwrap()).unwrap(),
        before
    );
    {
        let index = state.workspace_index_read();
        let edges = index.forward_edges_by_identity("angular", "Service", "Owner");
        let symbolic: Vec<_> = edges
            .iter()
            .filter(|edge| {
                edge.relation == SemanticRelation::Injects && edge.object.name == "MissingService"
            })
            .collect();
        assert_eq!(
            symbolic.len(),
            1,
            "restore must publish symbolic evidence before query reacquisition"
        );
        assert!(symbolic[0].object.canonical.is_none());
    }
    for (declaration, expected_count, canonical_token) in [
        ("declare class MissingService {}", 2, true),
        ("declare interface MissingService {}", 1, false),
        ("declare class MissingService {}", 2, true),
        ("declare interface UnrelatedNoise {}", 2, false),
    ] {
        std::fs::write(&globals, declaration).unwrap();
        let edges = dependencies(&state, p);
        assert_eq!(
            edges.len(),
            expected_count,
            "global mutation: {declaration}: {edges:?}"
        );
        let index = state.workspace_index_read();
        let typed = index.forward_edges_by_identity("angular", "Service", "Owner");
        let token: Vec<_> = typed
            .iter()
            .filter(|edge| {
                edge.relation == SemanticRelation::Injects && edge.object.name == "MissingService"
            })
            .collect();
        if expected_count == 1 {
            assert!(
                token.is_empty(),
                "a type-only global must not become symbolic"
            );
        } else {
            assert_eq!(token.len(), 1);
            assert_eq!(token[0].object.canonical.is_some(), canonical_token);
            if let Some(endpoint) = &token[0].object.canonical {
                assert!(
                    endpoint
                        .declaration()
                        .unwrap()
                        .source
                        .ends_with("globals.d.ts")
                );
                assert_eq!(endpoint.declaration().unwrap().name, "MissingService");
            }
        }
        assert_eq!(std::fs::read_to_string(&file).unwrap(), source);
    }
    std::fs::remove_file(&globals).unwrap();
    assert_eq!(
        dependencies(&state, p).len(),
        2,
        "removed ambient binding restores symbolic compatibility only"
    );
}

#[test]
fn issue_162_constructor_authority_aliases_keep_two_same_named_endpoints() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = fixture(false);
    let p = root.path();
    let state = crate::mcp::McpState::new(config);
    let file = p.join("app.ts");
    std::fs::write(&file, "import { Injectable } from '@angular/core'; import { Dependency } from './left'; @Injectable() export class Owner { constructor(value:Dependency) {} }").unwrap();
    assert_eq!(
        dependencies(&state, p).len(),
        1,
        "nonempty constructor control"
    );
    let source = r#"
import { Injectable as Service } from '@angular/core';
import { Dependency as Left } from './left';
import { Dependency as Right } from './right';
import { MissingExport } from './unrelated-missing';
function unrelated<T>(value:T) { return value; }
@Service() export class Owner {
    constructor(private readonly first: Left, public second: Right, repeated: Left) {}
    note = 'constructor(fake: FakeToken)';
    method(value: Right) { return unrelated(value); }
}
"#;
    std::fs::write(&file, source).unwrap();
    for _ in 0..2 {
        let edges = dependencies(&state, p);
        assert_eq!(
            edges.len(),
            2,
            "repeated constructor token deduplicates; both owners survive: {edges:?}"
        );
        let mut actual: Vec<_> = edges
            .iter()
            .map(|edge| {
                assert_eq!(
                    edge["object"]["name"], "Dependency",
                    "written aliases are not canonical token names"
                );
                crate::dictionary::path::canonical_identity_key(
                    edge["object"]["file"].as_str().unwrap(),
                )
            })
            .collect();
        actual.sort();
        let mut expected: Vec<_> = ["left.ts", "right.ts"]
            .iter()
            .map(|name| {
                crate::dictionary::path::canonical_identity_key(&p.join(name).to_string_lossy())
            })
            .collect();
        expected.sort();
        assert_eq!(
            actual, expected,
            "constructor facts must join actual declaration owners"
        );
    }
    assert_eq!(std::fs::read_to_string(file).unwrap(), source);
}

#[test]
fn issue_162_constructor_authority_namespace_and_field_share_exact_token_fact() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = fixture(false);
    let p = root.path();
    let state = crate::mcp::McpState::new(config);
    let source = r#"
import { Injectable as Service, inject as obtain } from '@angular/core';
import { Dependency as Left } from './left';
import * as right from './right';
const noise = { constructor: 'not a dependency' };
@Service() export class Owner {
    constructor(readonly left: Left, rightValue: right.Dependency) {}
    sameToken = obtain(Left);
    method(value: right.Dependency) { return value; }
}
"#;
    std::fs::write(p.join("app.ts"), source).unwrap();
    let edges = dependencies(&state, p);
    assert_eq!(
        edges.len(),
        2,
        "constructor and field uses of one token share a fact; the namespace target stays distinct: {edges:?}"
    );
    let index = state.workspace_index_read();
    let typed = index.forward_edges_by_identity("angular", "Service", "Owner");
    let injected: Vec<_> = typed
        .iter()
        .filter(|edge| edge.relation == crate::layers::meta::semantic::SemanticRelation::Injects)
        .collect();
    assert_eq!(injected.len(), 2);
    assert!(
        injected
            .iter()
            .all(|edge| edge.subject.canonical.is_some() && edge.object.canonical.is_some()),
        "neither constructor aliases nor namespace roots may survive as symbolic replacements"
    );
}

#[test]
fn issue_162_constructor_authority_restart_config_and_negative_candidate_mutations() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = fixture(true);
    let p = root.path();
    let file = p.join("app.ts").to_string_lossy().into_owned();
    let project = |targets: &[&str]| {
        json!({"compilerOptions": {
            "module":"esnext", "moduleResolution":"bundler", "moduleDetection":"legacy",
            "noLib":true, "types":[], "paths":{"@tokens/right":targets}
        }})
        .to_string()
    };
    let original_config = project(&["./preferred.ts", "./right.ts"]);
    std::fs::write(p.join("tsconfig.json"), &original_config).unwrap();
    let source = r#"
import { Injectable as Service, inject as obtain } from '@angular/core';
import { Dependency as Left } from './left';
import { Dependency as Right } from '@tokens/right';
const noise = 'constructor(fake: Fake)';
@Service() export class Owner {
    constructor(private readonly left:Left, right:Right, repeated:Left) {}
    field = obtain(Left);
    unrelated(value:Right) { return value; }
}
"#;
    std::fs::write(&file, source).unwrap();
    let state = crate::mcp::McpState::new(config.clone());
    tool(
        &state,
        p,
        "compress_code_context",
        json!({"filePath":file,"fidelity":"high"}),
    );
    assert_eq!(dependencies(&state, p).len(), 2);
    state.flush_persistence();
    let alias = state.alias_for_path(&file).unwrap();
    let snapshot = state.semantic_snapshot(&alias).unwrap();
    let before =
        serde_json::to_value(snapshot.authority.typescript_authority.as_ref().unwrap()).unwrap();
    assert_eq!(
        before["constructor_parameters"].as_array().unwrap().len(),
        3
    );
    drop(state);
    let state = crate::mcp::McpState::new(config);
    tool(&state, p, "restore_context", json!({"filePath":file}));
    let alias = state.alias_for_path(&file).unwrap();
    let restored = state.semantic_snapshot(&alias).unwrap();
    assert_eq!(
        serde_json::to_value(restored.authority.typescript_authority.as_ref().unwrap()).unwrap(),
        before
    );
    {
        let index = state.workspace_index_read();
        let edges = index.forward_edges_by_identity("angular", "Service", "Owner");
        let injected: Vec<_> = edges
            .iter()
            .filter(|edge| {
                edge.relation == crate::layers::meta::semantic::SemanticRelation::Injects
            })
            .collect();
        assert_eq!(
            injected.len(),
            2,
            "restore publishes two facts before reacquisition"
        );
        assert!(
            injected
                .iter()
                .all(|edge| edge.subject.canonical.is_some() && edge.object.canonical.is_some())
        );
    }
    let assert_files = |wanted: &[&str]| {
        let edges = dependencies(&state, p);
        let mut actual: Vec<_> = edges
            .iter()
            .map(|edge| {
                assert_eq!(edge["object"]["name"], "Dependency");
                crate::dictionary::path::canonical_identity_key(
                    edge["object"]["file"].as_str().unwrap(),
                )
            })
            .collect();
        actual.sort();
        let mut expected: Vec<_> = wanted
            .iter()
            .map(|name| {
                crate::dictionary::path::canonical_identity_key(&p.join(name).to_string_lossy())
            })
            .collect();
        expected.sort();
        assert_eq!(
            actual, expected,
            "only changed canonical facts may retract or move"
        );
        assert_eq!(std::fs::read_to_string(&file).unwrap(), source);
    };
    std::fs::write(p.join("right.ts"), "export interface Dependency {}").unwrap();
    assert_files(&["left.ts"]);
    std::fs::write(p.join("right.ts"), "export class Dependency {}").unwrap();
    assert_files(&["left.ts", "right.ts"]);
    std::fs::write(p.join("preferred.ts"), "export class Dependency {}").unwrap();
    assert_files(&["left.ts", "preferred.ts"]);
    std::fs::remove_file(p.join("preferred.ts")).unwrap();
    assert_files(&["left.ts", "right.ts"]);
    std::fs::write(p.join("tsconfig.json"), project(&["./left.ts"])).unwrap();
    assert_files(&["left.ts"]);
    std::fs::write(p.join("tsconfig.json"), original_config).unwrap();
    assert_files(&["left.ts", "right.ts"]);
}
