use crate::ir::opcodes::CoreOp;
#[cfg(any(windows, unix))]
use crate::mcp::context_store::ContextStore;
use crate::mcp::tool_handlers::control_full_test_support;
use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};
use std::sync::Arc;

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
        "provide_code_context",
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

#[cfg(any(windows, unix))]
fn assert_hard_link_edit_refusal_preserves_one_durable_lifecycle_owner() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let owner = root.path().join("hard-link-owner.ts");
    let alternate = root.path().join("hard-link-alternate.ts");
    let old_body = "{\r\n    return \"same\";\r\n  }";
    let new_body = "{\r\n    return \"changed\";\r\n  }";
    let bytes = source(old_body);
    std::fs::write(&owner, &bytes).expect("owner fixture");
    std::fs::hard_link(&owner, &alternate).expect("hard-link fixture");
    let owner_text = owner.to_string_lossy().into_owned();
    let alternate_text = alternate.to_string_lossy().into_owned();
    let lifecycle_state = state(&root);

    let produced = baseline(&lifecycle_state, &owner_text, &root);
    assert!(produced.get("error").is_none(), "{produced}");
    let repeated = baseline(&lifecycle_state, &alternate_text, &root);
    assert!(repeated.get("error").is_none(), "{repeated}");
    let alias = lifecycle_state
        .alias_for_path(&owner_text)
        .expect("owner alias");
    assert_eq!(
        lifecycle_state.alias_for_path(&alternate_text).as_deref(),
        Some(alias.as_str())
    );
    let before_version = lifecycle_state.file_version(&alias);
    let before_ir = lifecycle_state.ir_context_read().get_ir(&alias).cloned();
    let before_hash = lifecycle_state
        .ir_context_read()
        .get_source_hash(&alias)
        .cloned();
    let before_edges = serde_json::to_value(lifecycle_state.semantic_edges(&alias)).unwrap();
    let before_fidelity = lifecycle_state.context_fidelity(&alias);

    let rejected = dispatch(
        &lifecycle_state,
        2,
        "apply_edit",
        replace_body(&alternate_text, old_body, new_body),
    );
    assert_eq!(rejected["error"]["code"], -32602, "{rejected}");
    assert_eq!(
        rejected["error"]["data"]["code"],
        "hard_link_edit_unsupported"
    );
    assert_eq!(std::fs::read(&owner).unwrap(), bytes);
    assert_eq!(std::fs::read(&alternate).unwrap(), bytes);
    assert_eq!(lifecycle_state.file_version(&alias), before_version);
    assert_eq!(
        lifecycle_state.ir_context_read().get_ir(&alias).cloned(),
        before_ir
    );
    assert_eq!(
        lifecycle_state
            .ir_context_read()
            .get_source_hash(&alias)
            .cloned(),
        before_hash
    );
    assert_eq!(
        serde_json::to_value(lifecycle_state.semantic_edges(&alias)).unwrap(),
        before_edges
    );
    assert_eq!(lifecycle_state.context_fidelity(&alias), before_fidelity);
    assert_eq!(lifecycle_state.pending_transition_count(&alias), 0);
    assert_eq!(
        lifecycle_state.alias_for_path(&owner_text).as_deref(),
        Some(alias.as_str())
    );
    assert_eq!(
        lifecycle_state.alias_for_path(&alternate_text).as_deref(),
        Some(alias.as_str())
    );

    {
        let guard = lifecycle_state.persistence_store_lock();
        let sqlite = guard.as_ref().unwrap().sqlite().unwrap();
        assert!(sqlite.has_context(&owner_text));
        assert!(!sqlite.has_context(&alternate_text));
        assert!(!sqlite.has_edit_intent(&owner_text).unwrap());
        assert!(!sqlite.has_edit_intent(&alternate_text).unwrap());
    }
    drop(lifecycle_state);

    let restarted = state(&root);
    let owner_alias = restarted.get_or_create_alias(owner_text.clone());
    let alternate_alias = restarted.get_or_create_alias(alternate_text);
    assert_eq!(owner_alias, alternate_alias);
    let canonical_owner = crate::dictionary::path::canonical_identity_key(&owner_text);
    assert_eq!(
        restarted.path_for_alias(&owner_alias).as_deref(),
        Some(canonical_owner.as_str())
    );
    let restored = dispatch(
        &restarted,
        3,
        "restore_context",
        json!({
            "filePath": alternate.to_string_lossy(),
            "workspaceRoot": root.path().to_string_lossy()
        }),
    );
    assert!(restored.get("error").is_none(), "{restored}");
    assert_eq!(restored["result"]["_meta"]["file"], canonical_owner);
}

