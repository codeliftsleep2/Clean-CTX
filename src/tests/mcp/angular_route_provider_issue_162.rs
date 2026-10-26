use serde_json::{Value, json};

fn route_provider_question(composed: bool) {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install(p);
    let router = p.join("node_modules/@angular/router");
    std::fs::create_dir_all(&router).unwrap();
    std::fs::write(
        router.join("package.json"),
        r#"{"name":"@angular/router","version":"20.0.0","types":"index.d.ts"}"#,
    )
    .unwrap();
    std::fs::write(router.join("index.d.ts"),
        "export interface Route {path?:string;component?:unknown;providers?:unknown[];} export type Routes=Route[];").unwrap();
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
    let expression = if composed {
        "[...[GROUP]]".to_owned()
    } else {
        format!("[{entries}]")
    };
    let source = format!(
        "import type {{Routes}} from '@angular/router'; import {{Host}} from './host'; import {{Token as Left}} from './left'; import {{Token as Right}} from './right'; import {{GROUP}} from './group'; export const routes:Routes=[{{path:'probe',component:Host,providers:{expression}}}];"
    );
    std::fs::write(p.join("routes.ts"), &source).unwrap();
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
            json!({"filePath":p.join("routes.ts"),"fidelity":"high"}),
        );
        let answer = call(
            "workspace_query",
            json!({"type":"forward_edges","domain":"angular","entity_type":"Route","name":"probe"}),
        );
        let edges = answer["edges"].as_array().unwrap();
        assert_eq!(
            edges
                .iter()
                .filter(|e| e["relation"] == "RouteMapsTo" && e["object"]["name"] == "Host")
                .count(),
            1,
            "existing route projection prerequisite, not native route-authority certification: {answer}"
        );
        let found: Vec<_> = edges
            .iter()
            .filter(|e| e["relation"] == "RegistersProvider")
            .collect();
        assert_eq!(
            found.len(),
            2,
            "route-declared provider membership omitted; composed={composed}; public={answer}"
        );
        assert_eq!(token_files(&found), expected);
        for edge in found {
            assert_eq!(edge["subject"]["entity_type"], "Route");
            assert_eq!(
                identity(edge["subject"]["file"].as_str().unwrap()),
                identity(&p.join("routes.ts").to_string_lossy())
            );
        }
        assert_eq!(
            std::fs::read_to_string(p.join("routes.ts")).unwrap(),
            source
        );
    }
}

#[test]
fn issue_162_route_provider_direct_membership() {
    route_provider_question(false);
}

#[test]
fn issue_162_route_provider_imported_membership() {
    route_provider_question(true);
}
