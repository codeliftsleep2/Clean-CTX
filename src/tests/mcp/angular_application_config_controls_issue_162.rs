use crate::layers::meta::semantic::SemanticRelation;
use serde_json::{Value, json};
use std::path::Path;

const PREFIX: &str = "import type {ApplicationConfig as NativeConfig} from '@angular/core'; import {Token as Left} from './left'; import {Token as Right} from './right'; import {GROUP} from './group';";
const GROUP: &str = "import {Token as Left} from './left'; import {Token as Right} from './right'; export const GROUP=[Left,{provide:Right,useValue:1},Left];";
const SOURCE: &str = "export const appConfig:NativeConfig={providers:[...GROUP]};";
const CORE: &str = r#"{"name":"@angular/core","version":"20.0.0","types":"index.d.ts"}"#;

fn setup(body: &str) -> (tempfile::TempDir, crate::config::CleanCtxConfig) {
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install(p);
    std::fs::write(p.join("node_modules/@angular/core/index.d.ts"),"export declare function Component(metadata:unknown):any; export interface ApplicationConfig {providers:unknown[];}").unwrap();
    for name in ["left.ts", "right.ts"] {
        std::fs::write(p.join(name), "export class Token {}").unwrap();
    }
    std::fs::write(p.join("group.ts"), GROUP).unwrap();
    std::fs::write(
        p.join("barrel.ts"),
        "export {GROUP as Published} from './group';",
    )
    .unwrap();
    std::fs::write(
        p.join("fake.ts"),
        "export interface ApplicationConfig {providers:unknown[];}",
    )
    .unwrap();
    std::fs::write(p.join("config.ts"), format!("{PREFIX}{body}")).unwrap();
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    config.persistence.enabled = true;
    config.persistence.db_path = p.join("authority.db").to_string_lossy().into_owned();
    (root, config)
}
fn raw(state: &crate::mcp::McpState, p: &Path, tool: &str, mut args: Value) -> Value {
    args["workspaceRoot"] = json!(p);
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(&json!(162), tool, &json!({"arguments":args}), state);
    crate::protocol::captured_responses().pop().unwrap()
}
fn call(state: &crate::mcp::McpState, p: &Path, tool: &str, args: Value) -> Value {
    let r = raw(state, p, tool, args);
    assert!(r.get("error").is_none(), "{r}");
    assert_ne!(r["result"]["isError"], true, "{r}");
    r["result"]["structuredContent"].clone()
}
fn capture(state: &crate::mcp::McpState, p: &Path, name: &str) {
    call(
        state,
        p,
        "compress_code_context",
        json!({"filePath":p.join(name),"fidelity":"high"}),
    );
}
fn proof(
    state: &crate::mcp::McpState,
    p: &Path,
    name: &str,
) -> crate::ir::typescript_resolution::TypeScriptAuthority {
    let alias = state
        .alias_for_path(&p.join(name).to_string_lossy())
        .unwrap();
    state
        .semantic_snapshot(&alias)
        .unwrap()
        .authority
        .typescript_authority
        .as_ref()
        .unwrap()
        .clone()
}
fn answers(state: &crate::mcp::McpState, p: &Path, expected: &[(&str, &str)]) {
    let r = call(
        state,
        p,
        "workspace_query",
        json!({"type":"forward_edges","domain":"angular","entity_type":"ApplicationConfig","name":"appConfig"}),
    );
    let edges: Vec<_> = r["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["relation"] == "RegistersProvider")
        .collect();
    let key = |s: &str| crate::dictionary::path::canonical_identity_key(s);
    let mut actual: Vec<_> = edges
        .iter()
        .map(|edge| {
            assert_eq!(edge["object"]["name"], "Token");
            assert_eq!(edge["subject"]["entity_type"], "ApplicationConfig");
            assert!(
                edge["subject"].get("canonical").is_none()
                    && edge["object"].get("canonical").is_none()
            );
            (
                key(edge["subject"]["file"].as_str().unwrap()),
                key(edge["object"]["file"].as_str().unwrap()),
            )
        })
        .collect();
    actual.sort();
    let mut expected: Vec<_> = expected
        .iter()
        .map(|(owner, token)| {
            (
                key(&p.join(owner).to_string_lossy()),
                key(&p.join(token).to_string_lossy()),
            )
        })
        .collect();
    expected.sort();
    assert_eq!(actual, expected, "{r}");
}
const BOTH: &[(&str, &str)] = &[("config.ts", "left.ts"), ("config.ts", "right.ts")];

#[test]
fn issue_162_application_config_provider_native_roots_refusals_and_position_policy() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup(SOURCE);
    let p = root.path();
    let state = crate::mcp::McpState::new(config);
    capture(&state, p, "config.ts");
    answers(&state, p, BOTH);
    for body in [
        SOURCE.to_owned(),
        SOURCE.replace(":NativeConfig", ":core.ApplicationConfig")
            + " import type * as core from '@angular/core';",
        SOURCE.replace(":NativeConfig", ":ApplicationConfig")
            + " import type {ApplicationConfig} from '@angular/core';",
    ] {
        std::fs::write(p.join("config.ts"), format!("{PREFIX}{body}")).unwrap();
        answers(&state, p, BOTH);
    }
    for body in [
        SOURCE.replace(":NativeConfig", ""),
        SOURCE.replace("const appConfig", "let appConfig"),
        SOURCE.replace(":NativeConfig", ":Fake") + " type Fake={providers:unknown[]};",
        SOURCE.replace(":NativeConfig", ":Foreign")
            + " import type {ApplicationConfig as Foreign} from './fake';",
        SOURCE.replace("{providers:[...GROUP]}", "{...{providers:[...GROUP]}}"),
        SOURCE.replace("providers:", "viewProviders:"),
        format!("{SOURCE}{SOURCE}"),
        "export const appConfig:NativeConfig=other; const other={providers:[Left]};".into(),
        "const unrelated={providers:[Left]}; export const appConfig:NativeConfig={};".into(),
    ] {
        std::fs::write(p.join("config.ts"), format!("{PREFIX}{body}")).unwrap();
        answers(&state, p, &[]);
    }
    std::fs::write(p.join("config.ts"), format!("{PREFIX}{SOURCE}")).unwrap();
    capture(&state, p, "config.ts");
    answers(&state, p, BOTH);
    let authority = proof(&state, p, "config.ts");
    let mut edges = crate::angular_meta::application_config_providers::project(&authority);
    edges.retain(|e| e.relation == SemanticRelation::RegistersProvider);
    for edge in &mut edges {
        edge.relation = SemanticRelation::RegistersViewProvider;
    }
    assert!(!authority.admits_relation_evidence(&edges));
}

