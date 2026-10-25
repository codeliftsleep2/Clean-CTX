use crate::layers::meta::semantic::SemanticRelation;
use serde_json::{Value, json};
use std::path::Path;

fn call(state: &crate::mcp::McpState, root: &Path, tool: &str, mut args: Value) -> Value {
    args["workspaceRoot"] = json!(root);
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(&json!(162), tool, &json!({"arguments":args}), state);
    let response = crate::protocol::captured_responses().pop().unwrap();
    assert!(response.get("error").is_none(), "{response}");
    response["result"]["structuredContent"].clone()
}

#[test]
fn issue_162_provider_factory_callable_boundary_preserves_legacy_and_refuses_bad_roles() {
    provider_boundary(false);
}

#[test]
fn issue_162_provider_existing_boundary_refuses_invalid_owner_fields_and_targets() {
    provider_boundary(true);
}

fn provider_boundary(existing: bool) {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install(p);
    std::fs::write(p.join("token.ts"), "export class Token {}").unwrap();
    std::fs::write(
        p.join("foreign.ts"),
        "export function Component(metadata:unknown){return metadata;}",
    )
    .unwrap();
    let declaration = "export function makeAudit(){return {};}";
    std::fs::write(p.join("factory.ts"), declaration).unwrap();
    let source = "import {Component as View} from '@angular/core'; import {Token} from './token'; import {makeAudit as Build} from './factory'; @View({providers:[{provide:Token,useFactory:Build}]}) export class ProviderHost {}";
    let field = if existing {
        "useExisting"
    } else {
        "useFactory"
    };
    let source = source.replace("useFactory", field);
    let source = source.as_str();
    let relation = if existing {
        SemanticRelation::ProviderUsesExisting
    } else {
        SemanticRelation::ProviderUsesFactory
    };
    std::fs::write(p.join("app.ts"), source).unwrap();
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    let capture = || {
        call(
            &state,
            p,
            "compress_code_context",
            json!({"filePath":p.join("app.ts"),"fidelity":"high"}),
        );
        let alias = state
            .alias_for_path(&p.join("app.ts").to_string_lossy())
            .unwrap();
        let snapshot = state.semantic_snapshot(&alias).unwrap();
        snapshot
            .edges
            .iter()
            .filter(|edge| edge.relation == relation)
            .cloned()
            .collect::<Vec<_>>()
    };
    let baseline = capture();
    assert_eq!(baseline.len(), 1, "nonempty declared factory control");
    assert!(baseline[0].subject.canonical.is_some() && baseline[0].object.canonical.is_some());
    for callable in [
        "export const makeAudit=()=>({});",
        "export const makeAudit=function(){return {};};",
    ] {
        std::fs::write(p.join("factory.ts"), callable).unwrap();
        let legacy = capture();
        assert_eq!(
            legacy.len(),
            1,
            "preserve existing callable-variable projection"
        );
        assert_eq!(legacy[0].object.name, "makeAudit");
        if existing {
            assert!(
                legacy[0].subject.canonical.is_some() && legacy[0].object.canonical.is_some(),
                "alias target is the callable object token, without factory execution"
            );
        } else {
            assert!(
                legacy[0].subject.canonical.is_none() && legacy[0].object.canonical.is_none(),
                "callable variables gain no canonical factory authority"
            );
        }
    }
    std::fs::write(p.join("factory.ts"), declaration).unwrap();
    let written = format!("{field}:Build");
    for refused in [
        source.replace("from '@angular/core'", "from './foreign'"),
        source.replace(
            "import {makeAudit as Build}",
            "import type {makeAudit as Build}",
        ),
        source.replace(&written, &format!("{written},{written}")),
        source.replace(&written, &format!("{written},useClass:Token")),
        source.replace(&written, &format!("{field}:Missing")),
        source.replace(&written, &format!("{field}:Build()")),
        source.replace("provide:Token,", ""),
        source.replace(&format!(",{written}"), ""),
    ] {
        std::fs::write(p.join("app.ts"), refused).unwrap();
        assert!(
            capture().is_empty(),
            "invalid factory proof cannot publish a strategy"
        );
        std::fs::write(p.join("app.ts"), source).unwrap();
        let restored = capture();
        assert_eq!(restored.len(), 1, "source repair");
        assert!(restored[0].subject.canonical.is_some() && restored[0].object.canonical.is_some());
    }
    assert_eq!(std::fs::read_to_string(p.join("app.ts")).unwrap(), source);
}

#[test]
fn issue_162_provider_factory_same_named_functions_retain_exact_identity() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install(p);
    for name in ["left.ts", "right.ts"] {
        std::fs::write(p.join(name), "export function makeAudit(){return {};}").unwrap();
    }
    std::fs::write(
        p.join("barrel.ts"),
        "export {makeAudit as build} from './left';",
    )
    .unwrap();
    std::fs::write(p.join("token.ts"), "export class Token {}").unwrap();
    let source = r#"
import {Component as View} from '@angular/core';
import {Token as Audit} from './token';
import {build as Left} from './barrel';
import * as right from './right';
const prose='useFactory:Phantom';
function noise<T>(Left:T){return Left;}
@View({providers:[{provide:Audit,useFactory:Left},
 {provide:Audit,useFactory:right.makeAudit},{provide:Audit,useFactory:Left}]})
export class ProviderHost {}
"#;
    std::fs::write(p.join("app.ts"), source).unwrap();
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    call(
        &state,
        p,
        "workspace_query",
        json!({"type":"entities_in_file","file_path":p.join("app.ts"),"fidelity":"high"}),
    );
    let registration = call(
        &state,
        p,
        "workspace_query",
        json!({"type":"forward_edges","domain":"angular","entity_type":"Component","name":"ProviderHost"}),
    );
    assert_eq!(
        registration["edges"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|edge| edge["relation"] == "RegistersProvider")
            .count(),
        1,
        "nonempty established factory registration control"
    );
    call(
        &state,
        p,
        "compress_code_context",
        json!({"filePath":p.join("app.ts"),"fidelity":"high"}),
    );
    let alias = state
        .alias_for_path(&p.join("app.ts").to_string_lossy())
        .unwrap();
    let snapshot = state.semantic_snapshot(&alias).unwrap();
    let factories: Vec<_> = snapshot
        .edges
        .iter()
        .filter(|edge| edge.relation == SemanticRelation::ProviderUsesFactory)
        .cloned()
        .collect();
    assert_eq!(
        factories.len(),
        3,
        "retain all asserting factory occurrences"
    );
    let actual: std::collections::BTreeSet<_> = factories
        .iter()
        .map(|edge| {
            assert!(
                edge.subject.canonical.is_some(),
                "factory relation must retain canonical token identity"
            );
            let declaration = edge
                .object
                .canonical
                .as_ref()
                .and_then(|endpoint| endpoint.declaration())
                .expect("factory target must retain exact function declaration identity");
            assert_eq!(declaration.name, "makeAudit");
            assert_eq!(declaration.kind, "function_declaration");
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
        "equal function names must retain distinct targets"
    );
    assert!(snapshot.authority.admits_relation_evidence(&factories));
    assert_eq!(std::fs::read_to_string(p.join("app.ts")).unwrap(), source);
}
