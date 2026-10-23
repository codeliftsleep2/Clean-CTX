use crate::ir::typescript_resolution::{Resolution, TypeScriptAuthority};
use serde_json::json;
use std::path::Path;

fn provide(state: &crate::mcp::McpState, root: &Path, file: &str, id: i64) {
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(
        &json!(id),
        "provide_code_context",
        &json!({"arguments": {
            "filePath": file, "workspaceRoot": root, "fidelity": "high"
        }}),
        state,
    );
    let response = crate::protocol::captured_responses()
        .pop()
        .expect("MCP response");
    assert!(response.get("error").is_none(), "{response}");
}

fn retained(state: &crate::mcp::McpState, file: &str) -> TypeScriptAuthority {
    let alias = state.alias_for_path(file).expect("published source owner");
    state
        .semantic_snapshot(&alias)
        .expect("current session semantic snapshot")
        .authority
        .typescript_authority
        .expect("provide must retain typed TypeScript authority, including on warm requests")
}

fn assert_leaf(authority: &TypeScriptAuthority, source: &str, leaf: &str) {
    let start = source.rfind("Local").unwrap();
    assert!(
        authority.reference_sites.iter().any(|(from, to, result)| {
            *from == start
                && *to == start + "Local".len()
                && matches!(result, Resolution::Resolved(id)
                if id.name == "Leaf" && id.source.ends_with("leaf.ts")
                    && id.source_hash == crate::cache::content_hash(leaf.as_bytes())
                    && id.start < id.end)
        }),
        "exact dependency endpoint: {:?}",
        authority.reference_sites
    );
}

#[test]
fn issue_162_provide_retention_cold_warm_and_dependency_mutation() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    std::fs::write(p.join("tsconfig.json"),
        r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[]}}"#).unwrap();
    let leaf = "export class Leaf {}";
    std::fs::write(p.join("leaf.ts"), leaf).unwrap();
    let source = "import { Leaf as Local } from './leaf'; export const selected = Local;";
    let file = p.join("app.ts");
    std::fs::write(&file, source).unwrap();
    let file = file.to_str().unwrap();
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    let control = crate::mcp::tool_helpers::compile_source_ir_candidate(
        file,
        source,
        crate::compression::Fidelity::High,
        &state,
        p.to_str(),
    )
    .expect("nonempty source authority control");
    assert_leaf(control.typescript_authority(), source, leaf);
    assert!(control.typescript_authority().capture_current);
    provide(&state, p, file, 1);
    let first = retained(&state, file);
    assert_leaf(&first, source, leaf);
    provide(&state, p, file, 2);
    let warm = retained(&state, file);
    assert_eq!(
        serde_json::to_value(&first).unwrap(),
        serde_json::to_value(&warm).unwrap()
    );

    let changed = "export class Leaf { updated(): void {} }";
    std::fs::write(p.join("leaf.ts"), changed).unwrap();
    assert!(first.witnesses.iter().any(|witness| !witness.current()));
    let alias = state.alias_for_path(file).unwrap();
    assert!(
        state.semantic_snapshot(&alias).is_none(),
        "stale dependency proof must be refused"
    );
    assert_eq!(std::fs::read_to_string(file).unwrap(), source);
    provide(&state, p, file, 3);
    assert_leaf(&retained(&state, file), source, changed);
}
