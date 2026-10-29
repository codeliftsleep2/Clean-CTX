//! Supplemental generation-73 exits; no production behavior changes.
use super::angular_route_component_support::*;
use crate::ir::typescript_resolution::declaration_facets::ExpressionValue;
use crate::layers::meta::semantic::SemanticRelation;
use serde_json::json;

#[test]
fn issue_162_route_component_exit_crossed_field_container_source_proof() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("{path:'same',component:A},{path:'same',component:B}");
    let state = crate::mcp::McpState::new(config);
    let original = capture(&state, root.path(), "routes.ts");
    let expected = project(&original);
    assert_eq!(expected.len(), 2);
    for mode in 0..7 {
        let mut bad = original.clone();
        let facet = bad
            .declaration_facets
            .iter_mut()
            .find(|f| f.declaration.source == original.source && f.declaration.name == "routes")
            .unwrap();
        let array = facet.initializer.as_mut().unwrap();
        let ExpressionValue::Array(items) = &mut array.value else {
            panic!("array")
        };
        let other = items[1].clone();
        let first = &mut items[0];
        match mode {
            0 => first.start += 1,
            1 => first.end = other.end,
            2 => {
                let ExpressionValue::Object(other_fields) = other.value else {
                    panic!("other")
                };
                let borrowed = other_fields
                    .iter()
                    .find(|(key, _)| key == "component")
                    .unwrap()
                    .1
                    .clone();
                let ExpressionValue::Object(fields) = &mut first.value else {
                    panic!("object")
                };
                fields
                    .iter_mut()
                    .find(|(key, _)| key == "component")
                    .unwrap()
                    .1 = borrowed;
            }
            3 => {
                let ExpressionValue::Object(fields) = &mut first.value else {
                    panic!("object")
                };
                fields
                    .iter_mut()
                    .find(|(key, _)| key == "component")
                    .unwrap()
                    .0 = "providers".into();
            }
            4 => facet.declaration.source = root.path().join("borrowed.ts"),
            5 => facet.declaration.source_hash.push('x'),
            _ => array.start += 1,
        }
        assert!(
            !bad.admits_relation_evidence(&expected[..1]),
            "crossed proof {mode}"
        );
    }
}
#[test]
fn issue_162_route_component_exit_complete_sqlite_snapshot_and_index_before_query() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup(
        "{path:'same',component:A,providers:[Token]},{path:'same',component:A},{path:'same',component:B}",
    );
    let p = root.path();
    let state = crate::mcp::McpState::new(config.clone());
    let expected = project(&capture(&state, p, "routes.ts"));
    assert_eq!(expected.len(), 3);
    call(
        &state,
        p,
        "save_context",
        json!({"filePath":p.join("routes.ts")}),
    );
    drop(state);
    let state = crate::mcp::McpState::new(config);
    call(
        &state,
        p,
        "restore_context",
        json!({"filePath":p.join("routes.ts")}),
    );
    let alias = state
        .alias_for_path(&p.join("routes.ts").to_string_lossy())
        .unwrap();
    let snap = state.semantic_snapshot(&alias).unwrap();
    let exact = |a: &crate::layers::meta::semantic::SemanticEdge,
                 b: &crate::layers::meta::semantic::SemanticEdge| {
        a.relation == b.relation
            && a.subject.canonical == b.subject.canonical
            && a.object.canonical == b.object.canonical
            && a.subject.file == b.subject.file
            && a.object.file == b.object.file
    };
    assert_eq!(
        snap.edges
            .iter()
            .filter(|e| e.relation == SemanticRelation::RouteMapsTo)
            .count(),
        3
    );
    assert!(snap.authority.admits_relation_evidence(&expected));
    for e in &expected {
        assert_eq!(snap.edges.iter().filter(|x| exact(e, x)).count(), 1);
    }
    {
        let index = state.workspace_index_read();
        let indexed = index.forward_edges_by_identity("angular", "Route", "same");
        for e in &expected {
            assert_eq!(indexed.iter().filter(|x| exact(e, x)).count(), 1);
        }
        assert_eq!(
            indexed
                .iter()
                .filter(|e| e.relation == SemanticRelation::RouteMapsTo)
                .count(),
            3
        );
    }
    assert_eq!(rows(&forward(&state, p, "same")).len(), 3);
}
#[test]
fn issue_162_route_component_exit_package_and_config_stale_restore_repair() {
    let _serial = crate::protocol::handler_response_serial();
    for config_change in [false, true] {
        let (root, config) = setup("");
        let p = root.path();
        let tsconfig = |target: &str| {
            json!({"compilerOptions":{"module":"esnext","moduleResolution":"bundler","noLib":true,"types":[],"baseUrl":".","paths":{"@screen":[target]}}}).to_string()
        };
        std::fs::write(p.join("tsconfig.json"), tsconfig("./screen-a.ts")).unwrap();
        let owner = "import type {Routes} from '@angular/router'; import {Screen} from '@screen'; export const routes:Routes=[{path:'same',component:Screen}];";
        std::fs::write(p.join("routes.ts"), owner).unwrap();
        let state = crate::mcp::McpState::new(config.clone());
        let before = project(&capture(&state, p, "routes.ts"));
        assert_eq!(
            before.len(),
            1,
            "stage=configured-native-route-prerequisite"
        );
        assert!(
            before[0]
                .object
                .file
                .as_ref()
                .unwrap()
                .ends_with("screen-a.ts")
        );
        call(
            &state,
            p,
            "save_context",
            json!({"filePath":p.join("routes.ts")}),
        );
        drop(state);
        let dependency = if config_change {
            p.join("tsconfig.json")
        } else {
            p.join("node_modules/@angular/core/package.json")
        };
        let original = std::fs::read(&dependency).unwrap();
        let changed = if config_change {
            tsconfig("./screen-b.ts")
        } else {
            let mut manifest: serde_json::Value = serde_json::from_slice(&original).unwrap();
            manifest["version"] = json!("20.0.1");
            manifest.to_string()
        };
        std::fs::write(&dependency, changed).unwrap();
        let state = crate::mcp::McpState::new(config.clone());
        let refused = super::angular_bootstrap_inline_controls_tests::raw(
            &state,
            p,
            "restore_context",
            json!({"filePath":p.join("routes.ts")}),
        );
        assert!(
            refused.get("error").is_some(),
            "stale dependency must refuse: {refused}"
        );
        let now = project(&capture(&state, p, "routes.ts"));
        assert_eq!(now.len(), 1);
        assert!(
            now[0]
                .object
                .file
                .as_ref()
                .unwrap()
                .ends_with(if config_change {
                    "screen-b.ts"
                } else {
                    "screen-a.ts"
                })
        );
        drop(state);
        std::fs::write(&dependency, original).unwrap();
        let state = crate::mcp::McpState::new(config);
        let repaired = project(&capture(&state, p, "routes.ts"));
        assert_eq!(repaired.len(), 1);
        assert_eq!(repaired[0].object.canonical, before[0].object.canonical);
        assert_eq!(rows(&forward(&state, p, "same")).len(), 1);
        assert_eq!(std::fs::read_to_string(p.join("routes.ts")).unwrap(), owner);
    }
}
#[test]
fn issue_162_route_component_exit_registered_lazy_coexistence_and_lower_bound() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup(
        "{path:'same',component:A},{path:'same',loadComponent:()=>import('./lazy').then(m=>m.LazyScreen)}",
    );
    let p = root.path();
    std::fs::write(
        p.join("lazy.ts"),
        "import {Component} from '@angular/core'; @Component({}) export class LazyScreen {}",
    )
    .unwrap();
    let state = crate::mcp::McpState::new(config);
    let a = capture(&state, p, "routes.ts");
    assert_eq!(
        project(&a).len(),
        1,
        "eager proof never evaluates lazy callbacks"
    );
    let answer = forward(&state, p, "same");
    let edges = rows(&answer);
    assert_eq!(
        edges.len(),
        2,
        "one canonical eager plus one historical lazy fact"
    );
    assert_eq!(
        edges
            .iter()
            .filter(|e| e["object"]["name"] == "Screen")
            .count(),
        1
    );
    assert_eq!(
        edges
            .iter()
            .filter(|e| e["object"]["name"] == "LazyScreen")
            .count(),
        1
    );
    assert_eq!(answer["coverage"]["result_semantics"], "lower_bound");
    assert_eq!(answer["coverage"]["source_complete"], false);
    assert_eq!(answer["coverage"]["omitted_possible"], true);
    assert_eq!(forward(&state, p, "same")["edges"], answer["edges"]);
}
