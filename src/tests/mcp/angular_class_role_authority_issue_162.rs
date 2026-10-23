use serde_json::json;
use std::path::Path;

fn count(state: &crate::mcp::McpState, root: &Path, file: &Path, role: &str) -> usize {
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(
        &json!(162),
        "workspace_query",
        &json!({"arguments":{"type":"entities_in_file","file_path":file,
            "workspaceRoot":root,"fidelity":"high"}}),
        state,
    );
    let response = crate::protocol::captured_responses().pop().unwrap();
    assert!(response.get("error").is_none(), "{response}");
    response["result"]["structuredContent"]["entities"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|entity| {
            entity["domain"] == "angular"
                && entity["entity_type"] == role
                && entity["name"] == "Owner"
        })
        .count()
}

fn fixture(api: &str) -> (tempfile::TempDir, crate::mcp::McpState) {
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
    std::fs::write(
        package.join("index.d.ts"),
        format!("export declare function {api}(metadata: unknown): any;"),
    )
    .unwrap();
    std::fs::write(
        p.join("foreign.ts"),
        format!("export function {api}(metadata: unknown) {{ return metadata; }}"),
    )
    .unwrap();
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    (root, crate::mcp::McpState::new(config))
}

fn source(api: &str, local: &str, module: &str) -> String {
    format!(
        "import {{ {api} as {local} }} from '{module}'; @{local}({{name:'demo',selector:'[demo]'}}) class Owner {{}}"
    )
}

fn alias_contract(api: &str, role: &str) {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state) = fixture(api);
    let file = root.path().join("app.ts");
    std::fs::write(&file, source(api, api, "@angular/core")).unwrap();
    assert_eq!(
        count(&state, root.path(), &file, role),
        1,
        "unaliased {api} declaration must be publicly represented without unrelated relationships"
    );
    std::fs::write(&file, source(api, "RoleAlias", "@angular/core")).unwrap();
    assert_eq!(
        count(&state, root.path(), &file, role),
        1,
        "{api} alias must retain the canonical {role} role"
    );
    assert_eq!(count(&state, root.path(), &file, role), 1, "warm alias");
    std::fs::write(&file, format!("import type {{ {api} }} from '@angular/core'; @{api}({{name:'demo',selector:'[demo]'}}) class Owner {{}}" )).unwrap();
    assert_eq!(
        count(&state, root.path(), &file, role),
        0,
        "erased {api} cannot grant a runtime role"
    );
}

fn foreign_contract(api: &str, role: &str) {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state) = fixture(api);
    let file = root.path().join("app.ts");
    // Prove the exact source target before testing the public consumer.
    let text = source(api, api, "./foreign");
    std::fs::write(&file, &text).unwrap();
    let artifact = crate::mcp::tool_helpers::compile_source_ir_candidate(
        file.to_str().unwrap(),
        &text,
        crate::compression::Fidelity::High,
        &state,
        root.path().to_str(),
    )
    .unwrap();
    let authority = artifact.typescript_authority();
    let start = text.find(&format!("@{api}")).unwrap() + 1;
    assert!(
        authority.reference_sites.iter().any(|site| site.0 == start
            && matches!(&site.2, crate::ir::typescript_resolution::Resolution::Resolved(id)
            if id.source.ends_with("foreign.ts") && id.start < id.end)),
        "foreign endpoint control"
    );
    assert_eq!(
        count(&state, root.path(), &file, role),
        0,
        "foreign {api} spelling cannot grant a {role} role"
    );
}

#[test]
fn issue_162_class_role_injectable_alias_and_erasure() {
    alias_contract("Injectable", "Service");
}
#[test]
fn issue_162_class_role_module_alias_and_erasure() {
    alias_contract("NgModule", "Module");
}
#[test]
fn issue_162_class_role_directive_alias_and_erasure() {
    alias_contract("Directive", "Directive");
}
#[test]
fn issue_162_class_role_pipe_alias_and_erasure() {
    alias_contract("Pipe", "Pipe");
}
#[test]
fn issue_162_class_role_foreign_injectable() {
    foreign_contract("Injectable", "Service");
}
#[test]
fn issue_162_class_role_foreign_module() {
    foreign_contract("NgModule", "Module");
}
#[test]
fn issue_162_class_role_foreign_directive() {
    foreign_contract("Directive", "Directive");
}
#[test]
fn issue_162_class_role_foreign_pipe() {
    foreign_contract("Pipe", "Pipe");
}
