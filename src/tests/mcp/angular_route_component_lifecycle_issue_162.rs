//! Approved eager route production controls; all runtime execution is operator-owned.
use super::angular_route_component_support::*;
use crate::layers::meta::semantic::SemanticRelation;
use serde_json::json;
#[test]
fn issue_162_route_component_controls_sqlite_before_queries_and_stale_repair() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("{path:'same',component:A},{path:'same',component:B}");
    let p = root.path();
    let state = crate::mcp::McpState::new(config.clone());
    let a = capture(&state, p, "routes.ts");
    let expected = project(&a);
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
    let snap = state.semantic_snapshot(&alias).unwrap();
    let restored = snap.authority.typescript_authority.as_ref().unwrap();
    assert!(restored.admits_relation_evidence(&expected));
    assert_eq!(
        serde_json::to_value(project(restored)).unwrap(),
        serde_json::to_value(&expected).unwrap()
    );
    assert_eq!(rows(&forward(&state, p, "same")).len(), 2);
    drop(state);
    std::fs::write(p.join("screen-a.ts"), "export class Screen {}").unwrap();
    assert!(
        !a.witnesses
            .iter()
            .all(crate::ir::typescript_resolution::Witness::current)
    );
    let state = crate::mcp::McpState::new(config.clone());
    stale_restore(&state, p);
    let changed = capture(&state, p, "routes.ts");
    assert_eq!(project(&changed).len(), 1);
    assert_eq!(rows(&forward(&state, p, "same")).len(), 1);
    drop(state);
    std::fs::write(p.join("screen-a.ts"), SCREEN).unwrap();
    let state = crate::mcp::McpState::new(config);
    assert_eq!(project(&capture(&state, p, "routes.ts")).len(), 2);
    assert_eq!(rows(&forward(&state, p, "same")).len(), 2);
}
#[test]
fn issue_162_route_component_controls_field_object_provider_retirement() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) =
        setup("{path:'same',component:A,providers:[Token]},{path:'same',component:B}");
    let p = root.path();
    let state = crate::mcp::McpState::new(config);
    assert_eq!(project(&capture(&state, p, "routes.ts")).len(), 2);
    for (body, components, providers) in [
        (
            "{path:'same',providers:[Token]},{path:'same',component:B}",
            1,
            1,
        ),
        ("{path:'same',component:A},{path:'same',component:B}", 2, 0),
        ("{path:'same',component:B}", 1, 0),
        (
            "{path:'same',providers:[Token],children:[{path:'',component:B}]}",
            1,
            1,
        ),
        ("{path:'same',providers:[Token],children:[]}", 0, 1),
        ("{path:'same',children:[{path:'',component:B}]}", 1, 0),
        ("", 0, 0),
    ] {
        std::fs::write(
            p.join("routes.ts"),
            format!("{PREFIX} export const routes:Routes=[{body}];"),
        )
        .unwrap();
        let a = capture(&state, p, "routes.ts");
        assert_eq!(project(&a).len(), components);
        assert_eq!(
            crate::angular_meta::route_providers::project(&a)
                .iter()
                .filter(|e| e.relation == SemanticRelation::RegistersProvider)
                .count(),
            providers
        );
        assert_eq!(rows(&forward(&state, p, "same")).len(), components);
    }
}

