//! Investigation only: inline RegistersProvider owner policy is not production-approved.
use crate::ir::typescript_resolution::declaration_facets::ExpressionValue;
use crate::ir::typescript_resolution::{Namespace, Resolution, TypeScriptAuthority};
use crate::layers::meta::canonical_endpoint::CanonicalEndpoint;
use crate::layers::meta::semantic::SemanticRelation;
use serde_json::{Value, json};
use std::path::Path;

fn query(state: &crate::mcp::McpState, p: &Path, tool: &str, mut args: Value) -> Value {
    args["workspaceRoot"] = json!(p);
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(&json!(162), tool, &json!({"arguments":args}), state);
    let response = crate::protocol::captured_responses().pop().unwrap();
    assert!(response.get("error").is_none(), "{response}");
    assert_ne!(response["result"]["isError"], true, "{response}");
    response["result"]["structuredContent"].clone()
}
fn capture(state: &crate::mcp::McpState, p: &Path, file: &str) -> TypeScriptAuthority {
    query(
        state,
        p,
        "compress_code_context",
        json!({"filePath":p.join(file),"fidelity":"high"}),
    );
    let alias = state
        .alias_for_path(&p.join(file).to_string_lossy())
        .unwrap();
    state
        .semantic_snapshot(&alias)
        .unwrap()
        .authority
        .typescript_authority
        .unwrap()
}
fn identity(file: &str) -> String {
    crate::dictionary::path::canonical_identity_key(file)
}
fn setup(form: &str) -> (tempfile::TempDir, crate::mcp::McpState) {
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install(p);
    std::fs::write(p.join("node_modules/@angular/core/index.d.ts"), "export declare function Component(metadata:unknown):any; export declare function Injectable(metadata?:unknown):any; export interface ApplicationConfig {providers:unknown[];}").unwrap();
    let platform = p.join("node_modules/@angular/platform-browser");
    std::fs::create_dir_all(&platform).unwrap();
    std::fs::write(
        platform.join("package.json"),
        r#"{"name":"@angular/platform-browser","version":"20.0.0","types":"index.d.ts"}"#,
    )
    .unwrap();
    std::fs::write(
        platform.join("index.d.ts"),
        "export declare function bootstrapApplication(root:unknown,config?:unknown):unknown;",
    )
    .unwrap();
    std::fs::write(
        p.join("api.ts"),
        "export {bootstrapApplication as start} from '@angular/platform-browser';",
    )
    .unwrap();
    for side in ["left", "right"] {
        std::fs::write(p.join(format!("{side}-token.ts")), "export class Token {}").unwrap();
    }
    let tokens =
        "import {Token as Left} from './left-token'; import {Token as Right} from './right-token';";
    let (import, callee) = match form {
        "direct" => (
            "import {bootstrapApplication} from '@angular/platform-browser';",
            "bootstrapApplication",
        ),
        "alias" => (
            "import {bootstrapApplication as launch} from '@angular/platform-browser';",
            "launch",
        ),
        "barrel" => ("import {start as launch} from './api';", "launch"),
        "namespace" => (
            "import * as platform from '@angular/platform-browser';",
            "platform.bootstrapApplication",
        ),
        _ => panic!("unknown form"),
    };
    for side in ["left", "right"] {
        std::fs::write(p.join(format!("{side}-root.ts")), format!("import {{Component}} from '@angular/core'; {tokens} @Component({{providers:[Left,Right]}}) export class Root {{}}")).unwrap();
        std::fs::write(p.join(format!("{side}-config.ts")), format!("import type {{ApplicationConfig}} from '@angular/core'; {tokens} export const appConfig:ApplicationConfig={{providers:[Left,Right]}};")).unwrap();
        let inline =
            format!("{callee}(Root,{{providers:[Left,{{provide:Right,useValue:1}},Left]}});");
        std::fs::write(p.join(format!("{side}-main.ts")), format!("{import} {tokens} import {{Root}} from './{side}-root'; import {{appConfig}} from './{side}-config'; {inline}{inline} {callee}(Root,{{other:1}}); {callee}(Root,appConfig);")).unwrap();
    }
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    (root, crate::mcp::McpState::new(config))
}
fn prerequisites(state: &crate::mcp::McpState, p: &Path, side: &str) {
    let file = format!("{side}-main.ts");
    let authority = capture(state, p, &file);
    let edges = crate::angular_meta::bootstrap_references::project(&authority);
    assert_eq!(
        edges
            .iter()
            .filter(|e| e.relation == SemanticRelation::DeclaresBootstrapRoot)
            .count(),
        4,
        "stage=native-root; {authority:?}"
    );
    assert_eq!(
        edges
            .iter()
            .filter(|e| e.relation == SemanticRelation::UsesApplicationConfig)
            .count(),
        1,
        "stage=named-config"
    );
    assert!(authority.admits_relation_evidence(&edges));
    let roots: Vec<_> = edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::DeclaresBootstrapRoot)
        .collect();
    let mut inline_count = 0;
    for invocation in &authority.invocation_facets {
        let ExpressionValue::Call(callee, arguments) = &invocation.expression.value else {
            panic!("stage=call-structure")
        };
        let ExpressionValue::Reference(Resolution::Resolved(api)) = &callee.value else {
            panic!("stage=native-api")
        };
        assert!(crate::angular_meta::class_role_authority::public_entry_api(
            &authority,
            api,
            "@angular/platform-browser",
            "@angular/platform-browser",
            "bootstrapApplication",
            Namespace::Value
        ));
        assert_eq!(
            authority.demanded_reference_at(
                &invocation.source,
                &invocation.source_hash,
                callee.start,
                callee.end
            ),
            Some(api)
        );
        assert_eq!(arguments.len(), 2);
        let call_roots: Vec<_> = roots
            .iter()
            .filter(|e| {
                matches!(&e.subject.canonical,
            Some(CanonicalEndpoint::InvocationOccurrence {invocation_occurrence:v})
            if v.source==invocation.source && v.source_hash==invocation.source_hash
                && v.start==invocation.expression.start && v.end==invocation.expression.end)
            })
            .collect();
        assert_eq!(call_roots.len(), 1, "stage=exact-occurrence");
        let object = &arguments[1];
        let ExpressionValue::Object(fields) = &object.value else {
            continue;
        };
        assert!(
            authority.admits_demanded_structure(
                &invocation.source,
                &invocation.source_hash,
                object
            ),
            "stage=argument-object"
        );
        let providers: Vec<_> = fields
            .iter()
            .filter(|(name, _)| name == "providers")
            .collect();
        if providers.is_empty() {
            continue;
        }
        inline_count += 1;
        assert_eq!(providers.len(), 1);
        let values = &providers[0].1;
        assert!(
            authority.admits_demanded_structure(
                &invocation.source,
                &invocation.source_hash,
                values
            ),
            "stage=providers-field"
        );
        assert!(
            invocation.expression.start <= object.start
                && object.start < values.start
                && values.end < object.end
                && object.end <= invocation.expression.end
        );
        let ExpressionValue::Array(entries) = &values.value else {
            panic!("stage=array")
        };
        assert_eq!(entries.len(), 3);
        let ExpressionValue::Reference(Resolution::Resolved(left)) = &entries[0].value else {
            panic!("stage=left-token")
        };
        let ExpressionValue::Object(entry) = &entries[1].value else {
            panic!("stage=provider-entry")
        };
        assert!(authority.admits_demanded_structure(
            &invocation.source,
            &invocation.source_hash,
            &entries[1]
        ));
        let provide = &entry.iter().find(|(name, _)| name == "provide").unwrap().1;
        let ExpressionValue::Reference(Resolution::Resolved(right)) = &provide.value else {
            panic!("stage=right-token")
        };
        assert_ne!(left, right);
        for (token, occurrence, suffix) in [
            (left, &entries[0], "left-token.ts"),
            (right, provide, "right-token.ts"),
        ] {
            assert_eq!(
                identity(&token.source.to_string_lossy()),
                identity(&p.join(suffix).to_string_lossy())
            );
            assert_eq!(
                authority.demanded_reference_at(
                    &invocation.source,
                    &invocation.source_hash,
                    occurrence.start,
                    occurrence.end
                ),
                Some(token)
            );
        }
    }
    assert_eq!(inline_count, 2, "stage=two-inline-occurrences");
    let public = query(
        state,
        p,
        "workspace_query",
        json!({"type":"forward_edges","domain":"angular","entity_type":"ApplicationBootstrap","name":"bootstrapApplication"}),
    );
    let own: Vec<_> = public["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| {
            identity(e["subject"]["file"].as_str().unwrap())
                == identity(&p.join(&file).to_string_lossy())
        })
        .collect();
    assert_eq!(
        own.iter()
            .filter(|e| e["relation"] == "DeclaresBootstrapRoot")
            .count(),
        4,
        "stage=public-root-independence"
    );
    assert_eq!(
        own.iter()
            .filter(|e| e["relation"] == "UsesApplicationConfig")
            .count(),
        1,
        "stage=no-inline-config-declaration"
    );
}
fn run(form: &str) {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state) = setup(form);
    let p = root.path();
    // Nonempty existing owner/provider controls precede new owner expectations.
    for side in ["left", "right"] {
        capture(&state, p, &format!("{side}-root.ts"));
        capture(&state, p, &format!("{side}-config.ts"));
    }
    for (kind, name, suffix) in [
        ("Component", "Root", "root"),
        ("ApplicationConfig", "appConfig", "config"),
    ] {
        let answer = query(
            &state,
            p,
            "workspace_query",
            json!({"type":"forward_edges","domain":"angular","entity_type":kind,"name":name}),
        );
        let mut actual = answer["edges"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["relation"] == "RegistersProvider")
            .map(|e| {
                (
                    identity(e["subject"]["file"].as_str().unwrap()),
                    identity(e["object"]["file"].as_str().unwrap()),
                )
            })
            .collect::<Vec<_>>();
        let mut wanted = Vec::new();
        for side in ["left", "right"] {
            for token in ["left", "right"] {
                wanted.push((
                    identity(&p.join(format!("{side}-{suffix}.ts")).to_string_lossy()),
                    identity(&p.join(format!("{token}-token.ts")).to_string_lossy()),
                ));
            }
        }
        actual.sort();
        wanted.sort();
        assert_eq!(actual, wanted, "stage=existing-provider-control; {kind}");
    }
    for side in ["left", "right"] {
        prerequisites(&state, p, side);
    }
    let answer = query(
        &state,
        p,
        "workspace_query",
        json!({"type":"forward_edges","domain":"angular","entity_type":"ApplicationBootstrap","name":"bootstrapApplication"}),
    );
    let selected = answer["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["relation"] == "RegistersProvider")
        .collect::<Vec<_>>();
    let mut actual = selected
        .iter()
        .map(|e| {
            (
                identity(e["subject"]["file"].as_str().unwrap()),
                identity(e["object"]["file"].as_str().unwrap()),
            )
        })
        .collect::<Vec<_>>();
    let mut wanted = Vec::new();
    for side in ["left", "right"] {
        for _ in 0..2 {
            for token in ["left", "right"] {
                wanted.push((
                    identity(&p.join(format!("{side}-main.ts")).to_string_lossy()),
                    identity(&p.join(format!("{token}-token.ts")).to_string_lossy()),
                ));
            }
        }
    }
    actual.sort();
    wanted.sort();
    assert_eq!(
        actual, wanted,
        "stage=inline-membership; form={form}; {answer}"
    );
    assert!(
        selected
            .iter()
            .all(|e| e["subject"]["entity_type"] == "ApplicationBootstrap"
                && e["object"]["entity_type"] == "ProviderToken"
                && e["subject"].get("canonical").is_none()
                && e["object"].get("canonical").is_none())
    );
    let repeated = query(
        &state,
        p,
        "workspace_query",
        json!({"type":"forward_edges","domain":"angular","entity_type":"ApplicationBootstrap","name":"bootstrapApplication"}),
    );
    assert_eq!(
        answer["edges"], repeated["edges"],
        "stage=repeated-public-answer"
    );
}
#[test]
fn issue_162_bootstrap_inline_provider_direct() {
    run("direct");
}
#[test]
fn issue_162_bootstrap_inline_provider_alias() {
    run("alias");
}
#[test]
fn issue_162_bootstrap_inline_provider_barrel() {
    run("barrel");
}
#[test]
fn issue_162_bootstrap_inline_provider_namespace() {
    run("namespace");
}

