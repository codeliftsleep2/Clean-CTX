use serde_json::{Value, json};

fn composition_question(form: &str, view: bool) {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install(p);
    for file in ["left.ts", "right.ts"] {
        std::fs::write(p.join(file), "export class Token {}").unwrap();
    }
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
    let entries =
        "{provide:Left,useValue:1},{provide:right.Token,useValue:2},{provide:Left,useValue:3}";
    let field = if view { "viewProviders" } else { "providers" };
    let relation = if view {
        "RegistersViewProvider"
    } else {
        "RegistersProvider"
    };
    for control in [true, false] {
        let (binding, expression) = if control {
            (String::new(), format!("[{entries}]"))
        } else {
            match form {
                "reference" => (format!("const GROUP=[{entries}];"), "GROUP".to_owned()),
                "spread" => (format!("const GROUP=[{entries}];"), "[...GROUP]".to_owned()),
                "nested" => (String::new(), format!("[[{entries}]]")),
                _ => panic!("unknown composition fixture"),
            }
        };
        let source = format!(
            "import {{Component as View}} from '@angular/core'; import {{Token as Left}} from './left'; import * as right from './right'; {binding} @View({{{field}:{expression}}}) export class Host {{}}"
        );
        std::fs::write(p.join("app.ts"), &source).unwrap();
        for _ in 0..2 {
            call(
                "compress_code_context",
                json!({"filePath":p.join("app.ts"),"fidelity":"high"}),
            );
            let snapshot = state
                .semantic_snapshot(
                    &state
                        .alias_for_path(&p.join("app.ts").to_string_lossy())
                        .unwrap(),
                )
                .unwrap();
            let authority = snapshot.authority.typescript_authority.as_ref().unwrap();
            let owner = authority
                .declaration_facets
                .iter()
                .find(|facet| facet.declaration.name == "Host")
                .unwrap();
            assert!(
                matches!(
                    crate::angular_meta::class_role_authority::declaration_role(
                        authority,
                        &owner.declaration
                    ),
                    Some(crate::angular_meta::decorators::ClassKind::Component)
                ),
                "exact native Component prerequisite"
            );
            let retained_shapes: Vec<_> = owner.decorators.iter().flat_map(|decorator| &decorator.arguments)
                .filter_map(|argument| match &argument.value {
                    crate::ir::typescript_resolution::declaration_facets::ExpressionValue::Object(fields) => Some(fields), _ => None,
                }).flatten().map(|(name,value)| (name,format!("{:?}",value.value))).collect();
            let initializers: Vec<_> = authority
                .declaration_facets
                .iter()
                .filter(|facet| facet.initializer.is_some())
                .map(|facet| facet.declaration.name.as_str())
                .collect();
            let response = call(
                "workspace_query",
                json!({"type":"forward_edges","domain":"angular","entity_type":"Component","name":"Host"}),
            );
            let found: Vec<_> = response["edges"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|edge| edge["relation"] == relation)
                .collect();
            assert_eq!(
                found.len(),
                2,
                "source-declared composition preserves exact token registrations; control={control}, form={form}, field={field}, retained={retained_shapes:?}, initializer declarations={initializers:?}, public={found:?}"
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
                .map(|file| {
                    crate::dictionary::path::canonical_identity_key(&p.join(file).to_string_lossy())
                })
                .collect();
            assert_eq!(actual, expected);
        }
        assert_eq!(std::fs::read_to_string(p.join("app.ts")).unwrap(), source);
    }
}

#[test]
fn issue_162_provider_composition_reference_ordinary() {
    composition_question("reference", false);
}
#[test]
fn issue_162_provider_composition_reference_view() {
    composition_question("reference", true);
}
#[test]
fn issue_162_provider_composition_spread_ordinary() {
    composition_question("spread", false);
}
#[test]
fn issue_162_provider_composition_spread_view() {
    composition_question("spread", true);
}
#[test]
fn issue_162_provider_composition_nested_ordinary() {
    composition_question("nested", false);
}
#[test]
fn issue_162_provider_composition_nested_view() {
    composition_question("nested", true);
}