#[test]
fn issue_162_route_component_controls_cold_warm_refresh_independent_root_retirement() {
    let _serial = crate::protocol::handler_response_serial();
    for import in [
        "import {Screen as A} from './screen-a';",
        "import {Screen as Named} from './screen-a';",
        "import {A} from './route-api';",
        "import * as ns from './route-api';",
        "import {Component} from '@angular/core'; @Component({}) class A {}",
    ] {
        let (root, config) = setup("");
        let p = root.path();
        std::fs::write(
            p.join("route-api.ts"),
            "export {Screen as A} from './screen-a';",
        )
        .unwrap();
        let value = if import.contains("Named") {
            "Named"
        } else if import.contains("ns") {
            "ns.A"
        } else {
            "A"
        };
        let source = format!(
            "import type {{Routes}} from '@angular/router'; {import} export const routes:Routes=[{{path:'lifecycle',component:{value}}}];"
        );
        for file in ["one.ts", "two.ts"] {
            std::fs::write(p.join(file), &source).unwrap();
        }
        let state = crate::mcp::McpState::new(config);
        // First acquisition is the registered public query, not capture().
        assert_eq!(
            rows(&forward(&state, p, "lifecycle")).len(),
            2,
            "cold {import}"
        );
        assert_eq!(
            rows(&forward(&state, p, "lifecycle")).len(),
            2,
            "warm {import}"
        );
        call(&state, p, "refresh_workspace", json!({}));
        assert_eq!(
            rows(&forward(&state, p, "lifecycle")).len(),
            2,
            "refresh {import}"
        );
        let scoped = call(
            &state,
            p,
            "workspace_query",
            json!({"type":"forward_edges","domain":"angular","entity_type":"Route","name":"lifecycle","withinPath":p.join("one.ts")}),
        );
        assert_eq!(rows(&scoped).len(), 1);
        assert_eq!(rows(&forward(&state, p, "lifecycle")).len(), 2);
        std::fs::remove_file(p.join("one.ts")).unwrap();
        call(&state, p, "refresh_workspace", json!({}));
        assert_eq!(rows(&forward(&state, p, "lifecycle")).len(), 1);
        std::fs::write(p.join("one.ts"), &source).unwrap();
        call(&state, p, "refresh_workspace", json!({}));
        assert_eq!(rows(&forward(&state, p, "lifecycle")).len(), 2);
    }
}

#[test]
fn issue_162_route_component_controls_unchanged_owner_barrel_api_repair() {
    let _serial = crate::protocol::handler_response_serial();
    for mode in 0..3 {
        let (root, config) = setup("");
        let p = root.path();
        std::fs::write(
            p.join("route-api.ts"),
            "export {Screen as A} from './screen-a';",
        )
        .unwrap();
        let source = "import type {Routes} from '@angular/router'; import {A} from './route-api'; export const routes:Routes=[{path:'same',component:A}];";
        std::fs::write(p.join("routes.ts"), source).unwrap();
        let state = crate::mcp::McpState::new(config.clone());
        let a = capture(&state, p, "routes.ts");
        assert_eq!(project(&a).len(), 1);
        call(
            &state,
            p,
            "save_context",
            json!({"filePath":p.join("routes.ts")}),
        );
        drop(state);
        let dependency = p.join(match mode {
            0 => "route-api.ts",
            1 => "node_modules/@angular/core/index.d.ts",
            _ => "screen-a.ts",
        });
        let original = std::fs::read(&dependency).unwrap();
        std::fs::write(
            &dependency,
            match mode {
                0 => "export {};",
                1 => "export declare function Foreign(metadata:unknown):any;",
                _ => "export class Screen {}",
            },
        )
        .unwrap();
        assert!(
            !a.witnesses
                .iter()
                .all(crate::ir::typescript_resolution::Witness::current)
        );
        let state = crate::mcp::McpState::new(config.clone());
        stale_restore(&state, p);
        let changed = capture(&state, p, "routes.ts");
        assert!(project(&changed).is_empty(), "dependency {mode}");
        assert!(rows(&forward(&state, p, "same")).is_empty());
        drop(state);
        std::fs::write(&dependency, original).unwrap();
        let state = crate::mcp::McpState::new(config);
        assert_eq!(project(&capture(&state, p, "routes.ts")).len(), 1);
        assert_eq!(rows(&forward(&state, p, "same")).len(), 1);
        assert_eq!(
            std::fs::read_to_string(p.join("routes.ts")).unwrap(),
            source
        );
    }
}
