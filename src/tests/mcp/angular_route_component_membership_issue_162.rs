//! Investigation only: exact eager RouteOccurrence membership is a test hypothesis.
use super::angular_bootstrap_inline_controls_tests::{call, capture};
use crate::ir::typescript_resolution::declaration_facets::{ExpressionFacet, ExpressionValue};
use crate::ir::typescript_resolution::demanded_occurrences::DemandedValue;
use crate::ir::typescript_resolution::{Namespace, Resolution, TypeScriptAuthority};
use crate::layers::meta::canonical_endpoint::CanonicalEndpoint;
use serde_json::{Value, json};
use std::path::Path;
fn key(p: &Path) -> String {
    crate::dictionary::path::canonical_identity_key(&p.to_string_lossy())
}
fn setup(form: &str) -> (tempfile::TempDir, crate::mcp::McpState) {
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install(p);
    std::fs::write(
        p.join("node_modules/@angular/core/index.d.ts"),
        "export declare function Component(metadata:unknown):any;",
    )
    .unwrap();
    let router = p.join("node_modules/@angular/router");
    std::fs::create_dir_all(&router).unwrap();
    std::fs::write(
        router.join("package.json"),
        r#"{"name":"@angular/router","version":"20.0.0","types":"index.d.ts"}"#,
    )
    .unwrap();
    std::fs::write(router.join("index.d.ts"),"export interface Route {path?:string;component?:unknown;providers?:unknown[];children?:Route[];} export type Routes=Route[];").unwrap();
    for side in ["left", "right"] {
        std::fs::write(p.join(format!("{side}-token.ts")), "export class Token {}").unwrap();
        std::fs::write(p.join(format!("{side}.ts")),format!("import {{Component}} from '@angular/core'; import {{Token}} from './{side}-token'; @Component({{providers:[Token]}}) export class Screen {{}}")).unwrap();
    }
    std::fs::write(
        p.join("targets.ts"),
        "export {Screen as Left} from './left'; export {Screen as Right} from './right';",
    )
    .unwrap();
    for side in ["one", "two"] {
        let (imports,a,b)=match form {
            "direct"=>("import {Screen} from './left'; import {Screen as Right} from './right';".to_owned(),"Screen","Right"),
            "alias"=>("import {Screen as A} from './left'; import {Screen as B} from './right';".to_owned(),"A","B"),
            "barrel"=>("import {Left as A,Right as B} from './targets';".to_owned(),"A","B"),
            "namespace"=>("import * as targets from './targets';".to_owned(),"targets.Left","targets.Right"),
            "local"=>("import {Component} from '@angular/core'; import {Screen as Right} from './right'; @Component({providers:[Token]}) class Screen {}".to_owned(),"Screen","Right"),
            _=>panic!("unknown route"),
        };
        let array = format!(
            "[{{path:'same',component:{a},providers:[Token]}},{{path:'same',component:{b}}},{{path:'same',component:{a}}},{{path:'',component:{a}}},{{path:'',component:{b}}},{{path:'parent',children:[{{path:'child',component:{a}}},{{path:'child',component:{b}}}]}}]"
        );
        std::fs::write(p.join(format!("{side}-routes.ts")),format!("import type {{Routes}} from '@angular/router'; import {{Token}} from './left-token'; {imports} export const routes:Routes={array};")).unwrap();
    }
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    (root, crate::mcp::McpState::new(config))
}
fn observe(state: &crate::mcp::McpState, p: &Path) -> Value {
    let mut forward = Vec::new();
    for name in ["same", "parent/child"] {
        let answer = call(
            state,
            p,
            "workspace_query",
            json!({"type":"forward_edges","domain":"angular","entity_type":"Route","name":name}),
        );
        forward.push(answer);
    }
    let reverse = call(
        state,
        p,
        "workspace_query",
        json!({"type":"reverse_edges","domain":"angular","entity_type":"Component","name":"Screen"}),
    );
    json!({"forward":forward,"reverse":reverse})
}
fn objects(
    authority: &TypeScriptAuthority,
    value: &ExpressionFacet,
    output: &mut Vec<(
        usize,
        usize,
        crate::ir::typescript_resolution::DeclarationId,
    )>,
    observed: &Value,
) {
    assert!(
        authority.admits_demanded_structure(&authority.source, &authority.source_hash, value),
        "stage=route-structure-prerequisite {value:?}"
    );
    match &value.value {
        ExpressionValue::Array(values) => {
            for v in values {
                objects(authority, v, output, observed);
            }
        }
        ExpressionValue::Object(fields) => {
            if let Some((_, children)) = fields.iter().find(|(name, _)| name == "children") {
                objects(authority, children, output, observed);
                return;
            }
            let candidates = fields
                .iter()
                .filter(|(name, _)| name == "component")
                .collect::<Vec<_>>();
            assert_eq!(
                candidates.len(),
                1,
                "stage=component-field-prerequisite exact retained component association missing/competing; object={value:?}; observed_public={observed}"
            );
            let target = &candidates[0].1;
            let ExpressionValue::Reference(Resolution::Resolved(id)) = &target.value else {
                panic!("stage=component-reference-prerequisite {target:?}")
            };
            assert_eq!(
                authority.demanded_reference_at(
                    &authority.source,
                    &authority.source_hash,
                    target.start,
                    target.end
                ),
                Some(id),
                "stage=component-reference-prerequisite"
            );
            assert_eq!(
                crate::angular_meta::class_role_authority::declaration_role(authority, id),
                Some(crate::angular_meta::decorators::ClassKind::Component),
                "stage=component-role-prerequisite"
            );
            assert!(
                authority.source.as_path() != id.source.as_path()
                    || authority.declarations.contains(id)
            );
            output.push((value.start, value.end, id.clone()));
        }
        _ => panic!("stage=route-object-prerequisite {value:?}"),
    }
}
fn question(form: &str) {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state) = setup(form);
    let p = root.path();
    for f in ["left.ts", "right.ts"] {
        let a = capture(&state, p, f);
        let id = a
            .declarations
            .iter()
            .find(|id| id.name == "Screen" && id.source == a.source)
            .unwrap();
        assert_eq!(
            crate::angular_meta::class_role_authority::declaration_role(&a, id),
            Some(crate::angular_meta::decorators::ClassKind::Component)
        );
    }
    let component = call(
        &state,
        p,
        "workspace_query",
        json!({"type":"forward_edges","domain":"angular","entity_type":"Component","name":"Screen"}),
    );
    assert!(
        component["edges"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["relation"] == "RegistersProvider")
            .count()
            >= 2,
        "stage=component-provider-prerequisite {component}"
    );
    let mut captured = Vec::new();
    for file in ["one-routes.ts", "two-routes.ts"] {
        let a = capture(&state, p, file);
        let root = a
            .declaration_facets
            .iter()
            .find(|f| f.declaration.name == "routes" && f.declaration.source == a.source)
            .unwrap();
        let id = &root.declaration;
        assert!(
            matches!(
                a.demanded_at(&id.source, &id.source_hash, id.start, id.end),
                Some(DemandedValue::Declaration { is_const: true, .. })
            ),
            "stage=const-root-prerequisite"
        );
        let annotation = root.type_annotation.as_ref().unwrap();
        let ExpressionValue::Reference(Resolution::Resolved(api)) = &annotation.value else {
            panic!("stage=Routes-type-prerequisite")
        };
        assert!(
            crate::angular_meta::class_role_authority::public_entry_api(
                &a,
                api,
                "@angular/router",
                "@angular/router",
                "Routes",
                Namespace::Type
            ),
            "stage=native-Routes-prerequisite"
        );
        let projection = crate::angular_meta::route_providers::project(&a);
        let providers = projection
            .iter()
            .filter(|edge| {
                edge.relation == crate::layers::meta::semantic::SemanticRelation::RegistersProvider
            })
            .collect::<Vec<_>>();
        assert_eq!(providers.len(), 1, "stage=route-provider-prerequisite");
        assert!(
            matches!(
                providers[0].subject.canonical,
                Some(CanonicalEndpoint::RouteOccurrence { .. })
            ),
            "stage=route-occurrence-prerequisite"
        );
        captured.push(a);
    }
    let observed = observe(&state, p);
    let mut wanted = Vec::new();
    let mut forward_wanted = Vec::new();
    for a in &captured {
        let root = a
            .declaration_facets
            .iter()
            .find(|f| f.declaration.name == "routes" && f.declaration.source == a.source)
            .unwrap();
        let array = root.initializer.as_ref().unwrap();
        assert!(matches!(array.value, ExpressionValue::Array(_)));
        let mut members = Vec::new();
        objects(a, array, &mut members, &observed);
        assert_eq!(members.len(), 7);
        for (position, (_, _, id)) in members.into_iter().enumerate() {
            let pair = (key(&a.source), key(&id.source));
            // Fixture positions 3/4 are the two empty-path objects. Their exact
            // source/target pairs remain required in the reverse Component bucket.
            if position != 3 && position != 4 {
                forward_wanted.push(pair.clone());
            }
            wanted.push(pair);
        }
    }
    wanted.sort();
    forward_wanted.sort();
    for direction in ["forward", "reverse"] {
        let answers = if direction == "forward" {
            observed[direction].as_array().unwrap().clone()
        } else {
            vec![observed[direction].clone()]
        };
        let mut actual = answers
            .iter()
            .flat_map(|v| v["edges"].as_array().unwrap())
            .filter(|e| e["relation"] == "RouteMapsTo")
            .map(|e| {
                (
                    key(Path::new(e["subject"]["file"].as_str().unwrap_or(""))),
                    key(Path::new(e["object"]["file"].as_str().unwrap_or(""))),
                )
            })
            .collect::<Vec<_>>();
        actual.sort();
        assert_eq!(
            &actual,
            if direction == "forward" {
                &forward_wanted
            } else {
                &wanted
            },
            "stage=eager-route-membership direction={direction}; public={observed}"
        );
    }
    let repeated = observe(&state, p);
    for i in 0..2 {
        assert_eq!(
            repeated["forward"][i]["edges"], observed["forward"][i]["edges"],
            "stable repeated forward facts"
        );
    }
    assert_eq!(
        repeated["reverse"]["edges"], observed["reverse"]["edges"],
        "stable repeated reverse facts"
    );
}
#[test]
fn issue_162_route_component_direct() {
    question("direct");
}
#[test]
fn issue_162_route_component_alias() {
    question("alias");
}
#[test]
fn issue_162_route_component_barrel() {
    question("barrel");
}
#[test]
fn issue_162_route_component_namespace() {
    question("namespace");
}
#[test]
fn issue_162_route_component_local() {
    question("local");
}