#[test]
fn issue_162_application_config_provider_shared_strategies_cycles_and_negative_siblings() {
    let _serial = crate::protocol::handler_response_serial();
    let source = "function make(){return {}} const cycle=[...cycle,Right]; export const appConfig:NativeConfig={providers:[...GROUP,...cycle,{provide:'str',useClass:Left},{provide:Left,useFactory:make},{provide:Right,useExisting:Left},{provide:Left,useValue:Right}]};";
    let (root, config) = setup(source);
    let p = root.path();
    let state = crate::mcp::McpState::new(config);
    capture(&state, p, "config.ts");
    let authority = proof(&state, p, "config.ts");
    let edges = crate::angular_meta::application_config_providers::project(&authority);
    assert!(authority.admits_relation_evidence(&edges));
    assert_eq!(
        edges
            .iter()
            .filter(|e| e.relation == SemanticRelation::RegistersProvider)
            .count(),
        8
    );
    for relation in [
        SemanticRelation::ProviderUsesClass,
        SemanticRelation::ProviderUsesFactory,
        SemanticRelation::ProviderUsesExisting,
        SemanticRelation::ProviderUsesValue,
    ] {
        assert!(edges.iter().any(|e| e.relation == relation), "{edges:?}");
    }
    std::fs::write(
        p.join("ambiguous.ts"),
        "export * from '@angular/core'; export * from './fake';",
    )
    .unwrap();
    for (extra, expression) in [
        (
            "import type {Token as OnlyType} from './right';",
            "[Left,OnlyType]",
        ),
        ("", "[Left,{provide:Right,useClass:Right,useValue:1}]"),
        ("", "[Left,{provide:Left,provide:Right,useValue:1}]"),
        ("const arbitrary=()=>[Right];", "[Left,...arbitrary()]"),
        ("", "[Left,...missing]"),
    ] {
        std::fs::write(
            p.join("config.ts"),
            format!(
                "{PREFIX}{extra} export const appConfig:NativeConfig={{providers:{expression}}};"
            ),
        )
        .unwrap();
        answers(&state, p, &[("config.ts", "left.ts")]);
    }
    std::fs::write(p.join("config.ts"),format!("{PREFIX}import type {{ApplicationConfig as Ambiguous}} from './ambiguous'; export const appConfig:Ambiguous={{providers:[Left]}};")).unwrap();
    answers(&state, p, &[]);
}

