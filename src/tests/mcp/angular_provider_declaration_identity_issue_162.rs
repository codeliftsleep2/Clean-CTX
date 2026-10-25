use serde_json::{Value, json};
use std::path::Path;

#[test]
fn issue_162_provider_existing_alias_preserves_distinct_registration_tokens() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install(p);
    std::fs::write(p.join("token.ts"), "export class AuditApi {}").unwrap();
    std::fs::write(p.join("existing.ts"), "export class ExistingApi {}").unwrap();
    std::fs::write(
        p.join("api.ts"),
        "export {Component as View} from '@angular/core';",
    )
    .unwrap();
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    let control = "import {Component} from '@angular/core'; import {AuditApi} from './token'; import {ExistingApi} from './existing'; @Component({providers:[ExistingApi,{provide:AuditApi,useClass:ExistingApi}]}) export class ProviderHost {}";
    std::fs::write(p.join("app.ts"), control).unwrap();
    let ordinary = registrations(&state, p);
    assert_eq!(
        ordinary.len(),
        2,
        "nonempty established class-provider control"
    );
    let source = r#"
import {View} from './api';
import {AuditApi as Token} from './token';
import * as existing from './existing';
const prose='useExisting:Phantom';
function noise<T>(existing:T){return existing;}
@View({providers:[existing.ExistingApi,
 {provide:Token,useExisting:existing.ExistingApi},
 {provide:Token,useExisting:existing.ExistingApi}]})
export class ProviderHost {}
"#;
    std::fs::write(p.join("app.ts"), source).unwrap();
    for _ in 0..2 {
        let found = registrations(&state, p);
        assert_eq!(
            found.len(),
            2,
            "alias registration retains its own token; no injector satisfaction inferred: {found:?}"
        );
        let actual: std::collections::BTreeSet<_> = found
            .iter()
            .map(|edge| {
                (
                    edge["object"]["name"].as_str().unwrap().to_owned(),
                    crate::dictionary::path::canonical_identity_key(
                        edge["object"]["file"].as_str().unwrap(),
                    ),
                )
            })
            .collect();
        let expected: std::collections::BTreeSet<_> =
            [("AuditApi", "token.ts"), ("ExistingApi", "existing.ts")]
                .into_iter()
                .map(|(name, file)| {
                    (
                        name.to_owned(),
                        crate::dictionary::path::canonical_identity_key(
                            &p.join(file).to_string_lossy(),
                        ),
                    )
                })
                .collect();
        assert_eq!(actual, expected);
    }
    assert_eq!(std::fs::read_to_string(p.join("app.ts")).unwrap(), source);
}

#[test]
fn issue_162_provider_strategy_same_named_classes_retain_declaration_identity() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install(p);
    std::fs::write(p.join("token.ts"), "export class AuditApi {}").unwrap();
    for name in ["left.ts", "right.ts"] {
        std::fs::write(p.join(name), "export class Implementation {}").unwrap();
    }
    let source = r#"
import {Component as View} from '@angular/core';
import {AuditApi as Token} from './token';
import {Implementation as Left} from './left';
import * as right from './right';
const prose="useClass:Phantom";
function noise<T>(Left:T){return Left;}
@View({providers:[{provide:Token,useClass:Left},{provide:Token,useClass:right.Implementation}]})
export class ProviderHost {}
"#;
    std::fs::write(p.join("app.ts"), source).unwrap();
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    let found = registrations(&state, p);
    assert_eq!(found.len(), 1, "nonempty shared-token registration control");
    assert_eq!(found[0]["object"]["name"], "AuditApi");
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(
        &json!(162),
        "compress_code_context",
        &json!({"arguments":{"workspaceRoot":p,"filePath":p.join("app.ts"),"fidelity":"high"}}),
        &state,
    );
    let response = crate::protocol::captured_responses().pop().unwrap();
    assert!(response.get("error").is_none(), "{response}");
    let alias = state
        .alias_for_path(&p.join("app.ts").to_string_lossy())
        .unwrap();
    let snapshot = state.semantic_snapshot(&alias).unwrap();
    let strategies: Vec<_> = snapshot
        .edges
        .iter()
        .filter(|edge| {
            edge.relation == crate::layers::meta::semantic::SemanticRelation::ProviderUsesClass
        })
        .cloned()
        .collect();
    assert_eq!(strategies.len(), 2, "retain both strategy occurrences");
    let actual: std::collections::BTreeSet<_> = strategies
        .iter()
        .map(|edge| {
            assert!(
                edge.subject.canonical.is_some(),
                "strategy must retain typed token identity"
            );
            let declaration = edge
                .object
                .canonical
                .as_ref()
                .and_then(|endpoint| endpoint.declaration())
                .expect("strategy target must retain typed declaration identity");
            assert_eq!(declaration.name, "Implementation");
            crate::dictionary::path::canonical_identity_key(&declaration.source.to_string_lossy())
        })
        .collect();
    let expected: std::collections::BTreeSet<_> = ["left.ts", "right.ts"]
        .into_iter()
        .map(|name| {
            crate::dictionary::path::canonical_identity_key(&p.join(name).to_string_lossy())
        })
        .collect();
    assert_eq!(
        actual, expected,
        "same labels must retain distinct strategy targets"
    );
    assert!(snapshot.authority.admits_relation_evidence(&strategies));
    assert_eq!(std::fs::read_to_string(p.join("app.ts")).unwrap(), source);
}

