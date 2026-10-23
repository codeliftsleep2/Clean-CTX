//! Issue #162 foundation contracts; operator RED required before repair.
use crate::mcp::McpState;
use serde_json::{Value, json};
use std::path::Path;

fn state() -> McpState {
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    McpState::new(config)
}

fn write(root: &Path, file: &str, source: &str) {
    let file = root.join(file);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(file, source).unwrap();
}

fn call(state: &McpState, root: &Path, mut args: Value) -> Value {
    args["workspaceRoot"] = json!(root);
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(
        &json!(162),
        "workspace_query",
        &json!({"arguments":args}),
        state,
    );
    let response = crate::protocol::captured_responses().pop().unwrap();
    assert!(response.get("error").is_none(), "{response:#}");
    assert_ne!(response["result"]["isError"], true, "{response:#}");
    response["result"]["structuredContent"].clone()
}

fn edges(state: &McpState, root: &Path, kind: &str, name: &str) -> Value {
    call(
        state,
        root,
        json!({"type":"forward_edges", "domain":"angular", "entity_type":kind, "name":name}),
    )
}

fn membership(result: &Value, relation: &str, expected: &[(&str, &str)]) {
    let mut actual: Vec<_> = result["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|edge| edge["relation"] == relation)
        .map(|edge| {
            (
                edge["object"]["name"].as_str().unwrap().to_string(),
                edge["object"]["entity_type"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    actual.sort();
    actual.dedup();
    let mut expected: Vec<_> = expected
        .iter()
        .map(|(name, kind)| (name.to_string(), kind.to_string()))
        .collect();
    expected.sort();
    assert_eq!(actual, expected, "{result:#}");
}

fn fixture(root: &Path) {
    write(
        root,
        "package.json",
        r#"{"name":"angular-authority-fixture","private":true,"dependencies":{"@angular/core":"19.0.0"}}"#,
    );
    write(
        root,
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler","experimentalDecorators":true}}"#,
    );
    // Synthetic package evidence only; no npm install or provider is needed.
    write(
        root,
        "node_modules/@angular/core/package.json",
        r#"{"name":"@angular/core","version":"19.0.0","exports":{".":{"types":"./index.d.ts","default":"./index.d.ts"}}}"#,
    );
    write(
        root,
        "node_modules/@angular/core/index.d.ts",
        "export declare function Component(value: any): ClassDecorator; export declare function Directive(value: any): ClassDecorator; export declare function Pipe(value: any): ClassDecorator; export declare function NgModule(value: any): ClassDecorator;",
    );
    write(
        root,
        "leaf.ts",
        "import { Directive } from '@angular/core'; @Directive({selector:'[leaf]',standalone:false}) export class Leaf {}",
    );
    module(root, "import { Leaf } from './leaf';", "Leaf");
}

fn module(root: &Path, imports: &str, entries: &str) {
    write(
        root,
        "app.module.ts",
        &format!(
            "import {{ NgModule }} from '@angular/core'; {imports} @NgModule({{declarations:[{entries}]}}) export class AppModule {{}}"
        ),
    );
}

fn expect_leaf(state: &McpState, root: &Path) {
    membership(
        &edges(state, root, "Module", "AppModule"),
        "DeclaresInModule",
        &[("Leaf", "Directive")],
    );
}

#[test]
fn issue_162_existing_direct_control_and_157_lifecycle() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    let state = state();
    expect_leaf(&state, root.path());
    write(
        root.path(),
        "leaf.ts",
        "import { Pipe } from '@angular/core'; @Pipe({name:'leaf',standalone:false}) export class Leaf {}",
    );
    membership(
        &edges(&state, root.path(), "Module", "AppModule"),
        "DeclaresInModule",
        &[("Leaf", "Pipe")],
    );
}

#[test]
fn issue_162_nested_imported_arrays_keep_their_lexical_source() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    write(
        root.path(),
        "core.ts",
        "import { Leaf } from './leaf'; export const CORE = [Leaf];",
    );
    write(
        root.path(),
        "shared.ts",
        "import { CORE } from './core'; const LOCAL = [...CORE]; export const SHARED = [...LOCAL];",
    );
    module(
        root.path(),
        "import { SHARED } from './shared';",
        "...SHARED",
    );
    expect_leaf(&state(), root.path());
}

#[test]
fn issue_162_composition_cycle_retains_independently_proven_leaf() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    module(
        root.path(),
        "import { Leaf } from './leaf'; const A = [...B, Leaf]; const B = [...A];",
        "...A",
    );
    expect_leaf(&state(), root.path());
}

#[test]
fn issue_162_barrel_chain_retargets_without_target_edits() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    write(
        root.path(),
        "barrel/index.ts",
        "export * from '../forward';",
    );
    write(root.path(), "forward.ts", "export { Leaf } from './leaf';");
    module(root.path(), "import { Leaf } from './barrel';", "Leaf");
    let state = state();
    expect_leaf(&state, root.path());
    write(
        root.path(),
        "other.ts",
        "import { Pipe } from '@angular/core'; @Pipe({name:'leaf',standalone:false}) export class Leaf {}",
    );
    write(root.path(), "forward.ts", "export { Leaf } from './other';");
    membership(
        &edges(&state, root.path(), "Module", "AppModule"),
        "DeclaresInModule",
        &[("Leaf", "Pipe")],
    );
}

#[test]
fn issue_162_competing_star_exports_do_not_choose_first() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    expect_leaf(&state(), root.path());
    write(
        root.path(),
        "other.ts",
        "import { Pipe } from '@angular/core'; @Pipe({name:'leaf',standalone:false}) export class Leaf {}",
    );
    write(
        root.path(),
        "barrel.ts",
        "export * from './leaf'; export * from './other';",
    );
    module(root.path(), "import { Leaf } from './barrel';", "Leaf");
    membership(
        &edges(&state(), root.path(), "Module", "AppModule"),
        "DeclaresInModule",
        &[],
    );
}

