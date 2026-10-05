use super::super::provide::publication_race_test_support;
use crate::compression::Fidelity;
use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};
use std::sync::Arc;

fn dispatch(state: &crate::mcp::McpState, id: i64, tool: &str, file: &str, root: &str) {
    dispatch_tools_call(
        &json!(id),
        tool,
        &json!({ "arguments": {
            "filePath": file,
            "workspaceRoot": root,
            "fidelity": "high"
        }}),
        state,
    );
}

fn response(id: i64) -> Value {
    crate::protocol::captured_responses()
        .iter()
        .find(|response| response["id"] == id)
        .cloned()
        .expect("captured response")
}

fn assert_newer_authority(
    state: &crate::mcp::McpState,
    file: &str,
    newer_source: &str,
    newer_entity: &str,
    older_entity: &str,
) {
    let alias = state.alias_for_path(file).expect("semantic owner alias");
    let newer_hash = state.cache_read().compute_hash(newer_source.as_bytes());
    assert_eq!(
        state.ir_context_read().get_source_hash(&alias),
        Some(&newer_hash)
    );
    assert_eq!(state.context_fidelity(&alias), Some(Fidelity::High));
    let index = state.workspace_index_read();
    assert!(!index.find_entities_by_name(newer_entity).is_empty());
    assert!(index.find_entities_by_name(older_entity).is_empty());
    drop(index);
    let edges = serde_json::to_string(&state.semantic_edges(&alias)).expect("semantic edges");
    assert!(edges.contains(newer_entity), "{edges}");
    assert!(!edges.contains(older_entity), "{edges}");
}

#[test]
fn stale_first_baseline_delta_cannot_replace_newer_publication() {
    let _serial = crate::protocol::handler_response_serial();
    crate::protocol::captured_responses().clear();
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
        .join("delta-publication-race.db")
        .to_string_lossy()
        .into();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into());
    let state = Arc::new(crate::mcp::McpState::new(config));
    let file_text = file.to_string_lossy().into_owned();
    let root_text = root.path().to_string_lossy().into_owned();

    publication_race_test_support::arm(&file_text);
    let worker_state = Arc::clone(&state);
    let worker_file = file_text.clone();
    let worker_root = root_text.clone();
    let older = std::thread::spawn(move || {
        crate::protocol::captured_responses().clear();
        dispatch(
            &worker_state,
            1,
            "delta_code_context",
            &worker_file,
            &worker_root,
        );
        response(1)
    });
    publication_race_test_support::wait_until_paused();

    std::fs::write(&file, newer_source).expect("newer source");
    state.invalidate_source_cache(&file_text);
    dispatch(&state, 2, "provide_code_context", &file_text, &root_text);
    publication_race_test_support::release();
    let older_response = older.join().expect("older worker");

    assert!(older_response.get("error").is_some(), "{older_response}");
    assert_newer_authority(&state, &file_text, newer_source, "NewerOwner", "OlderOwner");
    let store = state.persistence_store_lock();
    let durable = store
        .as_ref()
        .and_then(|store| store.sqlite())
        .and_then(|sqlite| sqlite.load_durable_context(&file_text, None).ok().flatten())
        .expect("durable checkpoint");
    assert_eq!(
        durable.source_hash,
        state.cache_read().compute_hash(newer_source.as_bytes())
    );
}

#[test]
fn delta_derivation_rejects_a_superseded_baseline_epoch() {
    let _serial = crate::protocol::handler_response_serial();
    crate::protocol::captured_responses().clear();
    let root = tempfile::tempdir().expect("workspace");
    let file = root.path().join("owner.ts");
    let initial_source = "export class InitialOwner { initial(): void {} }\n";
    let candidate_source = "export class CandidateOwner { candidate(): void {} }\n";
    let newer_source = "export class NewerOwner { newer(): void {} }\n";
    std::fs::write(&file, initial_source).expect("initial source");

    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into());
    let state = Arc::new(crate::mcp::McpState::new(config));
    let file_text = file.to_string_lossy().into_owned();
    let root_text = root.path().to_string_lossy().into_owned();
    dispatch(&state, 10, "delta_code_context", &file_text, &root_text);
    let alias = state.alias_for_path(&file_text).expect("baseline alias");
    assert_eq!(state.file_version(&alias), Some(1));

    std::fs::write(&file, candidate_source).expect("candidate source");
    state.invalidate_source_cache(&file_text);
    publication_race_test_support::arm(&file_text);
    let worker_state = Arc::clone(&state);
    let worker_file = file_text.clone();
    let worker_root = root_text.clone();
    let candidate = std::thread::spawn(move || {
        crate::protocol::captured_responses().clear();
        dispatch(
            &worker_state,
            11,
            "delta_code_context",
            &worker_file,
            &worker_root,
        );
        response(11)
    });
    publication_race_test_support::wait_until_paused();

    std::fs::write(&file, newer_source).expect("newer source");
    state.invalidate_source_cache(&file_text);
    dispatch(&state, 12, "provide_code_context", &file_text, &root_text);
    publication_race_test_support::release();
    let candidate_response = candidate.join().expect("candidate worker");

    assert!(
        candidate_response.get("error").is_some(),
        "{candidate_response}"
    );
    assert_eq!(state.pending_transition_count(&alias), 0);
    assert_newer_authority(
        &state,
        &file_text,
        newer_source,
        "NewerOwner",
        "CandidateOwner",
    );
}
