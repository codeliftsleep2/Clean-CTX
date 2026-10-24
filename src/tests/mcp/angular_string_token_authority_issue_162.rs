use serde_json::{Value, json};
use std::path::Path;

fn fixture() -> (tempfile::TempDir, crate::mcp::McpState) {
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install(p);
    // Selected Angular 20 API shapes: Inject accepts any token, while inject
    // uses ProviderToken. String support is not generalized across APIs.
    std::fs::write(
        p.join("node_modules/@angular/core/index.d.ts"),
        r#"
export declare function Injectable(metadata?: unknown):any;
export declare function Component(metadata:unknown):any;
export declare function Inject(token:any):any;
export declare function Optional():any;
export interface Type<T> { new (...args:any[]):T; }
export declare class InjectionToken<T> { constructor(description:string); }
export type ProviderToken<T> = Type<T> | InjectionToken<T>;
export declare function inject<T>(token:ProviderToken<T>):T;
"#,
    )
    .unwrap();
    std::fs::write(p.join("foreign.ts"), "export function Inject(token:unknown){return token;} export function inject(token:unknown){return token;}").unwrap();
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    (root, crate::mcp::McpState::new(config))
}

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

fn relations(
    state: &crate::mcp::McpState,
    root: &Path,
    kind: &str,
    owner: &str,
    relation: &str,
) -> Vec<Value> {
    query(
        state,
        root,
        json!({"type":"entities_in_file","file_path":root.join("app.ts"),"fidelity":"high"}),
    );
    query(
        state,
        root,
        json!({"type":"forward_edges","domain":"angular","entity_type":kind,"name":owner}),
    )["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|edge| edge["relation"] == relation)
        .cloned()
        .collect()
}

fn names(edges: &[Value]) -> Vec<String> {
    let mut names: Vec<_> = edges
        .iter()
        .map(|edge| edge["object"]["name"].as_str().unwrap().to_owned())
        .collect();
    names.sort();
    names
}

#[test]
fn issue_162_string_token_constructor_mixed_values_and_declaration_identity() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state) = fixture();
    let p = root.path();
    let control = "import {Injectable} from '@angular/core'; class AuditApi {} @Injectable() export class Owner {constructor(dep:AuditApi) {}}";
    std::fs::write(p.join("app.ts"), control).unwrap();
    assert_eq!(
        names(&relations(&state, p, "Service", "Owner", "Injects")),
        vec!["AuditApi".to_owned()]
    );
    let source = r#"
import {Injectable as Service, Inject as Select, Optional as Maybe, inject as obtain} from '@angular/core';
import * as core from '@angular/core';
import {inject as impostor} from './foreign';
import {Absent} from './unrelated-missing';
class AuditApi {}
const noise = "@Inject('phantom') constructor(fake: Phantom)";
function unrelated<T>(value:T) {return value;}
@Service() export class Owner {
    constructor(@Maybe() @Select('audit.api') first:unknown,
        @core.Inject("audit\u002eapi") equivalent:unknown,
        @Select('other.api') distinct:unknown,
        @Select('AuditApi') stringNamesake:unknown,
        plain:AuditApi) {}
    foreign = impostor('phantom');
    unsupportedApi = obtain('field-only');
    method(Select:(token:unknown)=>unknown) {return Select('phantom');}
}
"#;
    std::fs::write(p.join("app.ts"), source).unwrap();
    for _ in 0..2 {
        let edges = relations(&state, p, "Service", "Owner", "Injects");
        assert_eq!(
            edges.len(),
            4,
            "equivalent strings deduplicate, distinct strings and a declaration remain distinct: {edges:?}"
        );
        let mut expected = ["\"audit.api\"", "\"other.api\"", "\"AuditApi\"", "AuditApi"]
            .map(str::to_owned)
            .to_vec();
        expected.sort();
        assert_eq!(names(&edges), expected);
        assert!(
            edges
                .iter()
                .all(|edge| edge["object"].get("canonical").is_none())
        );
    }
    assert_eq!(std::fs::read_to_string(p.join("app.ts")).unwrap(), source);
}

#[test]
fn issue_162_string_token_provider_and_constructor_share_value_label() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state) = fixture();
    let source = r#"
import {Component, Injectable as Service, Inject as Select} from '@angular/core';
const noise = "provide:'phantom'";
@Component({selector:'x-host',providers:[{provide:"audit\u002eapi",useValue:'actual-value'}]})
export class ProviderHost {}
@Service() export class Owner {constructor(@Select('audit.api') dep:unknown) {}}
"#;
    std::fs::write(root.path().join("app.ts"), source).unwrap();
    let providers = relations(
        &state,
        root.path(),
        "Component",
        "ProviderHost",
        "RegistersProvider",
    );
    assert_eq!(
        names(&providers),
        vec!["\"audit.api\"".to_owned()],
        "provider must render the proved string value, not a raw fragment"
    );
    let consumers = relations(&state, root.path(), "Service", "Owner", "Injects");
    assert_eq!(
        names(&consumers),
        names(&providers),
        "same token value label; no provider-satisfaction assertion"
    );
}

#[test]
fn issue_162_string_token_injection_token_descriptions_do_not_merge_objects() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state) = fixture();
    std::fs::write(root.path().join("tokens.ts"), "import {InjectionToken} from '@angular/core'; export const FIRST=new InjectionToken<unknown>('audit.api'); export const SECOND=new InjectionToken<unknown>('audit.api');").unwrap();
    std::fs::write(
        root.path().join("barrel.ts"),
        "export {FIRST as Left, SECOND as Right} from './tokens';",
    )
    .unwrap();
    std::fs::write(root.path().join("app.ts"), "import {Injectable as Service, Inject as Select} from '@angular/core'; import {Left as A, Right as B} from './barrel'; const noise='audit.api'; @Service() export class Owner {constructor(@Select(A) first:unknown, @Select(B) second:unknown) {}}").unwrap();
    let edges = relations(&state, root.path(), "Service", "Owner", "Injects");
    assert_eq!(
        names(&edges),
        vec!["FIRST".to_owned(), "SECOND".to_owned()],
        "equal descriptions are not object-token identity"
    );
}

#[test]
fn issue_162_string_token_api_guard_does_not_expand_field_inject() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state) = fixture();
    std::fs::write(root.path().join("app.ts"), "import {Injectable as Service, inject as obtain} from '@angular/core'; class AuditApi {} @Service() export class Owner {good=obtain(AuditApi); unsupported=obtain('audit.api'); note='inject(phantom)';}").unwrap();
    let edges = relations(&state, root.path(), "Service", "Owner", "Injects");
    assert_eq!(
        names(&edges),
        vec!["AuditApi".to_owned()],
        "a string-capable internal carrier must not broaden inject's admitted API forms"
    );
}
