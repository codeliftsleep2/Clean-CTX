//! Owner-approved lazy investigation only; no capture or lazy semantics change.
use super::angular_route_component_support::{call, capture, setup};
use crate::ir::typescript_resolution::declaration_facets::{ExpressionFacet, ExpressionValue};
use crate::layers::meta::canonical_endpoint::CanonicalEndpoint;
use crate::layers::meta::semantic::SemanticRelation;
use serde_json::{Value, json};
use std::path::Path;
fn observe(state: &crate::mcp::McpState, p: &Path) -> Value {
    let forward:Vec<_>=["same","parent/child"].into_iter().map(|name|call(state,p,"workspace_query",json!({"type":"forward_edges","domain":"angular","entity_type":"Route","name":name}))).collect();
    let reverse = call(
        state,
        p,
        "workspace_query",
        json!({"type":"reverse_edges","domain":"angular","entity_type":"Component","name":"Screen"}),
    );
    json!({"forward":forward,"reverse":reverse})
}
fn fields(
    a: &crate::ir::typescript_resolution::TypeScriptAuthority,
    value: &ExpressionFacet,
    seen: &mut usize,
    observed: &Value,
) {
    assert!(
        a.admits_demanded_structure(&a.source, &a.source_hash, value),
        "stage=route-object-structure; {observed}"
    );
    match &value.value {
        ExpressionValue::Array(values) => {
            for v in values {
                fields(a, v, seen, observed);
            }
        }
        ExpressionValue::Object(entries) => {
            // Eager-only control and parent container are separate from lazy objects.
            if entries.iter().any(|(k, _)| k == "component") {
                return;
            }
            if let Some((_, children)) = entries.iter().find(|(k, _)| k == "children") {
                fields(a, children, seen, observed);
                return;
            }
            let values: Vec<_> = entries
                .iter()
                .filter(|(k, _)| k == "loadComponent")
                .collect();
            assert_eq!(
                values.len(),
                1,
                "stage=loadComponent-field-prerequisite object={}..{}; public={observed}",
                value.start,
                value.end
            );
            let callback = &values[0].1;
            assert!(
                value.start <= callback.start
                    && callback.start < callback.end
                    && callback.end <= value.end,
                "stage=field-value-containment"
            );
            assert!(
                a.admits_demanded_structure(&a.source, &a.source_hash, callback),
                "stage=callback-structure-prerequisite callback={callback:?}; public={observed}"
            );
            *seen += 1;
        }
        _ => panic!("stage=route-container-prerequisite {value:?}"),
    }
}
fn question(form: &str) {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("");
    let p = root.path();
    std::fs::write(p.join("lazy-left.ts"), "export {Screen} from './screen-a';").unwrap();
    std::fs::write(
        p.join("lazy-right.ts"),
        "export {Screen} from './screen-b';",
    )
    .unwrap();
    std::fs::write(
        p.join("lazy-alias-left.ts"),
        "export {Screen as PublicScreen} from './screen-a';",
    )
    .unwrap();
    std::fs::write(
        p.join("lazy-alias-right.ts"),
        "export {Screen as PublicScreen} from './screen-b';",
    )
    .unwrap();
    let (left, right, member) = match form {
        "direct" => ("./screen-a", "./screen-b", "Screen"),
        "reexport" => ("./lazy-left", "./lazy-right", "Screen"),
        _ => ("./lazy-alias-left", "./lazy-alias-right", "PublicScreen"),
    };
    let lazy = |module: &str| format!("()=>import('{module}').then(m=>m.{member})");
    for file in ["lazy-one.ts", "lazy-two.ts"] {
        let a = lazy(left);
        let b = lazy(right);
        std::fs::write(p.join(file),format!("import type {{Routes}} from '@angular/router'; import {{Screen as Eager}} from './screen-a'; import {{Token}} from './left'; export const routes:Routes=[{{path:'eager',component:Eager}},{{path:'same',loadComponent:{a},providers:[Token]}},{{path:'same',loadComponent:{b}}},{{path:'same',loadComponent:{a}}},{{path:'',loadComponent:{a}}},{{path:'',loadComponent:{b}}},{{path:'parent',children:[{{path:'child',loadComponent:{a}}},{{path:'child',loadComponent:{b}}}]}}];")).unwrap();
    }
    let state = crate::mcp::McpState::new(config);
    // These controls prove independent native targets, never the callback export join.
    for file in ["screen-a.ts", "screen-b.ts"] {
        let a = capture(&state, p, file);
        let id = a
            .declarations
            .iter()
            .find(|id| id.name == "Screen")
            .unwrap();
        assert_eq!(
            crate::angular_meta::class_role_authority::declaration_role(&a, id),
            Some(crate::angular_meta::decorators::ClassKind::Component),
            "stage=native-Component-control"
        );
    }
    let mut authorities = Vec::new();
    for file in ["lazy-one.ts", "lazy-two.ts"] {
        let a = capture(&state, p, file);
        let eager = crate::angular_meta::route_components::project(&a);
        assert_eq!(eager.len(), 1, "stage=eager-sibling-control");
        let providers = crate::angular_meta::route_providers::project(&a);
        let registrations: Vec<_> = providers
            .iter()
            .filter(|e| e.relation == SemanticRelation::RegistersProvider)
            .collect();
        assert_eq!(registrations.len(), 1, "stage=provider-sibling-control");
        assert!(
            matches!(
                registrations[0].subject.canonical,
                Some(CanonicalEndpoint::RouteOccurrence { .. })
            ),
            "stage=native-root-object-control"
        );
        authorities.push(a);
    }
    let public = observe(&state, p);
    eprintln!("lazy-route-observed form={form} public={public}");
    // Field/callback acquisition probes only. Export/member and semantic assertions
    // remain unclassified until an observed complete neutral authority chain exists.
    for a in &authorities {
        let root = a
            .declaration_facets
            .iter()
            .find(|f| f.declaration.source == a.source && f.declaration.name == "routes")
            .unwrap();
        let mut seen = 0;
        fields(a, root.initializer.as_ref().unwrap(), &mut seen, &public);
        assert_eq!(seen, 7);
    }
}
#[test]
fn issue_162_lazy_route_direct_prerequisites() {
    question("direct");
}
#[test]
fn issue_162_lazy_route_reexport_prerequisites() {
    question("reexport");
}
#[test]
fn issue_162_lazy_route_alias_export_prerequisites() {
    question("alias");
}
#[test]
fn issue_162_lazy_route_unsupported_siblings_observed() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("{path:'eager',component:A,providers:[Token]}");
    let p = root.path();
    std::fs::write(
        p.join("private.ts"),
        "import {Component} from '@angular/core'; @Component({}) class Screen {}",
    )
    .unwrap();
    std::fs::write(p.join("foreign.ts"), "export class Screen {}").unwrap();
    std::fs::write(
        p.join("type-only.ts"),
        "export type {Screen} from './screen-a';",
    )
    .unwrap();
    std::fs::write(
        p.join("ambiguous.ts"),
        "export * from './screen-a'; export * from './screen-b';",
    )
    .unwrap();
    let callbacks = [
        "()=>import('./screen-a').then(m=>other.Screen)",
        "()=>import('./screen-a').then(m=>m['Screen'])",
        "()=>import(specifier).then(m=>m.Screen)",
        "()=>import('./missing').then(m=>m.Screen)",
        "()=>import('./private').then(m=>m.Screen)",
        "()=>import('./type-only').then(m=>m.Screen)",
        "()=>import('./ambiguous').then(m=>m.Screen)",
        "()=>import('./foreign').then(m=>m.Screen)",
        "async()=>{const m=await import('./screen-a');return m.Screen;}",
        "()=>condition?import('./screen-a').then(m=>m.Screen):import('./screen-b').then(m=>m.Screen)",
    ];
    let noise = callbacks
        .iter()
        .map(|c| format!("{{path:'noise',loadComponent:{c}}}"))
        .collect::<Vec<_>>()
        .join(",");
    std::fs::write(p.join("noise.ts"),format!("import type {{Routes}} from '@angular/router'; export const routes:Routes=[{noise},{{path:'noise',loadComponent:()=>import('./screen-a').then(m=>m.Screen),loadComponent:()=>import('./screen-b').then(m=>m.Screen)}}];")).unwrap();
    let state = crate::mcp::McpState::new(config);
    let a = capture(&state, p, "routes.ts");
    capture(&state, p, "noise.ts");
    assert_eq!(crate::angular_meta::route_components::project(&a).len(), 1);
    assert_eq!(
        crate::angular_meta::route_providers::project(&a)
            .iter()
            .filter(|e| e.relation == SemanticRelation::RegistersProvider)
            .count(),
        1
    );
    let old = call(
        &state,
        p,
        "workspace_query",
        json!({"type":"forward_edges","domain":"angular","entity_type":"Route","name":"noise"}),
    );
    eprintln!("unsupported-lazy-observed {old}");
    let sibling = call(
        &state,
        p,
        "workspace_query",
        json!({"type":"forward_edges","domain":"angular","entity_type":"Route","name":"eager"}),
    );
    assert_eq!(
        sibling["edges"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["relation"] == "RouteMapsTo")
            .count(),
        1
    );
}
