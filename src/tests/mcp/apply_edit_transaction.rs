use crate::ir::opcodes::CoreOp;
use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

fn state(root: &tempfile::TempDir) -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config.persistence.enabled = true;
    config.persistence.db_path = root
        .path()
        .join("apply-edit-transaction.db")
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

fn baseline(state: &crate::mcp::McpState, file: &str, root: &tempfile::TempDir) -> Value {
    dispatch(
        state,
        1,
        "compress_code_context",
        json!({
            "filePath": file,
            "workspaceRoot": root.path().to_string_lossy(),
            "fidelity": "edit"
        }),
    )
}

fn replace_body(file: &str, old: &str, new: &str) -> Value {
    json!({
        "filePath": file,
        "operations": [{
            "type": "replace_body",
            "target": "Unsafe.run",
            "expectedOldText": old,
            "newText": new
        }]
    })
}

fn source(body: &str) -> Vec<u8> {
    format!("\u{feff}export class Unsafe {{\r\n  run() {body}\r\n}}\r\n").into_bytes()
}

fn durable(state: &crate::mcp::McpState, file: &str) -> crate::ir::compiler::CompiledIR {
    let guard = state.persistence_store_lock();
    guard
        .as_ref()
        .unwrap()
        .sqlite()
        .unwrap()
        .load_durable_context(file, None)
        .unwrap()
        .unwrap()
        .ir
}

#[test]
fn registered_edit_is_byte_exact_and_commits_source_durable_and_live_together() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("unsafe.ts").to_string_lossy().into_owned();
    let old_body = "{\r\n    const label = \"café\";\r\n    if (label) {\r\n      return label;\r\n    }\r\n    return \"\";\r\n  }";
    let new_body = "{\r\n    const label = \"café🙂\";\r\n    if (label) {\r\n      return label;\r\n    }\r\n    return \"exact\";\r\n  }";
    let prior = source(old_body);
    std::fs::write(&file, &prior).unwrap();
    let state = state(&root);

    let produced = baseline(&state, &file, &root);
    assert!(produced.get("error").is_none(), "{produced}");
    let compact = produced["result"]["content"][0]["text"].as_str().unwrap();
    assert!(
        compact.contains(old_body),
        "full Body fallback was lost: {compact}"
    );

    let old_start = prior
        .windows(old_body.len())
        .position(|window| window == old_body.as_bytes())
        .unwrap();
    let expected = [
        &prior[..old_start],
        new_body.as_bytes(),
        &prior[old_start + old_body.len()..],
    ]
    .concat();
    let edited = dispatch(
        &state,
        2,
        "apply_edit",
        replace_body(&file, old_body, new_body),
    );
    assert!(edited.get("error").is_none(), "{edited}");
    assert_eq!(std::fs::read(&file).unwrap(), expected);
    assert_eq!(&expected[..old_start], &prior[..old_start]);
    assert_eq!(
        &expected[old_start + new_body.len()..],
        &prior[old_start + old_body.len()..]
    );
    assert!(expected.starts_with(&[0xEF, 0xBB, 0xBF]));
    assert!(expected.windows(2).any(|bytes| bytes == b"\r\n"));

    let persisted = durable(&state, &file);
    let (body, start, end) = persisted
        .instructions
        .iter()
        .find_map(|operation| match operation {
            CoreOp::Body(_, body, Some(start), Some(end)) if body == new_body => {
                Some((body, *start as usize, *end as usize))
            }
            _ => None,
        })
        .expect("byte-exact persisted Body");
    assert_eq!(body.as_bytes(), &expected[start..end]);
    let alias = state.alias_for_path(&file).unwrap();
    let expected_hash = state.cache_read().compute_hash(&expected);
    assert_eq!(state.file_version(&alias), Some(persisted.version));
    assert_eq!(
        state.ir_context_read().get_source_hash(&alias),
        Some(&expected_hash)
    );

    state.ir_context_lock().remove_file(&alias);
    let restored = dispatch(&state, 3, "restore_context", json!({ "filePath": file }));
    assert!(restored.get("error").is_none(), "{restored}");
    assert!(
        restored["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains(new_body)
    );
}