#[test]
fn issue_162_application_config_provider_membership_forgery_and_serde_refusal() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup(SOURCE);
    let p = root.path();
    let state = crate::mcp::McpState::new(config);
    capture(&state, p, "config.ts");
    let authority = proof(&state, p, "config.ts");
    let edges = crate::angular_meta::application_config_providers::project(&authority);
    assert!(authority.admits_relation_evidence(&edges));
    let restored = serde_json::from_value(serde_json::to_value(&authority).unwrap()).unwrap();
    assert!(crate::angular_meta::application_config_providers::admits(
        &restored, &edges[0]
    ));
    for mode in 0..17 {
        let mut bad = authority.clone();
        let index = bad
            .declaration_facets
            .iter()
            .position(|f| f.declaration.name == "appConfig")
            .unwrap();
        match mode {
            0 => bad.type_reference_sites.clear(),
            1 => bad.package_exports.clear(),
            2 => bad.demanded_occurrences.clear(),
            3 => bad.declaration_facets[index].initializer = None,
            4 => bad.declaration_facets[index].type_annotation = None,
            5 => {
                let duplicate = bad.declaration_facets[index].clone();
                bad.declaration_facets.push(duplicate);
            }
            6 => bad.declaration_facets[index]
                .declaration
                .source_hash
                .push('x'),
            7 => bad.references.retain(|(name, _, _)| name != "appConfig"),
            8 => bad.declarations.retain(|id| id.name != "appConfig"),
            10 => {
                use crate::ir::typescript_resolution::declaration_facets::ExpressionValue;
                let ExpressionValue::Object(fields) = &mut bad.declaration_facets[index]
                    .initializer
                    .as_mut()
                    .unwrap()
                    .value
                else {
                    panic!()
                };
                let duplicate = fields
                    .iter()
                    .find(|(key, _)| key == "providers")
                    .unwrap()
                    .clone();
                fields.push(duplicate);
            }
            11 => {
                use crate::ir::typescript_resolution::declaration_facets::ExpressionValue;
                let ExpressionValue::Object(fields) = &mut bad.declaration_facets[index]
                    .initializer
                    .as_mut()
                    .unwrap()
                    .value
                else {
                    panic!()
                };
                fields
                    .iter_mut()
                    .find(|(key, _)| key == "providers")
                    .unwrap()
                    .0 = "unrelated".into();
            }
            12 => {
                bad.declaration_facets[index]
                    .type_annotation
                    .as_mut()
                    .unwrap()
                    .start += 1
            }
            13 => {
                bad.declaration_facets[index]
                    .initializer
                    .as_mut()
                    .unwrap()
                    .start += 1
            }
            14 => {
                let annotation = bad.declaration_facets[index]
                    .type_annotation
                    .as_ref()
                    .unwrap();
                let duplicate = bad
                    .type_reference_sites
                    .iter()
                    .find(|(start, end, _)| *start == annotation.start && *end == annotation.end)
                    .unwrap()
                    .clone();
                bad.type_reference_sites.push(duplicate);
            }
            15 => {
                let annotation = bad.declaration_facets[index]
                    .type_annotation
                    .clone()
                    .unwrap();
                bad.declaration_facets[index].initializer = Some(annotation);
            }
            16 => bad.declaration_facets[index]
                .declaration
                .source
                .push("crossed.ts"),
            _ => {
                use crate::ir::typescript_resolution::declaration_facets::ExpressionValue;
                let ExpressionValue::Object(fields) = &mut bad.declaration_facets[index]
                    .initializer
                    .as_mut()
                    .unwrap()
                    .value
                else {
                    panic!()
                };
                fields
                    .iter_mut()
                    .find(|(key, _)| key == "providers")
                    .unwrap()
                    .1
                    .start += 1;
            }
        }
        assert!(!bad.admits_relation_evidence(&edges), "mode={mode}");
    }
    let mut forged = edges[0].clone();
    forged.subject.canonical = None;
    assert!(!authority.admits_relation_evidence(&[forged]));
    let mut forged = edges[0].clone();
    forged.subject.entity_type = "Component";
    assert!(!authority.admits_relation_evidence(&[forged]));
}

