//! Shared neutral mechanism, separate Angular roles and exact demand ownership.
use super::typescript_children_selection_support::*;
use crate::ir::typescript_resolution::declaration_facets::ExpressionValue;
use crate::ir::typescript_resolution::{Namespace, Resolution, TypeScriptAuthority};
use crate::layers::meta::canonical_endpoint::CanonicalEndpoint;
use crate::layers::meta::semantic::SemanticRelation;
use serde_json::json;

pub(super) fn setup_role(
    form: &str,
    role: &str,
) -> (tempfile::TempDir, crate::config::CleanCtxConfig) {
    let (root, config) = setup(form);
    let p = root.path();
    let core = p.join("node_modules/@angular/core/index.d.ts");
    let mut text = std::fs::read_to_string(&core).unwrap();
    text.push_str("\nexport declare function NgModule(metadata:unknown):any;\n");
    std::fs::write(core, text).unwrap();
    for side in ["a", "b"] {
        let text = match role {
            "module" => {
                "import {NgModule} from '@angular/core'; @NgModule({}) export class Screen {}"
            }
            "routes" => {
                "import {Routes} from '@angular/router'; import {Screen as Eager} from './screen-a'; import {Token} from './left'; export const Screen:Routes=[{path:'selected',component:Eager,providers:[Token]}];"
            }
            _ => "export const Screen=123;",
        };
        std::fs::write(p.join(format!("feature-{side}.ts")), text).unwrap();
        std::fs::write(
            p.join(format!("lazy-{side}.ts")),
            format!("export {{Screen}} from './feature-{side}';"),
        )
        .unwrap();
        std::fs::write(
            p.join(format!("alias-{side}.ts")),
            format!("export {{Screen as PublicScreen}} from './feature-{side}';"),
        )
        .unwrap();
    }
    for owner in ["one.ts", "two.ts"] {
        let file = p.join(owner);
        let text = std::fs::read_to_string(&file)
            .unwrap()
            .replace("import('./screen-a')", "import('./feature-a')")
            .replace("import('./screen-b')", "import('./feature-b')");
        std::fs::write(file, text).unwrap();
    }
    (root, config)
}

