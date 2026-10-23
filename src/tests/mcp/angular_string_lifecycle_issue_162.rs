use serde_json::{Value, json};
use std::path::Path;

fn call(state: &crate::mcp::McpState, root: &Path, tool: &str, mut args: Value) -> Value {
    args["workspaceRoot"] = json!(root);
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(&json!(162), tool, &json!({"arguments":args}), state);
    crate::protocol::captured_responses().pop().unwrap()
}

fn expected_proof(state: &crate::mcp::McpState, file: &str) -> Value {
    let alias = state.alias_for_path(file).expect("real source-owned alias");
    let snapshot = state
        .semantic_snapshot(&alias)
        .expect("real source-owned snapshot");
    let endpoints: Vec<_> = snapshot
        .edges
        .iter()
        .map(|edge| (&edge.subject.canonical, &edge.object.canonical))
        .collect();
    json!({"authority":snapshot.authority, "source_hash":snapshot.source_hash,
        "endpoints":endpoints})
}

fn values(value: &str) -> String {
    format!(
        "export const /* real trivia */ ROOT = '{value}'; export const ALIAS = ROOT; const unrelated = \"ROOT='phantom'\"; function noise<T>(ROOT:T) {{return ROOT;}}"
    )
}

fn first_source() -> &'static str {
    r#"
import {Injectable as Service, Component as View} from '@angular/core';
import {Select as Pick} from './api';
import {Shared} from './tokens';
const local = Shared;
const noise = "constructor(@Pick('phantom') x:unknown)";
@Service() export class Owner {
 constructor(@Pick(local) a:unknown, @Pick(Shared) b:unknown) {}
 method(Shared:unknown) {return Shared;}
}
@View({selector:'first-host', providers:[
 {provide:local,useValue:'one'}, {provide:'audit\u002eapi',useValue:'two'}
]}) export class ProviderHost {}
"#
}

fn setup() -> (tempfile::TempDir, crate::config::CleanCtxConfig, Vec<Value>) {
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install_inject_apis(p);
    let core = p.join("node_modules/@angular/core/index.d.ts");
    let mut api = std::fs::read_to_string(&core).unwrap();
    api.push_str("export declare function Inject(token:any):any;");
    std::fs::write(core, api).unwrap();
    std::fs::write(
        p.join("api.ts"),
        "export {Inject as Select} from '@angular/core';",
    )
    .unwrap();
    std::fs::write(p.join("values.ts"), values("audit.api")).unwrap();
    std::fs::write(
        p.join("tokens.ts"),
        "export {ALIAS as Shared} from './values';",
    )
    .unwrap();
    std::fs::write(p.join("app.ts"), first_source()).unwrap();
    std::fs::write(
        p.join("neighbor.ts"),
        r#"
import {Injectable as Service} from '@angular/core';
import * as api from './api';
import * as tokens from './tokens';
const misleading = "provide:'phantom'";
@Service() export class Owner {constructor(@api.Select(tokens.Shared) value:unknown) {}}
"#,
    )
    .unwrap();
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    config.persistence.enabled = true;
    config.persistence.db_path = p.join("strings.db").to_string_lossy().into_owned();
    let state = crate::mcp::McpState::new(config.clone());
    let mut expected = Vec::new();
    for name in ["app.ts", "neighbor.ts"] {
        let file = p.join(name).to_string_lossy().into_owned();
        let response = call(
            &state,
            p,
            "compress_code_context",
            json!({"filePath":file,"fidelity":"high"}),
        );
        assert!(response.get("error").is_none(), "{response}");
        let proof = expected_proof(&state, &file);
        let facets = proof["authority"]["typescript_authority"]["declaration_facets"]
            .as_array()
            .unwrap();
        for required in ["ROOT", "ALIAS"] {
            assert!(
                facets
                    .iter()
                    .any(|facet| facet["declaration"]["name"] == required
                        && facet["initializer"].is_object()),
                "nonempty immutable route {required}"
            );
        }
        expected.push(proof);
    }
    state.flush_persistence();
    let database = rusqlite::Connection::open(&config.persistence.db_path).unwrap();
    for (name, proof) in ["app.ts", "neighbor.ts"].iter().zip(&expected) {
        let file = p.join(name).to_string_lossy().into_owned();
        let encoded: String = database
            .query_row(
                "SELECT edges_json FROM semantic_edge_snapshots WHERE file_path = ?1",
                [&file],
                |row| row.get(0),
            )
            .unwrap();
        let durable: Value = serde_json::from_str(&encoded).unwrap();
        assert_eq!(
            durable["typescript_authority"], proof["authority"]["typescript_authority"],
            "actual SQLite must retain the complete route and witnesses"
        );
    }
    drop(database);
    (root, config, expected)
}

fn restore(state: &crate::mcp::McpState, root: &Path, name: &str) {
    let response = call(
        state,
        root,
        "restore_context",
        json!({"filePath":root.join(name)}),
    );
    assert!(response.get("error").is_none(), "{response}");
}

