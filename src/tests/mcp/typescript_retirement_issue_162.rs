use serde_json::{Value, json};

fn dispatch(state: &crate::mcp::McpState, tool: &str, args: Value) -> Value {
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(&json!(162), tool, &json!({"arguments": args}), state);
    let response = crate::protocol::captured_responses()
        .pop()
        .expect("MCP response");
    assert!(response.get("error").is_none(), "{tool}: {response}");
    response
}

#[test]
fn issue_162_retirement_delete_context_retires_only_owned_proof_and_preserves_source() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    std::fs::write(p.join("tsconfig.json"),
        r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[]}}"#).unwrap();
    std::fs::write(p.join("leaf.ts"), "export class Leaf {}").unwrap();
    let source = "import { Leaf as Local } from './leaf'; export const selected = Local;";
    let files = [p.join("a.ts"), p.join("b.ts")];
    for file in &files {
        std::fs::write(file, source).unwrap();
    }
    let mut config = crate::tests::test_config();
    config.persistence.enabled = true;
    config.persistence.db_path = p.join("retirement.db").to_string_lossy().into_owned();
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config.clone());
    let mut aliases = Vec::new();
    let mut proofs = Vec::new();
    for file in &files {
        dispatch(
            &state,
            "provide_code_context",
            json!({
                "filePath": file, "workspaceRoot": p, "fidelity": "edit"
            }),
        );
        let alias = state.alias_for_path(file.to_str().unwrap()).unwrap();
        let authority = state
            .semantic_snapshot(&alias)
            .unwrap()
            .authority
            .typescript_authority
            .expect("nonempty retained owner proof");
        assert!(
            authority
                .reference_sites
                .iter()
                .any(|(_, _, result)| matches!(result,
            crate::ir::typescript_resolution::Resolution::Resolved(id) if id.name == "Leaf"))
        );
        proofs.push(serde_json::to_value(authority).unwrap());
        aliases.push(alias);
    }
    dispatch(&state, "delete_context", json!({"filePath": files[0]}));
    assert!(
        state.semantic_snapshot(&aliases[0]).is_none(),
        "deleted owner retained proof"
    );
    assert_eq!(std::fs::read_to_string(&files[0]).unwrap(), source);
    let survivor = state
        .semantic_snapshot(&aliases[1])
        .expect("other owner remains current");
    assert_eq!(
        serde_json::to_value(survivor.authority.typescript_authority.unwrap()).unwrap(),
        proofs[1]
    );
    state.flush_persistence();
    let connection = rusqlite::Connection::open(&config.persistence.db_path).unwrap();
    let deleted_rows: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM semantic_edge_snapshots WHERE file_path = ?1",
            [files[0].to_str().unwrap()],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(deleted_rows, 0, "deleted durable proof must not resurrect");
    drop(connection);
    drop(state);
    let restarted = crate::mcp::McpState::new(config);
    dispatch(
        &restarted,
        "restore_context",
        json!({"filePath": files[1], "workspaceRoot": p}),
    );
    let alias = restarted
        .alias_for_path(files[1].to_str().unwrap())
        .unwrap();
    let survivor = restarted.semantic_snapshot(&alias).unwrap();
    assert_eq!(
        serde_json::to_value(survivor.authority.typescript_authority.unwrap()).unwrap(),
        proofs[1]
    );
}
