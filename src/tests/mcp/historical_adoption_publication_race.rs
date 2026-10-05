use super::historical_adoption_race_test_support;
use crate::compression::Fidelity;
use crate::mcp::tools::dispatch_tools_call;
use serde_json::json;
use std::sync::Arc;

fn dispatch(
    state: &crate::mcp::McpState,
    id: i64,
    tool: &str,
    file: &str,
    root: &str,
    fidelity: Option<&str>,
) {
    let mut arguments = json!({
        "filePath": file,
        "workspaceRoot": root,
    });
    if let Some(fidelity) = fidelity {
        arguments["fidelity"] = json!(fidelity);
    }
    dispatch_tools_call(&json!(id), tool, &json!({ "arguments": arguments }), state);
}

#[test]
fn older_in_flight_restore_cannot_replace_newer_same_owner_publication() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let path = root.path().join("owner.ts");
    let file = path.to_string_lossy().into_owned();
    let root_text = root.path().to_string_lossy().into_owned();
    let historical_source = "export class HistoricalOwner { oldMethod(): void {} }\n";
    let newer_source = "export class CurrentOwner { newMethod(): void {} }\n";
    std::fs::write(&path, historical_source).expect("historical source");

    let mut config = crate::tests::test_config();
    config.persistence.enabled = true;
    config.persistence.auto_save = true;
    config.persistence.db_path = root
        .path()
        .join("historical-adoption-race.db")
        .to_string_lossy()
        .into_owned();
    config.additional_roots.push(root_text.clone());
    let state = Arc::new(crate::mcp::McpState::new(config));

    // Establish durable H1 through the production checkpoint path.
    dispatch(
        &state,
        1,
        "compress_code_context",
        &file,
        &root_text,
        Some("low"),
    );
    let historical_hash = state
        .cache_read()
        .compute_hash(historical_source.as_bytes());
    {
        let store = state.persistence_store_lock();
        let durable = store
            .as_ref()
            .and_then(|store| store.sqlite())
            .and_then(|sqlite| sqlite.load_durable_context(&file, None).ok().flatten())
            .expect("durable H1");
        assert_eq!(durable.source_hash, historical_hash);
    }

    // A begins first and selects H1. The barrier is deliberately inside this
    // already-started restore operation, after durable selection/validation
    // but before live adoption.
    historical_adoption_race_test_support::arm(&file);
    let restore_state = Arc::clone(&state);
    let restore_file = file.clone();
    let restore_root = root_text.clone();
    let restore = std::thread::spawn(move || {
        dispatch(
            &restore_state,
            2,
            "restore_context",
            &restore_file,
            &restore_root,
            None,
        );
    });
    historical_adoption_race_test_support::wait_until_paused();

    // B is newer by operation-authority order, not by snapshot chronology or
    // completion time: its source transition and publication ticket are both
    // established after A began and selected H1.
    std::fs::write(&path, newer_source).expect("newer source");
    state.invalidate_source_cache(&file);
    dispatch(
        &state,
        3,
        "provide_code_context",
        &file,
        &root_text,
        Some("high"),
    );

    let alias = state.alias_for_path(&file).expect("semantic owner alias");
    let newer_hash = state.cache_read().compute_hash(newer_source.as_bytes());
    assert_eq!(
        state.ir_context_read().get_source_hash(&alias),
        Some(&newer_hash),
        "operation B must establish H2 before operation A resumes"
    );
    assert_eq!(state.context_fidelity(&alias), Some(Fidelity::High));
    assert!(
        !state
            .workspace_index_read()
            .find_entities_by_name("CurrentOwner")
            .is_empty(),
        "operation B must publish H2 into WorkspaceIndex"
    );

    historical_adoption_race_test_support::release();
    restore.join().expect("restore worker");

    assert_eq!(
        state.ir_context_read().get_source_hash(&alias),
        Some(&newer_hash),
        "an older in-flight historical adoption must not supersede a later-started same-owner authority operation merely by finishing last"
    );
    assert_eq!(state.context_fidelity(&alias), Some(Fidelity::High));
    let index = state.workspace_index_read();
    assert!(!index.find_entities_by_name("CurrentOwner").is_empty());
    assert!(index.find_entities_by_name("HistoricalOwner").is_empty());
}
