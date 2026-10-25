use serde_json::{Value, json};
use std::path::Path;

const SOURCE: &str = r#"
import {View} from './api';
import {AuditApi as Left} from './left';
import {AuditApi as Right} from './right';
const misleading="provide:Phantom";
function noise<T>(Left:T){return Left;}
@View({providers:[{provide:Left,useValue:1},{provide:Right,useValue:2},{provide:Left,useValue:3}]})
export class ProviderHost {}
"#;

const SHORTHAND_SOURCE: &str = r#"
import {View} from './api';
import {AuditApi as Left} from './left';
import * as right from './right';
const misleading="providers:[Phantom]";
function noise<T>(Left:T){return Left;}
@View({providers:[Left,right.AuditApi,Left]})
export class ProviderHost {}
"#;

fn call(state: &crate::mcp::McpState, root: &Path, tool: &str, mut args: Value) -> Value {
    args["workspaceRoot"] = json!(root);
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(&json!(162), tool, &json!({"arguments":args}), state);
    crate::protocol::captured_responses().pop().unwrap()
}

const CLASS_SOURCE: &str = r#"
import {View} from './api';
import {AuditApi as Left} from './left';
import * as right from './right';
const misleading="useClass:Phantom";
function noise<T>(Left:T){return Left;}
@View({providers:[{provide:Left,useClass:Left},
 {provide:right.AuditApi,useClass:right.AuditApi},{provide:Left,useClass:Left}]})
export class ProviderHost {}
"#;

const FACTORY_SOURCE: &str = r#"
import {View} from './api';
import {AuditApi as Left} from './left';
import {AuditApi as Right} from './right';
import {makeAudit as Build} from './factory-left';
import * as factories from './factory-right';
function noise<T>(Build:T){return Build;}
@View({providers:[{provide:Left,useFactory:Build},
 {provide:Right,useFactory:factories.makeAudit},{provide:Left,useFactory:Build}]})
export class ProviderHost {}
"#;

fn proof(state: &crate::mcp::McpState, file: &str) -> Value {
    let alias = state.alias_for_path(file).unwrap();
    let snapshot = state.semantic_snapshot(&alias).unwrap();
    let endpoints: Vec<_> = snapshot
        .edges
        .iter()
        .filter(|edge| {
            edge.subject.entity_type == "Component"
                && edge.relation
                    == crate::layers::meta::semantic::SemanticRelation::RegistersProvider
        })
        .map(|edge| (&edge.subject.canonical, &edge.object.canonical))
        .collect();
    assert_eq!(
        endpoints.len(),
        3,
        "retain all provider occurrences; public index answers deduplicate"
    );
    assert!(
        endpoints
            .iter()
            .all(|(owner, token)| owner.is_some() && token.is_some())
    );
    let strategies: Vec<_> = snapshot
        .edges
        .iter()
        .filter(|edge| {
            matches!(
                edge.relation,
                crate::layers::meta::semantic::SemanticRelation::ProviderUsesClass
                    | crate::layers::meta::semantic::SemanticRelation::ProviderUsesFactory
            )
        })
        .collect();
    assert!(snapshot.authority.admits_relation_evidence(&snapshot.edges));
    if !strategies.is_empty() {
        assert_eq!(strategies.len(), 3, "retain all class strategy occurrences");
        assert!(
            strategies
                .iter()
                .all(|edge| edge.subject.canonical.is_some() && edge.object.canonical.is_some())
        );
    }
    let strategies: Vec<_> = strategies
        .iter()
        .map(|edge| (&edge.subject.canonical, &edge.object.canonical))
        .collect();
    json!({"authority":snapshot.authority,"source_hash":snapshot.source_hash,"endpoints":endpoints,"strategies":strategies})
}

