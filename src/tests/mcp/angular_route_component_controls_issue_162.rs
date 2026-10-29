//! Approved eager route production controls; all runtime execution is operator-owned.
use super::angular_route_component_support::*;
use crate::ir::typescript_resolution::TypeScriptAuthority;
use crate::layers::meta::canonical_endpoint::CanonicalEndpoint;
use crate::layers::meta::semantic::SemanticRelation;
use std::path::Path;
#[test]
fn issue_162_route_component_controls_forged_crossed_borrowed_retained_proof() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup(
        "{path:'same',component:A,providers:[Token]},{path:'same',component:B},{path:'same',component:A}",
    );
    let state = crate::mcp::McpState::new(config);
    let a = capture(&state, root.path(), "routes.ts");
    let edges = project(&a);
    assert_eq!(edges.len(), 3);
    assert!(a.admits_relation_evidence(&edges));
    assert_ne!(edges[0].subject.canonical, edges[2].subject.canonical);
    let provider = crate::angular_meta::route_providers::project(&a)
        .into_iter()
        .find(|e| e.relation == SemanticRelation::RegistersProvider)
        .unwrap();
    assert_eq!(provider.subject.canonical, edges[0].subject.canonical);
    let restored: TypeScriptAuthority =
        serde_json::from_value(serde_json::to_value(&a).unwrap()).unwrap();
    assert!(restored.admits_relation_evidence(&edges));
    for mode in 0..12 {
        let mut edge = edges[0].clone();
        match mode {
            0 => edge.object = edges[1].object.clone(),
            1 => edge.subject.canonical = edges[1].subject.canonical.clone(),
            2 => edge.subject.file = edges[1].object.file.clone(),
            3 => edge.object.file = edge.subject.file.clone(),
            4 => edge.subject.name = "borrowed".into(),
            5 => edge.object.name = "borrowed".into(),
            6 => edge.object.canonical = None,
            7 => edge.subject.canonical = None,
            8 => edge.layer = "typescript",
            _ => {
                let Some(CanonicalEndpoint::RouteOccurrence { route_occurrence }) =
                    &mut edge.subject.canonical
                else {
                    panic!("route")
                };
                match mode {
                    9 => route_occurrence.source_hash.push('x'),
                    10 => route_occurrence.start += 1,
                    _ => route_occurrence.root.source_hash.push('x'),
                }
            }
        }
        assert!(!a.admits_relation_evidence(&[edge]), "forged edge {mode}");
    }
    for mode in 0..8 {
        let mut bad = a.clone();
        match mode {
            0 => bad.demanded_occurrences.clear(),
            1 => bad.declaration_facets.clear(),
            2 => bad.source_hash.push('x'),
            3 => bad.demanded_occurrences.retain(|v| !matches!(v.value, crate::ir::typescript_resolution::demanded_occurrences::DemandedValue::Reference(_))),
            4 => {
                use crate::ir::typescript_resolution::demanded_occurrences::DemandedValue;
                for item in &mut bad.demanded_occurrences {
                    if let DemandedValue::Declaration { id, is_const, .. } = &mut item.value
                        && id.source == bad.source && id.name == "routes" { *is_const = false; }
                }
            }
            5 => {
                use crate::ir::typescript_resolution::demanded_occurrences::DemandedValue;
                bad.demanded_occurrences.retain(|item| !matches!(&item.value,
                    DemandedValue::Structure {kind, ..} if kind == "object"));
            }
            6 => bad.type_reference_sites.clear(),
            _ => { let duplicate = bad.declaration_facets.iter().find(|f| f.declaration.name == "routes" && f.declaration.source == bad.source).unwrap().clone(); bad.declaration_facets.push(duplicate); }
        }
        assert!(
            !bad.admits_relation_evidence(&edges),
            "missing/crossed authority {mode}"
        );
    }
    let no_authority = crate::mcp::semantic_compilation::retained::RetainedAuthority::default();
    assert!(!no_authority.admits_relation_evidence(&edges));
}
#[test]
fn issue_162_route_component_controls_owned_refusal_and_provider_independence() {
    let _serial = crate::protocol::handler_response_serial();
    for refused in ["Missing", "Token", "A,component:B"] {
        let (root, config) = setup(&format!(
            "{{path:'bad',component:{refused},providers:[Token]}},{{path:'good',component:B}}"
        ));
        let state = crate::mcp::McpState::new(config);
        let a = capture(&state, root.path(), "routes.ts");
        let edges = project(&a);
        assert_eq!(edges.len(), 1, "{refused}");
        assert_eq!(edges[0].subject.name, "good");
        assert_eq!(
            crate::angular_meta::route_providers::project(&a)
                .iter()
                .filter(|e| e.relation == SemanticRelation::RegistersProvider)
                .count(),
            1
        );
        assert!(
            rows(&forward(&state, root.path(), "bad")).is_empty(),
            "no weak fallback for {refused}"
        );
        assert_eq!(rows(&forward(&state, root.path(), "good")).len(), 1);
    }
}
#[test]
fn issue_162_route_component_controls_selective_legacy_and_lazy_origin() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("{path:'native',component:A}");
    let p = root.path();
    std::fs::write(p.join("foreign.ts"), "export type Routes=unknown[];").unwrap();
    for (file, declaration) in [
        ("untyped.ts", "const legacyRoutes"),
        ("mutable.ts", "let routes:Routes"),
        ("foreign-routes.ts", "const routes:Routes"),
    ] {
        let import = if file == "foreign-routes.ts" {
            "./foreign"
        } else {
            "@angular/router"
        };
        std::fs::write(p.join(file), format!("import type {{Routes}} from '{import}'; import {{Screen}} from './screen-a'; export {declaration}=[{{path:'legacy',component:Screen}}];")).unwrap();
    }
    let state = crate::mcp::McpState::new(config);
    for file in ["untyped.ts", "mutable.ts", "foreign-routes.ts"] {
        let a = capture(&state, p, file);
        assert!(project(&a).is_empty());
    }
    let legacy = forward(&state, p, "legacy");
    assert_eq!(
        rows(&legacy).len(),
        2,
        "existing native-import compatibility"
    );
    for file in ["untyped.ts", "mutable.ts"] {
        assert!(
            rows(&legacy).iter().any(|edge| {
                Path::new(edge["subject"]["file"].as_str().unwrap()).file_name()
                    == Some(std::ffi::OsStr::new(file))
            }),
            "missing legacy owner {file}"
        );
    }
    assert!(
        !rows(&legacy).iter().any(|edge| edge["subject"]["file"]
            .as_str()
            .unwrap()
            .ends_with("foreign-routes.ts")),
        "foreign-only source remains outside Angular applicability"
    );
    let mut shape = crate::angular_meta::routing::extract_route_shape(
        "import {Routes} from '@angular/router'; const routes:Routes=[{path:'both',component:Screen}];", crate::compression::Fidelity::High).unwrap();
    // Simultaneous origins sharing vocabulary must remain separately selectable.
    shape.routes[0].resolved_load_component = Some(crate::angular_meta::routing::LazyRouteTarget {
        entity_type: "Component",
        name: "LazyScreen".into(),
    });
    shape.routes[0].resolved_load_children = Some(crate::angular_meta::routing::LazyRouteTarget {
        entity_type: "Module",
        name: "LazyModule".into(),
    });
    let owned = vec![shape.routes[0].eager_object_span];
    let original = shape.to_semantic_edges();
    let selective = shape.to_semantic_edges_selective(&owned);
    assert_eq!(original.len(), selective.len() + 1);
    assert!(
        selective
            .iter()
            .any(|e| e.relation == SemanticRelation::RouteMapsTo && e.object.name == "LazyScreen")
    );
    assert!(selective.iter().any(|e| e.relation == SemanticRelation::LoadsChildren && e.object.name == "LazyModule"));
    assert!(!selective.iter().any(|e| e.object.name == "Screen"));
}
#[test]
fn issue_162_route_component_controls_target_refusals_and_spread_compatibility() {
    let _serial = crate::protocol::handler_response_serial();
    for (extra, target) in [
        (
            "import type {Screen as OnlyType} from './screen-a';",
            "OnlyType",
        ),
        (
            "import {Screen as Foreign} from './foreign-screen';",
            "Foreign",
        ),
        (
            "import {Screen as Ambiguous} from './ambiguous';",
            "Ambiguous",
        ),
    ] {
        let (root, config) = setup("");
        let p = root.path();
        std::fs::write(p.join("foreign-screen.ts"), "function Component(_:unknown){return (_:unknown)=>{};} @Component({}) export class Screen {}").unwrap();
        std::fs::write(
            p.join("ambiguous.ts"),
            "export * from './screen-a'; export * from './screen-b';",
        )
        .unwrap();
        std::fs::write(p.join("routes.ts"), format!("{PREFIX} {extra} export const routes:Routes=[{{path:'bad',component:{target},providers:[Token]}},{{path:'good',component:B}}];")).unwrap();
        let state = crate::mcp::McpState::new(config);
        let a = capture(&state, p, "routes.ts");
        assert_eq!(project(&a).len(), 1, "{target}");
        assert!(rows(&forward(&state, p, "bad")).is_empty(), "{target}");
        assert_eq!(rows(&forward(&state, p, "good")).len(), 1);
    }
    let (root, config) = setup("{path:'spread',component:A,...extra},{path:'good',component:B}");
    let p = root.path();
    // Unsupported spread stays outside shared ordinary-object ownership.
    let state = crate::mcp::McpState::new(config);
    let a = capture(&state, p, "routes.ts");
    assert_eq!(project(&a).len(), 1);
    assert_eq!(
        rows(&forward(&state, p, "spread")).len(),
        1,
        "historical row remains; not canonical spread evaluation"
    );
}
