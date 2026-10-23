use serde_json::{Value, json};

const SOURCE: &str = "import { Leaf as Local } from './leaf'; export const selected = Local; export class App { run() { return 1; } }";

fn dispatch(state: &crate::mcp::McpState, tool: &str, args: Value) -> Value {
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(&json!(162), tool, &json!({"arguments": args}), state);
    let response = crate::protocol::captured_responses()
        .pop()
        .expect("registered response");
    assert!(response.get("error").is_none(), "{tool}: {response}");
    response
}

fn fixture() -> (
    tempfile::TempDir,
    crate::config::CleanCtxConfig,
    String,
    crate::mcp::McpState,
) {
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    std::fs::write(p.join("tsconfig.json"),
        r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[]}}"#).unwrap();
    std::fs::write(p.join("leaf.ts"), "export class Leaf {}").unwrap();
    let file = p.join("app.ts").to_string_lossy().into_owned();
    std::fs::write(&file, SOURCE).unwrap();
    let mut config = crate::tests::test_config();
    config.persistence.enabled = true;
    config.persistence.db_path = p
        .join("checkpoint-authority.db")
        .to_string_lossy()
        .into_owned();
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config.clone());
    dispatch(
        &state,
        "provide_code_context",
        json!({
            "filePath": file, "workspaceRoot": p, "fidelity": "edit"
        }),
    );
    let authority = retained(&state, &file);
    assert!(
        authority["reference_sites"]
            .as_array()
            .unwrap()
            .iter()
            .any(|site| site[2]["Resolved"]["name"] == "Leaf"),
        "nonempty exact endpoint: {authority}"
    );
    (root, config, file, state)
}

fn retained(state: &crate::mcp::McpState, file: &str) -> Value {
    let alias = state.alias_for_path(file).expect("source owner");
    let snapshot = state
        .semantic_snapshot(&alias)
        .expect("current session proof");
    serde_json::to_value(
        snapshot
            .authority
            .typescript_authority
            .expect("retained proof"),
    )
    .unwrap()
}

fn persisted(config: &crate::config::CleanCtxConfig, file: &str) -> Value {
    let connection = rusqlite::Connection::open(&config.persistence.db_path).unwrap();
    let encoded: String = connection.query_row(
        "SELECT edges_json FROM semantic_edge_snapshots WHERE context_id = (SELECT id FROM contexts WHERE file_path = ?1 ORDER BY updated_at DESC LIMIT 1) ORDER BY semantic_version DESC LIMIT 1",
        [file], |row| row.get(0)).unwrap();
    serde_json::from_str(&encoded).unwrap()
}

#[test]
fn issue_162_checkpoint_retention_explicit_save_preserves_current_proof() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config, file, state) = fixture();
    let expected = retained(&state, &file);
    state.flush_persistence();
    assert_eq!(
        persisted(&config, &file).get("typescript_authority"),
        Some(&expected)
    );
    dispatch(&state, "save_context", json!({"filePath": file}));
    state.flush_persistence();
    assert_eq!(
        persisted(&config, &file).get("typescript_authority"),
        Some(&expected),
        "explicit checkpoint must preserve typed proof"
    );
    drop(state);
    let restarted = crate::mcp::McpState::new(config);
    dispatch(
        &restarted,
        "restore_context",
        json!({"filePath": file, "workspaceRoot": root.path()}),
    );
    assert_eq!(retained(&restarted, &file), expected);
}

#[test]
fn issue_162_checkpoint_retention_noop_delta_replaces_source_snapshot_proof() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config, file, state) = fixture();
    let changed = format!("{SOURCE}\n");
    std::fs::write(&file, &changed).unwrap();
    state.invalidate_source_cache(&file);
    let response = dispatch(
        &state,
        "delta_code_context",
        json!({
            "filePath": file, "workspaceRoot": root.path(), "fidelity": "edit"
        }),
    );
    assert!(
        response["result"]["delta"].is_null(),
        "fixture must select no-op delta: {response}"
    );
    let expected = retained(&state, &file);
    assert_eq!(
        expected["source_hash"],
        crate::cache::content_hash(changed.as_bytes()),
        "a new source hash must replace old typed snapshot proof even without IR edits"
    );
    dispatch(&state, "save_context", json!({"filePath": file}));
    state.flush_persistence();
    assert_eq!(
        persisted(&config, &file).get("typescript_authority"),
        Some(&expected)
    );
    drop(state);
    let restarted = crate::mcp::McpState::new(config);
    dispatch(
        &restarted,
        "restore_context",
        json!({"filePath": file, "workspaceRoot": root.path()}),
    );
    assert_eq!(retained(&restarted, &file), expected);
}
