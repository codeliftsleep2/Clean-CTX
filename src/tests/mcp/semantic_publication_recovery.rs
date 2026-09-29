//! P9-20 registered semantic-publication recovery boundary.

use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

fn state(root: &tempfile::TempDir) -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config.persistence.enabled = true;
    config.persistence.db_path = root.path().join("p9-20.db").to_string_lossy().into_owned();
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

fn request(root: &tempfile::TempDir, file: &str) -> Value {
    json!({
        "filePath": file,
        "workspaceRoot": root.path().to_string_lossy(),
        "fidelity": "edit"
    })
}

fn baseline(state: &crate::mcp::McpState, root: &tempfile::TempDir, file: &str) {
    let response = dispatch(state, 1, "compress_code_context", request(root, file));
    assert!(response.get("error").is_none(), "{response}");
}

fn establish_irreconcilable_intent(
    state: &crate::mcp::McpState,
    file: &str,
    prior: &[u8],
    target: &[u8],
    disk: &[u8],
) {
    let alias = state.alias_for_path(file).expect("session alias");
    let prior_hash = state
        .ir_context_read()
        .get_source_hash(&alias)
        .cloned()
        .expect("prior hash");
    let prior_version = state.file_version(&alias).expect("prior version");
    let (mut ir, edges, target_hash) = crate::mcp::tool_helpers::compile_source_ir_candidate(
        file,
        std::str::from_utf8(target).expect("UTF-8 fixture"),
        crate::compression::Fidelity::Edit,
        state,
    )
    .expect("target compilation");
    ir.file_id = file.to_string();
    let intent = crate::mcp::sqlite_store::EditIntent {
        transition_id: format!("p9-20-{}", ir.version),
        file_path: file.to_string(),
        prior_hash,
        target_hash,
        prior_version,
        target_version: ir.version,
        prior_source: prior.to_vec(),
        target_source: target.to_vec(),
        target_ir: crate::ir::binary_wire::encode(&ir),
        target_edges: edges,
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
        .expect("edit intent");
    std::fs::write(file, disk).unwrap();
}

fn assert_recovery_failure(response: &Value) {
    assert!(
        response["error"]["message"]
            .as_str()
            .is_some_and(|message| message.contains("Edit recovery failed")),
        "{response}"
    );
}

#[test]
fn compression_and_provision_refuse_publication_before_recovery() {
    let _serial = crate::protocol::handler_response_serial();
    for tool in ["compress_code_context", "provide_code_context"] {
        let root = tempfile::tempdir().unwrap();
        let file = root
            .path()
            .join(format!("{tool}.ts"))
            .to_string_lossy()
            .into_owned();
        let prior = b"export class Prior { run() { return 1; } }\n";
        let target = b"export class Target { run() { return 2; } }\n";
        std::fs::write(&file, prior).unwrap();
        let initial = state(&root);
        baseline(&initial, &root, &file);
        establish_irreconcilable_intent(
            &initial,
            &file,
            prior,
            target,
            b"external bytes outside the owned transition\n",
        );
        drop(initial);

        let restarted = state(&root);
        let before_edges = restarted.workspace_index_read().edge_count();
        let response = dispatch(&restarted, 2, tool, request(&root, &file));
        assert_recovery_failure(&response);
        assert!(restarted.alias_for_path(&file).is_none());
        assert_eq!(restarted.workspace_index_read().edge_count(), before_edges);
    }
}

#[test]
fn save_refuses_to_checkpoint_split_source_authority() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("save.ts").to_string_lossy().into_owned();
    let prior = b"export class Prior { run() { return 1; } }\n";
    let target = b"export class Target { run() { return 2; } }\n";
    std::fs::write(&file, prior).unwrap();
    let state = state(&root);
    baseline(&state, &root, &file);
    let alias = state.alias_for_path(&file).unwrap();
    let prior_ir = state.ir_context_read().get_ir(&alias).cloned();
    let prior_hash = state.ir_context_read().get_source_hash(&alias).cloned();
    establish_irreconcilable_intent(
        &state,
        &file,
        prior,
        target,
        b"external bytes outside the owned transition\n",
    );

    let response = dispatch(&state, 3, "save_context", json!({ "filePath": &file }));
    assert_recovery_failure(&response);
    assert_eq!(state.ir_context_read().get_ir(&alias).cloned(), prior_ir);
    assert_eq!(
        state.ir_context_read().get_source_hash(&alias).cloned(),
        prior_hash
    );
}

