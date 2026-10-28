//! Written direct-cycle investigation; no runtime initialization validity claim.
use super::angular_bootstrap_inline_controls_tests::{call, capture};
use super::angular_config_composition_controls_tests::{NATIVE, setup};
use super::angular_nested_config_input_tests::role;
use serde_json::json;
use std::path::Path;
fn pairs(state: &crate::mcp::McpState, p: &Path) -> Vec<(String, String)> {
    let mut result = Vec::new();
    for name in ["A", "B"] {
        let answer = call(
            state,
            p,
            "workspace_query",
            json!({"type":"forward_edges","domain":"angular","entity_type":"ApplicationConfig","name":name}),
        );
        for edge in answer["edges"].as_array().unwrap() {
            if edge["relation"] == "IncludesApplicationConfig" {
                result.push((
                    edge["subject"]["name"].as_str().unwrap().to_owned(),
                    edge["object"]["name"].as_str().unwrap().to_owned(),
                ));
            }
        }
    }
    result.sort();
    result
}
fn run(mode: &str) {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("Base,Feature");
    let p = root.path();
    let (a,b)=match mode {
 "self"=>(format!("{NATIVE} export const A:ApplicationConfig=combine(A,Base);"),None),
 "same-file"=>(format!("{NATIVE} export const A:ApplicationConfig=combine(B,Base); export const B:ApplicationConfig=combine(A,Feature);"),None),
 "multi-file"=>(format!("{NATIVE} import {{B}} from './cycle-b'; export const A:ApplicationConfig=combine(B,Base);"),Some(format!("{NATIVE} import {{A}} from './cycle-a'; export const B:ApplicationConfig=combine(A,Feature);"))),
 "foreign-producer"=>(format!("{NATIVE} import {{B}} from './cycle-b'; export const A:ApplicationConfig=combine(B,Base);"),Some("import type {ApplicationConfig} from '@angular/core'; import {mergeApplicationConfig} from './fake-merge'; import {A} from './cycle-a'; export const B:ApplicationConfig=mergeApplicationConfig(A);".into())),_=>panic!()};
    std::fs::write(p.join("cycle-a.ts"), &a).unwrap();
    if let Some(b) = &b {
        std::fs::write(p.join("cycle-b.ts"), b).unwrap();
    }
    let state = crate::mcp::McpState::new(config);
    let control = capture(&state, p, "result.ts");
    assert_eq!(
        crate::angular_meta::config_composition::project(&control).len(),
        2,
        "stage=independent-leaf-control"
    );
    let authority = capture(&state, p, "cycle-a.ts");
    let first = role(&authority, "A", &authority.source);
    let expression = first.initializer.as_ref().unwrap();
    let crate::ir::typescript_resolution::declaration_facets::ExpressionValue::Call(_, args) =
        &expression.value
    else {
        panic!()
    };
    let crate::ir::typescript_resolution::declaration_facets::ExpressionValue::Reference(
        crate::ir::typescript_resolution::Resolution::Resolved(target),
    ) = &args[0].value
    else {
        panic!("stage=cycle-reference")
    };
    let expected_name = if mode == "self" { "A" } else { "B" };
    assert_eq!(target.name, expected_name);
    assert_eq!(
        authority.demanded_reference_at(
            &first.declaration.source,
            &first.declaration.source_hash,
            args[0].start,
            args[0].end
        ),
        Some(target)
    );
    if mode != "foreign-producer" {
        let other = role(&authority, expected_name, &target.source);
        assert_eq!(other.declaration, *target, "stage=cycle-exact-target-role");
    }
    if mode == "multi-file" {
        let b = capture(&state, p, "cycle-b.ts");
        role(&b, "B", &b.source);
    }
    let mut expected = vec![("A".to_owned(), "leafConfig".to_owned())];
    if mode == "self" {
        expected.push(("A".into(), "A".into()));
    } else if mode != "foreign-producer" {
        expected.extend([
            ("A".into(), "B".into()),
            ("B".into(), "A".into()),
            ("B".into(), "leafConfig".into()),
        ]);
    }
    expected.sort();
    let actual = pairs(&state, p);
    assert_eq!(actual, expected, "stage=written-cycle; mode={mode}");
    assert_eq!(pairs(&state, p), actual, "stage=cycle-repeated-query");
    if mode == "multi-file" {
        let original = std::fs::read(p.join("cycle-b.ts")).unwrap();
        std::fs::write(
            p.join("cycle-b.ts"),
            format!("{NATIVE} export const B:ApplicationConfig=combine(Feature);"),
        )
        .unwrap();
        let mut broken = expected.clone();
        broken.retain(|(subject, target)| !(subject == "B" && target == "A"));
        assert_eq!(pairs(&state, p), broken, "stage=broken-written-edge");
        std::fs::write(p.join("cycle-b.ts"), original).unwrap();
        assert_eq!(pairs(&state, p), expected, "stage=restored-written-edge");
    }
}
#[test]
fn issue_162_nested_config_cycle_self() {
    run("self");
}
#[test]
fn issue_162_nested_config_cycle_same_file() {
    run("same-file");
}
#[test]
fn issue_162_nested_config_cycle_multi_file() {
    run("multi-file");
}
#[test]
fn issue_162_nested_config_cycle_foreign_producer() {
    run("foreign-producer");
}
