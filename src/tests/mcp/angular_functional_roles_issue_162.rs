use serde_json::{Value, json};
use std::path::Path;

fn fixture() -> (tempfile::TempDir, crate::mcp::McpState) {
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
    std::fs::write(p.join("node_modules/@angular/router/index.d.ts"),"export type CanActivateFn=(...args:unknown[])=>unknown; export type CanMatchFn=(...args:unknown[])=>unknown; export type ResolveFn<T>=(...args:unknown[])=>T;").unwrap();
    std::fs::create_dir_all(p.join("node_modules/@angular/common/http")).unwrap();
    std::fs::write(p.join("node_modules/@angular/common/package.json"),r#"{"name":"@angular/common","version":"20.0.0","types":"index.d.ts","exports":{".":{"types":"./index.d.ts"},"./http":{"types":"./http/index.d.ts"}}}"#).unwrap();
    std::fs::write(
        p.join("node_modules/@angular/common/index.d.ts"),
        "export interface UnrelatedRootApi {}",
    )
    .unwrap();
    std::fs::write(
        p.join("node_modules/@angular/common/http/index.d.ts"),
        "export type HttpInterceptorFn=(...args:unknown[])=>unknown;",
    )
    .unwrap();
    std::fs::write(
        p.join("tokens.ts"),
        "export class AuditApi {} export class RefusedApi {}",
    )
    .unwrap();
    std::fs::write(p.join("api.ts"),"export {inject as obtain} from '@angular/core'; export type {CanMatchFn as Match, ResolveFn as Resolution} from '@angular/router'; export type {HttpInterceptorFn as Intercept} from '@angular/common/http';").unwrap();
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    (root, crate::mcp::McpState::new(config))
}

fn edges(state: &crate::mcp::McpState, root: &Path, kind: &str, name: &str) -> Vec<Value> {
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
    query(json!({"type":"forward_edges","domain":"angular","entity_type":kind,"name":name}))["edges"]
        .as_array().unwrap().iter().filter(|edge| edge["relation"] == "Injects").cloned().collect()
}

fn control(state: &crate::mcp::McpState, root: &Path) {
    std::fs::write(root.join("app.ts"),"import {inject} from '@angular/core'; import type {CanActivateFn} from '@angular/router'; import {AuditApi} from './tokens'; export const control:CanActivateFn=()=>inject(AuditApi);").unwrap();
    let found = edges(state, root, "Guard", "control");
    assert_eq!(found.len(), 1, "nonempty migrated role/API/token control");
    assert_eq!(found[0]["object"]["name"], "AuditApi");
}

fn exact_token(found: &[Value], root: &Path) {
    assert_eq!(found.len(), 1, "exact functional dependency: {found:?}");
    assert_eq!(found[0]["object"]["name"], "AuditApi");
    assert_eq!(
        crate::dictionary::path::canonical_identity_key(
            found[0]["object"]["file"].as_str().unwrap()
        ),
        crate::dictionary::path::canonical_identity_key(&root.join("tokens.ts").to_string_lossy())
    );
}

#[test]
fn issue_162_functional_roles_match_alias_rejects_inner_role_decoy() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state) = fixture();
    let p = root.path();
    control(&state, p);
    let source = r#"
import * as api from './api'; import type {Match} from './api';
import {AuditApi,RefusedApi} from './tokens';
const misleading="const loose:Match=()=>api.obtain(RefusedApi)";
export const match:Match=()=>api.obtain(AuditApi);
export const loose=()=>{const decoy:Match=()=>true;api.obtain(RefusedApi);return decoy;};
"#;
    std::fs::write(p.join("app.ts"), source).unwrap();
    exact_token(&edges(&state, p, "Guard", "match"), p);
    assert!(
        edges(&state, p, "Guard", "loose").is_empty(),
        "an inner annotation cannot grant the enclosing owner a role"
    );
}

#[test]
fn issue_162_functional_roles_generic_resolver_alias() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state) = fixture();
    let p = root.path();
    control(&state, p);
    let source = "import {obtain} from './api'; import type {Resolution} from './api'; import {AuditApi} from './tokens'; const prose='Resolution<unknown>'; export const resolve:Resolution<unknown>=()=>obtain(AuditApi);";
    std::fs::write(p.join("app.ts"), source).unwrap();
    for _ in 0..2 {
        exact_token(&edges(&state, p, "Resolver", "resolve"), p);
    }
    assert_eq!(std::fs::read_to_string(p.join("app.ts")).unwrap(), source);
}

#[test]
fn issue_162_functional_roles_http_subpath_keeps_owning_package_boundary() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state) = fixture();
    let p = root.path();
    control(&state, p);
    let source = "import * as api from './api'; import type {Intercept} from './api'; import {AuditApi} from './tokens'; const prose='@angular/common/http'; export const intercept:Intercept=()=>api.obtain(AuditApi);";
    std::fs::write(p.join("app.ts"), source).unwrap();
    for _ in 0..2 {
        exact_token(&edges(&state, p, "Interceptor", "intercept"), p);
    }
    assert_eq!(std::fs::read_to_string(p.join("app.ts")).unwrap(), source);
}