fn setup(source: &str) -> (tempfile::TempDir, crate::config::CleanCtxConfig, Vec<Value>) {
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install(p);
    std::fs::write(
        p.join("node_modules/@angular/core/index.d.ts"),
        "export declare function Component(metadata:unknown):any;",
    )
    .unwrap();
    for name in ["left.ts", "right.ts"] {
        std::fs::write(p.join(name), "export class AuditApi {}").unwrap();
    }
    for name in ["factory-left.ts", "factory-right.ts"] {
        std::fs::write(p.join(name), "export function makeAudit(){return {};}").unwrap();
    }
    std::fs::write(
        p.join("api.ts"),
        "export {Component as View} from '@angular/core';",
    )
    .unwrap();
    for name in ["app.ts", "neighbor.ts"] {
        std::fs::write(p.join(name), source).unwrap();
    }
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config.persistence.enabled = true;
    config.persistence.db_path = p
        .join("provider-declarations.db")
        .to_string_lossy()
        .into_owned();
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
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
        expected.push(proof(&state, &file));
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
            durable["typescript_authority"],
            proof["authority"]["typescript_authority"]
        );
        let endpoints: Vec<_> = durable["edges"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|edge| {
                edge["subject"]["entity_type"] == "Component"
                    && edge["relation"] == "RegistersProvider"
            })
            .map(|edge| json!([edge["subject"]["canonical"], edge["object"]["canonical"]]))
            .collect();
        assert_eq!(
            json!(endpoints),
            proof["endpoints"],
            "complete SQLite endpoints"
        );
        let strategies: Vec<_> = durable["edges"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|edge| {
                edge["relation"] == "ProviderUsesClass" || edge["relation"] == "ProviderUsesFactory"
            })
            .map(|edge| json!([edge["subject"]["canonical"], edge["object"]["canonical"]]))
            .collect();
        assert_eq!(
            json!(strategies),
            proof["strategies"],
            "complete SQLite class strategy endpoints"
        );
    }
    drop(database);
    (root, config, expected)
}

