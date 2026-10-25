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

fn registrations(state: &crate::mcp::McpState, root: &Path) -> Vec<Value> {
    call(
        state,
        root,
        "workspace_query",
        json!({"type":"entities_in_file","file_path":root.join("app.ts"),"fidelity":"high"}),
    );
    call(state, root, "workspace_query", json!({"type":"forward_edges","domain":"angular","entity_type":"Component","name":"ProviderHost"}))["edges"]
        .as_array().unwrap().iter().filter(|edge| edge["relation"] == "RegistersProvider").cloned().collect()
}

#[test]
fn issue_162_provider_value_namespace_preserves_distinct_registration_tokens() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install(p);
    for name in ["left.ts", "right.ts"] {
        std::fs::write(p.join(name), "export class Token {} export class Value {}").unwrap();
    }
    std::fs::write(p.join("barrel.ts"), "export {Token,Value} from './right';").unwrap();
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    let control = "import {Component} from '@angular/core'; import {Token,Value} from './left'; class OtherToken {} @Component({providers:[{provide:Token,useValue:Value},{provide:OtherToken,useValue:Value}]}) export class ProviderHost {}";
    std::fs::write(p.join("app.ts"), control).unwrap();
    assert_eq!(
        registrations(&state, p).len(),
        2,
        "nonempty established reference-valued provider control"
    );
    let source = r#"
import {Component as View} from '@angular/core';
import {Token as Left,Value as LeftValue} from './left';
import * as right from './barrel';
const prose='useValue:Phantom';
function noise<T>(LeftValue:T){return LeftValue;}
@View({providers:[{provide:Left,useValue:LeftValue},
 {provide:right.Token,useValue:right.Value},{provide:Left,useValue:LeftValue}]})
export class ProviderHost {}
"#;
    std::fs::write(p.join("app.ts"), source).unwrap();
    for _ in 0..2 {
        let found = registrations(&state, p);
        assert_eq!(
            found.len(),
            2,
            "namespace value reference preserves its own registration: {found:?}"
        );
        let actual: std::collections::BTreeSet<_> = found
            .iter()
            .map(|edge| {
                assert_eq!(edge["object"]["name"], "Token");
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
