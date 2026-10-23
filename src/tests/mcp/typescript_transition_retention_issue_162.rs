use crate::ir::typescript_resolution::Resolution;
use serde_json::{Value, json};

const OLD_BODY: &str = "{ return 1; }";
const NEW_BODY: &str = "{ return 2; }";

fn source(body: &str) -> String {
    format!(
        "import {{ Leaf as Local }} from './leaf'; export const selected = Local; export class App {{ run() {body} }}"
    )
}

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
    std::fs::write(&file, source(OLD_BODY)).unwrap();
    let mut config = crate::tests::test_config();
    config.persistence.enabled = true;
    config.persistence.db_path = p
        .join("transition-authority.db")
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
    assert_current(&state, &file, &source(OLD_BODY));
    (root, config, file, state)
}

fn assert_current(state: &crate::mcp::McpState, file: &str, expected_source: &str) -> Value {
    let alias = state.alias_for_path(file).expect("published source owner");
    let snapshot = state
        .semantic_snapshot(&alias)
        .expect("current semantic snapshot");
    let authority = snapshot
        .authority
        .typescript_authority
        .expect("transition must retain typed authority");
    let hash = crate::cache::content_hash(expected_source.as_bytes());
    assert_eq!(snapshot.source_hash, hash);
    assert_eq!(authority.source_hash, hash);
    let start = expected_source.rfind("Local").unwrap();
    assert!(
        authority
            .reference_sites
            .iter()
            .any(|(from, to, result)| *from == start
                && *to == start + "Local".len()
                && matches!(result, Resolution::Resolved(id)
                if id.name == "Leaf" && id.source.ends_with("leaf.ts") && id.start < id.end)),
        "exact endpoint after transition: {:?}",
        authority.reference_sites
    );
    serde_json::to_value(authority).unwrap()
}

fn assert_durable_and_restart(
    state: crate::mcp::McpState,
    config: crate::config::CleanCtxConfig,
    root: &std::path::Path,
    file: &str,
    expected: Value,
) {
    state.flush_persistence();
    let connection = rusqlite::Connection::open(&config.persistence.db_path).unwrap();
    let encoded: String = connection.query_row(
        "SELECT edges_json FROM semantic_edge_snapshots WHERE file_path = ?1 ORDER BY semantic_version DESC LIMIT 1",
        [file], |row| row.get(0)).unwrap();
    let persisted: Value = serde_json::from_str(&encoded).unwrap();
    assert_eq!(persisted.get("typescript_authority"), Some(&expected));
    drop(connection);
    drop(state);
    let restarted = crate::mcp::McpState::new(config);
    dispatch(
        &restarted,
        "restore_context",
        json!({"filePath": file, "workspaceRoot": root}),
    );
    assert_eq!(
        assert_current(&restarted, file, &source(NEW_BODY)),
        expected
    );
}

#[test]
fn issue_162_transition_retention_apply_edit_replaces_and_restores_proof() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config, file, state) = fixture();
    dispatch(
        &state,
        "apply_edit",
        json!({
            "filePath": file, "workspaceRoot": root.path(),
            "operations": [{"type": "replace_body", "target": "App.run",
                "expectedOldText": OLD_BODY, "newText": NEW_BODY}]
        }),
    );
    assert_eq!(std::fs::read_to_string(&file).unwrap(), source(NEW_BODY));
    let expected = assert_current(&state, &file, &source(NEW_BODY));
    assert_durable_and_restart(state, config, root.path(), &file, expected);
}

#[test]
fn issue_162_transition_retention_delta_acceptance_replaces_and_restores_proof() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config, file, state) = fixture();
    std::fs::write(&file, source(NEW_BODY)).unwrap();
    state.invalidate_source_cache(&file);
    let delta = dispatch(
        &state,
        "delta_code_context",
        json!({
            "filePath": file, "workspaceRoot": root.path(), "fidelity": "edit"
        }),
    );
    assert!(
        delta["result"]["delta"].is_object(),
        "nonempty transition: {delta}"
    );
    dispatch(
        &state,
        "apply_delta",
        json!({
            "delta": delta["result"]["delta"],
            "currentVersion": delta["result"]["from_version"]
        }),
    );
    let expected = assert_current(&state, &file, &source(NEW_BODY));
    assert_durable_and_restart(state, config, root.path(), &file, expected);
}
