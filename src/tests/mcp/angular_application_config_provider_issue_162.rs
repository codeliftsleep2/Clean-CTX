use serde_json::{Value, json};

fn application_config_question(form: &str) {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install(p);
    std::fs::write(p.join("node_modules/@angular/core/index.d.ts"),
        "export declare function Component(metadata:unknown):any; export interface ApplicationConfig {providers:unknown[];}").unwrap();
    for file in ["left.ts", "right.ts"] {
        std::fs::write(p.join(file), "export class Token {}").unwrap();
    }
    let entries = "{provide:Left,useValue:1},{provide:Right,useValue:2},{provide:Left,useValue:3}";
    std::fs::write(p.join("group.ts"),format!(
        "import {{Token as Left}} from './left'; import {{Token as Right}} from './right'; export const GROUP=[{entries}];"
    )).unwrap();
    std::fs::write(p.join("host.ts"),format!(
        "import {{Component}} from '@angular/core'; import {{Token as Left}} from './left'; import {{Token as Right}} from './right'; @Component({{providers:[{entries}]}}) export class Host {{}}"
    )).unwrap();
    std::fs::write(
        p.join("barrel.ts"),
        "export {GROUP as Published} from './group';",
    )
    .unwrap();
    let (extra, expression) = match form {
        "direct" => (String::new(), format!("[{entries}]")),
        "local" => (
            format!("const LOCAL=[{entries}]; const ALIAS=LOCAL;"),
            "[...[ALIAS]]".into(),
        ),
        "barrel" => (
            "import {Published as Imported} from './barrel';".into(),
            "[...Imported]".into(),
        ),
        "namespace" => (
            "import * as groups from './barrel';".into(),
            "[...[groups.Published]]".into(),
        ),
        _ => panic!("unknown configuration form"),
    };
    let source = format!(
        "import type {{ApplicationConfig as Config}} from '@angular/core'; import {{Token as Left}} from './left'; import {{Token as Right}} from './right'; {extra} export const appConfig:Config={{providers:{expression}}};"
    );
    std::fs::write(p.join("config.ts"), &source).unwrap();
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
        assert_ne!(response["result"]["isError"], true, "{response}");
        response["result"]["structuredContent"].clone()
    };
    call(
        "compress_code_context",
        json!({"filePath":p.join("host.ts"),"fidelity":"high"}),
    );
    let snapshot = state
        .semantic_snapshot(
            &state
                .alias_for_path(&p.join("host.ts").to_string_lossy())
                .unwrap(),
        )
        .unwrap();
    let authority = snapshot.authority.typescript_authority.as_ref().unwrap();
    let host = authority
        .declaration_facets
        .iter()
        .find(|f| f.declaration.name == "Host")
        .unwrap();
    assert_eq!(
        crate::angular_meta::class_role_authority::declaration_role(authority, &host.declaration),
        Some(crate::angular_meta::decorators::ClassKind::Component)
    );
    let control = call(
        "workspace_query",
        json!({"type":"forward_edges","domain":"angular","entity_type":"Component","name":"Host"}),
    );
    let registered: Vec<_> = control["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["relation"] == "RegistersProvider")
        .collect();
    assert_eq!(
        registered.len(),
        2,
        "nonempty established Component registration prerequisite: {control}"
    );
    let identity = |value: &str| crate::dictionary::path::canonical_identity_key(value);
    let expected: std::collections::BTreeSet<_> = ["left.ts", "right.ts"]
        .into_iter()
        .map(|file| identity(&p.join(file).to_string_lossy()))
        .collect();
    let token_files = |edges: &[&Value]| {
        edges
            .iter()
            .map(|edge| {
                assert_eq!(edge["object"]["name"], "Token");
                identity(edge["object"]["file"].as_str().unwrap())
            })
            .collect::<std::collections::BTreeSet<_>>()
    };
    assert_eq!(token_files(&registered), expected);
    for _ in 0..2 {
        call(
            "compress_code_context",
            json!({"filePath":p.join("config.ts"),"fidelity":"high"}),
        );
        let snapshot = state
            .semantic_snapshot(
                &state
                    .alias_for_path(&p.join("config.ts").to_string_lossy())
                    .unwrap(),
            )
            .unwrap();
        let authority = snapshot.authority.typescript_authority.as_ref().unwrap();
        let anchors: Vec<_> = authority
            .type_reference_sites
            .iter()
            .filter(|(_, _, resolution)| {
                let crate::ir::typescript_resolution::Resolution::Resolved(id) = resolution else {
                    return false;
                };
                crate::angular_meta::class_role_authority::public_entry_api(
                    authority,
                    id,
                    "@angular/core",
                    "@angular/core",
                    "ApplicationConfig",
                    crate::ir::typescript_resolution::Namespace::Type,
                )
            })
            .collect();
        assert_eq!(
            anchors.len(),
            1,
            "exact native ApplicationConfig type prerequisite; form={form}"
        );
        let owner = authority
            .declarations
            .iter()
            .find(|id| {
                id.name == "appConfig"
                    && id.kind == "variable_declarator"
                    && id.source == authority.source
            })
            .unwrap();
        let (start, end, _) = anchors[0];
        assert!(
            owner.start <= *start && *end <= owner.end,
            "native type occurrence must belong to the candidate config declaration"
        );
        let answer = call(
            "workspace_query",
            json!({"type":"forward_edges","domain":"angular","entity_type":"ApplicationConfig","name":"appConfig"}),
        );
        let edges = answer["edges"].as_array().unwrap();
        let found: Vec<_> = edges
            .iter()
            .filter(|e| e["relation"] == "RegistersProvider")
            .collect();
        assert_eq!(
            found.len(),
            2,
            "config-declared provider membership omitted; form={form}; public={answer}"
        );
        assert_eq!(token_files(&found), expected);
        for edge in found {
            assert_eq!(edge["subject"]["entity_type"], "ApplicationConfig");
            assert_eq!(
                identity(edge["subject"]["file"].as_str().unwrap()),
                identity(&p.join("config.ts").to_string_lossy())
            );
        }
        assert_eq!(
            std::fs::read_to_string(p.join("config.ts")).unwrap(),
            source
        );
    }
}

#[test]
fn issue_162_application_config_provider_direct() {
    application_config_question("direct");
}
#[test]
fn issue_162_application_config_provider_local_composition() {
    application_config_question("local");
}
#[test]
fn issue_162_application_config_provider_imported_barrel() {
    application_config_question("barrel");
}
#[test]
fn issue_162_application_config_provider_imported_namespace() {
    application_config_question("namespace");
}