fn answers(state: &crate::mcp::McpState, root: &Path, owners: &[&str], tokens: &[&str]) {
    for name in owners {
        let response = call(
            state,
            root,
            "workspace_query",
            json!({"type":"entities_in_file","file_path":root.join(name),"fidelity":"high"}),
        );
        assert!(response.get("error").is_none(), "{response}");
    }
    let response = call(
        state,
        root,
        "workspace_query",
        json!({"type":"forward_edges","domain":"angular","entity_type":"Component","name":"ProviderHost"}),
    );
    assert!(response.get("error").is_none(), "{response}");
    let selected: Vec<_> = response["result"]["structuredContent"]["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|edge| edge["relation"] == "RegistersProvider")
        .collect();
    let mut actual: Vec<_> = selected
        .iter()
        .map(|edge| {
            assert_eq!(edge["object"]["name"], "AuditApi");
            assert!(edge["object"].get("canonical").is_none());
            (
                crate::dictionary::path::canonical_identity_key(
                    edge["subject"]["file"].as_str().unwrap(),
                ),
                crate::dictionary::path::canonical_identity_key(
                    edge["object"]["file"].as_str().unwrap(),
                ),
            )
        })
        .collect();
    actual.sort();
    let mut expected: Vec<_> = owners
        .iter()
        .flat_map(|owner| {
            tokens.iter().map(move |token| {
                (
                    crate::dictionary::path::canonical_identity_key(
                        &root.join(owner).to_string_lossy(),
                    ),
                    crate::dictionary::path::canonical_identity_key(
                        &root.join(token).to_string_lossy(),
                    ),
                )
            })
        })
        .collect();
    expected.sort();
    assert_eq!(
        actual, expected,
        "exact source-owned provider registration identities"
    );
}

fn strategy_answers(
    state: &crate::mcp::McpState,
    root: &Path,
    source: &str,
    owners: usize,
    tokens: &[&str],
) {
    if source == SOURCE {
        return;
    }
    let factory = source.contains("useFactory:");
    let relation = if factory {
        "ProviderUsesFactory"
    } else {
        "ProviderUsesClass"
    };
    let response = call(
        state,
        root,
        "workspace_query",
        json!({"type":"forward_edges","domain":"angular","entity_type":"ProviderToken","name":"AuditApi"}),
    );
    assert!(response.get("error").is_none(), "{response}");
    let mut actual: Vec<_> = response["result"]["structuredContent"]["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|edge| edge["relation"] == relation)
        .map(|edge| {
            assert!(edge["subject"].get("canonical").is_none());
            assert!(edge["object"].get("canonical").is_none());
            assert_eq!(
                edge["object"]["name"],
                if factory { "makeAudit" } else { "AuditApi" }
            );
            let token = crate::dictionary::path::canonical_identity_key(
                edge["subject"]["file"].as_str().unwrap(),
            );
            let target = crate::dictionary::path::canonical_identity_key(
                edge["object"]["file"].as_str().unwrap(),
            );
            (token, target)
        })
        .collect();
    actual.sort();
    let mut expected: Vec<_> = (0..owners)
        .flat_map(|_| {
            tokens.iter().map(|token| {
                let target = if factory {
                    format!("factory-{token}")
                } else {
                    (*token).to_owned()
                };
                (
                    crate::dictionary::path::canonical_identity_key(
                        &root.join(token).to_string_lossy(),
                    ),
                    crate::dictionary::path::canonical_identity_key(
                        &root.join(target).to_string_lossy(),
                    ),
                )
            })
        })
        .collect();
    expected.sort();
    assert_eq!(
        actual, expected,
        "source-owned class strategy occurrences after lifecycle transition"
    );
}

#[test]
fn issue_162_provider_declaration_lifecycle_restart_retains_exact_authority_and_endpoints() {
    restart(SOURCE);
}

#[test]
fn issue_162_provider_shorthand_lifecycle_restart_retains_exact_authority_and_endpoints() {
    restart(SHORTHAND_SOURCE);
}

#[test]
fn issue_162_provider_class_strategy_lifecycle_restart_retains_exact_pairs() {
    restart(CLASS_SOURCE);
}

#[test]
fn issue_162_provider_factory_lifecycle_restart_retains_exact_pairs() {
    restart(FACTORY_SOURCE);
}

fn restart(source: &str) {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config, expected) = setup(source);
    let state = crate::mcp::McpState::new(config);
    for (name, expected) in ["app.ts", "neighbor.ts"].iter().zip(expected) {
        let file = root.path().join(name).to_string_lossy().into_owned();
        let response = call(
            &state,
            root.path(),
            "restore_context",
            json!({"filePath":file}),
        );
        assert!(response.get("error").is_none(), "{response}");
        assert_eq!(proof(&state, &file), expected, "before query reacquisition");
        if source != SOURCE {
            assert_eq!(expected["strategies"].as_array().unwrap().len(), 3);
        }
        let owner = state.semantic_owner_path(&file);
        let index = state.workspace_index_read();
        assert_eq!(
            serde_json::to_value(index.retained_typescript_authority(&owner).unwrap()).unwrap(),
            expected["authority"]["typescript_authority"]
        );
    }
    answers(
        &state,
        root.path(),
        &["app.ts", "neighbor.ts"],
        &["left.ts", "right.ts"],
    );
    strategy_answers(&state, root.path(), source, 2, &["left.ts", "right.ts"]);
}

#[test]
fn issue_162_provider_declaration_lifecycle_stale_tokens_repair_and_independent_retirement() {
    stale_tokens_repair_and_retirement(SOURCE);
}

#[test]
fn issue_162_provider_shorthand_lifecycle_stale_tokens_repair_and_independent_retirement() {
    stale_tokens_repair_and_retirement(SHORTHAND_SOURCE);
}

#[test]
fn issue_162_provider_class_strategy_lifecycle_stale_repair_and_independent_retirement() {
    stale_tokens_repair_and_retirement(CLASS_SOURCE);
}

#[test]
fn issue_162_provider_factory_lifecycle_stale_repair_and_independent_retirement() {
    stale_tokens_repair_and_retirement(FACTORY_SOURCE);
}

#[test]
fn issue_162_provider_class_strategy_lifecycle_independent_target_stale_restore_and_repair() {
    let _serial = crate::protocol::handler_response_serial();
    let source = CLASS_SOURCE.replace("provide:right.AuditApi", "provide:Left");
    let (root, config, _) = setup(&source);
    let p = root.path();
    std::fs::write(p.join("right.ts"), "export interface AuditApi {}").unwrap();
    let state = crate::mcp::McpState::new(config);
    // Refuse stale adoption before any query can reacquire the provider proof.
    for name in ["app.ts", "neighbor.ts"] {
        let file = p.join(name).to_string_lossy().into_owned();
        let response = call(&state, p, "restore_context", json!({"filePath":file}));
        assert!(
            response.get("error").is_some(),
            "stale target adopted: {response}"
        );
        assert!(state.alias_for_path(&file).is_none());
    }
    for restored in [false, true] {
        if restored {
            std::fs::write(p.join("right.ts"), "export class AuditApi {}").unwrap();
        }
        answers(&state, p, &["app.ts", "neighbor.ts"], &["left.ts"]);
        let response = call(
            &state,
            p,
            "workspace_query",
            json!({"type":"forward_edges","domain":"angular","entity_type":"ProviderToken","name":"AuditApi"}),
        );
        assert!(response.get("error").is_none(), "{response}");
        let mut targets: Vec<_> = response["result"]["structuredContent"]["edges"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|edge| edge["relation"] == "ProviderUsesClass")
            .map(|edge| {
                assert_eq!(
                    crate::dictionary::path::canonical_identity_key(
                        edge["subject"]["file"].as_str().unwrap()
                    ),
                    crate::dictionary::path::canonical_identity_key(
                        &p.join("left.ts").to_string_lossy()
                    ),
                    "token stays independent of target edits"
                );
                crate::dictionary::path::canonical_identity_key(
                    edge["object"]["file"].as_str().unwrap(),
                )
            })
            .collect();
        targets.sort();
        let names = if restored {
            vec!["left.ts", "right.ts"]
        } else {
            vec!["left.ts"]
        };
        let mut expected: Vec<_> = (0..2)
            .flat_map(|_| {
                names.iter().map(|name| {
                    crate::dictionary::path::canonical_identity_key(&p.join(name).to_string_lossy())
                })
            })
            .collect();
        expected.sort();
        assert_eq!(targets, expected, "independent target retraction/repair");
        for name in ["app.ts", "neighbor.ts"] {
            assert_eq!(std::fs::read_to_string(p.join(name)).unwrap(), source);
        }
    }
}

fn stale_tokens_repair_and_retirement(source: &str) {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config, _) = setup(source);
    let p = root.path();
    std::fs::write(p.join("right.ts"), "export interface AuditApi {}").unwrap();
    let state = crate::mcp::McpState::new(config);
    for name in ["app.ts", "neighbor.ts"] {
        let file = p.join(name).to_string_lossy().into_owned();
        let response = call(&state, p, "restore_context", json!({"filePath":file}));
        assert!(
            response.get("error").is_some(),
            "stale token proof adopted: {response}"
        );
        assert!(state.alias_for_path(&file).is_none());
    }
    answers(&state, p, &["app.ts", "neighbor.ts"], &["left.ts"]);
    strategy_answers(&state, p, source, 2, &["left.ts"]);
    std::fs::write(p.join("right.ts"), "export class AuditApi {}").unwrap();
    answers(
        &state,
        p,
        &["app.ts", "neighbor.ts"],
        &["left.ts", "right.ts"],
    );
    assert_eq!(std::fs::read_to_string(p.join("app.ts")).unwrap(), source);
    strategy_answers(&state, p, source, 2, &["left.ts", "right.ts"]);
    assert_eq!(
        std::fs::read_to_string(p.join("neighbor.ts")).unwrap(),
        source
    );
    std::fs::write(p.join("app.ts"), "export class ProviderHost {}").unwrap();
    let response = call(
        &state,
        p,
        "workspace_query",
        json!({"type":"entities_in_file","file_path":p.join("app.ts"),"fidelity":"high"}),
    );
    assert!(response.get("error").is_none(), "{response}");
    answers(&state, p, &["neighbor.ts"], &["left.ts", "right.ts"]);
    strategy_answers(&state, p, source, 1, &["left.ts", "right.ts"]);
    std::fs::remove_file(p.join("neighbor.ts")).unwrap();
    let response = call(&state, p, "refresh_workspace", json!({}));
    assert!(response.get("error").is_none(), "{response}");
    answers(&state, p, &[], &[]);
    strategy_answers(&state, p, source, 0, &[]);
}