#[test]
fn issue_162_provider_strategy_namespace_refuses_erased_and_competing_fields() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install(p);
    std::fs::write(p.join("token.ts"), "export class AuditApi {}").unwrap();
    std::fs::write(
        p.join("implementation.ts"),
        "export class Implementation {} export function makeAudit(){return new Implementation();}",
    )
    .unwrap();
    let barrel = "export {Implementation,makeAudit} from './implementation';";
    std::fs::write(p.join("barrel.ts"), barrel).unwrap();
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    for (strategy, endpoint) in [("useClass", "Implementation"), ("useFactory", "makeAudit")] {
        let source = format!(
            "import {{Component as View}} from '@angular/core'; import {{AuditApi as Token}} from './token'; import * as impl from './barrel'; @View({{providers:[{{provide:Token,{strategy}:impl.{endpoint}}}]}}) export class ProviderHost {{}}"
        );
        std::fs::write(p.join("app.ts"), &source).unwrap();
        assert_eq!(registrations(&state, p).len(), 1, "nonempty {strategy}");
        std::fs::write(
            p.join("barrel.ts"),
            "export type {Implementation,makeAudit} from './implementation';",
        )
        .unwrap();
        assert!(
            registrations(&state, p).is_empty(),
            "erased {strategy} must not retain runtime registration"
        );
        std::fs::write(p.join("barrel.ts"), barrel).unwrap();
        assert_eq!(registrations(&state, p).len(), 1, "repaired {strategy}");
        assert_eq!(std::fs::read_to_string(p.join("app.ts")).unwrap(), source);
        let duplicate = source.replace(
            &format!("{strategy}:impl.{endpoint}"),
            &format!("{strategy}:impl.{endpoint},{strategy}:impl.{endpoint}"),
        );
        std::fs::write(p.join("app.ts"), duplicate).unwrap();
        assert!(
            registrations(&state, p).is_empty(),
            "duplicate {strategy} fields cannot supply unique retained proof"
        );
        std::fs::write(p.join("app.ts"), &source).unwrap();
        assert_eq!(
            registrations(&state, p).len(),
            1,
            "source repair {strategy}"
        );
    }
}

#[test]
fn issue_162_provider_strategy_namespace_preserves_registration() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install(p);
    std::fs::write(p.join("token.ts"), "export class AuditApi {}").unwrap();
    std::fs::write(
        p.join("implementation.ts"),
        "export class Implementation {} export function makeAudit(){return new Implementation();}",
    )
    .unwrap();
    std::fs::write(
        p.join("barrel.ts"),
        "export {Implementation,makeAudit} from './implementation';",
    )
    .unwrap();
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    for (strategy, endpoint) in [("useClass", "Implementation"), ("useFactory", "makeAudit")] {
        let control = format!(
            "import {{Component}} from '@angular/core'; import {{AuditApi}} from './token'; import {{{endpoint}}} from './implementation'; @Component({{providers:[{{provide:AuditApi,{strategy}:{endpoint}}}]}}) export class ProviderHost {{}}"
        );
        std::fs::write(p.join("app.ts"), control).unwrap();
        let ordinary = registrations(&state, p);
        assert_eq!(ordinary.len(), 1, "nonempty established {strategy} control");
        assert_eq!(ordinary[0]["object"]["name"], "AuditApi");
        let source = format!(
            "import {{Component as View}} from '@angular/core'; import {{AuditApi as Token}} from './token'; import * as impl from './barrel'; const prose='useClass:Phantom'; function noise<T>(impl:T){{return impl;}} @View({{providers:[{{provide:Token,{strategy}:impl.{endpoint}}}]}}) export class ProviderHost {{}}"
        );
        std::fs::write(p.join("app.ts"), &source).unwrap();
        for _ in 0..2 {
            let found = registrations(&state, p);
            assert_eq!(
                found.len(),
                1,
                "proved namespace {strategy} must preserve its registration: {found:?}"
            );
            assert_eq!(found[0]["object"]["name"], "AuditApi");
            assert_eq!(
                crate::dictionary::path::canonical_identity_key(
                    found[0]["object"]["file"].as_str().unwrap()
                ),
                crate::dictionary::path::canonical_identity_key(
                    &p.join("token.ts").to_string_lossy()
                )
            );
        }
        assert_eq!(std::fs::read_to_string(p.join("app.ts")).unwrap(), source);
    }
}

