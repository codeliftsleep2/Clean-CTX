//! Production lifecycle contract for `persistence.auto_save`.

use crate::compression::Fidelity;
use crate::mcp::context_store::ContextStore;
use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

fn config(
    root: &tempfile::TempDir,
    enabled: bool,
    auto_save: bool,
    database: &str,
) -> crate::config::CleanCtxConfig {
    let mut config = crate::tests::test_config();
    config.persistence.enabled = enabled;
    config.persistence.auto_save = auto_save;
    config.persistence.db_path = root.path().join(database).to_string_lossy().into_owned();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    config
}

fn dispatch(state: &crate::mcp::McpState, id: i64, tool: &str, arguments: Value) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&json!(id), tool, &json!({ "arguments": arguments }), state);
    crate::protocol::captured_responses()
        .pop()
        .expect("registered handler response")
}

fn write_source(root: &tempfile::TempDir, name: &str, owner: &str) -> String {
    let path = root.path().join(name);
    std::fs::write(
        &path,
        format!("export class {owner} {{ run(value: number): number {{ return value + 1; }} }}\n"),
    )
    .expect("source fixture");
    path.to_string_lossy().into_owned()
}

fn context_args(file: &str, root: &tempfile::TempDir, fidelity: &str) -> Value {
    json!({
        "filePath": file,
        "workspaceRoot": root.path().to_string_lossy(),
        "fidelity": fidelity
    })
}

fn assert_not_durable(state: &crate::mcp::McpState, file: &str) {
    let store = state.persistence_store_lock();
    let sqlite = store
        .as_ref()
        .expect("persistence store")
        .sqlite()
        .expect("SQLite store");
    assert!(
        !sqlite.has_context(file),
        "unexpected checkpoint for {file}"
    );
}

fn assert_durable(state: &crate::mcp::McpState, file: &str) {
    let store = state.persistence_store_lock();
    let sqlite = store
        .as_ref()
        .expect("persistence store")
        .sqlite()
        .expect("SQLite store");
    assert!(sqlite.has_context(file), "missing checkpoint for {file}");
}

#[test]
fn disabled_persistence_publishes_only_session_state() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let file = write_source(&root, "disabled.ts", "DisabledOwner");
    let state = crate::mcp::McpState::new(config(&root, false, true, "disabled.db"));

    let response = dispatch(
        &state,
        1,
        "provide_code_context",
        context_args(&file, &root, "low"),
    );
    assert!(response.get("error").is_none(), "{response}");
    assert!(state.persistence_store_lock().is_none());
    let alias = state.alias_for_path(&file).expect("session alias");
    assert!(state.ir_context_read().has_file(&alias));
}

#[test]
fn auto_save_false_keeps_read_producers_session_only() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let provided = write_source(&root, "provided.ts", "ProvidedOwner");
    let compressed = write_source(&root, "compressed.ts", "CompressedOwner");
    let delta = write_source(&root, "delta.ts", "DeltaOwner");
    let state = crate::mcp::McpState::new(config(&root, true, false, "manual.db"));

    for (id, tool, file) in [
        (10, "provide_code_context", &provided),
        (11, "compress_code_context", &compressed),
        (12, "delta_code_context", &delta),
    ] {
        let response = dispatch(&state, id, tool, context_args(file, &root, "low"));
        assert!(response.get("error").is_none(), "{tool}: {response}");
        assert_not_durable(&state, file);
        let alias = state.alias_for_path(file).expect("session alias");
        assert!(state.ir_context_read().has_file(&alias));
    }
}

#[test]
fn auto_save_true_checkpoints_all_canonical_read_producers() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let provided = write_source(&root, "provided.ts", "ProvidedOwner");
    let compressed = write_source(&root, "compressed.ts", "CompressedOwner");
    let delta = write_source(&root, "delta.ts", "DeltaOwner");
    let state = crate::mcp::McpState::new(config(&root, true, true, "automatic.db"));

    for (id, tool, file) in [
        (20, "provide_code_context", &provided),
        (21, "compress_code_context", &compressed),
        (22, "delta_code_context", &delta),
    ] {
        let response = dispatch(&state, id, tool, context_args(file, &root, "high"));
        assert!(response.get("error").is_none(), "{tool}: {response}");
        assert_durable(&state, file);
    }
}

