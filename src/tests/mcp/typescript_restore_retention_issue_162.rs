use serde_json::{Value, json};
use std::path::Path;

fn dispatch(state: &crate::mcp::McpState, tool: &str, file: &str, root: &Path) -> Value {
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(
        &json!(162),
        tool,
        &json!({"arguments": {
            "filePath": file, "workspaceRoot": root, "fidelity": "high"
        }}),
        state,
    );
    crate::protocol::captured_responses()
        .pop()
        .expect("registered MCP response")
}

fn persisted_fixture() -> (
    tempfile::TempDir,
    crate::config::CleanCtxConfig,
    String,
    Value,
) {
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    std::fs::write(p.join("tsconfig.json"),
        r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[]}}"#).unwrap();
    std::fs::write(p.join("leaf.ts"), "export class Leaf {}").unwrap();
    let source = "import { Leaf as Local } from './leaf'; export const selected = Local;";
    let file = p.join("app.ts");
    std::fs::write(&file, source).unwrap();
    let file = file.to_string_lossy().into_owned();
    let mut config = crate::tests::test_config();
    config.persistence.enabled = true;
    config.persistence.db_path = p
        .join("restore-authority.db")
        .to_string_lossy()
        .into_owned();
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config.clone());
    let response = dispatch(&state, "compress_code_context", &file, p);
    assert!(response.get("error").is_none(), "{response}");
    state.flush_persistence();
    let alias = state.alias_for_path(&file).unwrap();
    let snapshot = state.semantic_snapshot(&alias).unwrap();
    let authority = snapshot
        .authority
        .typescript_authority
        .as_ref()
        .expect("producer proof");
    let start = source.rfind("Local").unwrap();
    assert!(
        authority
            .reference_sites
            .iter()
            .any(|(from, _, result)| *from == start
                && matches!(result,
            crate::ir::typescript_resolution::Resolution::Resolved(id)
                if id.name == "Leaf" && id.source.ends_with("leaf.ts") && id.start < id.end))
    );
    let expected = serde_json::to_value(authority).unwrap();
    drop(state);
    (root, config, file, expected)
}

#[test]
fn issue_162_restore_retention_restart_preserves_exact_current_payload() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config, file, expected) = persisted_fixture();
    let state = crate::mcp::McpState::new(config);
    assert!(state.alias_for_path(&file).is_none());
    let response = dispatch(&state, "restore_context", &file, root.path());
    assert!(response.get("error").is_none(), "{response}");
    let alias = state.alias_for_path(&file).expect("restored source owner");
    let snapshot = state
        .semantic_snapshot(&alias)
        .expect("current restored snapshot");
    let authority = snapshot
        .authority
        .typescript_authority
        .expect("restore must retain typed authority");
    assert_eq!(serde_json::to_value(authority).unwrap(), expected);
}

#[test]
fn issue_162_restore_retention_rejects_dependency_mutation_before_live_adoption() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config, file, _) = persisted_fixture();
    let source = std::fs::read_to_string(&file).unwrap();
    std::fs::write(root.path().join("leaf.ts"), "export interface Leaf {}").unwrap();
    let state = crate::mcp::McpState::new(config);
    let response = dispatch(&state, "restore_context", &file, root.path());
    assert!(
        response.get("error").is_some(),
        "stale proof adopted: {response}"
    );
    assert!(
        state.alias_for_path(&file).is_none(),
        "failed admission must not install an owner"
    );
    assert_eq!(std::fs::read_to_string(file).unwrap(), source);
}

#[test]
fn issue_162_restore_retention_legacy_projection_does_not_invent_authority() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config, file, _) = persisted_fixture();
    {
        let connection = rusqlite::Connection::open(&config.persistence.db_path).unwrap();
        let encoded: String = connection
            .query_row(
                "SELECT edges_json FROM semantic_edge_snapshots WHERE file_path = ?1",
                [&file],
                |row| row.get(0),
            )
            .unwrap();
        let mut legacy: Value = serde_json::from_str(&encoded).unwrap();
        assert!(
            legacy
                .as_object_mut()
                .unwrap()
                .remove("typescript_authority")
                .is_some()
        );
        assert_eq!(
            connection
                .execute(
                    "UPDATE semantic_edge_snapshots SET edges_json = ?1 WHERE file_path = ?2",
                    rusqlite::params![serde_json::to_string(&legacy).unwrap(), &file]
                )
                .unwrap(),
            1
        );
    }
    let state = crate::mcp::McpState::new(config);
    let response = dispatch(&state, "restore_context", &file, root.path());
    assert!(response.get("error").is_none(), "{response}");
    let alias = state
        .alias_for_path(&file)
        .expect("compatible legacy projection");
    let snapshot = state.semantic_snapshot(&alias).unwrap();
    assert!(snapshot.authority.typescript_authority.is_none());
}