fn registrations(state: &crate::mcp::McpState, root: &Path) -> Vec<Value> {
    let query = |mut args: Value| {
        args["workspaceRoot"] = json!(root);
        crate::protocol::captured_responses().clear();
        crate::mcp::tools::dispatch_tools_call(
            &json!(162),
            "workspace_query",
            &json!({"arguments":args}),
            state,
        );
        let response = crate::protocol::captured_responses().pop().unwrap();
        assert!(response.get("error").is_none(), "{response}");
        response["result"]["structuredContent"].clone()
    };
    query(json!({"type":"entities_in_file","file_path":root.join("app.ts"),"fidelity":"high"}));
    query(json!({"type":"forward_edges","domain":"angular","entity_type":"Component","name":"ProviderHost"}))["edges"]
        .as_array().unwrap().iter().filter(|edge| edge["relation"] == "RegistersProvider").cloned().collect()
}

#[test]
fn issue_162_provider_declaration_identity_same_named_tokens_and_aliases() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install(p);
    for name in ["left.ts", "right.ts"] {
        std::fs::write(p.join(name), "export class AuditApi {}").unwrap();
    }
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    let control = "import {Component} from '@angular/core'; import {AuditApi} from './left'; @Component({providers:[{provide:AuditApi,useValue:1}]}) export class ProviderHost {}";
    std::fs::write(p.join("app.ts"), control).unwrap();
    let ordinary = registrations(&state, p);
    assert_eq!(
        ordinary.len(),
        1,
        "nonempty established class-token registration"
    );
    assert_eq!(ordinary[0]["object"]["name"], "AuditApi");
    let source = r#"
import {Component as View} from '@angular/core';
import {AuditApi as Left} from './left';
import {AuditApi as Right} from './right';
const misleading="provide:Phantom";
function noise<T>(Left:T){return Left;}
@View({providers:[{provide:Left,useValue:1},{provide:Right,useValue:2},{provide:Left,useValue:3}]})
export class ProviderHost {}
"#;
    std::fs::write(p.join("app.ts"), source).unwrap();
    for _ in 0..2 {
        let found = registrations(&state, p);
        assert_eq!(
            found.len(),
            2,
            "two token identities; repeated registration is not a scope/precedence claim: {found:?}"
        );
        let actual: std::collections::BTreeSet<_> = found
            .iter()
            .map(|edge| {
                assert_eq!(
                    edge["object"]["name"], "AuditApi",
                    "label follows the proved declaration, not its local alias"
                );
                assert!(edge["object"].get("canonical").is_none());
                crate::dictionary::path::canonical_identity_key(
                    edge["object"]["file"].as_str().unwrap(),
                )
            })
            .collect();
        let expected: std::collections::BTreeSet<_> = ["left.ts", "right.ts"]
            .into_iter()
            .map(|name| {
                crate::dictionary::path::canonical_identity_key(&p.join(name).to_string_lossy())
            })
            .collect();
        assert_eq!(actual, expected);
    }
    assert_eq!(std::fs::read_to_string(p.join("app.ts")).unwrap(), source);
}

#[test]
fn issue_162_provider_declaration_refuses_type_only_and_foreign_owner_roles() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install(p);
    std::fs::write(
        p.join("tokens.ts"),
        "export class AuditApi {} export class RefusedApi {}",
    )
    .unwrap();
    std::fs::write(
        p.join("foreign.ts"),
        "export function Component(metadata:unknown){return metadata;}",
    )
    .unwrap();
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    let source = r#"
import {Component as View} from '@angular/core';
import {AuditApi as Real} from './tokens';
import type {RefusedApi as OnlyType} from './tokens';
const prose="provide:OnlyType";
function noise<T>(Real:T){return Real;}
@View({providers:[{provide:Real,useValue:1},{provide:OnlyType,useValue:2}]})
export class ProviderHost {}
"#;
    std::fs::write(p.join("app.ts"), source).unwrap();
    let found = registrations(&state, p);
    assert_eq!(
        found.len(),
        1,
        "type-only value use cannot create another token fact: {found:?}"
    );
    assert_eq!(found[0]["object"]["name"], "AuditApi");
    let foreign = source.replace("from '@angular/core'", "from './foreign'");
    std::fs::write(p.join("app.ts"), foreign).unwrap();
    assert!(
        registrations(&state, p).is_empty(),
        "foreign owner API cannot grant registration authority"
    );
    std::fs::write(p.join("app.ts"), source).unwrap();
    let repaired = registrations(&state, p);
    assert_eq!(repaired.len(), 1);
    assert_eq!(repaired[0]["object"]["name"], "AuditApi");
}

