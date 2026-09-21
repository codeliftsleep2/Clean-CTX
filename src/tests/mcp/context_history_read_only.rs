//! Registered P9-21 read-only context-history contract.

use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

fn state(root: &tempfile::TempDir) -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config.persistence.enabled = true;
    config.persistence.db_path = root
        .path()
        .join("context-history.db")
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

fn compile(state: &crate::mcp::McpState, root: &tempfile::TempDir, file: &str) {
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

fn history(state: &crate::mcp::McpState, id: i64, file: &str) -> Value {
    dispatch(state, id, "context_history", json!({ "filePath": file }))
}

#[test]
fn registered_description_declares_read_only_ownership() {
    let tool = crate::mcp::tools::tool_list()
        .into_iter()
        .find(|tool| tool["name"] == "context_history")
        .expect("registered context_history tool");
    let description = tool["description"].as_str().unwrap();
    assert!(description.contains("without creating"));
    assert!(description.contains("mutating context ownership"));
}

#[test]
fn tracked_and_durable_only_history_are_read_without_publication() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let file = root
        .path()
        .join("tracked.ts")
        .to_string_lossy()
        .into_owned();
    std::fs::write(&file, "export class Tracked { run() {} }\n").unwrap();
    let initial = state(&root);
    compile(&initial, &root, &file);
    let alias = initial.alias_for_path(&file).expect("tracked alias");
    let live_version = initial.file_version(&alias).expect("live version");

    let tracked = history(&initial, 2, &file);
    let tracked_text = tracked["result"]["content"][0]["text"].as_str().unwrap();
    assert!(tracked_text.contains("IR Baseline: yes"), "{tracked}");
    assert!(
        tracked_text.contains(&format!("IR Version: {live_version}")),
        "{tracked}"
    );
    assert!(tracked_text.contains("Context Store: yes"), "{tracked}");
    drop(initial);

    let restarted = state(&root);
    assert!(restarted.alias_for_path(&file).is_none());
    let durable_only = history(&restarted, 3, &file);
    let durable_text = durable_only["result"]["content"][0]["text"]
        .as_str()
        .unwrap();
    assert!(durable_text.contains("IR Baseline: no"), "{durable_only}");
    assert!(
        durable_text.contains("Context Store: yes"),
        "{durable_only}"
    );
    assert!(restarted.alias_for_path(&file).is_none());
    assert_eq!(restarted.workspace_index_read().edge_count(), 0);
}

#[test]
fn unknown_repeated_reads_do_not_change_alias_order_or_session_owners() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let unknown = root
        .path()
        .join("unknown.ts")
        .to_string_lossy()
        .into_owned();
    let owned = root.path().join("owned.ts").to_string_lossy().into_owned();
    std::fs::write(&owned, "export class Owned {}\n").unwrap();
    let state = state(&root);
    let before_edges = state.workspace_index_read().edge_count();

    for id in [10, 11] {
        let response = history(&state, id, &unknown);
        assert!(response.get("error").is_none(), "{response}");
        assert!(state.alias_for_path(&unknown).is_none());
    }
    assert_eq!(state.workspace_index_read().edge_count(), before_edges);
    assert!(state.semantic_edges("α1").is_none());
    assert!(state.context_fidelity("α1").is_none());
    assert!(!state.ir_context_read().has_file("α1"));

    compile(&state, &root, &owned);
    assert_eq!(state.alias_for_path(&owned).as_deref(), Some("α1"));
}

#[test]
fn history_does_not_resolve_a_pending_edit_intent() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let file = root
        .path()
        .join("pending.ts")
        .to_string_lossy()
        .into_owned();
    let prior = b"export class Pending { run() { return 1; } }\n";
    let target = b"export class Pending { run() { return 2; } }\n";
    std::fs::write(&file, prior).unwrap();
    let initial = state(&root);
    compile(&initial, &root, &file);
    let alias = initial.alias_for_path(&file).unwrap();
    let prior_hash = initial
        .ir_context_read()
        .get_source_hash(&alias)
        .cloned()
        .unwrap();
    let prior_version = initial.file_version(&alias).unwrap();
    let (mut target_ir, target_edges, target_hash) =
        crate::mcp::tool_helpers::compile_source_ir_candidate(
            &file,
            std::str::from_utf8(target).unwrap(),
            crate::compression::Fidelity::Edit,
            &initial,
        )
        .unwrap();
    target_ir.file_id.clone_from(&file);
    let intent = crate::mcp::sqlite_store::EditIntent {
        transition_id: "history-pending".to_string(),
        file_path: file.clone(),
        prior_hash,
        target_hash,
        prior_version,
        target_version: target_ir.version,
        prior_source: prior.to_vec(),
        target_source: target.to_vec(),
        target_ir: crate::ir::binary_wire::encode(&target_ir),
        target_edges,
        fidelity: crate::compression::Fidelity::Edit,
        stage_path: String::new(),
    };
    initial
        .persistence_store_lock()
        .as_ref()
        .unwrap()
        .sqlite()
        .unwrap()
        .establish_edit_intent(&intent)
        .unwrap();
    std::fs::write(&file, target).unwrap();
    drop(initial);

    let restarted = state(&root);
    let response = history(&restarted, 20, &file);
    assert!(response.get("error").is_none(), "{response}");
    assert!(restarted.alias_for_path(&file).is_none());
    assert!(
        restarted
            .persistence_store_lock()
            .as_ref()
            .unwrap()
            .sqlite()
            .unwrap()
            .has_edit_intent(&file)
            .unwrap()
    );
    assert_eq!(restarted.workspace_index_read().edge_count(), 0);
}