#[test]
fn issue_162_bootstrap_inline_provider_existing_refusals_and_named_preservation() {
    let _serial = crate::protocol::handler_response_serial();
    for (prefix, expression, roots, configs) in [
        (
            "import {bootstrapApplication as launch} from '@angular/platform-browser';",
            "launch(Root)",
            1,
            0,
        ),
        (
            "import {bootstrapApplication as launch} from '@angular/platform-browser';",
            "launch(Root,{other:1})",
            1,
            0,
        ),
        (
            "import {bootstrapApplication as launch} from '@angular/platform-browser';",
            "launch(Root,appConfig)",
            1,
            1,
        ),
        (
            "import {bootstrapApplication as launch} from '@angular/platform-browser';",
            "launch(...[Root],{providers:[Left]})",
            0,
            0,
        ),
        (
            "import {bootstrapApplication as launch} from '@angular/platform-browser';",
            "launch(Root,...[{providers:[Left]}])",
            1,
            0,
        ),
        (
            "function launch(root:unknown,config:unknown){}",
            "launch(Root,{providers:[Left]})",
            0,
            0,
        ),
        (
            "import type {bootstrapApplication as launch} from '@angular/platform-browser';",
            "launch(Root,{providers:[Left]})",
            0,
            0,
        ),
        (
            "import {start as launch} from './missing-api';",
            "launch(Root,{providers:[Left]})",
            0,
            0,
        ),
    ] {
        let (root, state) = setup("direct");
        let p = root.path();
        // Finish all fixture writes before the first native capture.
        for side in ["left", "right"] {
            std::fs::write(p.join(format!("{side}-main.ts")),format!("{prefix} import {{Root}} from './{side}-root'; import {{appConfig}} from './{side}-config'; import {{Token as Left}} from './left-token'; {expression};")).unwrap();
        }
        for side in ["left", "right"] {
            let authority = capture(&state, p, &format!("{side}-main.ts"));
            let edges = crate::angular_meta::bootstrap_references::project(&authority);
            assert_eq!(
                edges
                    .iter()
                    .filter(|e| e.relation == SemanticRelation::DeclaresBootstrapRoot)
                    .count(),
                roots,
                "{expression}"
            );
            assert_eq!(
                edges
                    .iter()
                    .filter(|e| e.relation == SemanticRelation::UsesApplicationConfig)
                    .count(),
                configs,
                "{expression}"
            );
            if let Some(root_edge) = edges
                .iter()
                .find(|e| e.relation == SemanticRelation::DeclaresBootstrapRoot)
            {
                let mut borrowed = root_edge.clone();
                borrowed.relation = SemanticRelation::RegistersProvider;
                borrowed.object.entity_type = "ProviderToken";
                assert!(
                    !authority.admits_relation_evidence(&[borrowed]),
                    "valid call/root cannot supply provider entry authority"
                );
            }
        }
        let answer = query(
            &state,
            p,
            "workspace_query",
            json!({"type":"forward_edges","domain":"angular","entity_type":"ApplicationBootstrap","name":"bootstrapApplication"}),
        );
        assert!(
            !answer["edges"]
                .as_array()
                .unwrap()
                .iter()
                .any(|e| e["relation"] == "RegistersProvider"),
            "{expression}: {answer}"
        );
    }
}