#[test]
fn issue_162_application_config_provider_complete_sqlite_same_name_identity_restore() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup(SOURCE);
    let p = root.path();
    std::fs::write(p.join("neighbor.ts"), format!("{PREFIX}{SOURCE}")).unwrap();
    let state = crate::mcp::McpState::new(config.clone());
    let mut expected = Vec::new();
    for name in ["config.ts", "neighbor.ts"] {
        capture(&state, p, name);
        let alias = state
            .alias_for_path(&p.join(name).to_string_lossy())
            .unwrap();
        let snapshot = state.semantic_snapshot(&alias).unwrap();
        let registrations: Vec<_> = snapshot
            .edges
            .iter()
            .filter(|e| e.relation == SemanticRelation::RegistersProvider)
            .map(|e| json!([e.subject.canonical, e.object.canonical]))
            .collect();
        assert_eq!(registrations.len(), 3);
        assert!(snapshot.authority.admits_relation_evidence(&snapshot.edges));
        expected.push(json!({"authority":snapshot.authority.typescript_authority,"registrations":registrations}));
        call(&state, p, "save_context", json!({"filePath":p.join(name)}));
    }
    drop(state);
    let db = rusqlite::Connection::open(&config.persistence.db_path).unwrap();
    for (name, expected) in ["config.ts", "neighbor.ts"].into_iter().zip(&expected) {
        let encoded: String = db
            .query_row(
                "SELECT edges_json FROM semantic_edge_snapshots WHERE file_path=?1",
                [p.join(name).to_string_lossy().into_owned()],
                |r| r.get(0),
            )
            .unwrap();
        let durable: Value = serde_json::from_str(&encoded).unwrap();
        assert_eq!(durable["typescript_authority"], expected["authority"]);
        let registrations: Vec<_> = durable["edges"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["relation"] == "RegistersProvider")
            .map(|e| json!([e["subject"]["canonical"], e["object"]["canonical"]]))
            .collect();
        assert_eq!(json!(registrations), expected["registrations"]);
    }
    drop(db);
    let state = crate::mcp::McpState::new(config);
    for (name, expected) in ["config.ts", "neighbor.ts"].into_iter().zip(expected) {
        call(
            &state,
            p,
            "restore_context",
            json!({"filePath":p.join(name)}),
        );
        let owner = state.semantic_owner_path(&p.join(name).to_string_lossy());
        assert_eq!(
            serde_json::to_value(
                state
                    .workspace_index_read()
                    .retained_typescript_authority(&owner)
                    .unwrap()
            )
            .unwrap(),
            expected["authority"]
        );
        let alias = state
            .alias_for_path(&p.join(name).to_string_lossy())
            .unwrap();
        let snapshot = state.semantic_snapshot(&alias).unwrap();
        let registrations: Vec<_> = snapshot
            .edges
            .iter()
            .filter(|e| e.relation == SemanticRelation::RegistersProvider)
            .map(|e| json!([e.subject.canonical, e.object.canonical]))
            .collect();
        assert_eq!(
            json!(registrations),
            expected["registrations"],
            "before queries"
        );
    }
    answers(
        &state,
        p,
        &[
            ("config.ts", "left.ts"),
            ("config.ts", "right.ts"),
            ("neighbor.ts", "left.ts"),
            ("neighbor.ts", "right.ts"),
        ],
    );
}

