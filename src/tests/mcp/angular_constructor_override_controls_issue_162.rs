use crate::layers::meta::semantic::SemanticRelation;

#[test]
fn issue_162_constructor_decorator_authority_override_eligibility_and_evaluation_scope() {
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install_inject_apis(p);
    let core = p.join("node_modules/@angular/core/index.d.ts");
    let mut api = std::fs::read_to_string(&core).unwrap();
    api.push_str("export declare function Inject(token:unknown):any; export declare function Optional():any;");
    std::fs::write(core, api).unwrap();
    std::fs::write(
        p.join("left.ts"),
        "export class Inferred {} export interface Erased {}",
    )
    .unwrap();
    std::fs::write(p.join("right.ts"), "export class Dependency {}").unwrap();
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    for (parameter, expected) in [
        ("@Maybe() @Override(Right) private readonly dep:Erased", 1),
        ("@Override(Right) Right:Erased", 1),
        ("@Override(Erased) dep:Inferred", 0),
        ("@Override(Ghost) dep:Inferred", 0),
        ("@Override(Right) @Override(Inferred) dep:Inferred", 0),
    ] {
        let source = format!(
            "import {{Injectable as Service, Inject as Override, Optional as Maybe}} from '@angular/core'; import {{Inferred, Erased}} from './left'; import {{Dependency as Right}} from './right'; import type {{Dependency as Ghost}} from './right'; const noise='@Inject(Fake)'; @Service() export class Owner {{ constructor({parameter}) {{}} method(Right:unknown) {{return Right;}} }}"
        );
        let file = p.join("app.ts");
        std::fs::write(&file, &source).unwrap();
        let artifact = crate::mcp::tool_helpers::compile_source_ir_candidate(
            file.to_str().unwrap(),
            &source,
            crate::compression::Fidelity::High,
            &state,
            p.to_str(),
        )
        .unwrap();
        let (edges, _, _) = artifact.published_semantics();
        let injected: Vec<_> = edges
            .iter()
            .filter(|edge| edge.relation == SemanticRelation::Injects)
            .collect();
        assert_eq!(
            injected.len(),
            expected,
            "override must use runtime argument proof, never annotation fallback: {parameter}: {edges:?}"
        );
        if expected == 1 {
            let endpoint = injected[0]
                .object
                .canonical
                .as_ref()
                .expect("override token must have canonical value proof");
            assert!(endpoint.declaration().unwrap().source.ends_with("right.ts"));
            assert_eq!(endpoint.declaration().unwrap().name, "Dependency");
        }
        assert!(
            artifact
                .typescript_authority()
                .admits_relation_evidence(&edges)
        );
        assert_eq!(std::fs::read_to_string(file).unwrap(), source);
    }
}

fn override_tool(
    state: &crate::mcp::McpState,
    root: &std::path::Path,
    name: &str,
    mut arguments: serde_json::Value,
) -> serde_json::Value {
    arguments["workspaceRoot"] = serde_json::json!(root);
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(
        &serde_json::json!(162),
        name,
        &serde_json::json!({"arguments":arguments}),
        state,
    );
    crate::protocol::captured_responses().pop().unwrap()
}

