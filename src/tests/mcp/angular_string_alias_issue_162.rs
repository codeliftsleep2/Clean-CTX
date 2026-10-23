use serde_json::{Value, json};
use std::path::Path;

fn fixture() -> (tempfile::TempDir, crate::mcp::McpState) {
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install_inject_apis(p);
    let core = p.join("node_modules/@angular/core/index.d.ts");
    let mut api = std::fs::read_to_string(&core).unwrap();
    api.push_str("export declare function Inject(token:any):any; export declare class InjectionToken<T> {constructor(description:string);}");
    std::fs::write(core, api).unwrap();
    std::fs::write(
        p.join("api.ts"),
        "export {Inject as Select} from '@angular/core';",
    )
    .unwrap();
    std::fs::write(p.join("values.ts"), values("audit.api")).unwrap();
    std::fs::write(
        p.join("tokens.ts"),
        "export {ALIAS as Shared, Mutable, CycleA, OBJECT} from './values';",
    )
    .unwrap();
    std::fs::write(
        p.join("foreign.ts"),
        "export function Inject(token:unknown) {return token;}",
    )
    .unwrap();
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    (root, crate::mcp::McpState::new(config))
}

fn values(value: &str) -> String {
    format!(
        r#"
import {{InjectionToken}} from '@angular/core';
export const ROOT = '{value}';
export const ALIAS = ROOT;
export let Mutable = 'unproved.mutable';
export const CycleA = CycleB;
export const CycleB = CycleA;
export const OBJECT = new InjectionToken<unknown>('audit.api');
const unrelated = "export const ALIAS = 'wrong'";
function noise<T>(ROOT:T) {{ return ROOT; }}
"#
    )
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

fn names(
    state: &crate::mcp::McpState,
    root: &Path,
    kind: &str,
    owner: &str,
    relation: &str,
) -> Vec<String> {
    query(
        state,
        root,
        json!({"type":"entities_in_file", "file_path":root.join("app.ts"), "fidelity":"high"}),
    );
    let response = query(
        state,
        root,
        json!({"type":"forward_edges", "domain":"angular", "entity_type":kind, "name":owner}),
    );
    let mut names: Vec<_> = response["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|edge| edge["relation"] == relation)
        .map(|edge| edge["object"]["name"].as_str().unwrap().to_owned())
        .collect();
    names.sort();
    names
}

#[test]
fn issue_162_string_alias_constructor_barrel_local_namespace_and_dependency_mutation() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state) = fixture();
    let p = root.path();
    let control = "import {Injectable, Inject} from '@angular/core'; @Injectable() export class Owner {constructor(@Inject('audit.api') value:unknown){}}";
    std::fs::write(p.join("app.ts"), control).unwrap();
    assert_eq!(
        names(&state, p, "Service", "Owner", "Injects"),
        ["\"audit.api\""]
    );
    let source = r#"
import {Injectable as Service} from '@angular/core';
import {Select} from './api';
import {Shared} from './tokens';
import * as bucket from './tokens';
import {Inject as Foreign} from './foreign';
const local = Shared;
const fake = "@Select('phantom')";
function noise<Select>(Shared:Select) { return Shared; }
@Service() export class Owner {
 constructor(@Select(local) a:unknown, @Select(Shared) b:unknown,
  @Select(bucket.Shared) c:unknown, @Select('audit\u002eapi') direct:unknown) {}
 method(local:unknown) { return local; }
}
@Foreign('unrelated') export class Neighbor {}
"#;
    std::fs::write(p.join("app.ts"), source).unwrap();
    for _ in 0..2 {
        assert_eq!(
            names(&state, p, "Service", "Owner", "Injects"),
            ["\"audit.api\""],
            "immutable strings must join by cooked value across aliases, not their declaration names"
        );
    }
    std::fs::write(p.join("values.ts"), values("other.api")).unwrap();
    assert_eq!(
        names(&state, p, "Service", "Owner", "Injects"),
        ["\"audit.api\"", "\"other.api\""],
        "unchanged consumer must replace the derived value while retaining its direct literal"
    );
    std::fs::write(p.join("values.ts"), values("audit.api")).unwrap();
    assert_eq!(
        names(&state, p, "Service", "Owner", "Injects"),
        ["\"audit.api\""]
    );
    assert_eq!(std::fs::read_to_string(p.join("app.ts")).unwrap(), source);
}

#[test]
fn issue_162_string_alias_provider_joins_value_without_merging_object_description() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state) = fixture();
    let p = root.path();
    let source = r#"
import {Component as View} from '@angular/core';
import {Shared, OBJECT} from './tokens';
import * as bucket from './tokens';
const local = Shared;
const unrelated = {provide:'phantom'};
function noise(Shared:unknown) { return Shared; }
@View({selector:'audit-host', providers:[
 {provide:local,useValue:1}, {provide:bucket.Shared,useValue:2},
 {provide:'audit\u002eapi',useValue:3}, {provide:OBJECT,useValue:4}
]}) export class ProviderHost {}
"#;
    std::fs::write(p.join("app.ts"), source).unwrap();
    assert_eq!(
        names(&state, p, "Component", "ProviderHost", "RegistersProvider"),
        ["\"audit.api\"", "OBJECT"],
        "equal string values converge; the equal-description object remains a distinct token"
    );
    std::fs::write(p.join("values.ts"), values("other.api")).unwrap();
    assert_eq!(
        names(&state, p, "Component", "ProviderHost", "RegistersProvider"),
        ["\"audit.api\"", "\"other.api\"", "OBJECT"]
    );
    std::fs::write(p.join("values.ts"), values("audit.api")).unwrap();
    assert_eq!(
        names(&state, p, "Component", "ProviderHost", "RegistersProvider"),
        ["\"audit.api\"", "OBJECT"]
    );
    assert_eq!(std::fs::read_to_string(p.join("app.ts")).unwrap(), source);
}

#[test]
fn issue_162_string_alias_unsafe_derivations_do_not_gain_string_proof() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state) = fixture();
    let p = root.path();
    let source = r#"
import {Injectable as Service} from '@angular/core';
import {Select} from './api';
import {Mutable, CycleA} from './tokens';
import type {Shared as Erased} from './tokens';
import {Inject as Foreign} from './foreign';
const opaque = (() => 'unproved.call')();
@Service() export class Owner {
 constructor(@Select('safe.control') good:unknown, @Select(Mutable) a:unknown,
 @Select(CycleA) b:unknown, @Select(opaque) c:unknown, @Select(Erased) d:unknown) {}
}
@Service() export class Other {constructor(@Foreign('unproved.foreign') value:unknown) {}}
"#;
    std::fs::write(p.join("app.ts"), source).unwrap();
    let all = names(&state, p, "Service", "Owner", "Injects");
    let strings: Vec<_> = all
        .iter()
        .filter(|name| name.starts_with('"'))
        .cloned()
        .collect();
    assert_eq!(
        strings,
        ["\"safe.control\""],
        "mutable/cyclic/call/type-only references must not become proved string values"
    );
    assert!(
        !names(&state, p, "Service", "Other", "Injects")
            .iter()
            .any(|name| name.starts_with('"'))
    );
    // This control concerns string proof only. It does not certify legacy
    // symbolic/object projections, injector scope, or arbitrary evaluation.
}