fn edges(
    state: &crate::mcp::McpState,
    root: &Path,
    kind: &str,
    owner: &str,
    relation: &str,
) -> Vec<Value> {
    let response = call(
        state,
        root,
        "workspace_query",
        json!({"type":"forward_edges",
        "domain":"angular","entity_type":kind,"name":owner}),
    );
    assert!(response.get("error").is_none(), "{response}");
    response["result"]["structuredContent"]["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|edge| edge["relation"] == relation)
        .cloned()
        .collect()
}

fn assert_answers(state: &crate::mcp::McpState, root: &Path, value: &str, provider_count: usize) {
    let consumers = edges(state, root, "Service", "Owner", "Injects");
    assert_eq!(
        consumers.len(),
        2,
        "same-named owners retain independent source facts: {consumers:?}"
    );
    assert!(
        consumers
            .iter()
            .all(|edge| edge["object"]["name"] == format!("\"{value}\""))
    );
    let origins: std::collections::BTreeSet<_> = consumers
        .iter()
        .map(|edge| {
            crate::dictionary::path::canonical_identity_key(
                edge["subject"]["file"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(origins.len(), 2);
    assert_eq!(
        edges(
            state,
            root,
            "Component",
            "ProviderHost",
            "RegistersProvider"
        )
        .len(),
        provider_count
    );
}

#[test]
fn issue_162_string_lifecycle_restart_preserves_complete_route_and_index_proof() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config, expected) = setup();
    let p = root.path();
    let state = crate::mcp::McpState::new(config);
    for (name, proof) in ["app.ts", "neighbor.ts"].iter().zip(&expected) {
        restore(&state, p, name);
        let file = p.join(name).to_string_lossy().into_owned();
        assert_eq!(
            expected_proof(&state, &file),
            *proof,
            "session must restore exact proof before any public query can reacquire it"
        );
        {
            let owner = state.semantic_owner_path(&file);
            let index = state.workspace_index_read();
            let authority = index
                .retained_typescript_authority(&owner)
                .expect("real restored index authority");
            assert_eq!(
                serde_json::to_value(authority).unwrap(),
                proof["authority"]["typescript_authority"]
            );
        }
    }
    assert_answers(&state, p, "audit.api", 1);
}

#[test]
fn issue_162_string_lifecycle_stale_initializer_refuses_restore_and_reacquires() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config, _) = setup();
    let p = root.path();
    let original = std::fs::read(p.join("app.ts")).unwrap();
    std::fs::write(p.join("values.ts"), values("other.api")).unwrap();
    let state = crate::mcp::McpState::new(config);
    for name in ["app.ts", "neighbor.ts"] {
        let file = p.join(name).to_string_lossy().into_owned();
        let response = call(&state, p, "restore_context", json!({"filePath":file}));
        assert!(
            response.get("error").is_some(),
            "stale derived string adopted: {response}"
        );
        assert!(state.alias_for_path(&file).is_none());
    }
    for name in ["app.ts", "neighbor.ts"] {
        let file = p.join(name).to_string_lossy().into_owned();
        let response = call(
            &state,
            p,
            "workspace_query",
            json!({"type":"entities_in_file", "file_path":file,"fidelity":"high"}),
        );
        assert!(response.get("error").is_none(), "{response}");
    }
    assert_answers(&state, p, "other.api", 2);
    assert_eq!(std::fs::read(p.join("app.ts")).unwrap(), original);
}

#[test]
fn issue_162_string_lifecycle_post_restore_mutation_and_independent_source_retirement() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config, _) = setup();
    let p = root.path();
    let state = crate::mcp::McpState::new(config);
    restore(&state, p, "app.ts");
    restore(&state, p, "neighbor.ts");
    assert_answers(&state, p, "audit.api", 1);
    let neighbor = std::fs::read(p.join("neighbor.ts")).unwrap();
    std::fs::write(p.join("values.ts"), values("other.api")).unwrap();
    assert_answers(&state, p, "other.api", 2);
    std::fs::write(p.join("values.ts"), values("audit.api")).unwrap();
    assert_answers(&state, p, "audit.api", 1);
    let replacement = first_source().replace(
        "constructor(@Pick(local) a:unknown, @Pick(Shared) b:unknown) {}",
        "",
    );
    assert_ne!(replacement, first_source(), "real source edit control");
    std::fs::write(p.join("app.ts"), replacement).unwrap();
    let response = call(
        &state,
        p,
        "workspace_query",
        json!({"type":"entities_in_file", "file_path":p.join("app.ts"),"fidelity":"high"}),
    );
    assert!(response.get("error").is_none(), "{response}");
    let remaining = edges(&state, p, "Service", "Owner", "Injects");
    assert_eq!(
        remaining.len(),
        1,
        "replacement must retire only that source's dependency fact"
    );
    assert_eq!(
        crate::dictionary::path::canonical_identity_key(
            remaining[0]["subject"]["file"].as_str().unwrap()
        ),
        crate::dictionary::path::canonical_identity_key(&p.join("neighbor.ts").to_string_lossy())
    );
    std::fs::remove_file(p.join("app.ts")).unwrap();
    // External deletion uses the documented current-source reconciliation
    // boundary; discovery does not act as an implicit filesystem watcher.
    let response = call(&state, p, "refresh_workspace", json!({}));
    assert!(response.get("error").is_none(), "{response}");
    assert!(edges(&state, p, "Component", "ProviderHost", "RegistersProvider").is_empty());
    assert_eq!(edges(&state, p, "Service", "Owner", "Injects").len(), 1);
    assert_eq!(std::fs::read(p.join("neighbor.ts")).unwrap(), neighbor);
}
