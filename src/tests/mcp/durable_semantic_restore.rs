//! Production-path contracts for durable semantic-edge restoration.

use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

fn state(root: &tempfile::TempDir) -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config.persistence.enabled = true;
    config.persistence.db_path = root
        .path()
        .join("durable-semantics.db")
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

fn args(path: &str, root: &tempfile::TempDir) -> Value {
    json!({
        "filePath": path,
        "workspaceRoot": root.path(),
        "fidelity": "low"
    })
}

fn edge_json(state: &crate::mcp::McpState, alias: &str) -> Value {
    serde_json::to_value(
        state
            .semantic_edges(alias)
            .expect("authoritative semantic edges"),
    )
    .expect("serializable semantic edges")
}

fn indexed_component_edges(state: &crate::mcp::McpState, component: &str) -> Value {
    let index = state.workspace_index_read();
    serde_json::to_value(index.forward_edges_by_identity("angular", "Component", component))
        .expect("serializable indexed component edges")
}

#[test]
fn generated_delta_persists_and_restores_complete_target_edges() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("temp workspace");
    let path = root.path().join("component.ts");
    let file = path.to_string_lossy().into_owned();
    let baseline = "import { Component } from '@angular/core';\n@Component({ selector: 'app-old', template: '' })\nexport class AppComponent { run(): void { helper(); } }\n";
    let target = baseline.replace("app-old", "app-new");
    std::fs::write(&path, baseline).expect("baseline source");
    let producer = state(&root);

    let first = dispatch(&producer, 1, "delta_code_context", args(&file, &root));
    assert!(first.get("error").is_none(), "{first}");
    let alias = producer.alias_for_path(&file).expect("session alias");
    assert!(
        edge_json(&producer, &alias)
            .as_array()
            .is_some_and(|edges| !edges.is_empty()),
        "framework/meta edges must exist in the production fixture"
    );

    std::fs::write(&path, target).expect("target source");
    producer.invalidate_source_cache(&file);
    let generated = dispatch(&producer, 2, "delta_code_context", args(&file, &root));
    let delta = generated["result"]["delta"].clone();
    let from = generated["result"]["from_version"].clone();
    assert_eq!(delta["dv"], 2, "{generated}");
    assert!(delta["target_hash"].as_str().is_some(), "{generated}");
    let baseline_ir = producer.ir_context_read().get_ir(&alias).cloned();
    let baseline_edges = edge_json(&producer, &alias);
    let baseline_index = indexed_component_edges(&producer, "AppComponent");
    assert!(baseline_index.to_string().contains("app-old"));
    assert_eq!(producer.pending_transition_count(&alias), 1);

    let repeated = dispatch(&producer, 25, "delta_code_context", args(&file, &root));
    assert_eq!(repeated["result"]["delta"], delta, "{repeated}");
    assert_eq!(repeated["result"]["from_version"], from, "{repeated}");
    assert_eq!(producer.pending_transition_count(&alias), 1);
    assert_eq!(
        producer.ir_context_read().get_ir(&alias).cloned(),
        baseline_ir
    );
    assert_eq!(edge_json(&producer, &alias), baseline_edges);
    assert_eq!(
        indexed_component_edges(&producer, "AppComponent"),
        baseline_index
    );

    for (id, mismatched) in [
        (20, {
            let mut value = delta.clone();
            value["file"] = json!("wrong-file");
            value
        }),
        (21, {
            let mut value = delta.clone();
            value["from"] = json!(99);
            value
        }),
        (22, {
            let mut value = delta.clone();
            value["to"] = json!(99);
            value
        }),
        (23, {
            let mut value = delta.clone();
            value["target_hash"] = json!("wrong-target-hash");
            value
        }),
        (24, {
            let mut value = delta.clone();
            value["edits"][0]["at"] = json!(99);
            value
        }),
    ] {
        let rejected = dispatch(
            &producer,
            id,
            "apply_delta",
            json!({ "delta": mismatched, "currentVersion": from.clone() }),
        );
        assert!(rejected.get("error").is_some(), "{rejected}");
    }
    assert_eq!(
        producer.ir_context_read().get_ir(&alias).cloned(),
        baseline_ir
    );
    assert_eq!(edge_json(&producer, &alias), baseline_edges);
    assert_eq!(
        indexed_component_edges(&producer, "AppComponent"),
        baseline_index
    );
    assert_eq!(producer.pending_transition_count(&alias), 1);

    let restarted_without_pending = state(&root);
    let restored_baseline = dispatch(
        &restarted_without_pending,
        3,
        "restore_context",
        json!({ "filePath": file.clone(), "workspaceRoot": root.path() }),
    );
    assert!(
        restored_baseline.get("error").is_none(),
        "{restored_baseline}"
    );
    let standalone = dispatch(
        &restarted_without_pending,
        4,
        "apply_delta",
        json!({ "delta": delta.clone(), "currentVersion": from.clone() }),
    );
    assert!(
        standalone["error"]["message"]
            .as_str()
            .is_some_and(|message| message.contains("authoritative semantic-edge snapshot")),
        "{standalone}"
    );

    let applied = dispatch(
        &producer,
        5,
        "apply_delta",
        json!({ "delta": delta.clone(), "currentVersion": from }),
    );
    assert!(applied.get("error").is_none(), "{applied}");
    let expected_edges = edge_json(&producer, &alias);
    let applied_index = indexed_component_edges(&producer, "AppComponent");
    assert_ne!(expected_edges, baseline_edges);
    assert_ne!(applied_index, baseline_index);
    assert!(applied_index.to_string().contains("app-new"));
    assert!(!applied_index.to_string().contains("app-old"));
    assert_eq!(producer.pending_transition_count(&alias), 0);
    assert!(
        producer
            .pending_transition(
                &alias,
                &file,
                &serde_json::from_value(delta.clone()).unwrap()
            )
            .is_err()
    );

    let restarted = state(&root);
    let restored = dispatch(
        &restarted,
        6,
        "restore_context",
        json!({ "filePath": file.clone(), "workspaceRoot": root.path() }),
    );
    assert!(restored.get("error").is_none(), "{restored}");
    let restored_alias = restarted.alias_for_path(&file).expect("restored alias");
    assert_eq!(edge_json(&restarted, &restored_alias), expected_edges);
    assert_eq!(
        restarted.workspace_index_read().edge_count(),
        producer.workspace_index_read().edge_count(),
        "WorkspaceIndex must hydrate the exact restored edge set"
    );

    let replayed = dispatch(
        &producer,
        7,
        "apply_delta",
        json!({ "delta": delta, "currentVersion": 1 }),
    );
    assert!(replayed.get("error").is_some(), "{replayed}");
}

