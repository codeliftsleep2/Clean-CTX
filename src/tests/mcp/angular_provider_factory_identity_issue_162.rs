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
