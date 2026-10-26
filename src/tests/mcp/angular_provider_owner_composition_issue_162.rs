use serde_json::{Value, json};

fn owner_composition_question(form: &str, owner_api: &str) {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install(p);
    std::fs::write(p.join("node_modules/@angular/core/index.d.ts"),
        "export declare function Component(metadata:unknown):any; export declare function Directive(metadata:unknown):any; export declare function NgModule(metadata:unknown):any;").unwrap();
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
    use crate::angular_meta::decorators::ClassKind;
    let owner_kind = match owner_api {
        "Directive" => ClassKind::Directive,
        "NgModule" => ClassKind::Module,
        _ => panic!("unknown provider owner"),
    };
    std::fs::write(
        p.join("group.ts"),
        format!("import {{Token as Left}} from './left'; import * as right from './right'; export const GROUP=[{entries}];"),
    ).unwrap();
    std::fs::write(
        p.join("barrel.ts"),
        "export {GROUP as Published} from './group';",
    )
    .unwrap();
    for (stage, api, kind, composed) in [
        ("direct-owner", owner_api, owner_kind, false),
        (
            "composed-component",
            "Component",
            ClassKind::Component,
            true,
        ),
        ("composed-owner", owner_api, owner_kind, true),
    ] {
        let (binding, expression) = if !composed {
            (String::new(), format!("[{entries}]"))
        } else {
            match form {
                "nested" => (String::new(), format!("[[{entries}]]")),
                "reference" => (format!("const GROUP=[{entries}];"), "GROUP".to_owned()),
                "spread" => (format!("const GROUP=[{entries}];"), "[...GROUP]".to_owned()),
                "import-alias" => (
                    "import {GROUP as Imported} from './group';".to_owned(),
                    "Imported".to_owned(),
                ),
                "import-barrel" => (
                    "import {Published as Imported} from './barrel';".to_owned(),
                    "[...Imported]".to_owned(),
                ),
                "import-namespace" => (
                    "import * as groups from './barrel';".to_owned(),
                    "[...[groups.Published]]".to_owned(),
                ),
                _ => panic!("unknown owner composition form"),
            }
        };
        let entity_type = match kind {
            ClassKind::Directive => "Directive",
            ClassKind::Module => "Module",
            ClassKind::Component => "Component",
            _ => unreachable!(),
        };
        let selector = if api == "Directive" {
            "selector:'[probe]',"
        } else {
            ""
        };
        let source = format!(
            "import {{{api} as View}} from '@angular/core'; import {{Token as Left}} from './left'; import * as right from './right'; {binding} @View({{{selector}providers:{expression}}}) export class Host {{}}"
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
            assert_eq!(
                crate::angular_meta::class_role_authority::declaration_role(
                    authority,
                    &owner.declaration
                ),
                Some(kind),
                "exact native {api} prerequisite for {stage}/{form}"
            );
            let retained_shapes: Vec<_> = owner.decorators.iter().flat_map(|decorator| &decorator.arguments)
                .filter_map(|argument| match &argument.value {
                    crate::ir::typescript_resolution::declaration_facets::ExpressionValue::Object(fields) => Some(fields), _ => None,
                }).flatten().map(|(name,value)| (name,format!("{:?}",value.value))).collect();
            let initializers: Vec<_> = authority
                .declaration_facets
                .iter()
                .filter(|facet| facet.initializer.is_some())
                .map(|facet| (&facet.declaration, &facet.initializer))
                .collect();
            let response = call(
                "workspace_query",
                json!({"type":"forward_edges","domain":"angular","entity_type":entity_type,"name":"Host"}),
            );
            let found: Vec<_> = response["edges"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|edge| edge["relation"] == "RegistersProvider")
                .collect();
            assert_eq!(
                found.len(),
                2,
                "source-declared owner composition preserves exact token registrations; stage={stage}, owner={api}, form={form}, retained={retained_shapes:?}, initializer declarations={initializers:?}, public={found:?}"
            );
            for edge in &found {
                assert_eq!(edge["subject"]["name"], "Host");
                assert_eq!(edge["subject"]["entity_type"], entity_type);
                assert_eq!(
                    crate::dictionary::path::canonical_identity_key(
                        edge["subject"]["file"].as_str().unwrap()
                    ),
                    crate::dictionary::path::canonical_identity_key(
                        &p.join("app.ts").to_string_lossy()
                    ),
                    "asserting owner remains app.ts, not the contributor",
                );
            }
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
fn issue_162_provider_owner_composition_directive_nested() {
    owner_composition_question("nested", "Directive");
}

#[test]
fn issue_162_provider_owner_composition_directive_reference() {
    owner_composition_question("reference", "Directive");
}

#[test]
fn issue_162_provider_owner_composition_directive_spread() {
    owner_composition_question("spread", "Directive");
}

#[test]
fn issue_162_provider_owner_composition_directive_import_alias() {
    owner_composition_question("import-alias", "Directive");
}

#[test]
fn issue_162_provider_owner_composition_directive_import_barrel() {
    owner_composition_question("import-barrel", "Directive");
}

#[test]
fn issue_162_provider_owner_composition_directive_import_namespace() {
    owner_composition_question("import-namespace", "Directive");
}

#[test]
fn issue_162_provider_owner_composition_ngmodule_nested() {
    owner_composition_question("nested", "NgModule");
}

#[test]
fn issue_162_provider_owner_composition_ngmodule_reference() {
    owner_composition_question("reference", "NgModule");
}

#[test]
fn issue_162_provider_owner_composition_ngmodule_spread() {
    owner_composition_question("spread", "NgModule");
}

#[test]
fn issue_162_provider_owner_composition_ngmodule_import_alias() {
    owner_composition_question("import-alias", "NgModule");
}

#[test]
fn issue_162_provider_owner_composition_ngmodule_import_barrel() {
    owner_composition_question("import-barrel", "NgModule");
}

#[test]
fn issue_162_provider_owner_composition_ngmodule_import_namespace() {
    owner_composition_question("import-namespace", "NgModule");
}