#[test]
fn stale_external_source_is_rejected_before_intent_and_can_be_explicitly_reconciled() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("unsafe.ts").to_string_lossy().into_owned();
    let old_body = "{\n    return \"old\";\n  }";
    let external_body = "{\n    return \"external\";\n  }";
    let target_body = "{\n    return \"target\";\n  }";
    std::fs::write(&file, source(old_body)).unwrap();
    let state = state(&root);
    assert!(baseline(&state, &file, &root).get("error").is_none());
    let alias = state.alias_for_path(&file).unwrap();
    let prior_version = state.file_version(&alias);
    let prior_index = state.workspace_index_read().edge_count();
    let external = source(external_body);
    std::fs::write(&file, &external).unwrap();
    state.invalidate_source_cache(&file);

    let rejected = dispatch(
        &state,
        10,
        "apply_edit",
        replace_body(&file, external_body, target_body),
    );
    assert_eq!(rejected["error"]["data"]["code"], "stale_edit_source");
    assert!(rejected["error"]["data"]["expectedLiveHash"].is_string());
    assert!(rejected["error"]["data"]["expectedDurableHash"].is_string());
    assert!(rejected["error"]["data"]["actualSourceHash"].is_string());
    assert_eq!(std::fs::read(&file).unwrap(), external);
    assert_eq!(state.file_version(&alias), prior_version);
    assert_eq!(state.workspace_index_read().edge_count(), prior_index);
    assert!(
        !state
            .persistence_store_lock()
            .as_ref()
            .unwrap()
            .sqlite()
            .unwrap()
            .has_edit_intent(&file)
            .unwrap()
    );

    let reconciled = baseline(&state, &file, &root);
    assert!(reconciled.get("error").is_none(), "{reconciled}");
    let accepted = dispatch(
        &state,
        11,
        "apply_edit",
        replace_body(&file, external_body, target_body),
    );
    assert!(accepted.get("error").is_none(), "{accepted}");
}

#[test]
fn durable_edit_failure_restores_exact_prior_bytes_and_preserves_live_state() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("unsafe.ts").to_string_lossy().into_owned();
    let old_body = "{\r\n    return \"café\";\r\n  }";
    let target_body = "{\r\n    return \"changed\";\r\n  }";
    let prior = source(old_body);
    std::fs::write(&file, &prior).unwrap();
    let state = state(&root);
    assert!(baseline(&state, &file, &root).get("error").is_none());
    let alias = state.alias_for_path(&file).unwrap();
    let prior_ir = state.ir_context_read().get_ir(&alias).cloned().unwrap();
    let prior_version = state.file_version(&alias);
    let prior_edges = serde_json::to_value(state.semantic_edges(&alias)).unwrap();

    crate::mcp::sqlite_store::fail_next_semantic_save(&file);
    let failed = dispatch(
        &state,
        20,
        "apply_edit",
        replace_body(&file, old_body, target_body),
    );
    assert!(failed.get("error").is_some(), "{failed}");
    assert_eq!(std::fs::read(&file).unwrap(), prior);
    assert_eq!(state.ir_context_read().get_ir(&alias), Some(&prior_ir));
    assert_eq!(state.file_version(&alias), prior_version);
    assert_eq!(
        serde_json::to_value(state.semantic_edges(&alias)).unwrap(),
        prior_edges
    );
    assert!(
        !state
            .persistence_store_lock()
            .as_ref()
            .unwrap()
            .sqlite()
            .unwrap()
            .has_edit_intent(&file)
            .unwrap()
    );
}

