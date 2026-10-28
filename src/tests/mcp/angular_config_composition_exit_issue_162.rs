//! Supplemental named-config approval obligations; no evaluated merge semantics.
use super::angular_bootstrap_inline_controls_tests::{call, capture, raw};
use super::angular_config_composition_controls_tests::{NATIVE, facts, setup};
use serde_json::json;
const IMPORT: &str = "import {mergeApplicationConfig as combine} from '@angular/core';";
#[test]
fn issue_162_config_composition_exit_api_type_and_input_refusals() {
    let _serial = crate::protocol::handler_response_serial();
    for mode in 0..11 {
        let (root, config) = setup("Base,Feature");
        let p = root.path();
        std::fs::write(
            p.join("ambiguous-api.ts"),
            "export * from '@angular/core'; export * from './fake-merge';",
        )
        .unwrap();
        std::fs::write(
            p.join("ambiguous-input.ts"),
            "export * from './base'; export * from './feature';",
        )
        .unwrap();
        std::fs::write(
            p.join("fake-type.ts"),
            "export interface ApplicationConfig {providers:unknown[];}",
        )
        .unwrap();
        let mut prefix = NATIVE.to_owned();
        let mut declaration = "export const appConfig:ApplicationConfig";
        let mut initializer = "=combine(Base,Feature)";
        match mode {
            0 => prefix = prefix.replace(IMPORT, "function combine(...args:unknown[]){return {};}"),
            1 => {
                prefix = prefix.replace(
                    IMPORT,
                    "import type {mergeApplicationConfig as combine} from '@angular/core';",
                )
            }
            2 => {
                prefix = prefix.replace(
                    IMPORT,
                    "import {mergeApplicationConfig as combine} from './missing-api';",
                )
            }
            3 => {
                prefix = prefix.replace(
                    IMPORT,
                    "import {mergeApplicationConfig as combine} from './ambiguous-api';",
                )
            }
            4 => {
                prefix = prefix.replace(
                    "import type {ApplicationConfig} from '@angular/core';",
                    "import type {ApplicationConfig} from './fake-type';",
                )
            }
            5 => initializer = "",
            6 => {
                prefix.push_str(" import {leafConfig as Ambiguous} from './ambiguous-input';");
                initializer = "=combine(Base,Ambiguous)";
            }
            7 => {
                prefix.push_str(" const NotConfig={providers:[]};");
                initializer = "=combine(Base,NotConfig)";
            }
            8 => {
                prefix.push_str(" function scoped(combine:unknown){ const appConfig:ApplicationConfig=combine(Base,Feature); }");
                declaration = "export const appConfig:ApplicationConfig";
                initializer = "={providers:[]}";
            }
            9 => {
                prefix.push_str(" const nested:ApplicationConfig=combine(Base,Feature);");
                initializer = "=combine(Base,nested)";
            }
            _ => {
                prefix.push_str(" const Alias=Base;");
                initializer = "=combine(Base,Alias)";
            }
        }
        std::fs::write(
            p.join("result.ts"),
            format!("{prefix} {declaration}{initializer};"),
        )
        .unwrap();
        let state = crate::mcp::McpState::new(config);
        facts(
            &state,
            p,
            if matches!(mode, 6 | 7 | 9 | 10) {
                &["base.ts"]
            } else {
                &[]
            },
        );
        if mode == 9 {
            let answer = call(
                &state,
                p,
                "workspace_query",
                json!({"type":"forward_edges","domain":"angular","entity_type":"ApplicationConfig","name":"nested"}),
            );
            let mut targets = answer["edges"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|e| {
                    e["relation"] == "IncludesApplicationConfig" && e["subject"]["name"] == "nested"
                })
                .map(|e| {
                    crate::dictionary::path::canonical_identity_key(
                        e["object"]["file"].as_str().unwrap(),
                    )
                })
                .collect::<Vec<_>>();
            targets.sort();
            let mut expected = ["base.ts", "feature.ts"]
                .iter()
                .map(|f| {
                    crate::dictionary::path::canonical_identity_key(&p.join(f).to_string_lossy())
                })
                .collect::<Vec<_>>();
            expected.sort();
            assert_eq!(
                targets, expected,
                "independent nested declaration inputs survive"
            );
        }
    }
}
#[test]
fn issue_162_config_composition_exit_duplicate_and_cross_source_native_proof() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("Base,Feature");
    let p = root.path();
    let state = crate::mcp::McpState::new(config);
    let authority = capture(&state, p, "result.ts");
    let edges = crate::angular_meta::config_composition::project(&authority);
    assert_eq!(edges.len(), 2);
    let result = authority
        .declaration_facets
        .iter()
        .find(|f| f.declaration.name == "appConfig")
        .unwrap();
    let initializer = result.initializer.as_ref().unwrap();
    let annotation = result.type_annotation.as_ref().unwrap();
    for mode in 0..7 {
        let mut bad = authority.clone();
        if mode == 0 {
            bad.declaration_facets.push(result.clone());
        } else {
            let (start, end) = if mode < 4 {
                (initializer.start, initializer.end)
            } else {
                (annotation.start, annotation.end)
            };
            let i = bad
                .demanded_occurrences
                .iter()
                .position(|v| v.source == authority.source && v.start == start && v.end == end)
                .unwrap();
            match mode {
                1 => {
                    let duplicate = bad.demanded_occurrences[i].clone();
                    bad.demanded_occurrences.push(duplicate);
                }
                2 | 4 => {
                    bad.demanded_occurrences.remove(i);
                }
                3 | 5 => bad.demanded_occurrences[i].source_hash.push('x'),
                _ => bad.demanded_occurrences[i].source.push("crossed.ts"),
            }
        }
        assert!(!bad.admits_relation_evidence(&edges), "mode={mode}");
    }
    let first = call(
        &state,
        p,
        "workspace_query",
        json!({"type":"forward_edges","domain":"angular","entity_type":"ApplicationConfig","name":"appConfig"}),
    );
    let second = call(
        &state,
        p,
        "workspace_query",
        json!({"type":"forward_edges","domain":"angular","entity_type":"ApplicationConfig","name":"appConfig"}),
    );
    assert_eq!(first["edges"], second["edges"]);
}
#[test]
fn issue_162_config_composition_exit_api_barrel_package_config_stale_repair() {
    let _serial = crate::protocol::handler_response_serial();
    for mode in 0..4 {
        let (root, config) = setup("Base,Feature");
        let p = root.path();
        std::fs::write(
            p.join("merge-api.ts"),
            "export {mergeApplicationConfig as combine} from '@angular/core';",
        )
        .unwrap();
        let source = format!(
            "{} export const appConfig:ApplicationConfig=combine(Base,Feature);",
            NATIVE.replace(IMPORT, "import {combine} from './merge-api';")
        );
        std::fs::write(p.join("result.ts"), &source).unwrap();
        let state = crate::mcp::McpState::new(config.clone());
        capture(&state, p, "result.ts");
        facts(&state, p, &["base.ts", "feature.ts"]);
        call(
            &state,
            p,
            "save_context",
            json!({"filePath":p.join("result.ts")}),
        );
        drop(state);
        let (file, replacement) = match mode {
            0 => (
                "merge-api.ts",
                "export {mergeApplicationConfig as combine} from './fake-merge';",
            ),
            1 => (
                "node_modules/@angular/core/package.json",
                r#"{"name":"@angular/core","version":"20.0.0","exports":{".":null}}"#,
            ),
            2 => (
                "tsconfig.json",
                r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","noLib":true,"types":[],"paths":{"@angular/core":["./fake-merge.ts"]}}}"#,
            ),
            _ => (
                "node_modules/@angular/core/index.d.ts",
                "export declare function Component(metadata:unknown):any; export interface ApplicationConfig {providers:unknown[];}",
            ),
        };
        let original = std::fs::read(p.join(file)).unwrap();
        std::fs::write(p.join(file), replacement).unwrap();
        let state = crate::mcp::McpState::new(config);
        let rejected = raw(
            &state,
            p,
            "restore_context",
            json!({"filePath":p.join("result.ts")}),
        );
        assert!(
            rejected.get("error").is_some() || rejected["result"]["isError"] == true,
            "{rejected}"
        );
        facts(&state, p, &[]);
        std::fs::write(p.join(file), original).unwrap();
        facts(&state, p, &["base.ts", "feature.ts"]);
        assert_eq!(
            std::fs::read_to_string(p.join("result.ts")).unwrap(),
            source
        );
    }
}
#[test]
fn issue_162_config_composition_exit_independent_caller_result_retirement() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("Base,Feature");
    let p = root.path();
    let state = crate::mcp::McpState::new(config);
    facts(&state, p, &["base.ts", "feature.ts"]);
    std::fs::remove_file(p.join("caller.ts")).unwrap();
    call(&state, p, "refresh_workspace", json!({}));
    facts(&state, p, &["base.ts", "feature.ts"]);
    let feature = std::fs::read(p.join("feature.ts")).unwrap();
    std::fs::remove_file(p.join("feature.ts")).unwrap();
    call(&state, p, "refresh_workspace", json!({}));
    facts(&state, p, &["base.ts"]);
    std::fs::write(p.join("feature.ts"), feature).unwrap();
    facts(&state, p, &["base.ts", "feature.ts"]);
    let original_result = std::fs::read(p.join("result.ts")).unwrap();
    std::fs::write(p.join("result.ts"), "export const Replaced=1;").unwrap();
    call(&state, p, "refresh_workspace", json!({}));
    facts(&state, p, &[]);
    std::fs::remove_file(p.join("result.ts")).unwrap();
    call(&state, p, "refresh_workspace", json!({}));
    facts(&state, p, &[]);
    // External new-path creation requires the existing refresh notification.
    std::fs::write(p.join("result.ts"), original_result).unwrap();
    call(&state, p, "refresh_workspace", json!({}));
    facts(&state, p, &["base.ts", "feature.ts"]);
}
