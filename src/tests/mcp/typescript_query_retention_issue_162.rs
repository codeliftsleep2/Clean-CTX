use crate::ir::typescript_resolution::Resolution;
use serde_json::json;

fn query(state: &crate::mcp::McpState, root: &std::path::Path, file: &str, kind: &str) {
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(
        &json!(162),
        "workspace_query",
        &json!({"arguments": {"type": kind, "file_path": file,
            "workspaceRoot": root, "fidelity": "high"}}),
        state,
    );
    let response = crate::protocol::captured_responses()
        .pop()
        .expect("MCP response");
    assert!(response.get("error").is_none(), "{response}");
}

fn exercise(kind: &str) {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    std::fs::write(p.join("tsconfig.json"),
        r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[]}}"#).unwrap();
    let leaf = "export class Leaf {}";
    std::fs::write(p.join("leaf.ts"), leaf).unwrap();
    let source = "import { Leaf as Local } from './leaf'; export const selected = Local;";
    let file = p.join("app.ts").to_string_lossy().into_owned();
    std::fs::write(&file, source).unwrap();
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    let control = crate::mcp::tool_helpers::compile_source_ir_candidate(
        &file,
        source,
        crate::compression::Fidelity::High,
        &state,
        p.to_str(),
    )
    .unwrap();
    let start = source.rfind("Local").unwrap();
    assert!(
        control
            .typescript_authority()
            .reference_sites
            .iter()
            .any(|(from, _, result)| *from == start
                && matches!(result, Resolution::Resolved(id)
            if id.name == "Leaf" && id.source.ends_with("leaf.ts")))
    );
    let canonical = state.semantic_owner_path(&file);
    query(&state, p, &file, kind);
    assert!(
        state.alias_for_path(&file).is_none(),
        "query must not install a session context"
    );
    let first = state
        .workspace_index_read()
        .retained_typescript_authority(&canonical)
        .cloned()
        .expect("publishing file query must retain typed source authority");
    assert_eq!(
        serde_json::to_value(&first).unwrap(),
        serde_json::to_value(control.typescript_authority()).unwrap()
    );
    query(&state, p, &file, kind);
    let warm = state
        .workspace_index_read()
        .retained_typescript_authority(&canonical)
        .cloned()
        .unwrap();
    assert_eq!(
        serde_json::to_value(&first).unwrap(),
        serde_json::to_value(warm).unwrap()
    );

    let changed = "export class Leaf { updated(): void {} }";
    std::fs::write(p.join("leaf.ts"), changed).unwrap();
    assert!(first.witnesses.iter().any(|witness| !witness.current()));
    assert!(
        !state
            .workspace_index_read()
            .has_current_semantic_projection(
                &canonical,
                crate::workspace::index::SemanticFidelity::High,
                &crate::cache::content_hash(source.as_bytes())
            )
    );
    query(&state, p, &file, kind);
    let after = state
        .workspace_index_read()
        .retained_typescript_authority(&canonical)
        .cloned()
        .unwrap();
    assert!(
        after
            .reference_sites
            .iter()
            .any(|(from, _, result)| *from == start
                && matches!(result, Resolution::Resolved(id)
            if id.source_hash == crate::cache::content_hash(changed.as_bytes())))
    );
    assert!(state.alias_for_path(&file).is_none());
}

#[test]
fn issue_162_query_retention_entities_cold_warm_and_dependency_mutation() {
    exercise("entities_in_file");
}

#[test]
fn issue_162_query_retention_declarations_cold_warm_and_dependency_mutation() {
    exercise("declarations_in_file");
}
