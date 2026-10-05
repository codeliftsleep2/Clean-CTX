use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

fn state_with_config(
    root: &tempfile::TempDir,
    configure: impl FnOnce(&mut crate::config::CleanCtxConfig),
) -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config.persistence.enabled = true;
    config.persistence.db_path = root
        .path()
        .join("restore-compatibility.db")
        .to_string_lossy()
        .into_owned();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    configure(&mut config);
    crate::mcp::McpState::new(config)
}

fn dispatch(state: &crate::mcp::McpState, id: i64, tool: &str, arguments: Value) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&json!(id), tool, &json!({ "arguments": arguments }), state);
    crate::protocol::captured_responses()
        .pop()
        .expect("registered response")
}

fn assert_clean_rejection(state: &crate::mcp::McpState, file: &str, response: &Value, reason: &str) {
    assert_eq!(response["error"]["data"]["reason"], reason, "{response}");
    assert!(state.alias_for_path(file).is_none());
    assert_eq!(state.workspace_index_read().edge_count(), 0);
    assert!(state.llm_text_cache_lock().is_empty());
}

fn save_baseline(
    root: &tempfile::TempDir,
    name: &str,
    source: &str,
    configure: impl FnOnce(&mut crate::config::CleanCtxConfig),
) -> String {
    let path = root.path().join(name);
    let file = path.to_string_lossy().into_owned();
    std::fs::write(&path, source).expect("source");
    let producer = state_with_config(root, configure);
    let saved = dispatch(
        &producer,
        100,
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

fn restore(state: &crate::mcp::McpState, root: &tempfile::TempDir, file: &str) -> Value {
    dispatch(
        state,
        101,
        "restore_context",
        json!({ "filePath": file, "workspaceRoot": root.path() }),
    )
}

fn rewrite_identity(
    state: &crate::mcp::McpState,
    file: &str,
    slot: usize,
    table: &str,
    column: &str,
    rewrite: impl FnOnce(String) -> String,
) {
    let store = state.persistence_store_lock();
    let sqlite = store.as_ref().unwrap().sqlite().unwrap();
    let stored = sqlite
        .stored_compatibility_json(file)
        .expect("stored identities");
    let encoded = rewrite(stored[slot].clone().expect("identity JSON"));
    let escaped = encoded.replace('\'', "''");
    sqlite
        .execute_batch(&format!("UPDATE {table} SET {column} = '{escaped}'"))
        .expect("rewrite identity");
}

#[test]
fn restore_rejects_canonical_type_alias_config_before_live_mutation() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("temporary workspace");
    let path = root.path().join("model.ts");
    let file = path.to_string_lossy().into_owned();
    std::fs::write(&path, "export class Model { value: UserId; }\n").expect("source");

    let producer = state_with_config(&root, |_| {});
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
    drop(producer);

    let restarted = state_with_config(&root, |config| {
        config
            .type_aliases
            .insert("uid".to_string(), "UserId".to_string());
    });
    assert!(restarted.alias_for_path(&file).is_none());
    assert_eq!(restarted.workspace_index_read().edge_count(), 0);
    assert!(restarted.llm_text_cache_lock().is_empty());

    let rejected = dispatch(
        &restarted,
        2,
        "restore_context",
        json!({ "filePath": file, "workspaceRoot": root.path() }),
    );
    assert_eq!(
        rejected["error"]["data"]["reason"],
        "canonical_configuration_incompatible",
        "{rejected}"
    );
    assert_eq!(
        rejected["error"]["data"]["component"],
        "canonical_configuration",
        "{rejected}"
    );
    assert!(restarted.alias_for_path(&file).is_none());
    assert_eq!(restarted.workspace_index_read().edge_count(), 0);
    assert!(restarted.llm_text_cache_lock().is_empty());
}

#[test]
fn restore_rejects_marker_only_configuration_mismatch() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let file = save_baseline(
        &root,
        "marker.ts",
        "import { Component } from '@angular/core';\n@Component({selector:'x'})\nexport class App {}\n",
        |_| {},
    );
    let restarted = state_with_config(&root, |config| {
        config
            .meta_layers
            .entry("angular".to_string())
            .or_default()
            .rxjs
            .min_pipe_operators = 9;
    });
    let rejected = restore(&restarted, &root, &file);
    assert_clean_rejection(
        &restarted,
        &file,
        &rejected,
        "canonical_configuration_incompatible",
    );
}

#[test]
fn restore_rejects_semantic_config_enabled_to_disabled() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let file = save_baseline(
        &root,
        "semantic-off.ts",
        "import { Component } from '@angular/core';\n@Component({selector:'x'})\nexport class App {}\n",
        |_| {},
    );
    let restarted = state_with_config(&root, |config| {
        config
            .meta_layers
            .entry("angular".to_string())
            .or_default()
            .enabled = false;
    });
    let rejected = restore(&restarted, &root, &file);
    assert_clean_rejection(
        &restarted,
        &file,
        &rejected,
        "semantic_configuration_incompatible",
    );
}