#[test]
fn issue_162_provider_declaration_admission_refuses_forged_endpoint_proof() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install(p);
    std::fs::write(p.join("tokens.ts"), "export class AuditApi {}").unwrap();
    let source = "import {Component as View} from '@angular/core'; import {AuditApi as Token} from './tokens'; @View({providers:[{provide:Token,useValue:1}]}) export class ProviderHost {}";
    std::fs::write(p.join("app.ts"), source).unwrap();
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    assert_eq!(registrations(&state, p).len(), 1, "nonempty public control");
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(
        &json!(162),
        "compress_code_context",
        &json!({"arguments":{"workspaceRoot":p,"filePath":p.join("app.ts"),"fidelity":"high"}}),
        &state,
    );
    let response = crate::protocol::captured_responses().pop().unwrap();
    assert!(response.get("error").is_none(), "{response}");
    let alias = state
        .alias_for_path(&p.join("app.ts").to_string_lossy())
        .unwrap();
    let snapshot = state.semantic_snapshot(&alias).unwrap();
    let selected: Vec<_> = snapshot
        .edges
        .iter()
        .filter(|edge| {
            edge.relation == crate::layers::meta::semantic::SemanticRelation::RegistersProvider
        })
        .cloned()
        .collect();
    assert_eq!(selected.len(), 1);
    assert!(snapshot.authority.admits_relation_evidence(&selected));
    for change in 0..5 {
        let mut forged = selected.clone();
        let declaration = forged[0]
            .object
            .canonical
            .as_mut()
            .unwrap()
            .declaration_mut()
            .unwrap();
        match change {
            0 => declaration.source_hash = "forged".into(),
            1 => declaration.start += 1,
            2 => declaration.name = "RefusedApi".into(),
            3 => declaration.source = p.join("foreign.ts"),
            _ => forged[0].subject.canonical = None,
        }
        assert!(
            !snapshot.authority.admits_relation_evidence(&forged),
            "forged provider endpoint admitted: {change}"
        );
    }
}

#[test]
fn issue_162_provider_shorthand_declaration_alias_and_namespace_identity() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install(p);
    for name in ["left.ts", "right.ts"] {
        std::fs::write(p.join(name), "export class AuditApi {}").unwrap();
    }
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    std::fs::write(p.join("app.ts"),"import {Component} from '@angular/core'; import {AuditApi} from './left'; @Component({providers:[AuditApi]}) export class ProviderHost {}").unwrap();
    let ordinary = registrations(&state, p);
    assert_eq!(
        ordinary.len(),
        1,
        "nonempty established shorthand class registration"
    );
    assert_eq!(ordinary[0]["object"]["name"], "AuditApi");
    let source = r#"
import {Component as View} from '@angular/core';
import {AuditApi as Left} from './left';
import * as right from './right';
const prose="providers:[Phantom]";
function noise<T>(Left:T){return Left;}
@View({providers:[Left,right.AuditApi,Left]}) export class ProviderHost {}
"#;
    std::fs::write(p.join("app.ts"), source).unwrap();
    for _ in 0..2 {
        let found = registrations(&state, p);
        assert_eq!(
            found.len(),
            2,
            "shorthand aliases retain two declaration identities without repeating the left registration: {found:?}"
        );
        let actual: std::collections::BTreeSet<_> = found
            .iter()
            .map(|edge| {
                assert_eq!(edge["object"]["name"], "AuditApi");
                crate::dictionary::path::canonical_identity_key(
                    edge["object"]["file"].as_str().unwrap(),
                )
            })
            .collect();
        let expected: std::collections::BTreeSet<_> = ["left.ts", "right.ts"]
            .into_iter()
            .map(|name| {
                crate::dictionary::path::canonical_identity_key(&p.join(name).to_string_lossy())
            })
            .collect();
        assert_eq!(actual, expected);
    }
    assert_eq!(std::fs::read_to_string(p.join("app.ts")).unwrap(), source);
}