#[cfg(windows)]
#[test]
fn windows_hard_link_edit_refusal_preserves_one_durable_lifecycle_owner() {
    assert_hard_link_edit_refusal_preserves_one_durable_lifecycle_owner();
}

#[cfg(unix)]
#[test]
fn unix_hard_link_edit_refusal_preserves_one_durable_lifecycle_owner() {
    assert_hard_link_edit_refusal_preserves_one_durable_lifecycle_owner();
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
    assert!(
        control_full_test_support::has_method_body(&produced, "run", old_body),
        "full Body fallback was lost: {produced}"
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
    assert!(control_full_test_support::has_method_body(
        &restored, "run", new_body
    ));
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
fn edit_rejects_source_authority_published_after_candidate_preparation() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("authority-race.ts");
    let file = path.to_string_lossy().into_owned();
    let old_body = "{\n    return \"old\";\n  }";
    let edit_body = "{\n    return \"edit\";\n  }";
    let newer_body = "{\n    return \"newer\";\n  }";
    std::fs::write(&path, source(old_body)).unwrap();
    let state = Arc::new(state(&root));
    assert!(baseline(&state, &file, &root).get("error").is_none());

    crate::mcp::tool_handlers::edit::race_test_support::arm(&file);
    let worker_state = Arc::clone(&state);
    let worker_file = file.clone();
    let edit = std::thread::spawn(move || {
        crate::protocol::captured_responses().clear();
        crate::mcp::tool_handlers::edit::handle_apply_edit(
            &json!(10),
            &json!({ "arguments": replace_body(&worker_file, old_body, edit_body) }),
            &worker_state,
        );
        crate::protocol::captured_responses()
            .pop()
            .expect("apply_edit response")
    });
    crate::mcp::tool_handlers::edit::race_test_support::wait_until_paused();

    let newer_source = source(newer_body);
    std::fs::write(&path, &newer_source).unwrap();
    state.invalidate_source_cache(&file);
    let published = baseline(&state, &file, &root);
    assert!(published.get("error").is_none(), "{published}");

    crate::mcp::tool_handlers::edit::race_test_support::release();
    let edit_response = edit.join().unwrap();

    assert_eq!(
        edit_response["error"]["data"]["code"], "stale_edit_source",
        "an edit prepared from superseded H1 must reject rather than overwrite H3: {edit_response}"
    );
    assert_eq!(std::fs::read(&path).unwrap(), newer_source);
    let alias = state.alias_for_path(&file).unwrap();
    let newer_hash = state.cache_read().compute_hash(&newer_source);
    assert_eq!(
        state.ir_context_read().get_source_hash(&alias),
        Some(&newer_hash)
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
    let identities = crate::mcp::compatibility::derive_identities(
        target_text,
        std::path::Path::new(&file),
        &initial_state.config,
    )
    .expect("target compatibility identities");
    initial_state
        .persistence_store_lock()
        .as_ref()
        .unwrap()
        .sqlite()
        .unwrap()
        .establish_compatible_edit_intent(&intent, &identities)
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
    assert!(control_full_test_support::has_method_body(
        &restored,
        "run",
        target_body
    ));
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
