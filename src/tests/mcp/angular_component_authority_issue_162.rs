use crate::ir::typescript_resolution::{Resolution, TypeScriptAuthority, Witness};
use serde_json::json;
use std::path::{Path, PathBuf};

const API: &str = "export declare function Component(metadata: unknown): any;";

fn config(root: &Path, redirect: bool) {
    let mut options = json!({"module":"esnext", "moduleResolution":"bundler",
        "moduleDetection":"legacy", "noLib":true, "types":[]});
    if redirect {
        options["paths"] = json!({"@angular/core":["./foreign.ts"]});
    }
    std::fs::write(
        root.join("tsconfig.json"),
        json!({"compilerOptions": options}).to_string(),
    )
    .unwrap();
}

fn component_source(module: &str, local: &str, owner: &str) -> String {
    format!(
        "import {{ Component as {local} }} from '{module}'; @{local}({{selector:'x-demo', template:''}}) class {owner} {{}}"
    )
}

fn components(state: &crate::mcp::McpState, root: &Path, file: &Path, owner: &str) -> usize {
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(
        &json!(162),
        "workspace_query",
        &json!({"arguments": {"type":"entities_in_file", "file_path":file,
            "workspaceRoot":root, "fidelity":"high"}}),
        state,
    );
    let response = crate::protocol::captured_responses()
        .pop()
        .expect("registered MCP response");
    assert!(response.get("error").is_none(), "{response}");
    response["result"]["structuredContent"]["entities"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|entity| {
            entity["domain"] == "angular"
                && entity["entity_type"] == "Component"
                && entity["name"] == owner
        })
        .count()
}

fn fixture() -> (tempfile::TempDir, crate::mcp::McpState, TypeScriptAuthority) {
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    config(p, false);
    let package = p.join("node_modules/@angular/core");
    std::fs::create_dir_all(&package).unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{"name":"@angular/core","version":"20.0.0","types":"index.d.ts"}"#,
    )
    .unwrap();
    std::fs::write(package.join("index.d.ts"), API).unwrap();
    std::fs::write(
        p.join("foreign.ts"),
        "export function Component(metadata: unknown) { return metadata; }",
    )
    .unwrap();
    let source = component_source("@angular/core", "Component", "Control");
    let file = p.join("control.ts");
    std::fs::write(&file, &source).unwrap();
    let mut cfg = crate::tests::test_config();
    cfg.additional_roots.push(p.to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(cfg);
    let artifact = crate::mcp::tool_helpers::compile_source_ir_candidate(
        file.to_str().unwrap(),
        &source,
        crate::compression::Fidelity::High,
        &state,
        p.to_str(),
    )
    .unwrap();
    let authority = artifact.typescript_authority().clone();
    assert!(
        authority.disposition.is_none(),
        "{:?}",
        authority.disposition
    );
    let target = source.find("@Component").unwrap() + 1;
    let suffix = PathBuf::from("node_modules").join("@angular/core/index.d.ts");
    assert!(
        authority
            .reference_sites
            .iter()
            .any(|(start, end, result)| *start == target
                && *end == target + "Component".len()
                && matches!(result, Resolution::Resolved(id)
                if Path::new(&id.source).ends_with(&suffix)
                    && id.source_hash == crate::cache::content_hash(API.as_bytes())
                    && id.start < id.end)),
        "canonical decorator control: {:?}",
        authority.reference_sites
    );
    assert!(authority.witnesses.iter().any(|witness| matches!(witness,
        Witness::Content {path, ..} if path.ends_with("node_modules/@angular/core/package.json"))));
    assert_eq!(
        components(&state, p, &file, "Control"),
        1,
        "unaliased real API control"
    );
    (root, state, authority)
}

