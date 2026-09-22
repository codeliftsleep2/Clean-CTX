//! Registered-dispatch contracts for file-scoped semantic-context deletion.

use crate::mcp::context_store::ContextStore;
use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

fn state(root: &tempfile::TempDir) -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config.persistence.enabled = true;
    config.persistence.db_path = root
        .path()
        .join("delete-context.db")
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

fn compile(state: &crate::mcp::McpState, id: i64, file: &str, root: &str) -> Value {
    dispatch(
        state,
        id,
        "compress_code_context",
        json!({ "filePath": file, "workspaceRoot": root, "fidelity": "low" }),
    )
}

fn establish_deletion_intent(
    state: &crate::mcp::McpState,
    file: &str,
    stage_path: &str,
    transition: &str,
) {
    let source = std::fs::read(file).expect("current source");
    let alias = state.alias_for_path(file).expect("alias");
    let prior_hash = state
        .ir_context_read()
        .get_source_hash(&alias)
        .cloned()
        .expect("source hash");
    let prior_version = state.file_version(&alias).expect("version");
    let (mut target_ir, target_edges, target_hash) =
        crate::mcp::tool_helpers::compile_source_ir_candidate(
            file,
            std::str::from_utf8(&source).expect("UTF-8 fixture"),
            crate::compression::Fidelity::Edit,
            state,
        )
        .expect("target compilation");
    target_ir.file_id = file.to_string();
    let intent = crate::mcp::sqlite_store::EditIntent {
        transition_id: transition.to_string(),
        file_path: file.to_string(),
        prior_hash,
        target_hash,
        prior_version,
        target_version: target_ir.version,
        prior_source: source.clone(),
        target_source: source,
        target_ir: crate::ir::binary_wire::encode(&target_ir),
        target_edges,
        fidelity: crate::compression::Fidelity::Edit,
        stage_path: stage_path.to_string(),
    };
    state
        .persistence_store_lock()
        .as_ref()
        .unwrap()
        .sqlite()
        .unwrap()
        .establish_edit_intent(&intent)
        .expect("edit intent");
}

#[test]
fn registered_delete_removes_exact_owned_context_without_touching_source_or_peer() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("temp workspace");
    let workspace = root.path().to_string_lossy().into_owned();
    let first = root.path().join("first.ts").to_string_lossy().into_owned();
    let second = root.path().join("second.ts").to_string_lossy().into_owned();
    let first_source = "export class First { run(): void {} }\n";
    std::fs::write(&first, first_source).expect("first source");
    std::fs::write(&second, "export class Second { stay(): void {} }\n").expect("second source");
    let state = state(&root);
    assert!(
        compile(&state, 1, &first, &workspace)
            .get("error")
            .is_none()
    );
    assert!(
        compile(&state, 2, &second, &workspace)
            .get("error")
            .is_none()
    );

    let first_alias = state.alias_for_path(&first).expect("first alias");
    let second_alias = state.alias_for_path(&second).expect("second alias");
    let second_ir = state
        .ir_context_read()
        .get_ir(&second_alias)
        .cloned()
        .expect("peer IR");
    std::fs::write(
        &first,
        "export class First { run(): void {} stop(): void {} }\n",
    )
    .expect("target source");
    state.invalidate_source_cache(&first);
    let generated = dispatch(
        &state,
        3,
        "delta_code_context",
        json!({ "filePath": &first, "workspaceRoot": &workspace, "fidelity": "low" }),
    );
    let delta: crate::ir::delta::SequenceDelta =
        serde_json::from_value(generated["result"]["delta"].clone()).expect("dv:2 delta");
    let applied = dispatch(
        &state,
        4,
        "apply_delta",
        json!({ "delta": generated["result"]["delta"], "currentVersion": delta.from }),
    );
    assert!(applied.get("error").is_none(), "{applied}");
    {
        let store = state.persistence_store_lock();
        let sqlite = store.as_ref().unwrap().sqlite().unwrap();
        let context_id = sqlite
            .current_context_id(&first)
            .expect("context lookup")
            .expect("durable context");
        assert!(
            sqlite.delta_count(&context_id) > 0,
            "persisted dv:2 history"
        );
        assert!(
            sqlite
                .load_durable_context(&first, None)
                .expect("durable state")
                .is_some(),
            "canonical baseline and semantic snapshot"
        );
    }
    std::fs::write(
        &first,
        "export class First { run(): void {} stop(): void {} pause(): void {} }\n",
    )
    .expect("second target source");
    state.invalidate_source_cache(&first);
    let pending = dispatch(
        &state,
        5,
        "delta_code_context",
        json!({ "filePath": &first, "workspaceRoot": &workspace, "fidelity": "low" }),
    );
    let pending_delta: crate::ir::delta::SequenceDelta =
        serde_json::from_value(pending["result"]["delta"].clone()).expect("pending dv:2 delta");
    assert!(
        state
            .pending_transition(&first_alias, &first, &pending_delta)
            .is_ok()
    );
    let source_before_delete = std::fs::read(&first).expect("source bytes");
    let stage = root
        .path()
        .join("first.stage")
        .to_string_lossy()
        .into_owned();
    std::fs::write(&stage, b"recovery artifact").unwrap();
    establish_deletion_intent(&state, &first, &stage, "delete-first");

    let deleted = dispatch(&state, 6, "delete_context", json!({ "filePath": &first }));
    assert_eq!(deleted["result"]["_meta"]["deleted"], 1, "{deleted}");
    assert_eq!(deleted["result"]["_meta"]["source_deleted"], false);
    assert_eq!(
        std::fs::read(&first).expect("source remains"),
        source_before_delete
    );
    assert!(!std::path::Path::new(&stage).exists());
    assert!(!state.ir_context_read().has_file(&first_alias));
    assert!(
        state
            .ir_context_read()
            .get_source_hash(&first_alias)
            .is_none()
    );
    assert!(state.alias_for_path(&first).is_none());
    assert!(state.persisted_path(&first_alias).is_none());
    assert!(state.context_fidelity(&first_alias).is_none());
    assert!(state.semantic_edges(&first_alias).is_none());
    assert!(
        state
            .pending_transition(&first_alias, &first, &pending_delta)
            .is_err()
    );
    assert!(!state.llm_text_cache_lock().contains_key(&first_alias));
    assert!(
        state
            .workspace_index_read()
            .entities_in_file(&crate::dictionary::path::canonical_identity_key(&first))
            .is_empty()
    );
    assert!(state.ir_context_read().has_file(&second_alias));
    assert_eq!(
        state.ir_context_read().get_ir(&second_alias),
        Some(&second_ir)
    );
    assert!(state.alias_for_path(&second).is_some());
    assert!(
        !state
            .workspace_index_read()
            .entities_in_file(&crate::dictionary::path::canonical_identity_key(&second))
            .is_empty()
    );

    {
        let store = state.persistence_store_lock();
        let sqlite = store.as_ref().unwrap().sqlite().unwrap();
        assert!(!sqlite.has_context(&first));
        assert!(!sqlite.has_edit_intent(&first).unwrap());
        assert!(sqlite.has_context(&second));
        assert!(
            sqlite
                .load_durable_context(&first, None)
                .expect("durable lookup")
                .is_none()
        );
    }
    let restore = dispatch(
        &state,
        7,
        "restore_context",
        json!({ "filePath": &first, "workspaceRoot": &workspace }),
    );
    assert!(restore.get("error").is_some(), "{restore}");
}