#[test]
fn restore_rejects_semantic_config_disabled_to_enabled() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let file = save_baseline(
        &root,
        "semantic-on.ts",
        "import { Component } from '@angular/core';\n@Component({selector:'x'})\nexport class App {}\n",
        |config| {
            config
                .meta_layers
                .entry("angular".to_string())
                .or_default()
                .enabled = false;
        },
    );
    let restarted = state_with_config(&root, |_| {});
    let rejected = restore(&restarted, &root, &file);
    assert_clean_rejection(
        &restarted,
        &file,
        &rejected,
        "semantic_configuration_incompatible",
    );
}

#[test]
fn restore_rejects_canonical_producer_generation_mismatch() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let file = save_baseline(&root, "canonical-generation.ts", "export class App {}\n", |_| {});
    let producer = state_with_config(&root, |_| {});
    rewrite_identity(
        &producer,
        &file,
        1,
        "contexts",
        "canonical_producer_identity",
        |json| json.replace("\"shared_canonical_pipeline\":1", "\"shared_canonical_pipeline\":99"),
    );
    drop(producer);
    let restarted = state_with_config(&root, |_| {});
    let rejected = restore(&restarted, &root, &file);
    assert_clean_rejection(&restarted, &file, &rejected, "canonical_producer_incompatible");
}

#[test]
fn restore_rejects_semantic_producer_generation_mismatch() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let file = save_baseline(&root, "semantic-generation.ts", "export class App {}\n", |_| {});
    let producer = state_with_config(&root, |_| {});
    rewrite_identity(
        &producer,
        &file,
        3,
        "semantic_edge_snapshots",
        "semantic_producer_identity",
        |json| json.replace("\"builtin_semantic\":1", "\"builtin_semantic\":99"),
    );
    drop(producer);
    let restarted = state_with_config(&root, |_| {});
    let rejected = restore(&restarted, &root, &file);
    assert_clean_rejection(&restarted, &file, &rejected, "semantic_producer_incompatible");
}

#[test]
fn restore_rejects_persisted_producer_when_current_set_lacks_it() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let file = save_baseline(&root, "producer-extra.ts", "export class App {}\n", |_| {});
    let producer = state_with_config(&root, |_| {});
    rewrite_identity(
        &producer,
        &file,
        1,
        "contexts",
        "canonical_producer_identity",
        |json| json.replace(
            "\"shared_canonical_pipeline\":1,",
            "\"angular_markers\":1,\"shared_canonical_pipeline\":1,",
        ),
    );
    drop(producer);
    let restarted = state_with_config(&root, |_| {});
    let rejected = restore(&restarted, &root, &file);
    assert_clean_rejection(&restarted, &file, &rejected, "canonical_producer_incompatible");
}

#[test]
fn restore_rejects_current_producer_when_persisted_set_lacks_it() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let file = save_baseline(&root, "producer-missing.ts", "export class App {}\n", |_| {});
    let producer = state_with_config(&root, |_| {});
    rewrite_identity(
        &producer,
        &file,
        1,
        "contexts",
        "canonical_producer_identity",
        |json| json.replace(",\"type_script_canonical\":1", ""),
    );
    drop(producer);
    let restarted = state_with_config(&root, |_| {});
    let rejected = restore(&restarted, &root, &file);
    assert_clean_rejection(&restarted, &file, &rejected, "canonical_producer_incompatible");
}

#[test]
fn restore_rejects_legacy_missing_identity() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let file = save_baseline(&root, "legacy.ts", "export class App {}\n", |_| {});
    let producer = state_with_config(&root, |_| {});
    {
        let store = producer.persistence_store_lock();
        let sqlite = store.as_ref().unwrap().sqlite().unwrap();
        sqlite
            .execute_batch("UPDATE contexts SET canonical_config_identity = NULL")
            .unwrap();
    }
    drop(producer);
    let restarted = state_with_config(&root, |_| {});
    let rejected = restore(&restarted, &root, &file);
    assert_clean_rejection(&restarted, &file, &rejected, "missing_legacy_identity");
}

#[test]
fn compatible_restart_reuses_and_publishes_complete_state() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let file = save_baseline(&root, "compatible.ts", "export class Compatible {}\n", |_| {});
    let restarted = state_with_config(&root, |_| {});
    let restored = restore(&restarted, &root, &file);
    assert!(restored.get("error").is_none(), "{restored}");
    let alias = restarted.alias_for_path(&file).expect("restored alias");
    assert!(restarted.ir_context_read().has_file(&alias));
    assert!(restarted.context_fidelity(&alias).is_some());
    assert_eq!(restarted.persisted_path(&alias).as_deref(), Some(file.as_str()));
    assert!(restarted.semantic_edges(&alias).is_some());
    assert!(restarted.llm_text_cache_lock().contains_key(&alias));
}
