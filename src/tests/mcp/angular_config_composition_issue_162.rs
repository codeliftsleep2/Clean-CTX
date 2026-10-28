//! Investigation hypotheses only: no merge result role or relation is production-approved.
use crate::ir::typescript_resolution::declaration_facets::ExpressionValue;
use crate::ir::typescript_resolution::demanded_occurrences::DemandedValue;
use crate::ir::typescript_resolution::{Namespace, Resolution, TypeScriptAuthority};
use crate::layers::meta::semantic::SemanticRelation;
use serde_json::{Value, json};
use std::path::Path;
fn call(state: &crate::mcp::McpState, p: &Path, tool: &str, mut args: Value) -> Value {
    args["workspaceRoot"] = json!(p);
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(&json!(162), tool, &json!({"arguments":args}), state);
    let response = crate::protocol::captured_responses().pop().unwrap();
    assert!(response.get("error").is_none(), "{response}");
    assert_ne!(response["result"]["isError"], true, "{response}");
    response["result"]["structuredContent"].clone()
}
fn capture(state: &crate::mcp::McpState, p: &Path, file: &str) -> TypeScriptAuthority {
    call(
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
fn key(file: &str) -> String {
    crate::dictionary::path::canonical_identity_key(file)
}
fn setup(form: &str, repeated: bool) -> (tempfile::TempDir, crate::mcp::McpState) {
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install(p);
    std::fs::write(p.join("node_modules/@angular/core/index.d.ts"),"export declare function Component(metadata:unknown):any; export interface ApplicationConfig {providers:unknown[];} export declare function mergeApplicationConfig(...configs:ApplicationConfig[]):ApplicationConfig;").unwrap();
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
        "export {mergeApplicationConfig as combine} from '@angular/core';",
    )
    .unwrap();
    let (import, callee) = match form {
        "direct" => (
            "import {mergeApplicationConfig} from '@angular/core';",
            "mergeApplicationConfig",
        ),
        "alias" => (
            "import {mergeApplicationConfig as combine} from '@angular/core';",
            "combine",
        ),
        "barrel" => ("import {combine} from './api';", "combine"),
        "namespace" => (
            "import * as core from '@angular/core';",
            "core.mergeApplicationConfig",
        ),
        _ => panic!(),
    };
    for side in ["left", "right"] {
        std::fs::write(p.join(format!("{side}-token.ts")), "export class Token {}").unwrap();
        for part in ["base", "feature"] {
            std::fs::write(p.join(format!("{side}-{part}.ts")),format!("import type {{ApplicationConfig}} from '@angular/core'; import {{Token}} from './{side}-token'; export const leafConfig:ApplicationConfig={{providers:[Token]}};")).unwrap();
        }
        std::fs::write(
            p.join(format!("{side}-root.ts")),
            "import {Component} from '@angular/core'; @Component({}) export class Root {}",
        )
        .unwrap();
        let second = if repeated { "Base" } else { "Feature" };
        std::fs::write(p.join(format!("{side}-result.ts")),format!("{import} import type {{ApplicationConfig}} from '@angular/core'; import {{bootstrapApplication}} from '@angular/platform-browser'; import {{Root}} from './{side}-root'; import {{leafConfig as Base}} from './{side}-base'; import {{leafConfig as Feature}} from './{side}-feature'; export const appConfig:ApplicationConfig={callee}(Base,{second}); bootstrapApplication(Root,appConfig);")).unwrap();
    }
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    (root, crate::mcp::McpState::new(config))
}
fn prerequisites(state: &crate::mcp::McpState, p: &Path, side: &str, repeated: bool) {
    let authority = capture(state, p, &format!("{side}-result.ts"));
    let facets = authority
        .declaration_facets
        .iter()
        .filter(|f| f.declaration.name == "appConfig" && f.declaration.source == authority.source)
        .collect::<Vec<_>>();
    assert_eq!(facets.len(), 1, "stage=result-declaration");
    let facet = facets[0];
    let id = &facet.declaration;
    let annotation = facet.type_annotation.as_ref().expect("stage=result-type");
    let initializer = facet
        .initializer
        .as_ref()
        .expect("stage=result-initializer");
    assert!(
        matches!(authority.demanded_at(&id.source,&id.source_hash,id.start,id.end),Some(DemandedValue::Declaration{id:actual,is_const:true,annotation:Some(a),initializer:Some(i),..}) if actual==id && *a==(annotation.start,annotation.end) && *i==(initializer.start,initializer.end)),
        "stage=exact-result-type-initializer-association"
    );
    let ExpressionValue::Reference(Resolution::Resolved(ty)) = &annotation.value else {
        panic!("stage=native-result-type")
    };
    assert!(crate::angular_meta::class_role_authority::public_entry_api(
        &authority,
        ty,
        "@angular/core",
        "@angular/core",
        "ApplicationConfig",
        Namespace::Type
    ));
    assert!(
        matches!(authority.demanded_at(&id.source,&id.source_hash,annotation.start,annotation.end),Some(DemandedValue::TypeReference(Resolution::Resolved(actual))) if actual==ty),
        "stage=type-occurrence-join"
    );
    let ExpressionValue::Call(callee, args) = &initializer.value else {
        panic!("stage=call-initializer")
    };
    let ExpressionValue::Reference(Resolution::Resolved(api)) = &callee.value else {
        panic!("stage=native-merge-api")
    };
    assert!(
        crate::angular_meta::class_role_authority::public_entry_api(
            &authority,
            api,
            "@angular/core",
            "@angular/core",
            "mergeApplicationConfig",
            Namespace::Value
        ),
        "stage=native-merge-api"
    );
    assert_eq!(
        authority.demanded_reference_at(&id.source, &id.source_hash, callee.start, callee.end),
        Some(api)
    );
    assert!(
        matches!(authority.demanded_at(&id.source,&id.source_hash,initializer.start,initializer.end),Some(DemandedValue::Structure{kind,children}) if kind=="call"&&children==&vec![("callee".into(),callee.start,callee.end),("argument:0".into(),args[0].start,args[0].end),("argument:1".into(),args[1].start,args[1].end)]),
        "stage=call-argument-structure"
    );
    assert_eq!(args.len(), 2);
    assert!(args[0].end <= args[1].start);
    let mut targets = Vec::new();
    for (position, arg) in args.iter().enumerate() {
        let ExpressionValue::Reference(Resolution::Resolved(target)) = &arg.value else {
            panic!("stage=input-reference")
        };
        assert_eq!(
            authority.demanded_reference_at(&id.source, &id.source_hash, arg.start, arg.end),
            Some(target)
        );
        let part = if position == 0 || repeated {
            "base"
        } else {
            "feature"
        };
        assert_eq!(
            key(&target.source.to_string_lossy()),
            key(&p.join(format!("{side}-{part}.ts")).to_string_lossy())
        );
        assert_ne!(target, id);
        assert!(authority.declaration_facets.iter().any(|f| {
            f.declaration == *target
                && f.initializer
                    .as_ref()
                    .is_some_and(|v| matches!(v.value, ExpressionValue::Object(_)))
        }));
        targets.push(target);
    }
    assert_eq!(
        targets[0] == targets[1],
        repeated,
        "same endpoints do not collapse argument positions"
    );
    let roots = crate::angular_meta::bootstrap_references::project(&authority);
    assert_eq!(
        roots
            .iter()
            .filter(|e| e.relation == SemanticRelation::DeclaresBootstrapRoot)
            .count(),
        1,
        "stage=native-bootstrap-root"
    );
    assert!(authority.admits_relation_evidence(&roots));
    let public = call(
        state,
        p,
        "workspace_query",
        json!({"type":"forward_edges","domain":"angular","entity_type":"ApplicationBootstrap","name":"bootstrapApplication"}),
    );
    assert!(
        public["edges"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["relation"] == "DeclaresBootstrapRoot"
                && key(e["subject"]["file"].as_str().unwrap())
                    == key(&p.join(format!("{side}-result.ts")).to_string_lossy())),
        "stage=public-root"
    );
}
fn run(form: &str, bootstrap: bool, repeated: bool) {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state) = setup(form, repeated);
    let p = root.path();
    for side in ["left", "right"] {
        for part in ["base", "feature"] {
            let authority = capture(&state, p, &format!("{side}-{part}.ts"));
            assert!(crate::angular_meta::application_config_providers::has_owner(&authority));
            assert!(
                !crate::angular_meta::application_config_providers::project(&authority).is_empty()
            );
        }
    }
    let leaves = call(
        &state,
        p,
        "workspace_query",
        json!({"type":"forward_edges","domain":"angular","entity_type":"ApplicationConfig","name":"leafConfig"}),
    );
    assert_eq!(
        leaves["edges"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["relation"] == "RegistersProvider")
            .count(),
        4,
        "stage=leaf-provider-controls"
    );
    for side in ["left", "right"] {
        prerequisites(&state, p, side, repeated);
    }
    let answer = call(
        &state,
        p,
        "workspace_query",
        json!({"type":"forward_edges","domain":"angular","entity_type":if bootstrap{"ApplicationBootstrap"}else{"ApplicationConfig"},"name":if bootstrap{"bootstrapApplication"}else{"appConfig"}}),
    );
    let relation = if bootstrap {
        "UsesApplicationConfig"
    } else {
        "IncludesApplicationConfig"
    };
    let mut rows = answer["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["relation"] == relation)
        .map(|e| {
            (
                key(e["subject"]["file"].as_str().unwrap()),
                key(e["object"]["file"].as_str().unwrap()),
            )
        })
        .collect::<Vec<_>>();
    rows.sort();
    let mut wanted = Vec::new();
    for side in ["left", "right"] {
        let result = key(&p.join(format!("{side}-result.ts")).to_string_lossy());
        if bootstrap {
            wanted.push((result.clone(), result));
        } else if !repeated {
            for part in ["base", "feature"] {
                wanted.push((
                    result.clone(),
                    key(&p.join(format!("{side}-{part}.ts")).to_string_lossy()),
                ));
            }
        }
    }
    wanted.sort();
    if repeated {
        // Investigation probe: retain two exact argument positions; do not freeze a
        // public one-row/two-row policy before owner disposition of the observed carrier.
        assert!(
            !rows.is_empty(),
            "stage=config-composition; repeated-input public multiplicity unresolved; {answer}"
        );
        assert!(rows.iter().all(|(owner, target)| {
            ["left", "right"].iter().any(|side| {
                *owner == key(&p.join(format!("{side}-result.ts")).to_string_lossy())
                    && *target == key(&p.join(format!("{side}-base.ts")).to_string_lossy())
            })
        }));
    } else {
        assert_eq!(
            rows,
            wanted,
            "stage={}; form={form}; {answer}",
            if bootstrap {
                "merged-bootstrap-reference"
            } else {
                "config-composition"
            }
        );
    }
    assert!(
        !answer["edges"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["relation"] == "RegistersProvider"),
        "no flattened providers"
    );
}
#[test]
fn issue_162_config_composition_direct() {
    run("direct", false, false);
}
#[test]
fn issue_162_config_composition_alias() {
    run("alias", false, false);
}
#[test]
fn issue_162_config_composition_barrel() {
    run("barrel", false, false);
}
#[test]
fn issue_162_config_composition_namespace() {
    run("namespace", false, false);
}
#[test]
fn issue_162_config_composition_bootstrap_direct() {
    run("direct", true, false);
}
#[test]
fn issue_162_config_composition_bootstrap_alias() {
    run("alias", true, false);
}
#[test]
fn issue_162_config_composition_bootstrap_barrel() {
    run("barrel", true, false);
}
#[test]
fn issue_162_config_composition_bootstrap_namespace() {
    run("namespace", true, false);
}
#[test]
fn issue_162_config_composition_repeated_input_probe() {
    run("direct", false, true);
}
