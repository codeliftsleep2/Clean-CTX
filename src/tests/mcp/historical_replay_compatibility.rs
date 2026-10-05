use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

fn state(root: &tempfile::TempDir) -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config.persistence.enabled = true;
    config.persistence.db_path = root
        .path()
        .join("historical-replay.db")
        .to_string_lossy()
        .into_owned();
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

fn baseline(root: &tempfile::TempDir, source: &str) -> String {
    let path = root.path().join("history.ts");
    let file = path.to_string_lossy().into_owned();
    std::fs::write(&path, source).expect("historical source");
    let producer = state(root);
    let saved = dispatch(
        &producer,
        1,
        "compress_code_context",
        json!({
            "filePath": file,
            "workspaceRoot": root.path(),
            "fidelity": "low"
        }),
    );
    assert!(saved.get("error").is_none(), "{saved}");
    file
}

fn replay(state: &crate::mcp::McpState, root: &tempfile::TempDir, file: &str) -> Value {
    dispatch(
        state,
        2,
        "replay_history",
        json!({
            "filePath": file,
            "workspaceRoot": root.path(),
            "targetSequence": 0
        }),
    )
}

fn assert_clean_rejection(
    state: &crate::mcp::McpState,
    file: &str,
    response: &Value,
    reason: &str,
) {
    assert_eq!(response["error"]["data"]["reason"], reason, "{response}");
    assert!(state.alias_for_path(file).is_none());
    assert_eq!(state.workspace_index_read().edge_count(), 0);
    assert!(state.llm_text_cache_lock().is_empty());
}

#[test]
fn compatible_historical_replay_keeps_h1_identity_and_never_covers_h2() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let h1 = "export class HistoricalOne {}\n";
    let h2 = "export class CurrentTwo {}\n";
    let file = baseline(&root, h1);
    std::fs::write(&file, h2).expect("current source");

    let restarted = state(&root);
    let replayed = replay(&restarted, &root, &file);
    assert!(replayed.get("error").is_none(), "{replayed}");
    let alias = restarted.alias_for_path(&file).expect("historical alias");
    let h1_hash = restarted.cache_read().compute_hash(h1.as_bytes());
    let h2_hash = restarted.cache_read().compute_hash(h2.as_bytes());
    let replayed_hash = restarted
        .ir_context_read()
        .get_source_hash(&alias)
        .cloned();
    assert_eq!(replayed_hash, Some(h1_hash.clone()));
    assert_ne!(h1_hash, h2_hash);
    let canonical = restarted.semantic_owner_path(&file);
    assert!(!restarted.workspace_index_read().has_current_semantic_projection(
        &canonical,
        crate::workspace::index::SemanticFidelity::Low,
        &h2_hash,
    ));
}

#[test]
fn historical_replay_rejects_producer_generation_mismatch() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let file = baseline(&root, "export class ProducerEpoch {}\n");
    let mutator = state(&root);
    {
        let store = mutator.persistence_store_lock();
        let sqlite = store.as_ref().unwrap().sqlite().unwrap();
        let identities = sqlite.stored_compatibility_json(&file).unwrap();
        let changed = identities[1]
            .as_ref()
            .unwrap()
            .replace("\"shared_canonical_pipeline\":1", "\"shared_canonical_pipeline\":99");
        sqlite
            .execute_batch(&format!(
                "UPDATE contexts SET canonical_producer_identity = '{}'",
                changed.replace('\'', "''")
            ))
            .unwrap();
    }
    drop(mutator);
    let restarted = state(&root);
    let rejected = replay(&restarted, &root, &file);
    assert_clean_rejection(
        &restarted,
        &file,
        &rejected,
        "canonical_producer_incompatible",
    );
}

#[test]
fn historical_replay_rejects_legacy_missing_identity() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let file = baseline(&root, "export class LegacyHistory {}\n");
    let mutator = state(&root);
    {
        let store = mutator.persistence_store_lock();
        let sqlite = store.as_ref().unwrap().sqlite().unwrap();
        sqlite
            .execute_batch("UPDATE contexts SET canonical_config_identity = NULL")
            .unwrap();
    }
    drop(mutator);
    let restarted = state(&root);
    let rejected = replay(&restarted, &root, &file);
    assert_clean_rejection(&restarted, &file, &rejected, "missing_legacy_identity");
}

#[test]
fn compatible_current_replay_preserves_existing_behavior() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let file = baseline(&root, "export class CurrentHistory {}\n");
    let restarted = state(&root);
    let replayed = replay(&restarted, &root, &file);
    assert!(replayed.get("error").is_none(), "{replayed}");
    let alias = restarted.alias_for_path(&file).expect("replayed alias");
    assert!(restarted.ir_context_read().has_file(&alias));
    assert!(restarted.semantic_edges(&alias).is_some());
    assert!(restarted.context_fidelity(&alias).is_some());
    assert_eq!(restarted.persisted_path(&alias).as_deref(), Some(file.as_str()));
}
