//! Production-boundary eager membership controls; no runtime router meaning.
pub(super) use super::angular_bootstrap_inline_controls_tests::{call, capture};
use crate::ir::typescript_resolution::TypeScriptAuthority;
use crate::layers::meta::semantic::SemanticEdge;
use serde_json::{Value, json};
use std::path::Path;
pub(super) const PREFIX: &str = "import type {Routes} from '@angular/router'; import {Screen as A} from './screen-a'; import {Screen as B} from './screen-b'; import {Token} from './left';";
pub(super) const SCREEN: &str =
    "import {Component} from '@angular/core'; @Component({}) export class Screen {}";
pub(super) fn setup(body: &str) -> (tempfile::TempDir, crate::config::CleanCtxConfig) {
    let (root, config) = super::angular_bootstrap_inline_controls_tests::setup("");
    let p = root.path();
    let router = p.join("node_modules/@angular/router");
    std::fs::create_dir_all(&router).unwrap();
    std::fs::write(
        router.join("package.json"),
        r#"{"name":"@angular/router","version":"20.0.0","types":"index.d.ts"}"#,
    )
    .unwrap();
    std::fs::write(router.join("index.d.ts"), "export interface Route {path?:string;component?:unknown;providers?:unknown[];children?:Route[];} export type Routes=Route[];").unwrap();
    for file in ["screen-a.ts", "screen-b.ts"] {
        std::fs::write(p.join(file), SCREEN).unwrap();
    }
    std::fs::write(
        p.join("routes.ts"),
        format!("{PREFIX} export const routes:Routes=[{body}];"),
    )
    .unwrap();
    (root, config)
}
pub(super) fn project(a: &TypeScriptAuthority) -> Vec<SemanticEdge> {
    crate::angular_meta::route_components::project(a)
}
pub(super) fn forward(state: &crate::mcp::McpState, p: &Path, name: &str) -> Value {
    call(
        state,
        p,
        "workspace_query",
        json!({"type":"forward_edges","domain":"angular","entity_type":"Route","name":name}),
    )
}
pub(super) fn rows(answer: &Value) -> Vec<&Value> {
    answer["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["relation"] == "RouteMapsTo")
        .collect()
}

pub(super) fn stale_restore(state: &crate::mcp::McpState, p: &Path) {
    let response = super::angular_bootstrap_inline_controls_tests::raw(
        state,
        p,
        "restore_context",
        json!({"filePath":p.join("routes.ts")}),
    );
    assert_eq!(response["error"]["code"], -32603, "{response}");
    assert_eq!(
        response["error"]["data"]["component"], "semantic_dependencies",
        "{response}"
    );
    assert_eq!(
        response["error"]["data"]["reason"], "semantic_dependency_stale",
        "{response}"
    );
}
