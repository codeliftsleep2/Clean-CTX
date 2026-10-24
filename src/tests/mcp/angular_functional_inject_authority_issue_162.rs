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

fn dependencies(state: &crate::mcp::McpState, root: &Path) -> Vec<String> {
    query(
        state,
        root,
        json!({"type":"entities_in_file","file_path":root.join("app.ts"),"fidelity":"high"}),
    );
    let response = query(
        state,
        root,
        json!({"type":"forward_edges","domain":"angular","entity_type":"Guard","name":"audit"}),
    );
    let mut tokens: Vec<_> = response["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|edge| edge["relation"] == "Injects")
        .map(|edge| edge["object"]["name"].as_str().unwrap().to_owned())
        .collect();
    tokens.sort();
    tokens
}

#[test]
fn issue_162_functional_inject_authority_aliases_and_namespace_amid_noise() {
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
    std::fs::write(
        p.join("tokens.ts"),
        "export class AuditApi {} export class OtherApi {} export class RefusedApi {}",
    )
    .unwrap();
    std::fs::write(p.join("api.ts"), "export {inject as obtain} from '@angular/core'; export type {CanActivateFn as Gate} from '@angular/router';").unwrap();
    std::fs::write(
        p.join("foreign.ts"),
        "export function inject(token:unknown){return token;}",
    )
    .unwrap();
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    std::fs::write(p.join("app.ts"), "import {inject} from '@angular/core'; import {CanActivateFn} from '@angular/router'; import {AuditApi} from './tokens'; export const audit:CanActivateFn=()=>{inject(AuditApi); return true;};").unwrap();
    assert_eq!(
        dependencies(&state, p),
        vec!["AuditApi"],
        "nonempty established functional guard control"
    );
    let source = r#"
import {obtain as local} from './api';
import * as api from './api';
import type {Gate} from './api';
import {AuditApi, OtherApi, RefusedApi} from './tokens';
import {inject as foreignInject} from './foreign';
const misleading = "const audit:Gate=()=>local(OtherApi)";
const ordinary = {obtain:()=>0};
function noise<T>(local:T) { return local; }
export const audit:Gate=()=>{
 local(AuditApi); api.obtain(OtherApi);
 foreignInject(RefusedApi); ordinary.obtain();
 return true;
};
function shadow(local:(token:unknown)=>unknown){return local(RefusedApi);}
"#;
    std::fs::write(p.join("app.ts"), source).unwrap();
    for _ in 0..2 {
        assert_eq!(
            dependencies(&state, p),
            vec!["AuditApi", "OtherApi"],
            "functional role and inject aliases require exact authority"
        );
    }
    assert_eq!(std::fs::read_to_string(p.join("app.ts")).unwrap(), source);
}
