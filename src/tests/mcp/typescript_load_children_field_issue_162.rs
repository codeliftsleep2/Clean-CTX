//! Field capture only; Unsupported callback values grant no lazy selection proof.
use super::angular_route_component_support::{call, capture, setup};
use crate::ir::typescript_resolution::{
    TypeScriptAuthority,
    declaration_facets::{ExpressionFacet, ExpressionValue},
};
use crate::layers::meta::semantic::SemanticRelation;
use serde_json::json;
fn walk<'a>(
    a: &TypeScriptAuthority,
    v: &'a ExpressionFacet,
    out: &mut Vec<(usize, usize, &'a ExpressionFacet)>,
) {
    assert!(a.admits_demanded_structure(&a.source, &a.source_hash, v));
    match &v.value {
        ExpressionValue::Array(values) => {
            for value in values {
                walk(a, value, out);
            }
        }
        ExpressionValue::Object(fields) => {
            for (name, value) in fields {
                if name == "loadChildren" {
                    assert!(
                        v.start <= value.start && value.start < value.end && value.end <= v.end
                    );
                    out.push((v.start, v.end, value));
                } else if name == "children" {
                    walk(a, value, out);
                }
            }
        }
        _ => panic!("ordinary field owning Object/Array required"),
    }
}
fn retained(a: &TypeScriptAuthority) -> Vec<(usize, usize, usize, usize)> {
    let root = a
        .declaration_facets
        .iter()
        .find(|f| f.declaration.source == a.source && f.declaration.name == "routes")
        .unwrap();
    assert_eq!(root.declaration.source_hash, a.source_hash);
    let mut out = Vec::new();
    walk(a, root.initializer.as_ref().unwrap(), &mut out);
    for (_, _, v) in &out {
        assert!(
            matches!(v.value, ExpressionValue::Unsupported(_)),
            "field acquisition must not invent callback authority: {v:?}"
        );
        let mut without_selection = a.clone();
        without_selection.arrow_selections.clear();
        assert!(
            !without_selection.admits_demanded_structure(&a.source, &a.source_hash, v),
            "Unsupported syntax alone grants no callback proof"
        );
    }
    out.iter()
        .map(|(s, e, v)| (*s, *e, v.start, v.end))
        .collect()
}
#[test]
fn issue_162_children_field_exact_objects_occurrences_serde_and_legacy_preservation() {
    let _serial = crate::protocol::handler_response_serial();
    for module in [true, false] {
        for form in ["direct", "reexport", "alias"] {
            let (root, config) = setup("");
            let p = root.path();
            let core = p.join("node_modules/@angular/core/index.d.ts");
            let mut declarations = std::fs::read_to_string(&core).unwrap();
            declarations.push_str("\nexport declare function NgModule(metadata:unknown):any;\n");
            std::fs::write(core, declarations).unwrap();
            let member = if module {
                "FeatureModule"
            } else {
                "childRoutes"
            };
            for side in ["a", "b"] {
                let target = if module {
                    "import {NgModule} from '@angular/core'; @NgModule({}) export class FeatureModule {}"
                } else {
                    "import {Routes} from '@angular/router'; export const childRoutes:Routes=[];"
                };
                std::fs::write(p.join(format!("target-{side}.ts")), target).unwrap();
                std::fs::write(
                    p.join(format!("api-{side}.ts")),
                    format!("export {{{member}}} from './target-{side}';"),
                )
                .unwrap();
                std::fs::write(
                    p.join(format!("alias-{side}.ts")),
                    format!("export {{{member} as PublicTarget}} from './target-{side}';"),
                )
                .unwrap();
            }
            let prefix = match form {
                "direct" => "target",
                "reexport" => "api",
                _ => "alias",
            };
            let export = if form == "alias" {
                "PublicTarget"
            } else {
                member
            };
            let left = format!("()=>import('./{prefix}-a').then(m=>m.{export})");
            let right = format!("()=>import('./{prefix}-b').then(m=>m.{export})");
            let source = format!(
                "import type {{Routes}} from '@angular/router'; import {{Screen as Eager}} from './screen-a'; import {{Token}} from './left'; export const routes:Routes=[{{path:'eager',component:Eager}},{{path:'same',loadChildren:{left},providers:[Token]}},{{path:'same',loadChildren:{right}}},{{path:'same',loadChildren:{left}}},{{path:'',loadChildren:{left}}},{{path:'',loadChildren:{right}}},{{path:'parent',children:[{{path:'child',loadChildren:{left}}},{{path:'child',loadChildren:{right}}}]}}];"
            );
            for owner in ["one.ts", "two.ts"] {
                std::fs::write(p.join(owner), &source).unwrap();
            }
            let state = crate::mcp::McpState::new(config);
            for owner in ["one.ts", "two.ts"] {
                let a = capture(&state, p, owner);
                let occurrences = retained(&a);
                assert_eq!(occurrences.len(), 7);
                let mut unique = occurrences.clone();
                unique.sort();
                unique.dedup();
                assert_eq!(unique.len(), 7);
                assert_eq!(a.arrow_selections.len(), 7);
                assert!(
                    a.arrow_selections
                        .iter()
                        .all(|proof| a.arrow_selection_at(proof.outer).is_some())
                );
                assert_eq!(crate::angular_meta::route_components::project(&a).len(), 1);
                assert_eq!(
                    crate::angular_meta::route_providers::project(&a)
                        .iter()
                        .filter(|e| e.relation == SemanticRelation::RegistersProvider)
                        .count(),
                    1
                );
                let restored: TypeScriptAuthority =
                    serde_json::from_value(serde_json::to_value(&a).unwrap()).unwrap();
                assert_eq!(retained(&restored), occurrences);
            }
            for name in ["same", "parent/child"] {
                let answer = call(
                    &state,
                    p,
                    "workspace_query",
                    json!({"type":"forward_edges","domain":"angular","entity_type":"Route","name":name}),
                );
                assert_eq!(
                    answer["edges"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .filter(|e| e["relation"] == "LoadsChildren")
                        .count(),
                    if name == "same" { 6 } else { 4 },
                    "approved canonical contribution: module={module} form={form}"
                );
            }
        }
    }
}
#[test]
fn issue_162_children_field_duplicate_and_nonroute_metadata_are_neutral() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup(
        "{path:'same',loadChildren:()=>import('./screen-a').then(m=>m.Screen),loadChildren:()=>import('./screen-b').then(m=>m.Screen),providers:[Token]}",
    );
    let p = root.path();
    std::fs::write(p.join("metadata.ts"),"import {Component} from '@angular/core'; @Component({loadChildren:()=>import('./screen-a').then(m=>m.Screen)}) export class Other {}").unwrap();
    let state = crate::mcp::McpState::new(config);
    let a = capture(&state, p, "routes.ts");
    let values = retained(&a);
    assert_eq!(values.len(), 2);
    assert_eq!(values[0].0, values[1].0);
    assert_ne!(values[0].2, values[1].2);
    let metadata = capture(&state, p, "metadata.ts");
    assert!(crate::angular_meta::route_components::project(&metadata).is_empty());
    assert!(crate::angular_meta::route_providers::project(&metadata).is_empty());
    let facet = metadata
        .declaration_facets
        .iter()
        .find(|f| f.declaration.source == metadata.source && f.declaration.name == "Other")
        .unwrap();
    let object = &facet.decorators[0].arguments[0];
    let ExpressionValue::Object(fields) = &object.value else {
        panic!("metadata Object")
    };
    assert_eq!(
        fields
            .iter()
            .filter(|(name, _)| name == "loadChildren")
            .count(),
        1
    );
}
#[test]
fn issue_162_children_field_sqlite_before_queries_and_owner_currentness() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup(
        "{path:'same',loadChildren:()=>import('./screen-a').then(m=>m.Screen),providers:[Token]},{path:'same',loadChildren:()=>import('./screen-a').then(m=>m.Screen)}",
    );
    let p = root.path();
    let state = crate::mcp::McpState::new(config.clone());
    let a = capture(&state, p, "routes.ts");
    let expected = retained(&a);
    assert_eq!(expected.len(), 2);
    call(
        &state,
        p,
        "save_context",
        json!({"filePath":p.join("routes.ts")}),
    );
    drop(state);
    let state = crate::mcp::McpState::new(config.clone());
    call(
        &state,
        p,
        "restore_context",
        json!({"filePath":p.join("routes.ts")}),
    );
    let alias = state
        .alias_for_path(&p.join("routes.ts").to_string_lossy())
        .unwrap();
    let snapshot = state.semantic_snapshot(&alias).unwrap();
    assert_eq!(
        retained(snapshot.authority.typescript_authority.as_ref().unwrap()),
        expected
    );
    let original = std::fs::read(p.join("routes.ts")).unwrap();
    std::fs::write(
        p.join("routes.ts"),
        "import type {Routes} from '@angular/router'; export const routes:Routes=[];",
    )
    .unwrap();
    drop(state);
    let changed_state = crate::mcp::McpState::new(config.clone());
    let changed = capture(&changed_state, p, "routes.ts");
    assert_ne!(changed.source_hash, snapshot.source_hash);
    assert!(!snapshot.authority.is_current(&changed.source_hash));
    assert!(retained(&changed).is_empty());
    drop(changed_state);
    std::fs::write(p.join("routes.ts"), original).unwrap();
    let state = crate::mcp::McpState::new(config);
    assert_eq!(retained(&capture(&state, p, "routes.ts")), expected);
}
