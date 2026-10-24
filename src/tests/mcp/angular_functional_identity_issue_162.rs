use serde_json::{Value, json};
use std::path::Path;

fn query(state: &crate::mcp::McpState, root: &Path, mut args: Value) -> Value {
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
}

fn dependencies(state: &crate::mcp::McpState, root: &Path) -> Vec<Value> {
    query(
        state,
        root,
        json!({"type":"entities_in_file","file_path":root.join("app.ts"),"fidelity":"high"}),
    );
    let result = query(
        state,
        root,
        json!({"type":"forward_edges","domain":"angular","entity_type":"Guard","name":"audit"}),
    );
    result["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|edge| edge["relation"] == "Injects")
        .cloned()
        .collect()
}

#[test]
fn issue_162_functional_identity_same_named_tokens_retain_distinct_declarations() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install(p);
    std::fs::write(
        p.join("node_modules/@angular/core/index.d.ts"),
        "export declare function inject(token:unknown):unknown;",
    )
    .unwrap();
    std::fs::create_dir_all(p.join("node_modules/@angular/router")).unwrap();
    std::fs::write(
        p.join("node_modules/@angular/router/package.json"),
        r#"{"name":"@angular/router","version":"20.0.0","types":"index.d.ts"}"#,
    )
    .unwrap();
    std::fs::write(
        p.join("node_modules/@angular/router/index.d.ts"),
        "export type CanActivateFn = (...args:unknown[])=>unknown;",
    )
    .unwrap();
    for file in ["left.ts", "right.ts"] {
        std::fs::write(p.join(file), "export class AuditApi {}").unwrap();
    }
    std::fs::write(p.join("api.ts"), "export {inject as obtain} from '@angular/core'; export type {CanActivateFn as Gate} from '@angular/router';").unwrap();
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    let control = "import {obtain} from './api'; import type {Gate} from './api'; import {AuditApi} from './left'; export const audit:Gate=()=>obtain(AuditApi);";
    std::fs::write(p.join("app.ts"), control).unwrap();
    let baseline = dependencies(&state, p);
    assert_eq!(baseline.len(), 1, "nonempty migrated functional control");
    assert_eq!(baseline[0]["object"]["name"], "AuditApi");
    let source = r#"
import * as api from './api';
import type {Gate} from './api';
import {AuditApi as Left} from './left';
import {AuditApi as Right} from './right';
const prose = "api.obtain(Phantom)";
function noise<T>(Left:T){return Left;}
export const audit:Gate=()=>{
 api.obtain(Left); api.obtain(Right); api.obtain(Left);
 return true;
};
"#;
    std::fs::write(p.join("app.ts"), source).unwrap();
    for _ in 0..2 {
        let edges = dependencies(&state, p);
        assert_eq!(
            edges.len(),
            2,
            "equal token names must preserve both declarations without repeating the left dependency: {edges:?}"
        );
        let actual: std::collections::BTreeSet<_> = edges
            .iter()
            .map(|edge| {
                assert_eq!(edge["object"]["name"], "AuditApi");
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
        assert_eq!(
            actual, expected,
            "public provenance follows the exact token declaration"
        );
    }
    assert_eq!(std::fs::read_to_string(p.join("app.ts")).unwrap(), source);
}