#[test]
fn issue_162_application_config_provider_stale_dependency_config_package_and_repair() {
    let _serial = crate::protocol::handler_response_serial();
    let body = "import {Published} from './barrel'; export const appConfig:NativeConfig={providers:[Left,...Published]};";
    for mode in 0..6 {
        let (root, config) = setup(body);
        let p = root.path();
        let state = crate::mcp::McpState::new(config.clone());
        capture(&state, p, "config.ts");
        answers(&state, p, BOTH);
        call(
            &state,
            p,
            "save_context",
            json!({"filePath":p.join("config.ts")}),
        );
        drop(state);
        let (file,replacement,expected)=match mode {
            0=>("group.ts",GROUP.replace("const GROUP","let GROUP"),&BOTH[..1]),
            1=>("right.ts","export interface Token {}".to_owned(),&BOTH[..1]),
            2=>("barrel.ts","export type {GROUP as Published} from './group';".to_owned(),&BOTH[..1]),
            3=>("node_modules/@angular/core/package.json",CORE.replace("20.0.0","20.0.1"),BOTH),
            4=>("node_modules/@angular/core/package.json",json!({"name":"@angular/core","version":"20.0.0","exports":{".":null}}).to_string(),&[][..]),
            _=>("tsconfig.json",json!({"compilerOptions":{"module":"esnext","moduleResolution":"bundler","noLib":true,"types":[],"paths":{"@angular/core":["./fake.ts"]}}}).to_string(),&[][..]),
        };
        let original = std::fs::read(p.join(file)).unwrap();
        std::fs::write(p.join(file), replacement).unwrap();
        let state = crate::mcp::McpState::new(config);
        let r = raw(
            &state,
            p,
            "restore_context",
            json!({"filePath":p.join("config.ts")}),
        );
        assert!(
            r.get("error").is_some() || r["result"]["isError"] == true,
            "{r}"
        );
        answers(&state, p, expected);
        std::fs::write(p.join(file), original).unwrap();
        answers(&state, p, BOTH);
        assert_eq!(
            std::fs::read_to_string(p.join("config.ts")).unwrap(),
            format!("{PREFIX}{body}")
        );
    }
}

#[test]
fn issue_162_application_config_provider_independent_field_contributor_and_config_retirement() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup(SOURCE);
    let p = root.path();
    std::fs::write(p.join("neighbor.ts"), format!("{PREFIX}{SOURCE}")).unwrap();
    let state = crate::mcp::McpState::new(config);
    for name in ["config.ts", "neighbor.ts"] {
        capture(&state, p, name);
    }
    let neighbor = &[("neighbor.ts", "left.ts"), ("neighbor.ts", "right.ts")][..];
    answers(&state, p, &[BOTH, neighbor].concat());
    std::fs::write(
        p.join("config.ts"),
        format!("{PREFIX}export const appConfig:NativeConfig={{}};"),
    )
    .unwrap();
    answers(&state, p, neighbor);
    std::fs::write(p.join("config.ts"), format!("{PREFIX}{SOURCE}")).unwrap();
    capture(&state, p, "config.ts");
    answers(&state, p, &[BOTH, neighbor].concat());
    std::fs::write(
        p.join("group.ts"),
        GROUP.replace("[Left,{provide:Right,useValue:1},Left]", "[Right]"),
    )
    .unwrap();
    answers(
        &state,
        p,
        &[("config.ts", "right.ts"), ("neighbor.ts", "right.ts")],
    );
    std::fs::remove_file(p.join("group.ts")).unwrap();
    answers(&state, p, &[]);
    std::fs::write(p.join("group.ts"), GROUP).unwrap();
    capture(&state, p, "config.ts");
    capture(&state, p, "neighbor.ts");
    for name in ["config.ts", "neighbor.ts"] {
        let owner = state.semantic_owner_path(&p.join(name).to_string_lossy());
        assert!(
            state
                .workspace_index_read()
                .retained_typescript_authority(&owner)
                .is_some(),
            "compression must publish retained index authority before a semantic query: {name}"
        );
    }
    answers(&state, p, &[BOTH, neighbor].concat());
    std::fs::write(p.join("config.ts"), "export class appConfig {}").unwrap();
    answers(&state, p, neighbor);
    std::fs::remove_file(p.join("neighbor.ts")).unwrap();
    call(&state, p, "refresh_workspace", json!({}));
    answers(&state, p, &[]);
}