#[test]
fn failed_automatic_checkpoint_publishes_no_semantic_candidate() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let file = write_source(&root, "failure.ts", "FailureOwner");
    let state = crate::mcp::McpState::new(config(&root, true, true, "failure.db"));
    crate::mcp::sqlite_store::fail_next_semantic_save(&file);

    let response = dispatch(
        &state,
        25,
        "provide_code_context",
        context_args(&file, &root, "low"),
    );
    assert!(response.get("error").is_some(), "{response}");
    let alias = state.alias_for_path(&file).expect("reserved alias");
    assert!(!state.ir_context_read().has_file(&alias));
    assert_eq!(state.context_fidelity(&alias), None);
    assert!(state.semantic_edges(&alias).is_none());
    assert!(
        state
            .workspace_index_read()
            .find_entities_by_name("FailureOwner")
            .is_empty()
    );
    assert_not_durable(&state, &file);
}

#[test]
fn explicit_save_persists_and_restores_complete_manual_checkpoint() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let file = write_source(&root, "manual.ts", "ManualOwner");
    let config = config(&root, true, false, "explicit.db");
    let state = crate::mcp::McpState::new(config.clone());

    let provided = dispatch(
        &state,
        30,
        "provide_code_context",
        context_args(&file, &root, "high"),
    );
    assert!(provided.get("error").is_none(), "{provided}");
    assert_not_durable(&state, &file);
    let alias = state.alias_for_path(&file).expect("session alias");
    let expected_version = state.file_version(&alias).expect("version");
    let expected_hash = state
        .ir_context_read()
        .get_source_hash(&alias)
        .cloned()
        .expect("source hash");
    let expected_edges = serde_json::to_value(
        state
            .semantic_edges(&alias)
            .expect("authoritative semantic edges"),
    )
    .expect("serializable edges");

    let saved = dispatch(
        &state,
        31,
        "save_context",
        json!({ "filePath": file.clone() }),
    );
    assert_eq!(saved["result"]["_meta"]["saved"], 1, "{saved}");
    {
        let store = state.persistence_store_lock();
        let sqlite = store.as_ref().unwrap().sqlite().unwrap();
        let durable = sqlite
            .load_durable_context(&file, None)
            .expect("durable lookup")
            .expect("durable checkpoint");
        assert_eq!(durable.ir.version, expected_version);
        assert_eq!(durable.source_hash, expected_hash);
        assert_eq!(durable.fidelity, Fidelity::High);
        assert_eq!(
            serde_json::to_value(&durable.semantic_edges).unwrap(),
            expected_edges
        );
    }
    drop(state);

    let restarted = crate::mcp::McpState::new(config);
    let restored = dispatch(
        &restarted,
        32,
        "restore_context",
        json!({ "filePath": file.clone() }),
    );
    assert!(restored.get("error").is_none(), "{restored}");
    let restored_alias = restarted.alias_for_path(&file).expect("restored alias");
    assert_eq!(
        restarted.file_version(&restored_alias),
        Some(expected_version)
    );
    assert_eq!(
        restarted.ir_context_read().get_source_hash(&restored_alias),
        Some(&expected_hash)
    );
    assert_eq!(
        restarted.context_fidelity(&restored_alias),
        Some(Fidelity::High)
    );
    assert_eq!(
        serde_json::to_value(
            restarted
                .semantic_edges(&restored_alias)
                .expect("restored semantic edges"),
        )
        .unwrap(),
        expected_edges
    );
}

#[test]
fn edit_read_establishes_durable_authority_when_auto_save_is_false() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let file = write_source(&root, "editable.ts", "EditableOwner");
    let state = crate::mcp::McpState::new(config(&root, true, false, "edit.db"));

    let response = dispatch(
        &state,
        40,
        "provide_code_context",
        context_args(&file, &root, "edit"),
    );
    assert!(response.get("error").is_none(), "{response}");
    assert_durable(&state, &file);
}