pub(super) fn selected_role(
    state: &crate::mcp::McpState,
    p: &std::path::Path,
    a: &TypeScriptAuthority,
    target: &crate::ir::typescript_resolution::DeclarationId,
    role: &str,
) {
    let facet = a
        .declaration_facets
        .iter()
        .find(|f| f.declaration == *target)
        .expect("stage=selected-target-facet-prerequisite");
    if role == "module" {
        assert_eq!(
            crate::angular_meta::class_role_authority::declaration_role(a, target),
            Some(crate::angular_meta::decorators::ClassKind::Module),
            "stage=selected-native-Module-prerequisite"
        );
    } else if role == "routes" {
        let independent = capture(
            state,
            p,
            target.source.file_name().unwrap().to_str().unwrap(),
        );
        let native = crate::angular_meta::route_providers::project(&independent);
        assert!(native.iter().any(|edge|edge.relation==SemanticRelation::RegistersProvider&&matches!(&edge.subject.canonical,Some(CanonicalEndpoint::RouteOccurrence{route_occurrence}) if route_occurrence.root==(&facet.declaration).into())),"stage=selected-native-const-Routes-prerequisite");
        let exact = independent
            .declaration_facets
            .iter()
            .find(|f| f.declaration == *target)
            .unwrap();
        assert_eq!(
            serde_json::to_value(&facet.type_annotation).unwrap(),
            serde_json::to_value(&exact.type_annotation).unwrap(),
            "stage=selected-Routes-annotation-prerequisite"
        );
        assert_eq!(
            serde_json::to_value(&facet.initializer).unwrap(),
            serde_json::to_value(&exact.initializer).unwrap(),
            "stage=selected-Routes-initializer-prerequisite"
        );
        let ExpressionValue::Reference(Resolution::Resolved(api)) =
            &facet.type_annotation.as_ref().unwrap().value
        else {
            panic!("stage=selected-Routes-type-reference-prerequisite")
        };
        assert!(
            crate::angular_meta::class_role_authority::public_entry_api(
                a,
                api,
                "@angular/router",
                "@angular/router",
                "Routes",
                Namespace::Type
            ),
            "stage=selected-native-Routes-package-prerequisite"
        );
    } else {
        assert!(crate::angular_meta::class_role_authority::declaration_role(a, target).is_none());
    }
}
#[test]
fn issue_162_shared_selection_module_routes_and_ordinary_exports() {
    let _serial = crate::protocol::handler_response_serial();
    for role in ["module", "routes", "ordinary"] {
        for form in ["direct", "reexport", "alias"] {
            let (root, config) = setup_role(form, role);
            let p = root.path();
            let state = crate::mcp::McpState::new(config);
            for owner in ["one.ts", "two.ts"] {
                let a = capture(&state, p, owner);
                let selections = proofs(&a);
                assert_eq!(
                    selections
                        .iter()
                        .filter(|s| s.target.source.ends_with("feature-a.ts"))
                        .count(),
                    4
                );
                assert_eq!(
                    selections
                        .iter()
                        .filter(|s| s.target.source.ends_with("feature-b.ts"))
                        .count(),
                    3
                );
                for selected in selections {
                    selected_role(&state, p, &a, &selected.target, role);
                }
                assert_eq!(crate::angular_meta::route_components::project(&a).len(), 1);
                assert!(
                    crate::angular_meta::route_lazy_components::project(&a).is_empty(),
                    "neutral loadChildren proof cannot publish Component membership"
                );
            }
            if role == "ordinary" {
                let response = call(
                    &state,
                    p,
                    "workspace_query",
                    json!({"type":"forward_edges","domain":"angular","entity_type":"Route","name":"same"}),
                );
                assert!(!response["edges"].as_array().unwrap().iter().any(|e|e["relation"]=="LoadsChildren"||e["relation"]=="RouteMapsTo"));
            }
        }
    }
}
fn rename_fields(v: &mut crate::ir::typescript_resolution::declaration_facets::ExpressionFacet) {
    match &mut v.value {
        ExpressionValue::Array(values) => {
            for v in values {
                rename_fields(v);
            }
        }
        ExpressionValue::Object(fields) => {
            for (name, v) in fields {
                if name == "loadChildren" {
                    *name = "providers".into();
                }
                rename_fields(v);
            }
        }
        _ => {}
    }
}
#[test]
fn issue_162_shared_selection_origin_duplicate_and_crossed_field_refusal() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("direct");
    let p = root.path();
    let state = crate::mcp::McpState::new(config.clone());
    let a = capture(&state, p, "one.ts");
    proofs(&a);
    let at = a.arrow_selections[0].outer;
    let mut bad = a.clone();
    for f in &mut bad.declaration_facets {
        if let Some(v) = &mut f.initializer {
            rename_fields(v);
        }
    }
    assert!(
        bad.arrow_selection_at(at).is_none(),
        "unsupported origin cannot borrow proof"
    );
    for field in ["loadChildren", "loadComponent"] {
        std::fs::write(p.join("one.ts"),format!("import type {{Routes}} from '@angular/router'; import {{Screen}} from './screen-a'; import {{Token}} from './left'; export const routes:Routes=[{{path:'eager',component:Screen,providers:[Token]}},{{path:'duplicate',{field}:()=>import('./screen-a').then(m=>m.Screen),{field}:()=>import('./screen-b').then(m=>m.Screen)}}];")).unwrap();
        let state = crate::mcp::McpState::new(config.clone());
        let duplicate = capture(&state, p, "one.ts");
        assert_eq!(duplicate.arrow_selections.len(), 2);
        assert!(
            duplicate
                .arrow_selections
                .iter()
                .all(|s| duplicate.arrow_selection_at(s.outer).is_none()),
            "duplicate origin {field}"
        );
    }
}

#[test]
fn issue_162_shared_selection_two_origins_and_unapproved_property() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("direct");
    let p = root.path();
    std::fs::write(p.join("one.ts"),"import type {Routes} from '@angular/router'; export const routes:Routes=[{path:'mixed',loadComponent:()=>import('./screen-a').then(m=>m.Screen),loadChildren:()=>import('./screen-b').then(m=>m.Screen)}];").unwrap();
    std::fs::write(p.join("metadata.ts"),"import {Component} from '@angular/core'; @Component({useFactory:()=>import('./screen-a').then(m=>m.Screen)}) export class Other {}").unwrap();
    let state = crate::mcp::McpState::new(config);
    let a = capture(&state, p, "one.ts");
    assert_eq!(a.arrow_selections.len(), 2);
    assert!(
        a.arrow_selections
            .iter()
            .all(|s| a.arrow_selection_at(s.outer).is_some())
    );
    assert_eq!(
        crate::angular_meta::route_lazy_components::project(&a).len(),
        1
    );
    let metadata = capture(&state, p, "metadata.ts");
    assert!(
        metadata.arrow_selections.is_empty(),
        "already-demanded unapproved property cannot nominate neutral selection"
    );
}
