use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

fn state(root: &tempfile::TempDir) -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config.persistence.enabled = true;
    config.persistence.db_path = root
        .path()
        .join("replay-edit-recovery.db")
        .to_string_lossy()
        .into_owned();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    crate::mcp::McpState::new(config)
}

fn dispatch(state: &crate::mcp::McpState, id: i64, tool: &str, args: Value) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&json!(id), tool, &json!({ "arguments": args }), state);
    crate::protocol::captured_responses()
        .pop()
        .expect("registered response")
}

fn source(value: &str) -> Vec<u8> {
    format!("export class Replay {{\r\n  run() {{\r\n    return \"{value}\";\r\n  }}\r\n}}\r\n")
        .into_bytes()
}

fn baseline(state: &crate::mcp::McpState, root: &tempfile::TempDir, file: &str) {
    let response = dispatch(
        state,
        1,
        "compress_code_context",
        json!({
            "filePath": file,
            "workspaceRoot": root.path().to_string_lossy(),
            "fidelity": "edit"
        }),
    );
    assert!(response.get("error").is_none(), "{response}");
}

fn establish_intent(
    state: &crate::mcp::McpState,
    file: &str,
    prior: Vec<u8>,
    target: Vec<u8>,
    transition: &str,
) -> u64 {
    let alias = state.alias_for_path(file).unwrap();
    let prior_hash = state
        .ir_context_read()
        .get_source_hash(&alias)
        .cloned()
        .unwrap();
    let prior_version = state.file_version(&alias).unwrap();
    let (mut target_ir, target_edges, target_hash) =
        crate::mcp::tool_helpers::compile_source_ir_candidate(
            file,
            std::str::from_utf8(&target).unwrap(),
            crate::compression::Fidelity::Edit,
            state,
        )
        .unwrap();
    target_ir.file_id = file.to_string();
    let target_version = target_ir.version;
    let intent = crate::mcp::sqlite_store::EditIntent {
        transition_id: transition.to_string(),
        file_path: file.to_string(),
        prior_hash,
        target_hash,
        prior_version,
        target_version,
        prior_source: prior,
        target_source: target,
        target_ir: crate::ir::binary_wire::encode(&target_ir),
        target_edges,
        fidelity: crate::compression::Fidelity::Edit,
        stage_path: String::new(),
    };
    state
        .persistence_store_lock()
        .as_ref()
        .unwrap()
        .sqlite()
        .unwrap()
        .establish_edit_intent(&intent)
        .unwrap();
    target_version
}

fn has_intent(state: &crate::mcp::McpState, file: &str) -> bool {
    state
        .persistence_store_lock()
        .as_ref()
        .unwrap()
        .sqlite()
        .unwrap()
        .has_edit_intent(file)
        .unwrap()
}

#[test]
fn registered_replay_without_intent_remains_unchanged() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("replay.ts").to_string_lossy().into_owned();
    std::fs::write(&file, source("prior")).unwrap();
    let state = state(&root);
    baseline(&state, &root, &file);

    let response = dispatch(&state, 10, "replay_history", json!({ "filePath": file }));
    assert!(response.get("error").is_none(), "{response}");
}

#[test]
fn registered_targeted_replay_resolves_prior_byte_intent_first() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("prior.ts").to_string_lossy().into_owned();
    let prior = source("prior");
    std::fs::write(&file, &prior).unwrap();
    let state = state(&root);
    baseline(&state, &root, &file);
    establish_intent(
        &state,
        &file,
        prior.clone(),
        source("target"),
        "prior-intent",
    );

    let response = dispatch(
        &state,
        20,
        "replay_history",
        json!({ "filePath": file, "targetSequence": 0 }),
    );
    assert!(response.get("error").is_none(), "{response}");
    assert_eq!(std::fs::read(&file).unwrap(), prior);
    assert!(!has_intent(&state, &file));
}

#[test]
fn registered_replay_recovers_target_commit_before_publication_after_restart() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("target.ts").to_string_lossy().into_owned();
    let prior = source("prior");
    let target = source("target🙂");
    std::fs::write(&file, &prior).unwrap();
    let initial = state(&root);
    baseline(&initial, &root, &file);
    let target_version = establish_intent(&initial, &file, prior, target.clone(), "target-intent");
    std::fs::write(&file, &target).unwrap();
    drop(initial);

    let restarted = state(&root);
    let response = dispatch(
        &restarted,
        30,
        "replay_history",
        json!({ "filePath": file }),
    );
    assert!(response.get("error").is_none(), "{response}");
    assert_eq!(response["result"]["_meta"]["version"], target_version);
    assert_eq!(std::fs::read(&file).unwrap(), target);
    assert!(!has_intent(&restarted, &file));
}

#[test]
fn failed_or_irreconcilable_recovery_cannot_publish_replay_state() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let file = root
        .path()
        .join("failure.ts")
        .to_string_lossy()
        .into_owned();
    let prior = source("prior");
    let target = source("target");
    std::fs::write(&file, &prior).unwrap();
    let state = state(&root);
    baseline(&state, &root, &file);
    establish_intent(&state, &file, prior, target.clone(), "failure-intent");
    std::fs::write(&file, &target).unwrap();
    let alias = state.alias_for_path(&file).unwrap();
    let live_ir = state.ir_context_read().get_ir(&alias).cloned().unwrap();
    let live_hash = state.ir_context_read().get_source_hash(&alias).cloned();
    let live_fidelity = state.context_fidelity(&alias);
    let live_edges = serde_json::to_value(state.semantic_edges(&alias)).unwrap();
    let index_count = state.workspace_index_read().edge_count();

    crate::mcp::sqlite_store::fail_next_semantic_save(&file);
    let failed = dispatch(&state, 40, "replay_history", json!({ "filePath": file }));
    assert!(failed.get("error").is_some(), "{failed}");
    assert_eq!(state.ir_context_read().get_ir(&alias), Some(&live_ir));
    assert_eq!(
        state.ir_context_read().get_source_hash(&alias),
        live_hash.as_ref()
    );
    assert_eq!(state.context_fidelity(&alias), live_fidelity);
    assert_eq!(
        serde_json::to_value(state.semantic_edges(&alias)).unwrap(),
        live_edges
    );
    assert_eq!(state.workspace_index_read().edge_count(), index_count);
    assert!(has_intent(&state, &file));

    std::fs::write(&file, source("irreconcilable")).unwrap();
    let irreconcilable = dispatch(&state, 41, "replay_history", json!({ "filePath": file }));
    assert!(irreconcilable.get("error").is_some(), "{irreconcilable}");
    assert_eq!(state.ir_context_read().get_ir(&alias), Some(&live_ir));
    assert_eq!(state.workspace_index_read().edge_count(), index_count);
    assert!(has_intent(&state, &file));
}