#[test]
fn issue_162_angular_component_authority_alias_reaches_public_query() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state, _) = fixture();
    let file = root.path().join("app.ts");
    std::fs::write(&file, component_source("@angular/core", "Paint", "Demo")).unwrap();
    assert_eq!(
        components(&state, root.path(), &file, "Demo"),
        1,
        "actual Component export must keep its role through a local alias"
    );
}

#[test]
fn issue_162_angular_component_authority_foreign_namesake_cannot_gain_role() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state, _) = fixture();
    let file = root.path().join("app.ts");
    std::fs::write(&file, component_source("./foreign", "Component", "Demo")).unwrap();
    assert_eq!(
        components(&state, root.path(), &file, "Demo"),
        0,
        "written decorator spelling cannot grant Angular API authority"
    );
}

#[test]
fn issue_162_angular_component_authority_package_spelling_cannot_override_target() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state, before) = fixture();
    config(root.path(), true);
    assert!(before.witnesses.iter().any(|witness| !witness.current()));
    let file = root.path().join("app.ts");
    std::fs::write(
        &file,
        component_source("@angular/core", "Component", "Demo"),
    )
    .unwrap();
    assert_eq!(
        components(&state, root.path(), &file, "Demo"),
        0,
        "package-looking import must not replace actual declaration/package proof"
    );
}

#[test]
fn issue_162_angular_component_boundary_barrel_alias_keeps_public_role() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state, _) = fixture();
    let file = root.path().join("app.ts");
    std::fs::write(
        root.path().join("bridge.ts"),
        "export { Component as Paint } from '@angular/core';",
    )
    .unwrap();
    std::fs::write(
        &file,
        "import { Paint as Tag } from './bridge'; @Tag({selector:'x-demo', template:''}) class Demo {}",
    )
    .unwrap();
    assert_eq!(components(&state, root.path(), &file, "Demo"), 1);
    assert_eq!(components(&state, root.path(), &file, "Demo"), 1);
}

#[test]
fn issue_162_angular_component_boundary_parameter_shadow_cannot_borrow_role() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state, _) = fixture();
    let file = root.path().join("app.ts");
    std::fs::write(
        &file,
        "import { Component as Paint } from '@angular/core'; function render(Paint: any) { @Paint({selector:'x-demo', template:''}) class Demo {} }",
    )
    .unwrap();
    assert_eq!(
        components(&state, root.path(), &file, "Demo"),
        0,
        "the decorator's parameter endpoint cannot borrow the imported API role"
    );
}

#[test]
fn issue_162_angular_component_boundary_type_only_import_cannot_grant_runtime_role() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state, _) = fixture();
    let file = root.path().join("app.ts");
    std::fs::write(
        &file,
        "import type { Component } from '@angular/core'; @Component({selector:'x-demo', template:''}) class Demo {}",
    )
    .unwrap();
    assert_eq!(
        components(&state, root.path(), &file, "Demo"),
        0,
        "an erased import cannot authorize a runtime decorator"
    );
}

#[test]
fn issue_162_angular_component_boundary_warm_package_role_mutation_and_restoration() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state, _) = fixture();
    let file = root.path().join("app.ts");
    let source = component_source("@angular/core", "Paint", "Demo");
    std::fs::write(&file, &source).unwrap();
    assert_eq!(components(&state, root.path(), &file, "Demo"), 1);
    assert_eq!(components(&state, root.path(), &file, "Demo"), 1);

    let manifest = root.path().join("node_modules/@angular/core/package.json");
    let original = std::fs::read(&manifest).unwrap();
    std::fs::write(
        &manifest,
        r#"{"name":"foreign-core","version":"20.0.0","types":"index.d.ts"}"#,
    )
    .unwrap();
    assert_eq!(std::fs::read_to_string(&file).unwrap(), source);
    assert_eq!(
        components(&state, root.path(), &file, "Demo"),
        0,
        "unchanged consumer bytes cannot preserve a retired package role"
    );
    std::fs::write(&manifest, original).unwrap();
    assert_eq!(components(&state, root.path(), &file, "Demo"), 1);
}
