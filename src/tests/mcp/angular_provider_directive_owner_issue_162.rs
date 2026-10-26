use serde_json::{Value, json};

#[test]
fn issue_162_provider_directive_owner_preserves_direct_registration() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install_inject_apis(p);
    for file in ["left.ts", "right.ts"] {
        std::fs::write(p.join(file), "export class Token {}").unwrap();
    }
    std::fs::write(
        p.join("api.ts"),
        "export {Directive as Behavior} from '@angular/core';",
    )
    .unwrap();
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    let call = |tool: &str, mut args: Value| {
        args["workspaceRoot"] = json!(p);
        crate::protocol::captured_responses().clear();
        crate::mcp::tools::dispatch_tools_call(
            &json!(162),
            tool,
            &json!({"arguments":args}),
            &state,
        );
        let response = crate::protocol::captured_responses().pop().unwrap();
        assert!(response.get("error").is_none(), "{response}");
        response["result"]["structuredContent"].clone()
    };
    let registrations = |kind: &str| {
        call(
            "compress_code_context",
            json!({"filePath":p.join("app.ts"),"fidelity":"high"}),
        );
        call("workspace_query", json!({"type":"forward_edges","domain":"angular","entity_type":kind,"name":"ProviderHost"}))["edges"]
            .as_array().unwrap().iter().filter(|edge| edge["relation"] == "RegistersProvider").cloned().collect::<Vec<_>>()
    };
    let control = "import {Component} from '@angular/core'; import {Token} from './left'; @Component({providers:[{provide:Token,useValue:1}]}) export class ProviderHost {}";
    std::fs::write(p.join("app.ts"), control).unwrap();
    assert_eq!(
        registrations("Component").len(),
        1,
        "nonempty established Component control"
    );
    for import in [
        "import {Directive as Behavior} from '@angular/core';",
        "import {Behavior} from './api';",
        "import * as api from './api';",
    ] {
        let decorator = if import.contains("* as") {
            "api.Behavior"
        } else {
            "Behavior"
        };
        let source = format!(
            "{import} import {{Token as Left}} from './left'; import * as right from './right'; @{decorator}({{selector:'[probe]',providers:[{{provide:Left,useValue:1}},{{provide:right.Token,useValue:2}},{{provide:Left,useValue:1}}]}}) export class ProviderHost {{}}"
        );
        std::fs::write(p.join("app.ts"), &source).unwrap();
        for _ in 0..2 {
            let found = registrations("Directive");
            let alias = state
                .alias_for_path(&p.join("app.ts").to_string_lossy())
                .unwrap();
            let snapshot = state.semantic_snapshot(&alias).unwrap();
            let authority = snapshot.authority.typescript_authority.as_ref().unwrap();
            let owner = authority
                .declaration_facets
                .iter()
                .find(|facet| facet.declaration.name == "ProviderHost")
                .unwrap();
            assert!(
                matches!(
                    crate::angular_meta::class_role_authority::declaration_role(
                        authority,
                        &owner.declaration
                    ),
                    Some(crate::angular_meta::decorators::ClassKind::Directive)
                ),
                "native exact Directive owner must precede the behavioral assertion"
            );
            assert_eq!(
                found.len(),
                2,
                "proved Directive directly registers both token identities: {found:?}"
            );
            let actual: std::collections::BTreeSet<_> = found
                .iter()
                .map(|edge| {
                    assert_eq!(edge["subject"]["entity_type"], "Directive");
                    assert_eq!(edge["object"]["name"], "Token");
                    crate::dictionary::path::canonical_identity_key(
                        edge["object"]["file"].as_str().unwrap(),
                    )
                })
                .collect();
            let expected = ["left.ts", "right.ts"]
                .into_iter()
                .map(|file| {
                    crate::dictionary::path::canonical_identity_key(&p.join(file).to_string_lossy())
                })
                .collect();
            assert_eq!(actual, expected);
        }
        assert_eq!(std::fs::read_to_string(p.join("app.ts")).unwrap(), source);
    }
}
