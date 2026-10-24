use serde_json::{Value, json};
use std::path::Path;

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
