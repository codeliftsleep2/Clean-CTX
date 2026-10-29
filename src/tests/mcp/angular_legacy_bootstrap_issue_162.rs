//! Investigation only: legacy receiver authority and relation vocabulary are hypotheses.
use super::angular_bootstrap_inline_controls_tests::{call, capture};
use super::angular_config_composition_controls_tests::setup as base_setup;
use crate::ir::invocation_evidence::{ArgumentQualifier, SourceSpan, WrittenCallee};
use crate::ir::typescript_resolution::{Namespace, TypeScriptAuthority};
use serde_json::json;
use std::path::Path;

fn key(p: &Path) -> String {
    crate::dictionary::path::canonical_identity_key(&p.to_string_lossy())
}
fn forms(form: &str) -> (&'static str, &'static str) {
    match form {
        "direct" => (
            "import {platformBrowserDynamic} from '@angular/platform-browser-dynamic';",
            "platformBrowserDynamic",
        ),
        "alias" => (
            "import {platformBrowserDynamic as platform} from '@angular/platform-browser-dynamic';",
            "platform",
        ),
        "barrel" => (
            "import {start as platform} from './legacy-api';",
            "platform",
        ),
        "namespace" => (
            "import * as legacy from '@angular/platform-browser-dynamic';",
            "legacy.platformBrowserDynamic",
        ),
        _ => panic!("unknown route"),
    }
}
fn setup(form: &str, refused: bool) -> (tempfile::TempDir, crate::mcp::McpState) {
    let (root, config) = base_setup("Base,Feature");
    let p = root.path();
    let core = p.join("node_modules/@angular/core/index.d.ts");
    let mut native = std::fs::read_to_string(&core).unwrap();
    native.push_str(" export declare function NgModule(metadata:unknown):any; export declare class PlatformRef { bootstrapModule(module:unknown):unknown; }");
    std::fs::write(core, native).unwrap();
    let package = p.join("node_modules/@angular/platform-browser-dynamic");
    std::fs::create_dir_all(&package).unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{"name":"@angular/platform-browser-dynamic","version":"20.0.0","types":"index.d.ts"}"#,
    )
    .unwrap();
    std::fs::write(package.join("index.d.ts"), "import {PlatformRef} from '@angular/core'; export declare function platformBrowserDynamic():PlatformRef;").unwrap();
    std::fs::write(
        p.join("legacy-api.ts"),
        "export {platformBrowserDynamic as start} from '@angular/platform-browser-dynamic';",
    )
    .unwrap();
    std::fs::write(
        p.join("legacy-fake.ts"),
        "export function platformBrowserDynamic(){return {bootstrapModule(module:unknown){}};}",
    )
    .unwrap();
    for side in ["left", "right"] {
        std::fs::write(p.join(format!("{side}-module.ts")), format!("import {{NgModule}} from '@angular/core'; import {{Token}} from './{side}'; @NgModule({{providers:[Token]}}) export class AppModule {{}}")).unwrap();
        let (import, factory) = forms(form);
        let body = if refused {
            "import {platformBrowserDynamic as foreign} from './legacy-fake'; foreign().bootstrapModule(AppModule); const receiver={bootstrapModule(module:unknown){}}; receiver.bootstrapModule(AppModule); function shadow(platformBrowserDynamic:any){platformBrowserDynamic().bootstrapModule(AppModule);}"
                .to_owned()
        } else {
            format!(
                "{import} {factory}().bootstrapModule(AppModule); {factory}().bootstrapModule(AppModule);"
            )
        };
        std::fs::write(
            p.join(format!("{side}-legacy.ts")),
            format!("import {{AppModule}} from './{side}-module'; {body}"),
        )
        .unwrap();
    }
    let state = crate::mcp::McpState::new(config);
    (root, state)
}
fn controls(state: &crate::mcp::McpState, p: &Path) -> Vec<TypeScriptAuthority> {
    let mut authorities = Vec::new();
    for side in ["left", "right"] {
        let authority = capture(state, p, &format!("{side}-module.ts"));
        let ids = authority
            .declarations
            .iter()
            .filter(|id| id.name == "AppModule" && id.source == authority.source)
            .collect::<Vec<_>>();
        assert_eq!(ids.len(), 1, "stage=module-prerequisite {authority:?}");
        assert_eq!(
            crate::angular_meta::class_role_authority::declaration_role(&authority, ids[0]),
            Some(crate::angular_meta::decorators::ClassKind::Module),
            "stage=module-role-prerequisite"
        );
        authorities.push(authority);
    }
    let providers = call(
        state,
        p,
        "workspace_query",
        json!({"type":"forward_edges","domain":"angular","entity_type":"Module","name":"AppModule"}),
    );
    let mut actual = providers["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["relation"] == "RegistersProvider")
        .map(|e| {
            (
                key(Path::new(e["subject"]["file"].as_str().unwrap())),
                key(Path::new(e["object"]["file"].as_str().unwrap())),
            )
        })
        .collect::<Vec<_>>();
    let mut wanted = ["left", "right"]
        .map(|side| {
            (
                key(&p.join(format!("{side}-module.ts"))),
                key(&p.join(format!("{side}.ts"))),
            )
        })
        .to_vec();
    actual.sort();
    wanted.sort();
    assert_eq!(actual, wanted, "stage=provider-prerequisite {providers}");
    let modern = call(
        state,
        p,
        "workspace_query",
        json!({"type":"forward_edges","domain":"angular","entity_type":"ApplicationBootstrap","name":"bootstrapApplication"}),
    );
    assert!(
        modern["edges"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["relation"] == "DeclaresBootstrapRoot"),
        "stage=modern-prerequisite {modern}"
    );
    super::angular_config_composition_controls_tests::facts(state, p, &["base.ts", "feature.ts"]);
    authorities
}
fn investigate(form: &str) {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state) = setup(form, false);
    let p = root.path();
    let modules = controls(&state, p);
    let (_, factory) = forms(form);
    for (side, module) in ["left", "right"].into_iter().zip(&modules) {
        let file = format!("{side}-legacy.ts");
        let source = std::fs::read_to_string(p.join(&file)).unwrap();
        let authority = capture(&state, p, &file);
        let artifact = crate::mcp::tool_helpers::compile_source_ir_candidate(
            &p.join(&file).to_string_lossy(),
            &source,
            crate::compression::Fidelity::High,
            &state,
            Some(&p.to_string_lossy()),
        )
        .expect("registered production compiler");
        let evidence = artifact
            .invocation_evidence()
            .expect("stage=syntax-prerequisite invocation inventory");
        let view = evidence.view(evidence.source_id(), &source).unwrap();
        let expression = format!("{factory}().bootstrapModule(AppModule)");
        let sites = source.match_indices(&expression).collect::<Vec<_>>();
        assert_eq!(sites.len(), 2);
        for (start, _) in sites {
            let end = start + expression.len();
            let outer = view
                .occurrences()
                .iter()
                .find(|c| c.span().start_byte() == start && c.span().end_byte() == end)
                .expect("stage=syntax-prerequisite outer call");
            let WrittenCallee::Member { receiver, terminal } = outer.callee() else {
                panic!("stage=syntax-prerequisite member {outer:?}")
            };
            assert_eq!(view.slice(*receiver).unwrap(), format!("{factory}()"));
            assert_eq!(view.slice(*terminal).unwrap(), "bootstrapModule");
            let inner = view
                .occurrences()
                .iter()
                .find(|c| c.span() == *receiver)
                .expect("stage=syntax-prerequisite exact inner call");
            assert!(inner.arguments().is_empty());
            assert_eq!(outer.arguments().len(), 1);
            let arg = &outer.arguments()[0];
            assert!(matches!(arg.qualifier(), ArgumentQualifier::Plain));
            assert_eq!(view.slice(arg.span()).unwrap(), "AppModule");
            let factory_span = SourceSpan::new(start, start + factory.len()).unwrap();
            let api=authority.value_reference_at(factory_span).unwrap_or_else(||panic!("stage=factory-acquisition-prerequisite missing retained factory; file={file}; authority={authority:?}"));
            assert!(
                crate::angular_meta::class_role_authority::public_entry_api(
                    &authority,
                    api,
                    "@angular/platform-browser-dynamic",
                    "@angular/platform-browser-dynamic",
                    "platformBrowserDynamic",
                    Namespace::Value
                ),
                "stage=factory-identity-prerequisite {api:?}"
            );
            let callee = SourceSpan::new(receiver.start_byte(), terminal.end_byte()).unwrap();
            let method=authority.value_reference_at(callee).unwrap_or_else(||panic!("stage=receiver-method-prerequisite missing exact method association; authority={authority:?}"));
            // This native declaration check is necessary, not a substitute for receiver proof.
            assert_eq!(method.name, "bootstrapModule", "stage=method-prerequisite");
            assert_eq!(
                key(&method.source),
                key(&p.join("node_modules/@angular/core/index.d.ts")),
                "stage=method-prerequisite"
            );
            let retained = authority
                .invocation_facets
                .iter()
                .find(|f| f.expression.start == start && f.expression.end == end)
                .expect("stage=receiver-method-prerequisite outer retained chain");
            let crate::ir::typescript_resolution::declaration_facets::ExpressionValue::Call(
                member,
                args,
            ) = &retained.expression.value
            else {
                panic!("stage=receiver-method-prerequisite outer call facet")
            };
            // Current direct-reference call proof alone cannot certify the native receiver return.
            assert!(
                matches!(
                    member.value,
                    crate::ir::typescript_resolution::declaration_facets::ExpressionValue::Member(
                        _,
                        _
                    )
                ),
                "stage=receiver-method-prerequisite exact retained receiver chain {retained:?}"
            );
            assert_eq!(args.len(), 1);
            let target = authority
                .value_reference_at(arg.span())
                .expect("stage=argument-prerequisite exact Module reference");
            let expected = module
                .declarations
                .iter()
                .find(|id| id.name == "AppModule" && id.source == module.source)
                .unwrap();
            assert_eq!(
                target, expected,
                "stage=argument-prerequisite same-name target"
            );
        }
    }
    for reverse in [false, true] {
        let result = call(
            &state,
            p,
            "workspace_query",
            json!({"type":if reverse{"reverse_edges"}else{"forward_edges"},"domain":"angular","entity_type":if reverse{"Module"}else{"ApplicationBootstrap"},"name":if reverse{"AppModule"}else{"bootstrapModule"}}),
        );
        let edges = result["edges"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["relation"] == "DeclaresBootstrapModule")
            .collect::<Vec<_>>();
        assert_eq!(edges.len(), 4, "stage=legacy-module-association {result}");
        for side in ["left", "right"] {
            assert_eq!(
                edges
                    .iter()
                    .filter(|e| key(Path::new(e["subject"]["file"].as_str().unwrap()))
                        == key(&p.join(format!("{side}-legacy.ts")))
                        && key(Path::new(e["object"]["file"].as_str().unwrap()))
                            == key(&p.join(format!("{side}-module.ts"))))
                    .count(),
                2,
                "exact independent occurrence multiset"
            );
        }
    }
}
#[test]
fn issue_162_legacy_bootstrap_direct() {
    investigate("direct");
}
#[test]
fn issue_162_legacy_bootstrap_alias() {
    investigate("alias");
}
#[test]
fn issue_162_legacy_bootstrap_barrel() {
    investigate("barrel");
}
#[test]
fn issue_162_legacy_bootstrap_namespace() {
    investigate("namespace");
}
#[test]
fn issue_162_legacy_bootstrap_foreign_shadow_siblings() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state) = setup("direct", true);
    let p = root.path();
    controls(&state, p);
    for side in ["left", "right"] {
        capture(&state, p, &format!("{side}-legacy.ts"));
    }
    let answer = call(
        &state,
        p,
        "workspace_query",
        json!({"type":"forward_edges","domain":"angular","entity_type":"ApplicationBootstrap","name":"bootstrapModule"}),
    );
    assert!(
        !answer["edges"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["relation"] == "DeclaresBootstrapModule"),
        "{answer}"
    );
    controls(&state, p);
}
