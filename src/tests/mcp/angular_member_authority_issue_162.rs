use serde_json::{Value, json};
use std::path::Path;

const APIS: &str = r#"
export declare function Component(metadata:unknown):any;
export declare const input: {<T>(value?:T):unknown; required:<T>()=>unknown;};
export declare function output<T>():unknown;
export declare const model: {<T>(value:T):unknown; required:<T>()=>unknown;};
"#;

fn fixture() -> (tempfile::TempDir, crate::mcp::McpState) {
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install(p);
    // Selected Angular 20 callable shapes; role comes from package/export
    // evidence, not these signatures or an external compiler oracle.
    std::fs::write(p.join("node_modules/@angular/core/index.d.ts"), APIS).unwrap();
    std::fs::write(p.join("foreign.ts"), APIS).unwrap();
    std::fs::write(
        p.join("api.ts"),
        "export {input as read, output as emit, model as bind} from '@angular/core';",
    )
    .unwrap();
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
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

fn members(state: &crate::mcp::McpState, root: &Path) -> Vec<(String, String)> {
    query(
        state,
        root,
        json!({"type":"entities_in_file","file_path":root.join("app.ts"),"fidelity":"high"}),
    );
    let result = query(
        state,
        root,
        json!({"type":"forward_edges","domain":"angular","entity_type":"Component","name":"Panel"}),
    );
    let mut actual: Vec<_> = result["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|edge| matches!(edge["relation"].as_str(), Some("HasInput" | "HasOutput")))
        .map(|edge| {
            (
                edge["relation"].as_str().unwrap().to_owned(),
                edge["object"]["name"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    actual.sort();
    actual
}

fn expected() -> Vec<(String, String)> {
    [
        ("HasInput", "requiredTitle"),
        ("HasInput", "selected"),
        ("HasInput", "title"),
        ("HasOutput", "changed"),
        ("HasOutput", "selected"),
    ]
    .into_iter()
    .map(|(relation, name)| (relation.to_owned(), name.to_owned()))
    .collect()
}

fn control() -> &'static str {
    "import {Component,input,output,model} from '@angular/core'; @Component({}) export class Panel {title=input(''); requiredTitle=input.required<string>(); changed=output<string>(); selected=model(false);}"
}

#[test]
fn issue_162_member_authority_barrel_namespace_aliases_amid_noise() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state) = fixture();
    let p = root.path();
    std::fs::write(p.join("app.ts"), control()).unwrap();
    assert_eq!(
        members(&state, p),
        expected(),
        "nonempty established member control"
    );
    let source = r#"
import {Component} from '@angular/core';
import {read as localInput, emit as localOutput} from './api';
import * as memberApi from './api';
import {input as foreignInput} from './foreign';
const prose = 'title=input(); @Component({})';
const object = {input:()=>0};
function wrap(value:unknown) {return value;}
function shadow(localInput:()=>unknown) {return localInput();}
@Component({selector:'audit-panel'}) export class Panel {
 title=localInput('');
 requiredTitle=memberApi.read.required<string>();
 changed=localOutput<string>();
 selected=memberApi.bind(false);
 foreign=foreignInput('');
 ordinary=object.input();
 nested=wrap(localInput(''));
 method(localOutput:()=>unknown) {return localOutput();}
}
"#;
    std::fs::write(p.join("app.ts"), source).unwrap();
    for _ in 0..2 {
        assert_eq!(
            members(&state, p),
            expected(),
            "actual member APIs must survive aliases while foreign/ordinary/nested sites stay out"
        );
    }
    assert_eq!(std::fs::read_to_string(p.join("app.ts")).unwrap(), source);
}

#[test]
fn issue_162_member_authority_package_spelling_cannot_restore_foreign_owner_or_api() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state) = fixture();
    let p = root.path();
    let source = control();
    std::fs::write(p.join("app.ts"), source).unwrap();
    assert_eq!(members(&state, p), expected());
    let original = std::fs::read(p.join("tsconfig.json")).unwrap();
    std::fs::write(
        p.join("tsconfig.json"),
        json!({"compilerOptions":{
            "module":"esnext","moduleResolution":"bundler","moduleDetection":"legacy",
            "noLib":true,"types":[],"paths":{"@angular/core":["./foreign.ts"]}
        }})
        .to_string(),
    )
    .unwrap();
    assert!(
        members(&state, p).is_empty(),
        "member extraction must not reintroduce Angular owner/API roles from package spelling"
    );
    std::fs::write(p.join("tsconfig.json"), original).unwrap();
    assert_eq!(
        members(&state, p),
        expected(),
        "repair must restore actual member facts"
    );
    assert_eq!(std::fs::read_to_string(p.join("app.ts")).unwrap(), source);
}
