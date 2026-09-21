use crate::ir::delta::SequenceDelta;
use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

fn state(root: &tempfile::TempDir) -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config.persistence.enabled = true;
    config.persistence.db_path = root
        .path()
        .join("delta-edit-recovery.db")
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
    format!("export class Delta {{\n  run() {{ return \"{value}\"; }}\n}}\n").into_bytes()
}

fn delta_request(root: &tempfile::TempDir, file: &str) -> Value {
    json!({
        "filePath": file,
        "workspaceRoot": root.path().to_string_lossy(),
        "fidelity": "edit"
    })
}

fn baseline(state: &crate::mcp::McpState, root: &tempfile::TempDir, file: &str) {
    let response = dispatch(state, 1, "delta_code_context", delta_request(root, file));
    assert!(response.get("error").is_none(), "{response}");
    assert!(response.pointer("/result/delta").is_none(), "{response}");
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
fn first_delta_after_restart_recovers_target_before_baseline_work() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("first.ts").to_string_lossy().into_owned();
    let prior = source("prior");
    let target = source("target");
    std::fs::write(&file, &prior).unwrap();
    let initial = state(&root);
    baseline(&initial, &root, &file);
    let target_version = establish_intent(&initial, &file, prior, target.clone(), "first-target");
    std::fs::write(&file, &target).unwrap();
    drop(initial);

    let restarted = state(&root);
    let recovered = dispatch(
        &restarted,
        10,
        "delta_code_context",
        delta_request(&root, &file),
    );
    assert!(recovered.get("error").is_none(), "{recovered}");
    assert_eq!(recovered["result"]["version"], target_version);
    assert_eq!(recovered["result"]["cached"], true);
    assert!(!has_intent(&restarted, &file));

    std::fs::write(&file, source("next")).unwrap();
    restarted.invalidate_source_cache(&file);
    let generated = dispatch(
        &restarted,
        11,
        "delta_code_context",
        delta_request(&root, &file),
    );
    assert_eq!(generated["result"]["delta"]["dv"], 2, "{generated}");
}

#[test]
fn existing_delta_baseline_recovers_prior_intent_then_generates_normally() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let file = root
        .path()
        .join("existing.ts")
        .to_string_lossy()
        .into_owned();
    let prior = source("prior");
    std::fs::write(&file, &prior).unwrap();
    let state = state(&root);
    baseline(&state, &root, &file);
    establish_intent(
        &state,
        &file,
        prior,
        source("uncommitted"),
        "existing-prior",
    );

    let recovered = dispatch(
        &state,
        20,
        "delta_code_context",
        delta_request(&root, &file),
    );
    assert!(recovered.get("error").is_none(), "{recovered}");
    assert!(!has_intent(&state, &file));
    std::fs::write(&file, source("changed")).unwrap();
    state.invalidate_source_cache(&file);
    let generated = dispatch(
        &state,
        21,
        "delta_code_context",
        delta_request(&root, &file),
    );
    assert_eq!(generated["result"]["delta"]["dv"], 2, "{generated}");
}

#[test]
fn apply_delta_recovers_target_before_rejecting_duplicate_transition() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("apply.ts").to_string_lossy().into_owned();
    let prior = source("prior");
    let changed = source("changed");
    std::fs::write(&file, &prior).unwrap();
    let state = state(&root);
    baseline(&state, &root, &file);
    std::fs::write(&file, &changed).unwrap();
    state.invalidate_source_cache(&file);
    let generated = dispatch(
        &state,
        30,
        "delta_code_context",
        delta_request(&root, &file),
    );
    let delta = generated["result"]["delta"].clone();
    let parsed: SequenceDelta = serde_json::from_value(delta.clone()).unwrap();
    establish_intent(&state, &file, prior, changed, "apply-target");

    let applied = dispatch(
        &state,
        31,
        "apply_delta",
        json!({ "delta": delta, "currentVersion": parsed.from }),
    );
    assert!(applied.get("error").is_some(), "{applied}");
    assert!(!has_intent(&state, &file));
    assert_eq!(state.file_version(&parsed.file), Some(parsed.to));
}

#[test]
fn apply_delta_recovery_failure_preserves_pending_and_live_authority() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let file = root
        .path()
        .join("apply-failure.ts")
        .to_string_lossy()
        .into_owned();
    let prior = source("prior");
    let changed = source("changed");
    std::fs::write(&file, &prior).unwrap();
    let state = state(&root);
    baseline(&state, &root, &file);
    let alias = state.alias_for_path(&file).unwrap();
    let live_ir = state.ir_context_read().get_ir(&alias).cloned().unwrap();
    std::fs::write(&file, &changed).unwrap();
    state.invalidate_source_cache(&file);
    let generated = dispatch(
        &state,
        35,
        "delta_code_context",
        delta_request(&root, &file),
    );
    let delta = generated["result"]["delta"].clone();
    let parsed: SequenceDelta = serde_json::from_value(delta.clone()).unwrap();
    establish_intent(&state, &file, prior, changed, "apply-failed-target");
    crate::mcp::sqlite_store::fail_next_semantic_save(&file);

    let failed = dispatch(
        &state,
        36,
        "apply_delta",
        json!({ "delta": delta, "currentVersion": parsed.from }),
    );
    assert!(
        failed["error"]["message"]
            .as_str()
            .is_some_and(|message| message.contains("Edit recovery failed")),
        "{failed}"
    );
    assert_eq!(state.ir_context_read().get_ir(&alias), Some(&live_ir));
    assert!(
        state
            .pending_transition(&parsed.file, &file, &parsed)
            .is_ok()
    );
    assert!(has_intent(&state, &file));
}

#[test]
fn recovery_failure_precedes_delta_compile_or_application_and_preserves_state() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let file = root
        .path()
        .join("failure.ts")
        .to_string_lossy()
        .into_owned();
    let prior = source("prior");
    let target = source("compile-must-not-run");
    std::fs::write(&file, &prior).unwrap();
    let state = state(&root);
    baseline(&state, &root, &file);
    establish_intent(&state, &file, prior, target.clone(), "failed-target");
    std::fs::write(&file, &target).unwrap();
    let alias = state.alias_for_path(&file).unwrap();
    let live_ir = state.ir_context_read().get_ir(&alias).cloned().unwrap();
    let live_hash = state.ir_context_read().get_source_hash(&alias).cloned();
    let index_count = state.workspace_index_read().edge_count();
    let pending_count = state.pending_transition_count(&alias);
    crate::mcp::sqlite_store::fail_next_semantic_save(&file);
    *crate::mcp::tool_helpers::TEST_INJECTED_SOURCE_FAILURE
        .lock()
        .unwrap() = Some("compile-must-not-run".to_string());

    let failed = dispatch(
        &state,
        40,
        "delta_code_context",
        delta_request(&root, &file),
    );
    *crate::mcp::tool_helpers::TEST_INJECTED_SOURCE_FAILURE
        .lock()
        .unwrap() = None;
    assert!(
        failed["error"]["message"]
            .as_str()
            .is_some_and(|message| message.contains("Edit recovery failed")),
        "{failed}"
    );
    assert_eq!(state.ir_context_read().get_ir(&alias), Some(&live_ir));
    assert_eq!(
        state.ir_context_read().get_source_hash(&alias),
        live_hash.as_ref()
    );
    assert_eq!(state.workspace_index_read().edge_count(), index_count);
    assert_eq!(state.pending_transition_count(&alias), pending_count);
    assert!(has_intent(&state, &file));
}
