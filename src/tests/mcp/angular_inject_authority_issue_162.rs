use serde_json::{Value, json};
use std::path::Path;

fn query(state: &crate::mcp::McpState, root: &Path, mut arguments: Value) -> Value {
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

fn fixture() -> (tempfile::TempDir, crate::mcp::McpState) {
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install(p);
    std::fs::write(p.join("node_modules/@angular/core/index.d.ts"), "export declare function Injectable(metadata?: unknown): any; export declare function inject(token: unknown): unknown;").unwrap();
    std::fs::write(p.join("token.ts"), "export class Dependency {}").unwrap();
    std::fs::write(
        p.join("barrel.ts"),
        "export { inject as resolveDependency } from '@angular/core';",
    )
    .unwrap();
    std::fs::write(
        p.join("foreign.ts"),
        "export function inject(token: unknown) { return token; }",
    )
    .unwrap();
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    (root, crate::mcp::McpState::new(config))
}

fn write_owner(root: &Path, imports: &str, callee: &str) {
    std::fs::write(root.join("app.ts"), format!("import {{ Injectable }} from '@angular/core'; {imports} @Injectable() export class Owner {{ dep = {callee}(Dependency); }}")).unwrap();
}

fn dependencies(state: &crate::mcp::McpState, root: &Path) -> Vec<Value> {
    let inventory = query(
        state,
        root,
        json!({"type":"entities_in_file","file_path":root.join("app.ts"),"fidelity":"high"}),
    );
    assert!(
        inventory["entities"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entity| entity["domain"] == "angular"
                && entity["entity_type"] == "Service"
                && entity["name"] == "Owner"),
        "canonical owner control: {inventory}"
    );
    let result = query(
        state,
        root,
        json!({"type":"forward_edges","domain":"angular","entity_type":"Service","name":"Owner"}),
    );
    result["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|edge| edge["relation"] == "Injects")
        .cloned()
        .collect()
}

#[test]
fn issue_162_inject_authority_barrel_alias_retains_api_and_token_endpoint() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state) = fixture();
    let p = root.path();
    write_owner(
        p,
        "import { inject } from '@angular/core'; import { Dependency } from './token';",
        "inject",
    );
    assert_eq!(
        dependencies(&state, p).len(),
        1,
        "unaliased extraction control"
    );
    write_owner(
        p,
        "import { resolveDependency as obtain } from './barrel'; import { Dependency } from './token';",
        "obtain",
    );
    let edges = dependencies(&state, p);
    assert_eq!(
        edges.len(),
        1,
        "canonical inject API through aliases: {edges:?}"
    );
    assert_eq!(edges[0]["object"]["name"], "Dependency");
    assert_eq!(
        edges[0]["object"]["file"],
        json!(crate::dictionary::path::canonical_identity_key(
            &p.join("token.ts").to_string_lossy()
        ))
    );
}

#[test]
fn issue_162_inject_authority_foreign_spelling_cannot_grant_dependency() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state) = fixture();
    let p = root.path();
    write_owner(
        p,
        "import { inject } from '@angular/core'; import { Dependency } from './token';",
        "inject",
    );
    assert_eq!(dependencies(&state, p).len(), 1);
    write_owner(
        p,
        "import { inject } from './foreign'; import { Dependency } from './token';",
        "inject",
    );
    assert!(
        dependencies(&state, p).is_empty(),
        "foreign inject spelling has no Angular API authority"
    );
    write_owner(
        p,
        "import { inject } from '@angular/core'; import { Dependency } from './token';",
        "inject",
    );
    assert_eq!(dependencies(&state, p).len(), 1);
}

#[test]
fn issue_162_inject_authority_type_only_token_cannot_gain_runtime_use() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state) = fixture();
    let p = root.path();
    write_owner(
        p,
        "import { inject } from '@angular/core'; import { Dependency } from './token';",
        "inject",
    );
    assert_eq!(dependencies(&state, p).len(), 1);
    write_owner(
        p,
        "import { inject } from '@angular/core'; import type { Dependency } from './token';",
        "inject",
    );
    assert!(
        dependencies(&state, p).is_empty(),
        "type-only token has no runtime injection authority"
    );
}