#[test]
fn issue_162_route_component_native_root_refusal_and_sibling_control() {
    let _serial = crate::protocol::handler_response_serial();
    for mode in 0..3 {
        let (root, state) = setup("alias");
        let p = root.path();
        std::fs::write(p.join("foreign-routes.ts"), "export type Routes=unknown[];").unwrap();
        let original = std::fs::read_to_string(p.join("one-routes.ts")).unwrap();
        let changed = match mode {
            0 => original.replace("from '@angular/router'", "from './foreign-routes'"),
            1 => original.replace("const routes:", "let routes:"),
            2 => original.replace("from '@angular/router'", "from './missing-router'"),
            _ => unreachable!(),
        };
        std::fs::write(p.join("one-routes.ts"), changed).unwrap();
        let refused = capture(&state, p, "one-routes.ts");
        assert!(
            crate::angular_meta::route_providers::project(&refused).is_empty(),
            "native root refusal mode={mode}"
        );
        let sibling = capture(&state, p, "two-routes.ts");
        assert_eq!(
            crate::angular_meta::route_providers::project(&sibling)
                .iter()
                .filter(|edge| edge.relation
                    == crate::layers::meta::semantic::SemanticRelation::RegistersProvider)
                .count(),
            1
        );
        // Record old routing contribution; root refusal is not a certificate for its rows.
        let observed = observe(&state, p);
        eprintln!("root-refusal mode={mode}; observed_public={observed}");
    }
}