#[test]
fn rejected_or_failed_delete_preserves_all_live_ownership() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("temp workspace");
    let workspace = root.path().to_string_lossy().into_owned();
    let file = root.path().join("owned.ts").to_string_lossy().into_owned();
    let wrong = root.path().join("wrong.ts").to_string_lossy().into_owned();
    std::fs::write(&file, "export class Owned { run(): void {} }\n").expect("source");
    let state = state(&root);
    assert!(
        compile(&state, 10, &file, &workspace)
            .get("error")
            .is_none()
    );
    let alias = state.alias_for_path(&file).expect("alias");
    let prior_ir = state.ir_context_read().get_ir(&alias).cloned();
    let prior_edges = serde_json::to_value(state.semantic_edges(&alias)).expect("edge snapshot");
    let prior_index_edges = state.workspace_index_read().edge_count();
    let prior_fidelity = state.context_fidelity(&alias);
    let prior_compact = state.llm_text_cache_lock().get(&alias).cloned();

    state.remember_persisted_path(&alias, &wrong);
    let mismatch = dispatch(&state, 11, "delete_context", json!({ "filePath": &file }));
    assert!(mismatch.get("error").is_some(), "{mismatch}");
    assert_eq!(state.ir_context_read().get_ir(&alias).cloned(), prior_ir);
    state.remember_persisted_path(&alias, &file);

    std::fs::write(
        &file,
        "export class Owned { run(): void {} pending(): void {} }\n",
    )
    .expect("pending target source");
    state.invalidate_source_cache(&file);
    let generated = dispatch(
        &state,
        12,
        "delta_code_context",
        json!({ "filePath": &file, "workspaceRoot": &workspace, "fidelity": "low" }),
    );
    let pending_delta: crate::ir::delta::SequenceDelta =
        serde_json::from_value(generated["result"]["delta"].clone()).expect("pending delta");
    assert!(
        state
            .pending_transition(&alias, &file, &pending_delta)
            .is_ok()
    );
    let source_before_failure = std::fs::read(&file).expect("source bytes");
    let stage = root
        .path()
        .join("owned.stage")
        .to_string_lossy()
        .into_owned();
    std::fs::write(&stage, b"retained recovery artifact").unwrap();
    establish_deletion_intent(&state, &file, &stage, "retain-on-failure");

    {
        let store = state.persistence_store_lock();
        let sqlite = store.as_ref().unwrap().sqlite().unwrap();
        sqlite
            .execute_batch("DROP TABLE semantic_edge_snapshots")
            .expect("force transactional deletion failure");
    }
    let failed = dispatch(&state, 13, "delete_context", json!({ "filePath": &file }));
    assert!(failed.get("error").is_some(), "{failed}");
    assert_eq!(state.ir_context_read().get_ir(&alias).cloned(), prior_ir);
    assert_eq!(
        serde_json::to_value(state.semantic_edges(&alias)).expect("edge snapshot"),
        prior_edges
    );
    assert_eq!(state.workspace_index_read().edge_count(), prior_index_edges);
    assert_eq!(state.persisted_path(&alias).as_deref(), Some(file.as_str()));
    assert_eq!(state.alias_for_path(&file).as_deref(), Some(alias.as_str()));
    assert_eq!(state.context_fidelity(&alias), prior_fidelity);
    assert_eq!(
        state.llm_text_cache_lock().get(&alias),
        prior_compact.as_ref()
    );
    assert!(
        state
            .pending_transition(&alias, &file, &pending_delta)
            .is_ok()
    );
    assert_eq!(
        std::fs::read(&file).expect("source remains"),
        source_before_failure
    );
    assert!(std::path::Path::new(&stage).exists());
    assert!(
        state
            .persistence_store_lock()
            .as_ref()
            .unwrap()
            .sqlite()
            .unwrap()
            .has_edit_intent(&file)
            .unwrap()
    );

    let missing = dispatch(
        &state,
        14,
        "delete_context",
        json!({ "filePath": root.path().join("missing.ts") }),
    );
    assert!(missing.get("error").is_some(), "{missing}");
}
