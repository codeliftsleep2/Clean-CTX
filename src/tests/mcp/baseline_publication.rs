use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

fn state_with_persistence(root: &tempfile::TempDir) -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config.persistence.enabled = true;
    config.persistence.db_path = root.path().join("p9-14.db").to_string_lossy().into_owned();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    crate::mcp::McpState::new(config)
}

fn dispatch(state: &crate::mcp::McpState, id: i64, tool: &str, arguments: Value) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&json!(id), tool, &json!({ "arguments": arguments }), state);
    crate::protocol::captured_responses()
        .pop()
        .expect("registered response")
}

fn args(file: &str, root: &tempfile::TempDir) -> Value {
    json!({
        "filePath": file,
        "workspaceRoot": root.path().to_string_lossy(),
        "fidelity": "low"
    })
}

#[test]
fn failed_compress_baseline_commit_publishes_no_candidate_live_state() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let file = root.path().join("owned.ts").to_string_lossy().into_owned();
    let peer = root.path().join("peer.ts").to_string_lossy().into_owned();
    std::fs::write(&file, "export class Before { run(): void {} }\n").unwrap();
    std::fs::write(&peer, "export class Peer { stay(): void {} }\n").unwrap();
    let state = state_with_persistence(&root);

    let prior_response = dispatch(&state, 1, "compress_code_context", args(&file, &root));
    assert!(prior_response.get("error").is_none(), "{prior_response}");
    let prior_render = prior_response["result"]["content"][0]["text"]
        .as_str()
        .expect("compact response")
        .to_string();
    assert!(
        dispatch(&state, 2, "compress_code_context", args(&peer, &root))
            .get("error")
            .is_none()
    );
    let alias = state.alias_for_path(&file).expect("owned alias");
    let peer_alias = state.alias_for_path(&peer).expect("peer alias");
    let prior_ir = state.ir_context_read().get_ir(&alias).cloned().unwrap();
    let prior_hash = state
        .ir_context_read()
        .get_source_hash(&alias)
        .cloned()
        .unwrap();
    let prior_version = state.file_version(&alias).unwrap();
    let prior_fidelity = state.context_fidelity(&alias);
    let prior_edges = serde_json::to_value(state.semantic_edges(&alias)).unwrap();
    let prior_index_count = state.workspace_index_read().edge_count();
    let prior_compact = state.llm_text_cache_lock().get(&alias).cloned();
    let prior_peer_ir = state
        .ir_context_read()
        .get_ir(&peer_alias)
        .cloned()
        .unwrap();

    std::fs::write(
        &file,
        "export class Candidate { changed(): void { throw new Error(); } }\n",
    )
    .unwrap();
    state.invalidate_source_cache(&file);
    crate::mcp::sqlite_store::fail_next_semantic_save(&file);
    let failed = dispatch(&state, 3, "compress_code_context", args(&file, &root));
    assert!(
        failed["error"]["message"]
            .as_str()
            .is_some_and(|message| message.contains("persisted atomically")),
        "{failed}"
    );

    let live = state.ir_context_read();
    assert_eq!(live.get_ir(&alias), Some(&prior_ir));
    assert_eq!(live.get_source_hash(&alias), Some(&prior_hash));
    assert_eq!(live.file_version(&alias), Some(prior_version));
    assert_eq!(live.get_ir(&peer_alias), Some(&prior_peer_ir));
    drop(live);
    assert_eq!(state.context_fidelity(&alias), prior_fidelity);
    assert_eq!(
        serde_json::to_value(state.semantic_edges(&alias)).unwrap(),
        prior_edges
    );
    assert_eq!(state.workspace_index_read().edge_count(), prior_index_count);
    assert!(
        state
            .workspace_index_read()
            .find_entities_by_name("Candidate")
            .is_empty()
    );
    assert!(
        !state
            .workspace_index_read()
            .find_entities_by_name("Before")
            .is_empty()
    );
    assert_eq!(
        state.llm_text_cache_lock().get(&alias),
        prior_compact.as_ref()
    );

    let restored = dispatch(
        &state,
        4,
        "restore_context",
        json!({ "filePath": file.clone() }),
    );
    assert!(restored.get("error").is_none(), "{restored}");
    assert_eq!(restored["result"]["content"][0]["text"], prior_render);
}

#[test]
fn failed_first_delta_baseline_commit_creates_no_live_owner() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let peer = root.path().join("peer.ts").to_string_lossy().into_owned();
    let target = root.path().join("target.ts").to_string_lossy().into_owned();
    std::fs::write(&peer, "export class Peer { stay(): void {} }\n").unwrap();
    std::fs::write(&target, "export class Target { start(): void {} }\n").unwrap();
    let state = state_with_persistence(&root);
    assert!(
        dispatch(&state, 10, "compress_code_context", args(&peer, &root))
            .get("error")
            .is_none()
    );
    let peer_alias = state.alias_for_path(&peer).expect("peer alias");
    let peer_ir = state
        .ir_context_read()
        .get_ir(&peer_alias)
        .cloned()
        .unwrap();
    let prior_index_count = state.workspace_index_read().edge_count();

    crate::mcp::sqlite_store::fail_next_semantic_save(&target);
    let failed = dispatch(&state, 11, "delta_code_context", args(&target, &root));
    assert!(failed.get("error").is_some(), "{failed}");
    assert!(state.alias_for_path(&target).is_none());
    assert_eq!(state.workspace_index_read().edge_count(), prior_index_count);
    assert!(
        state
            .workspace_index_read()
            .find_entities_by_name("Target")
            .is_empty()
    );
    assert_eq!(state.ir_context_read().get_ir(&peer_alias), Some(&peer_ir));

    let successful = dispatch(&state, 12, "delta_code_context", args(&target, &root));
    assert!(successful.get("error").is_none(), "{successful}");
    let target_alias = state.alias_for_path(&target).expect("committed alias");
    assert!(state.ir_context_read().get_ir(&target_alias).is_some());
    assert_eq!(state.file_version(&target_alias), Some(1));
    assert!(state.context_fidelity(&target_alias).is_some());
    assert!(state.semantic_edges(&target_alias).is_some());
    assert_eq!(
        state.persisted_path(&target_alias).as_deref(),
        Some(target.as_str())
    );
}
