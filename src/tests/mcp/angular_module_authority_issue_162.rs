use serde_json::{Value, json};
use std::path::Path;

fn call(state: &crate::mcp::McpState, root: &Path, mut arguments: Value) -> Value {
    arguments["workspaceRoot"] = json!(root);
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(
        &json!(162),
        "workspace_query",
        &json!({"arguments":arguments}),
        state,
    );
    let response = crate::protocol::captured_responses().pop().unwrap();
    assert!(response.get("error").is_none(), "{response}");
    response["result"]["structuredContent"].clone()
}

fn role(state: &crate::mcp::McpState, root: &Path, file: &Path, kind: &str, name: &str) -> usize {
    let result = call(
        state,
        root,
        json!({"type":"entities_in_file","file_path":file,"fidelity":"high"}),
    );
    result["entities"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|entity| {
            entity["domain"] == "angular" && entity["entity_type"] == kind && entity["name"] == name
        })
        .count()
}

fn members(state: &crate::mcp::McpState, root: &Path) -> Vec<Value> {
    let result = call(
        state,
        root,
        json!({"type":"forward_edges","domain":"angular",
        "entity_type":"Module","name":"AppModule"}),
    );
    result["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|edge| edge["relation"] == "DeclaresInModule")
        .cloned()
        .collect()
}

fn fixture() -> (tempfile::TempDir, crate::mcp::McpState) {
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    std::fs::write(p.join("tsconfig.json"), r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[]}}"#).unwrap();
    let package = p.join("node_modules/@angular/core");
    std::fs::create_dir_all(&package).unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{"name":"@angular/core","version":"20.0.0","types":"index.d.ts"}"#,
    )
    .unwrap();
    std::fs::write(package.join("index.d.ts"), "export declare function NgModule(metadata: unknown): any; export declare function Directive(metadata: unknown): any;").unwrap();
    std::fs::write(
        p.join("foreign.ts"),
        "export function Directive(metadata: unknown) { return metadata; }",
    )
    .unwrap();
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    (root, crate::mcp::McpState::new(config))
}

fn write_leaf(root: &Path, local: &str, module: &str) {
    std::fs::write(root.join("leaf.ts"), format!("import {{ Directive as {local} }} from '{module}'; @{local}({{selector:'[leaf]'}}) export class Leaf {{}}" )).unwrap();
}

fn write_owner(root: &Path, local: &str) {
    std::fs::write(root.join("app.ts"), format!("import {{ NgModule as {local} }} from '@angular/core'; import {{ Leaf }} from './leaf'; @{local}({{declarations:[Leaf]}}) export class AppModule {{}}" )).unwrap();
}

fn exact_member(state: &crate::mcp::McpState, root: &Path) {
    let edges = members(state, root);
    assert_eq!(edges.len(), 1, "one canonical module member: {edges:?}");
    assert_eq!(edges[0]["object"]["name"], "Leaf");
    assert_eq!(edges[0]["object"]["entity_type"], "Directive");
}

#[test]
fn issue_162_module_authority_owner_alias_retains_composition() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state) = fixture();
    let p = root.path();
    write_leaf(p, "Directive", "@angular/core");
    write_owner(p, "NgModule");
    assert_eq!(role(&state, p, &p.join("leaf.ts"), "Directive", "Leaf"), 1);
    assert_eq!(role(&state, p, &p.join("app.ts"), "Module", "AppModule"), 1);
    exact_member(&state, p);
    write_owner(p, "ModuleAlias");
    assert_eq!(role(&state, p, &p.join("app.ts"), "Module", "AppModule"), 1);
    exact_member(&state, p);
}

#[test]
fn issue_162_module_authority_target_alias_retains_composition() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state) = fixture();
    let p = root.path();
    write_leaf(p, "Directive", "@angular/core");
    write_owner(p, "NgModule");
    assert_eq!(role(&state, p, &p.join("app.ts"), "Module", "AppModule"), 1);
    exact_member(&state, p);
    write_leaf(p, "DirectiveAlias", "@angular/core");
    assert_eq!(role(&state, p, &p.join("leaf.ts"), "Directive", "Leaf"), 1);
    role(&state, p, &p.join("app.ts"), "Module", "AppModule");
    exact_member(&state, p);
}

#[test]
fn issue_162_module_authority_foreign_target_cannot_gain_membership() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state) = fixture();
    let p = root.path();
    write_leaf(p, "Directive", "@angular/core");
    write_owner(p, "NgModule");
    assert_eq!(role(&state, p, &p.join("leaf.ts"), "Directive", "Leaf"), 1);
    exact_member(&state, p);
    write_leaf(p, "Directive", "./foreign");
    assert_eq!(role(&state, p, &p.join("leaf.ts"), "Directive", "Leaf"), 0);
    role(&state, p, &p.join("app.ts"), "Module", "AppModule");
    assert!(
        members(&state, p).is_empty(),
        "a foreign decorator cannot authorize a Directive membership"
    );
    write_leaf(p, "Directive", "@angular/core");
    role(&state, p, &p.join("app.ts"), "Module", "AppModule");
    exact_member(&state, p);
}
