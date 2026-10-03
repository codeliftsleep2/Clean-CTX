use super::publication_race_test_support;
use crate::compression::Fidelity;
use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};
use std::sync::Arc;

fn dispatch(state: &crate::mcp::McpState, id: i64, file: &str, root: &str, fidelity: &str) {
    dispatch_tools_call(
        &json!(id),
        "provide_code_context",
        &json!({ "arguments": {
            "filePath": file,
            "workspaceRoot": root,
            "fidelity": fidelity
        }}),
        state,
    );
}

#[test]
fn older_lower_fidelity_snapshot_cannot_replace_newer_publication() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let file = root.path().join("owner.ts");
    let older_source = "export class OlderOwner { oldMethod(): void {} }\n";
    let newer_source = "export class NewerOwner { newMethod(): void {} }\n";
    std::fs::write(&file, older_source).expect("older source");

    let mut config = crate::tests::test_config();
    config.persistence.enabled = true;
    config.persistence.auto_save = true;
    config.persistence.db_path = root
        .path()
        .join("publication-race.db")
        .to_string_lossy()
        .into();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into());
    let state = Arc::new(crate::mcp::McpState::new(config));
    let file_text = file.to_string_lossy().into_owned();
    let root_text = root.path().to_string_lossy().into_owned();
    let older_hash = state.cache_read().compute_hash(older_source.as_bytes());
    let newer_hash = state.cache_read().compute_hash(newer_source.as_bytes());

    publication_race_test_support::arm(&file_text);
    let worker_state = Arc::clone(&state);
    let worker_file = file_text.clone();
    let worker_root = root_text.clone();
    let older = std::thread::spawn(move || {
        dispatch(&worker_state, 1, &worker_file, &worker_root, "low");
    });
    publication_race_test_support::wait_until_paused();

    std::fs::write(&file, newer_source).expect("newer source");
    state.invalidate_source_cache(&file_text);
    dispatch(&state, 2, &file_text, &root_text, "high");
    publication_race_test_support::release();
    older.join().expect("older worker");

    let alias = state
        .alias_for_path(&file_text)
        .expect("semantic owner alias");
    assert_eq!(
        state.ir_context_read().get_source_hash(&alias),
        Some(&newer_hash),
        "canonical IR must retain the newer source snapshot, not {older_hash}"
    );
    assert_eq!(state.context_fidelity(&alias), Some(Fidelity::High));
    let index = state.workspace_index_read();
    assert!(!index.find_entities_by_name("NewerOwner").is_empty());
    assert!(index.find_entities_by_name("OlderOwner").is_empty());
    drop(index);
    let edges: Value = serde_json::to_value(state.semantic_edges(&alias)).expect("semantic edges");
    assert!(edges.to_string().contains("NewerOwner"), "{edges}");
    assert!(!edges.to_string().contains("OlderOwner"), "{edges}");

    let store = state.persistence_store_lock();
    let durable = store
        .as_ref()
        .and_then(|store| store.sqlite())
        .and_then(|sqlite| sqlite.load_durable_context(&file_text, None).ok().flatten())
        .expect("durable checkpoint");
    assert_eq!(durable.source_hash, newer_hash);
    assert_eq!(durable.fidelity, Fidelity::High);
}