#[test]
fn issue_162_tsconfig_paths_retarget_invalidates_composition() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    write(
        root.path(),
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler","baseUrl":".","paths":{"@shared":["./leaf.ts"]}}}"#,
    );
    module(root.path(), "import { Leaf } from '@shared';", "Leaf");
    let state = state();
    expect_leaf(&state, root.path());
    write(
        root.path(),
        "other.ts",
        "import { Pipe } from '@angular/core'; @Pipe({name:'leaf',standalone:false}) export class Leaf {}",
    );
    write(
        root.path(),
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler","baseUrl":".","paths":{"@shared":["./other.ts"]}}}"#,
    );
    membership(
        &edges(&state, root.path(), "Module", "AppModule"),
        "DeclaresInModule",
        &[("Leaf", "Pipe")],
    );
}

#[test]
fn issue_162_package_exports_establish_source_identity() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    write(
        root.path(),
        "node_modules/@fixture/shared/package.json",
        r#"{"name":"@fixture/shared","version":"1.0.0","exports":{".":{"types":"./index.ts","default":"./index.ts"}}}"#,
    );
    write(
        root.path(),
        "node_modules/@fixture/shared/index.ts",
        "import { Directive } from '@angular/core'; @Directive({selector:'[leaf]',standalone:false}) export class Leaf {}",
    );
    module(
        root.path(),
        "import { Leaf } from '@fixture/shared';",
        "Leaf",
    );
    expect_leaf(&state(), root.path());
}

#[test]
fn issue_162_package_imports_establish_source_identity() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    write(
        root.path(),
        "package.json",
        r##"{"name":"angular-authority-fixture","private":true,"imports":{"#shared":"./leaf.ts"},"dependencies":{"@angular/core":"19.0.0"}}"##,
    );
    module(root.path(), "import { Leaf } from '#shared';", "Leaf");
    expect_leaf(&state(), root.path());
}

#[test]
fn issue_162_standalone_explicit_and_version_default_share_composition() {
    let _serial = crate::protocol::handler_response_serial();
    for flag in ["standalone:true,", ""] {
        let root = tempfile::tempdir().unwrap();
        fixture(root.path());
        write(
            root.path(),
            "leaf.ts",
            "import { Directive } from '@angular/core'; @Directive({selector:'[leaf]',standalone:true}) export class Leaf {}",
        );
        write(
            root.path(),
            "dashboard.ts",
            &format!(
                "import {{ Component }} from '@angular/core'; import {{ Leaf }} from './leaf'; @Component({{{flag}selector:'app-dashboard',imports:[Leaf],template:''}}) export class Dashboard {{}}"
            ),
        );
        membership(
            &edges(&state(), root.path(), "Component", "Dashboard"),
            "ImportsModule",
            &[("Leaf", "Directive")],
        );
    }
}

#[test]
fn issue_162_aliased_angular_decorator_preserves_declaration_kind() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    write(
        root.path(),
        "leaf.ts",
        "import { Directive as AngularDirective } from '@angular/core'; @AngularDirective({selector:'[leaf]',standalone:false}) export class Leaf {}",
    );
    expect_leaf(&state(), root.path());
}

#[test]
fn issue_162_unrelated_same_spelling_decorator_cannot_grant_authority() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    expect_leaf(&state(), root.path());
    write(
        root.path(),
        "fake.ts",
        "export function Directive(value: unknown): ClassDecorator { return () => {}; }",
    );
    write(
        root.path(),
        "leaf.ts",
        "import { Directive } from './fake'; @Directive({selector:'[leaf]'}) export class Leaf {}",
    );
    membership(
        &edges(&state(), root.path(), "Module", "AppModule"),
        "DeclaresInModule",
        &[],
    );
}

#[test]
fn issue_162_imported_declaration_provenance_is_not_asserting_owner() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    let state = state();
    let result = edges(&state, root.path(), "Module", "AppModule");
    membership(&result, "DeclaresInModule", &[("Leaf", "Directive")]);
    let edge = result["edges"]
        .as_array()
        .unwrap()
        .iter()
        .find(|edge| edge["relation"] == "DeclaresInModule")
        .unwrap();
    let leaf = state.semantic_owner_path(&root.path().join("leaf.ts").to_string_lossy());
    assert_eq!(
        edge["object"]["file"], leaf,
        "declaration identity must retain its own source: {result:#}"
    );
}