#[test]
fn failed_durable_commit_retains_pending_authority_and_live_state() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("temp workspace");
    let path = root.path().join("failure.ts");
    let file = path.to_string_lossy().into_owned();
    std::fs::write(&path, "export class Before { run(): void {} }\n").unwrap();
    let state = state(&root);
    assert!(
        dispatch(&state, 30, "delta_code_context", args(&file, &root))
            .get("error")
            .is_none()
    );
    let alias = state.alias_for_path(&file).expect("alias");
    let before = state.ir_context_read().get_ir(&alias).cloned();
    std::fs::write(&path, "export class After { run(): void {} }\n").unwrap();
    state.invalidate_source_cache(&file);
    let generated = dispatch(&state, 31, "delta_code_context", args(&file, &root));
    let delta_value = generated["result"]["delta"].clone();
    let delta: crate::ir::delta::SequenceDelta =
        serde_json::from_value(delta_value.clone()).expect("sequence delta");

    {
        let store = state.persistence_store_lock();
        let sqlite = store.as_ref().unwrap().sqlite().unwrap();
        sqlite
            .execute_batch("DROP TABLE semantic_edge_snapshots")
            .expect("force durable failure");
    }
    let rejected = dispatch(
        &state,
        32,
        "apply_delta",
        json!({ "delta": delta_value, "currentVersion": delta.from }),
    );
    assert!(rejected.get("error").is_some(), "{rejected}");
    assert_eq!(state.ir_context_read().get_ir(&alias).cloned(), before);
    assert!(state.pending_transition(&alias, &file, &delta).is_ok());
}

#[test]
fn malformed_or_mismatched_edge_snapshot_fails_without_session_mutation() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("temp workspace");
    let path = root.path().join("restore.ts");
    let file = path.to_string_lossy().into_owned();
    std::fs::write(
        &path,
        "import { Component } from '@angular/core';\n@Component({ selector: 'stable', template: '' })\nexport class StableComponent {}\n",
    )
    .expect("source");
    let state = state(&root);
    let compressed = dispatch(&state, 10, "compress_code_context", args(&file, &root));
    assert!(compressed.get("error").is_none(), "{compressed}");
    let alias = state.alias_for_path(&file).expect("alias");
    let prior_ir = state.ir_context_read().get_ir(&alias).cloned();
    let prior_edges = edge_json(&state, &alias);

    {
        let store = state.persistence_store_lock();
        let sqlite = store.as_ref().unwrap().sqlite().unwrap();
        sqlite
            .execute_batch("UPDATE semantic_edge_snapshots SET source_hash = 'wrong'")
            .expect("corrupt snapshot metadata");
    }
    let rejected = dispatch(
        &state,
        11,
        "restore_context",
        json!({ "filePath": file, "workspaceRoot": root.path() }),
    );
    assert!(rejected.get("error").is_some(), "{rejected}");
    assert_eq!(state.ir_context_read().get_ir(&alias).cloned(), prior_ir);
    assert_eq!(edge_json(&state, &alias), prior_edges);

    {
        let store = state.persistence_store_lock();
        let sqlite = store.as_ref().unwrap().sqlite().unwrap();
        sqlite
            .execute_batch(
                "UPDATE semantic_edge_snapshots
                 SET source_hash = (SELECT source_hash FROM contexts LIMIT 1),
                     edges_json = '{'",
            )
            .expect("malform snapshot");
    }
    let malformed = dispatch(
        &state,
        12,
        "restore_context",
        json!({ "filePath": file, "workspaceRoot": root.path() }),
    );
    assert!(malformed.get("error").is_some(), "{malformed}");
    assert_eq!(state.ir_context_read().get_ir(&alias).cloned(), prior_ir);
    assert_eq!(edge_json(&state, &alias), prior_edges);

    {
        let store = state.persistence_store_lock();
        let sqlite = store.as_ref().unwrap().sqlite().unwrap();
        sqlite
            .execute_batch("DELETE FROM semantic_edge_snapshots")
            .expect("remove snapshot");
    }
    let missing = dispatch(
        &state,
        13,
        "restore_context",
        json!({ "filePath": file, "workspaceRoot": root.path() }),
    );
    assert!(missing.get("error").is_some(), "{missing}");
    assert_eq!(state.ir_context_read().get_ir(&alias).cloned(), prior_ir);
    assert_eq!(edge_json(&state, &alias), prior_edges);
}