#[test]
fn candidate_compilation_failure_precedes_source_intent_and_live_mutation() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("unsafe.ts").to_string_lossy().into_owned();
    let old_body = "{\n    return \"old\";\n  }";
    let failed_body = "{\n    return \"candidate-failure-marker\";\n  }";
    let prior = source(old_body);
    std::fs::write(&file, &prior).unwrap();
    let state = state(&root);
    assert!(baseline(&state, &file, &root).get("error").is_none());
    let alias = state.alias_for_path(&file).unwrap();
    let prior_ir = state.ir_context_read().get_ir(&alias).cloned().unwrap();
    let prior_edges = serde_json::to_value(state.semantic_edges(&alias)).unwrap();
    *crate::mcp::tool_helpers::TEST_INJECTED_SOURCE_FAILURE
        .lock()
        .unwrap() = Some("candidate-failure-marker".to_string());

    let failed = dispatch(
        &state,
        30,
        "apply_edit",
        replace_body(&file, old_body, failed_body),
    );
    *crate::mcp::tool_helpers::TEST_INJECTED_SOURCE_FAILURE
        .lock()
        .unwrap() = None;

    assert!(failed.get("error").is_some(), "{failed}");
    assert_eq!(std::fs::read(&file).unwrap(), prior);
    assert_eq!(state.ir_context_read().get_ir(&alias), Some(&prior_ir));
    assert_eq!(
        serde_json::to_value(state.semantic_edges(&alias)).unwrap(),
        prior_edges
    );
    assert!(
        !state
            .persistence_store_lock()
            .as_ref()
            .unwrap()
            .sqlite()
            .unwrap()
            .has_edit_intent(&file)
            .unwrap()
    );
}

#[test]
fn restore_recovers_exact_target_after_restart_between_source_and_durable_commits() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("unsafe.ts").to_string_lossy().into_owned();
    let old_body = "{\r\n    return \"prior\";\r\n  }";
    let target_body = "{\r\n    return \"target🙂\";\r\n  }";
    let prior = source(old_body);
    let target = source(target_body);
    std::fs::write(&file, &prior).unwrap();
    let initial_state = state(&root);
    assert!(
        baseline(&initial_state, &file, &root)
            .get("error")
            .is_none()
    );
    let alias = initial_state.alias_for_path(&file).unwrap();
    let prior_hash = initial_state
        .ir_context_read()
        .get_source_hash(&alias)
        .cloned()
        .unwrap();
    let prior_version = initial_state.file_version(&alias).unwrap();
    let target_text = std::str::from_utf8(&target).unwrap();
    let (mut target_ir, target_edges, target_hash) =
        crate::mcp::tool_helpers::compile_source_ir_candidate(
            &file,
            target_text,
            crate::compression::Fidelity::Edit,
            &initial_state,
        )
        .unwrap();
    target_ir.file_id.clone_from(&file);
    let target_version = target_ir.version;
    let target_binary = crate::ir::binary_wire::encode(&target_ir);
    let intent = crate::mcp::sqlite_store::EditIntent {
        transition_id: "restart-target-transition".to_string(),
        file_path: file.clone(),
        prior_hash,
        target_hash,
        prior_version,
        target_version,
        prior_source: prior,
        target_source: target.clone(),
        target_ir: target_binary,
        target_edges,
        fidelity: crate::compression::Fidelity::Edit,
        stage_path: String::new(),
    };
    initial_state
        .persistence_store_lock()
        .as_ref()
        .unwrap()
        .sqlite()
        .unwrap()
        .establish_edit_intent(&intent)
        .unwrap();
    std::fs::write(&file, &target).unwrap();
    drop(initial_state);

    let restarted = state(&root);
    let restored = dispatch(
        &restarted,
        40,
        "restore_context",
        json!({ "filePath": file }),
    );
    assert!(restored.get("error").is_none(), "{restored}");
    assert_eq!(std::fs::read(&file).unwrap(), target);
    assert_eq!(durable(&restarted, &file).version, target_version);
    assert!(
        restored["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains(target_body)
    );
    assert!(
        !restarted
            .persistence_store_lock()
            .as_ref()
            .unwrap()
            .sqlite()
            .unwrap()
            .has_edit_intent(&file)
            .unwrap()
    );
}