#[test]
fn issue_162_constructor_decorator_lifecycle_restart_barrels_and_stale_restore() {
    use serde_json::json;
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install_inject_apis(p);
    let core = p.join("node_modules/@angular/core/index.d.ts");
    let mut api = std::fs::read_to_string(&core).unwrap();
    api.push_str("export declare function Inject(token:unknown):any;");
    std::fs::write(core, api).unwrap();
    std::fs::write(p.join("annotation.ts"), "export class Annotation {}").unwrap();
    std::fs::write(p.join("token.ts"), "export class Token {}").unwrap();
    std::fs::write(
        p.join("foreign.ts"),
        "export function Inject(value:unknown) {return value;}",
    )
    .unwrap();
    let route = "export {Inject as Select} from '@angular/core';";
    let tokens = "export {Token as PublicToken} from './token';";
    std::fs::write(p.join("api-barrel.ts"), route).unwrap();
    std::fs::write(p.join("tokens.ts"), tokens).unwrap();
    let source = r#"
import {Injectable as Service, inject as obtain} from '@angular/core';
import {Select} from './api-barrel';
import {PublicToken} from './tokens';
import {Annotation} from './annotation';
const noise = '@Inject(Fake) constructor(fake:Phantom)';
@Service() export class Owner {
    constructor(@Select(PublicToken) private readonly dependency:Annotation) {}
    sameToken = obtain(PublicToken);
    method(PublicToken:unknown) {return PublicToken;}
}
"#;
    let file = p.join("app.ts").to_string_lossy().into_owned();
    std::fs::write(&file, source).unwrap();
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    config.persistence.enabled = true;
    config.persistence.db_path = p.join("override.db").to_string_lossy().into_owned();
    let state = crate::mcp::McpState::new(config.clone());
    let response = override_tool(
        &state,
        p,
        "compress_code_context",
        json!({"filePath":file,"fidelity":"high"}),
    );
    assert!(response.get("error").is_none(), "{response}");
    state.flush_persistence();
    let alias = state.alias_for_path(&file).unwrap();
    let snapshot = state.semantic_snapshot(&alias).unwrap();
    let before =
        serde_json::to_value(snapshot.authority.typescript_authority.as_ref().unwrap()).unwrap();
    assert_eq!(
        before["constructor_parameters"][0]["decorator_facets"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    drop(state);
    let state = crate::mcp::McpState::new(config.clone());
    let response = override_tool(&state, p, "restore_context", json!({"filePath":file}));
    assert!(response.get("error").is_none(), "{response}");
    let alias = state.alias_for_path(&file).unwrap();
    let restored = state.semantic_snapshot(&alias).unwrap();
    assert_eq!(
        serde_json::to_value(restored.authority.typescript_authority.as_ref().unwrap()).unwrap(),
        before
    );
    {
        let index = state.workspace_index_read();
        let edges = index.forward_edges_by_identity("angular", "Service", "Owner");
        let selected: Vec<_> = edges
            .iter()
            .filter(|edge| edge.relation == SemanticRelation::Injects)
            .collect();
        assert_eq!(
            selected.len(),
            1,
            "restore must converge constructor and field before reacquisition"
        );
        assert_eq!(
            selected[0]
                .object
                .canonical
                .as_ref()
                .unwrap()
                .declaration()
                .unwrap()
                .name,
            "Token"
        );
    }
    let assert_targets = |expected: &[(&str, &str)]| {
        let response = override_tool(
            &state,
            p,
            "workspace_query",
            json!({"type":"entities_in_file","file_path":file,"fidelity":"high"}),
        );
        assert!(response.get("error").is_none(), "{response}");
        let response = override_tool(
            &state,
            p,
            "workspace_query",
            json!({"type":"forward_edges","domain":"angular","entity_type":"Service","name":"Owner"}),
        );
        assert!(response.get("error").is_none(), "{response}");
        let mut actual: Vec<_> = response["result"]["structuredContent"]["edges"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|edge| edge["relation"] == "Injects")
            .map(|edge| {
                (
                    edge["object"]["name"].as_str().unwrap().to_owned(),
                    crate::dictionary::path::canonical_identity_key(
                        edge["object"]["file"].as_str().unwrap(),
                    ),
                )
            })
            .collect();
        actual.sort();
        let mut wanted: Vec<_> = expected
            .iter()
            .map(|(name, file)| {
                (
                    (*name).to_owned(),
                    crate::dictionary::path::canonical_identity_key(
                        &p.join(file).to_string_lossy(),
                    ),
                )
            })
            .collect();
        wanted.sort();
        assert_eq!(actual, wanted);
        assert_eq!(std::fs::read_to_string(&file).unwrap(), source);
    };
    assert_targets(&[("Token", "token.ts")]);
    std::fs::write(
        p.join("api-barrel.ts"),
        "export {Inject as Select} from './foreign';",
    )
    .unwrap();
    assert_targets(&[("Annotation", "annotation.ts"), ("Token", "token.ts")]);
    std::fs::write(p.join("api-barrel.ts"), route).unwrap();
    assert_targets(&[("Token", "token.ts")]);
    std::fs::write(
        p.join("tokens.ts"),
        "export type {Token as PublicToken} from './token';",
    )
    .unwrap();
    assert_targets(&[]);
    std::fs::write(p.join("tokens.ts"), tokens).unwrap();
    assert_targets(&[("Token", "token.ts")]);
    drop(state);
    std::fs::write(
        p.join("tokens.ts"),
        "export type {Token as PublicToken} from './token';",
    )
    .unwrap();
    let state = crate::mcp::McpState::new(config);
    let response = override_tool(&state, p, "restore_context", json!({"filePath":file}));
    assert!(
        response.get("error").is_some() || response["result"]["isError"] == true,
        "stale decorator/token proof must be refused before adoption: {response}"
    );
}

#[test]
fn issue_162_constructor_authority_merged_type_family_keeps_runtime_class() {
    use serde_json::json;
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install_inject_apis(p);
    std::fs::write(p.join("left.ts"), "export class Dependency {}").unwrap();
    std::fs::write(p.join("right.ts"), "export class Dependency {}").unwrap();
    std::fs::write(
        p.join("merge-barrel.ts"),
        "export {Dependency as PublicLeft} from './left';",
    )
    .unwrap();
    let source = r#"
import {Injectable as Service, inject as obtain} from '@angular/core';
import {PublicLeft as First} from './merge-barrel';
import {Dependency as Right} from './right';
import {Missing} from './unrelated-missing';
const note = 'constructor(fake: Phantom)';
function unrelated<T>(value:T) {return value;}
@Service() export class Owner {
    constructor(private readonly first: First, right: Right) {}
    repeated = obtain(Right);
    method(first:Right) {return unrelated(first);}
}
"#;
    let file = p.join("app.ts");
    std::fs::write(&file, source).unwrap();
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    let assert_targets = |expected_count| {
        let response = override_tool(
            &state,
            p,
            "workspace_query",
            json!({"type":"entities_in_file","file_path":file,"fidelity":"high"}),
        );
        assert!(response.get("error").is_none(), "{response}");
        let response = override_tool(
            &state,
            p,
            "workspace_query",
            json!({"type":"forward_edges","domain":"angular","entity_type":"Service","name":"Owner"}),
        );
        assert!(response.get("error").is_none(), "{response}");
        let edges: Vec<_> = response["result"]["structuredContent"]["edges"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|edge| edge["relation"] == "Injects")
            .collect();
        assert_eq!(
            edges.len(),
            expected_count,
            "a legal type-only interface contributor cannot erase its runtime class token; duplicate classes cannot borrow merge authority: {response}"
        );
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
        let wanted = if expected_count == 2 {
            vec!["left.ts", "right.ts"]
        } else {
            vec!["right.ts"]
        };
        let mut wanted: Vec<_> = wanted
            .iter()
            .map(|name| {
                crate::dictionary::path::canonical_identity_key(&p.join(name).to_string_lossy())
            })
            .collect();
        wanted.sort();
        assert_eq!(actual, wanted);
    };
    assert_targets(2);
    let merged = "export class Dependency {} export interface Dependency { marker: string; } export const unrelated = ['noise'];";
    std::fs::write(p.join("left.ts"), merged).unwrap();
    assert_targets(2);
    {
        let owner = state.semantic_owner_path(&file.to_string_lossy());
        let index = state.workspace_index_read();
        let authority = index.retained_typescript_authority(&owner).unwrap();
        assert!(authority.constructor_parameters.iter().any(|parameter|
            matches!(&parameter.type_resolution, Some(crate::ir::typescript_resolution::Resolution::Merged(ids))
                if ids.len() == 2 && ids.iter().any(|id| id.kind == "interface_declaration"))
                && matches!(&parameter.value_resolution, Some(crate::ir::typescript_resolution::Resolution::Resolved(id))
                    if id.kind == "class_declaration")), "retain both meanings and every type contributor");
    }
    std::fs::write(p.join("left.ts"), "export class Dependency {} export class Dependency {} export interface Dependency {marker:string;}").unwrap();
    assert_targets(1);
    std::fs::write(p.join("left.ts"), merged).unwrap();
    assert_targets(2);
    assert_eq!(std::fs::read_to_string(file).unwrap(), source);
}

#[test]
fn issue_162_constructor_authority_merged_family_durable_roundtrip() {
    use serde_json::json;
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install_inject_apis(p);
    let merged = "export class Dependency {} export interface Dependency { marker:string; }";
    std::fs::write(p.join("left.ts"), merged).unwrap();
    std::fs::write(p.join("right.ts"), "export class Dependency {}").unwrap();
    let source = "import {Injectable as Service, inject as obtain} from '@angular/core'; import {Dependency as Left} from './left'; import {Dependency as Right} from './right'; const noise='constructor(fake:Phantom)'; @Service() export class Owner {constructor(private readonly first:Left, second:Right) {} repeated=obtain(Right); method(value:Left){return value;} }";
    let file = p.join("app.ts").to_string_lossy().into_owned();
    std::fs::write(&file, source).unwrap();
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    config.persistence.enabled = true;
    config.persistence.db_path = p.join("merged.db").to_string_lossy().into_owned();
    let state = crate::mcp::McpState::new(config.clone());
    let response = override_tool(
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
    assert!(authority.constructor_parameters.iter().any(|parameter|
        matches!(&parameter.type_resolution, Some(crate::ir::typescript_resolution::Resolution::Merged(ids)) if ids.len()==2)
        && parameter.runtime_class().is_some_and(|id| id.source.ends_with("left.ts"))));
    let expected = serde_json::to_value(authority).unwrap();
    drop(state);
    let state = crate::mcp::McpState::new(config);
    let response = override_tool(&state, p, "restore_context", json!({"filePath":file}));
    assert!(response.get("error").is_none(), "{response}");
    let alias = state.alias_for_path(&file).unwrap();
    let restored = state.semantic_snapshot(&alias).unwrap();
    assert_eq!(
        serde_json::to_value(restored.authority.typescript_authority.as_ref().unwrap()).unwrap(),
        expected
    );
    {
        let owner = state.semantic_owner_path(&file);
        let index = state.workspace_index_read();
        assert_eq!(
            serde_json::to_value(index.retained_typescript_authority(&owner).unwrap()).unwrap(),
            expected
        );
        let edges = index.forward_edges_by_identity("angular", "Service", "Owner");
        let injected: Vec<_> = edges
            .iter()
            .filter(|edge| edge.relation == SemanticRelation::Injects)
            .collect();
        assert_eq!(
            injected.len(),
            2,
            "both canonical facts must restore before query acquisition"
        );
        assert!(
            injected
                .iter()
                .all(|edge| edge.subject.canonical.is_some() && edge.object.canonical.is_some())
        );
    }
    std::fs::write(
        p.join("left.ts"),
        "export interface Dependency {marker:string;}",
    )
    .unwrap();
    let response = override_tool(
        &state,
        p,
        "workspace_query",
        json!({"type":"entities_in_file","file_path":file,"fidelity":"high"}),
    );
    assert!(response.get("error").is_none(), "{response}");
    let response = override_tool(
        &state,
        p,
        "workspace_query",
        json!({"type":"forward_edges","domain":"angular","entity_type":"Service","name":"Owner"}),
    );
    assert!(response.get("error").is_none(), "{response}");
    let edges: Vec<_> = response["result"]["structuredContent"]["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|edge| edge["relation"] == "Injects")
        .collect();
    assert_eq!(
        edges.len(),
        1,
        "restored interface contributors cannot retain a removed runtime class"
    );
    assert_eq!(
        edges[0]["object"]["file"],
        json!(crate::dictionary::path::canonical_identity_key(
            &p.join("right.ts").to_string_lossy()
        ))
    );
    assert_eq!(std::fs::read_to_string(file).unwrap(), source);
}
