//! Integrated generation-72 written Module bootstrap admission/lifecycle controls.
use super::angular_bootstrap_inline_controls_tests::{call, capture, raw};
use super::angular_config_composition_controls_tests::setup as base_setup;
use crate::ir::typescript_resolution::TypeScriptAuthority;
use serde_json::json;
use std::path::Path;
const FACTORY: &str = "import {PlatformRef} from '@angular/core'; export declare function platformBrowserDynamic():PlatformRef;";
pub(super) const PREFIX: &str = "import {platformBrowserDynamic as make} from '@angular/platform-browser-dynamic'; import {AppModule} from './module'; import {AppModule as Other} from './other-module';";
pub(super) const CALL: &str = "make().bootstrapModule(AppModule);";
pub(super) fn setup() -> (tempfile::TempDir, crate::config::CleanCtxConfig) {
    let (root, config) = base_setup("Base,Feature");
    let p = root.path();
    let core = p.join("node_modules/@angular/core/index.d.ts");
    let mut text = std::fs::read_to_string(&core).unwrap();
    text.push_str(" export declare function NgModule(metadata:unknown):any; export declare class PlatformRef {bootstrapModule(module:unknown):unknown;}");
    std::fs::write(core, text).unwrap();
    let package = p.join("node_modules/@angular/platform-browser-dynamic");
    std::fs::create_dir_all(&package).unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{"name":"@angular/platform-browser-dynamic","version":"20.0.0","types":"index.d.ts"}"#,
    )
    .unwrap();
    std::fs::write(package.join("index.d.ts"), FACTORY).unwrap();
    for file in ["module.ts", "other-module.ts"] {
        std::fs::write(p.join(file),"import {NgModule} from '@angular/core'; import {Token} from './left'; @NgModule({providers:[Token]}) export class AppModule {}").unwrap();
    }
    for file in ["legacy-a.ts", "legacy-b.ts"] {
        std::fs::write(p.join(file), format!("{PREFIX}{CALL}{CALL}")).unwrap();
    }
    (root, config)
}
fn key(p: &Path) -> String {
    crate::dictionary::path::canonical_identity_key(&p.to_string_lossy())
}
pub(super) fn facts(state: &crate::mcp::McpState, p: &Path, wanted: &[(&str, &str)]) {
    for reverse in [false, true] {
        let answer = call(
            state,
            p,
            "workspace_query",
            json!({"type":if reverse{"reverse_edges"}else{"forward_edges"},"domain":"angular","entity_type":if reverse{"Module"}else{"ApplicationBootstrap"},"name":if reverse{"AppModule"}else{"bootstrapModule"}}),
        );
        let edges = answer["edges"].as_array().unwrap();
        let mut actual = edges
            .iter()
            .filter(|e| e["relation"] == "DeclaresBootstrapModule")
            .map(|e| {
                (
                    key(Path::new(e["subject"]["file"].as_str().unwrap())),
                    key(Path::new(e["object"]["file"].as_str().unwrap())),
                )
            })
            .collect::<Vec<_>>();
        let mut expected = wanted
            .iter()
            .map(|(s, t)| (key(&p.join(s)), key(&p.join(t))))
            .collect::<Vec<_>>();
        actual.sort();
        expected.sort();
        assert_eq!(actual, expected, "{answer}");
        assert_eq!(answer["coverage"]["result_semantics"], "lower_bound");
        assert!(
            !edges
                .iter()
                .any(|e| e["subject"]["name"] == "bootstrapModule"
                    && e["relation"] == "RegistersProvider")
        );
    }
}
pub(super) const FOUR: [(&str, &str); 4] = [
    ("legacy-a.ts", "module.ts"),
    ("legacy-a.ts", "module.ts"),
    ("legacy-b.ts", "module.ts"),
    ("legacy-b.ts", "module.ts"),
];
#[test]
fn issue_162_legacy_controls_forged_neutral_target_and_missing_payload_refusal() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup();
    let state = crate::mcp::McpState::new(config);
    let p = root.path();
    let a = capture(&state, p, "legacy-a.ts");
    let edges = crate::angular_meta::legacy_bootstrap::project(&a);
    assert_eq!(edges.len(), 2);
    assert!(a.admits_relation_evidence(&edges));
    for mode in 0..12 {
        let mut bad = a.clone();
        let proof = bad.declared_return_members[0].clone();
        match mode {
            0=>bad.declared_return_members.clear(),
            1=>bad.declared_return_members.push(proof.clone()),
            2=>bad.declared_return_members[0].factory.source_hash.push('x'),
            3=>bad.declared_return_members[0].receiver.source_hash.push('x'),
            4=>bad.declared_return_members[0].method.source_hash.push('x'),
            5=>bad.declared_return_members[0].inner=bad.declared_return_members[1].inner,
            6=>bad.declared_return_members[0].property=bad.declared_return_members[1].property,
            7=>bad.invocation_facets.retain(|f|(f.expression.start,f.expression.end)!=proof.outer),
            8=>bad.package_exports.retain(|e|e.declaration!=proof.receiver),
            9=>bad.declaration_facets.retain(|f|f.declaration.name!="AppModule"),
            10=>bad.demanded_occurrences.retain(|s|!matches!(&s.value,crate::ir::typescript_resolution::demanded_occurrences::DemandedValue::Reference(crate::ir::typescript_resolution::Resolution::Resolved(id)) if id.name=="AppModule")),
            11=>bad.declared_return_members[0].source_hash.push('x'),
            _=>unreachable!(),
        }
        assert!(!bad.admits_relation_evidence(&edges[..1]), "mode={mode}");
        let restored: TypeScriptAuthority =
            serde_json::from_value(serde_json::to_value(&bad).unwrap()).unwrap();
        assert!(
            !restored.admits_relation_evidence(&edges[..1]),
            "serde mode={mode}"
        );
    }
    let mut wrong = edges[0].clone();
    // Borrow a different real Module, not an equal endpoint from the same Module.
    let other = capture(&state, p, "other-module.ts");
    let id = other
        .declarations
        .iter()
        .find(|id| id.name == "AppModule")
        .unwrap();
    wrong.object = crate::layers::meta::semantic::EntityRef::new("angular", "Module", &id.name)
        .with_file(id.source.to_string_lossy().into_owned());
    wrong.object.canonical = Some(
        crate::layers::meta::canonical_endpoint::CanonicalEndpoint::Declaration {
            declaration: id.into(),
            site: Some((id.start, id.end)),
        },
    );
    assert!(!a.admits_relation_evidence(&[wrong]));
    let missing = crate::mcp::semantic_compilation::retained::RetainedAuthority::default();
    assert!(!missing.admits_relation_evidence(&edges));
}
#[test]
fn issue_162_legacy_controls_native_return_and_target_refusal_siblings() {
    let _serial = crate::protocol::handler_response_serial();
    for mode in 0..7 {
        let (root, config) = setup();
        let p = root.path();
        let caller = match mode {
            0 => format!("{PREFIX} make().bootstrapModule(...[AppModule]);"),
            1 => format!("{PREFIX} const platform=make(); platform.bootstrapModule(AppModule);"),
            2 => format!("{PREFIX} make().bootstrapModule(Unknown);"),
            3 => format!("{PREFIX} make().bootstrapModule({{}});"),
            4 => {
                std::fs::write(p.join("module.ts"), "export class AppModule {}").unwrap();
                format!("{PREFIX}{CALL}")
            }
            5 => {
                std::fs::write(p.join("node_modules/@angular/platform-browser-dynamic/index.d.ts"),"import {PlatformRef} from '../../../fake-receiver'; export declare function platformBrowserDynamic():PlatformRef;").unwrap();
                std::fs::write(
                    p.join("fake-receiver.ts"),
                    "export declare class PlatformRef {bootstrapModule(module:unknown):unknown;}",
                )
                .unwrap();
                format!("{PREFIX}{CALL}")
            }
            6 => format!(
                "import type {{platformBrowserDynamic as make}} from '@angular/platform-browser-dynamic'; import {{AppModule}} from './module'; {CALL}"
            ),
            _ => unreachable!(),
        };
        std::fs::write(p.join("legacy-a.ts"), caller).unwrap();
        let state = crate::mcp::McpState::new(config);
        let a = capture(&state, p, "legacy-a.ts");
        assert!(
            crate::angular_meta::legacy_bootstrap::project(&a).is_empty(),
            "mode={mode}"
        );
        super::angular_config_composition_controls_tests::facts(
            &state,
            p,
            &["base.ts", "feature.ts"],
        );
    }
}
#[test]
fn issue_162_legacy_controls_sqlite_restore_before_public_query() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup();
    let p = root.path();
    let state = crate::mcp::McpState::new(config.clone());
    let mut before = Vec::new();
    for f in ["legacy-a.ts", "legacy-b.ts"] {
        let a = capture(&state, p, f);
        assert_eq!(crate::angular_meta::legacy_bootstrap::project(&a).len(), 2);
        before.push(a.declared_return_members);
        call(&state, p, "save_context", json!({"filePath":p.join(f)}));
    }
    drop(state);
    let restored = crate::mcp::McpState::new(config);
    for (i, f) in ["legacy-a.ts", "legacy-b.ts"].iter().enumerate() {
        call(
            &restored,
            p,
            "restore_context",
            json!({"filePath":p.join(f)}),
        );
        let alias = restored
            .alias_for_path(&p.join(f).to_string_lossy())
            .unwrap();
        let snap = restored.semantic_snapshot(&alias).unwrap();
        let a = snap.authority.typescript_authority.as_ref().unwrap();
        assert_eq!(a.declared_return_members, before[i]);
        let expected = crate::angular_meta::legacy_bootstrap::project(a);
        assert_eq!(expected.len(), 2);
        assert!(
            expected
                .iter()
                .all(|e| snap.edges.iter().any(|x| x.relation == e.relation
                    && x.subject.canonical == e.subject.canonical
                    && x.object.canonical == e.object.canonical))
        );
    }
    facts(&restored, p, &FOUR);
}
#[test]
fn issue_162_legacy_controls_cold_warm_refresh_and_independent_retirement() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup();
    let p = root.path();
    let state = crate::mcp::McpState::new(config);
    facts(&state, p, &FOUR);
    facts(&state, p, &FOUR);
    call(&state, p, "refresh_workspace", json!({}));
    facts(&state, p, &FOUR);
    let answer = call(
        &state,
        p,
        "workspace_query",
        json!({"type":"forward_edges","domain":"angular","entity_type":"ApplicationBootstrap","name":"bootstrapModule","withinPath":p.join("legacy-a.ts")}),
    );
    assert_eq!(
        answer["edges"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["relation"] == "DeclaresBootstrapModule")
            .count(),
        2
    );
    std::fs::write(
        p.join("legacy-a.ts"),
        format!("{PREFIX}{CALL}make().bootstrapModule(Other);"),
    )
    .unwrap();
    facts(
        &state,
        p,
        &[
            ("legacy-a.ts", "module.ts"),
            ("legacy-a.ts", "other-module.ts"),
            ("legacy-b.ts", "module.ts"),
            ("legacy-b.ts", "module.ts"),
        ],
    );
    std::fs::write(p.join("legacy-a.ts"), format!("{PREFIX}{CALL}")).unwrap();
    facts(
        &state,
        p,
        &[
            ("legacy-a.ts", "module.ts"),
            ("legacy-b.ts", "module.ts"),
            ("legacy-b.ts", "module.ts"),
        ],
    );
    std::fs::remove_file(p.join("legacy-a.ts")).unwrap();
    call(&state, p, "refresh_workspace", json!({}));
    facts(
        &state,
        p,
        &[("legacy-b.ts", "module.ts"), ("legacy-b.ts", "module.ts")],
    );
}
#[test]
fn issue_162_legacy_controls_stale_dependencies_and_unchanged_caller_repair() {
    let _serial = crate::protocol::handler_response_serial();
    for mode in 0..4 {
        let (root, config) = setup();
        let p = root.path();
        let state = crate::mcp::McpState::new(config.clone());
        let a = capture(&state, p, "legacy-a.ts");
        let edges = crate::angular_meta::legacy_bootstrap::project(&a);
        assert_eq!(edges.len(), 2);
        call(
            &state,
            p,
            "save_context",
            json!({"filePath":p.join("legacy-a.ts")}),
        );
        drop(state);
        let file = p.join(match mode {
            0 => "node_modules/@angular/platform-browser-dynamic/index.d.ts",
            1 => "node_modules/@angular/core/index.d.ts",
            2 => "module.ts",
            3 => "node_modules/@angular/platform-browser-dynamic/package.json",
            _ => unreachable!(),
        });
        let original = std::fs::read_to_string(&file).unwrap();
        let changed=match mode {0=>"export declare function platformBrowserDynamic():unknown;".into(),1=>original.replace("bootstrapModule(module:unknown)","other(module:unknown)"),2=>"export class AppModule {}".into(),3=>r#"{"name":"@angular/platform-browser-dynamic","version":"20.0.0","exports":{".":null}}"#.into(),_=>unreachable!()};
        std::fs::write(&file, changed).unwrap();
        assert!(!a.admits_relation_evidence(&edges));
        let next = crate::mcp::McpState::new(config.clone());
        let denied = raw(
            &next,
            p,
            "restore_context",
            json!({"filePath":p.join("legacy-a.ts")}),
        );
        assert!(
            denied.get("error").is_some() || denied["result"]["isError"] == true,
            "{denied}"
        );
        facts(&next, p, &[]);
        drop(next);
        std::fs::write(file, original).unwrap();
        let repaired = crate::mcp::McpState::new(config);
        facts(&repaired, p, &FOUR);
    }
}