#[test]
fn workspace_query_refuses_index_hydration_before_recovery() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let file = root
        .path()
        .join("Hydrated.ts")
        .to_string_lossy()
        .into_owned();
    let prior = b"export class Hydrated { run() { return 1; } }\n";
    let target = b"export class Hydrated { run() { return 2; } }\n";
    std::fs::write(&file, prior).unwrap();
    let initial = state(&root);
    baseline(&initial, &root, &file);
    establish_irreconcilable_intent(
        &initial,
        &file,
        prior,
        target,
        b"export class Hydrated { external() { return 3; } }\n",
    );
    drop(initial);

    let restarted = state(&root);
    let response = dispatch(
        &restarted,
        4,
        "workspace_query",
        json!({
            "type": "find_entities",
            "name": "Hydrated",
            "workspaceRoot": root.path().to_string_lossy()
        }),
    );
    assert_recovery_failure(&response);
    assert!(restarted.alias_for_path(&file).is_none());
    assert!(
        restarted
            .workspace_index_read()
            .entities_in_file(&crate::dictionary::path::canonical_identity_key(&file))
            .is_empty()
    );
}

#[test]
fn successful_recovery_preserves_registered_render_and_hydration_behavior() {
    let _serial = crate::protocol::handler_response_serial();
    for (tool, id) in [("compress_code_context", 20), ("provide_code_context", 21)] {
        let root = tempfile::tempdir().unwrap();
        let file = root
            .path()
            .join(format!("{tool}-ok.ts"))
            .to_string_lossy()
            .into_owned();
        let prior = b"export class Prior { run() { return 1; } }\n";
        let target = b"export class Target { run() { return 2; } }\n";
        std::fs::write(&file, prior).unwrap();
        let initial = state(&root);
        baseline(&initial, &root, &file);
        establish_irreconcilable_intent(&initial, &file, prior, target, target);
        drop(initial);

        let restarted = state(&root);
        let response = dispatch(&restarted, id, tool, request(&root, &file));
        assert!(response.get("error").is_none(), "{response}");
        assert!(restarted.alias_for_path(&file).is_some());
        assert!(
            response["result"]["content"][0]["text"]
                .as_str()
                .is_some_and(|text| text.contains("Target")),
            "{response}"
        );
    }

    let root = tempfile::tempdir().unwrap();
    let file = root
        .path()
        .join("HydratedOk.ts")
        .to_string_lossy()
        .into_owned();
    let prior = b"export class Prior { run() { return 1; } }\n";
    let target = b"export class HydratedOk { run() { return 2; } }\n";
    std::fs::write(&file, prior).unwrap();
    let initial = state(&root);
    baseline(&initial, &root, &file);
    establish_irreconcilable_intent(&initial, &file, prior, target, target);
    drop(initial);
    let restarted = state(&root);
    let response = dispatch(
        &restarted,
        22,
        "workspace_query",
        json!({
            "type": "find_entities",
            "name": "HydratedOk",
            "workspaceRoot": root.path().to_string_lossy()
        }),
    );
    assert!(response.get("error").is_none(), "{response}");
    assert!(
        response["result"]["structuredContent"]["count"]
            .as_u64()
            .unwrap()
            > 0
    );
}

#[test]
fn save_recovery_checkpoints_the_recovered_target() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let file = root
        .path()
        .join("save-ok.ts")
        .to_string_lossy()
        .into_owned();
    let prior = b"export class Prior { run() { return 1; } }\n";
    let target = b"export class Target { run() { return 2; } }\n";
    std::fs::write(&file, prior).unwrap();
    let state = state(&root);
    baseline(&state, &root, &file);
    establish_irreconcilable_intent(&state, &file, prior, target, target);

    let response = dispatch(&state, 23, "save_context", json!({ "filePath": &file }));
    assert!(response.get("error").is_none(), "{response}");
    let alias = state.alias_for_path(&file).unwrap();
    let restored = state.ir_context_read().get_ir(&alias).cloned().unwrap();
    assert!(
        restored
            .iter()
            .any(|tuple| tuple.iter().any(|part| part.contains("Target")))
    );
}
