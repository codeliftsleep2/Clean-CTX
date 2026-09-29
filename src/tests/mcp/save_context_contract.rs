//! Registered-dispatch contract for file-scoped persistence checkpoints.

use crate::ir::compiler::CompiledIR;
use crate::ir::delta::{SequenceDeltaComputer, SequenceEdit};
use crate::ir::opcodes::CoreOp;
use crate::mcp::context_store::ContextStore;
use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

fn state(root: &tempfile::TempDir) -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config.persistence.enabled = true;
    config.persistence.db_path = root
        .path()
        .join("save-context.db")
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

fn compile(state: &crate::mcp::McpState, path: &str, root: &str, id: i64) -> Value {
    dispatch(
        state,
        id,
        "compress_code_context",
        json!({
            "filePath": path, "workspaceRoot": root, "fidelity": "edit"
        }),
    )
}

#[test]
fn save_context_checkpoints_only_the_requested_file_and_reloads_exactly() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("temp workspace");
    let first = root.path().join("first.ts").to_string_lossy().into_owned();
    let second = root.path().join("second.ts").to_string_lossy().into_owned();
    let workspace = root.path().to_string_lossy().into_owned();
    std::fs::write(
        &first,
        "export class First { run(value: number): number { return value + 1; } }\n",
    )
    .expect("first source");
    std::fs::write(
        &second,
        "export class Second { stop(value: number): number { return value - 1; } }\n",
    )
    .expect("second source");
    let state = state(&root);
    assert!(
        compile(&state, &first, &workspace, 1)
            .get("error")
            .is_none()
    );
    assert!(
        compile(&state, &second, &workspace, 2)
            .get("error")
            .is_none()
    );

    let first_alias = state.alias_for_path(&first).expect("first alias");
    let compact_before = state
        .llm_text_cache_lock()
        .get(&first_alias)
        .cloned()
        .expect("compact output");
    let expected = state
        .ir_context_read()
        .get_ir(&first_alias)
        .cloned()
        .expect("canonical IR");
    {
        let store = state.persistence_store_lock();
        let mut sqlite = store
            .as_ref()
            .expect("persistence")
            .sqlite()
            .expect("SQLite");
        sqlite.clear_file(&first);
        sqlite.clear_file(&second);
    }

    let saved = dispatch(&state, 3, "save_context", json!({ "filePath": first }));
    assert_eq!(saved["result"]["_meta"]["saved"], 1, "{saved}");
    assert_eq!(saved["result"]["_meta"]["already_durable"], false);
    {
        let store = state.persistence_store_lock();
        let sqlite = store
            .as_ref()
            .expect("persistence")
            .sqlite()
            .expect("SQLite");
        let bytes = sqlite
            .baseline_binary(&first)
            .expect("query")
            .expect("checkpoint");
        assert_eq!(&bytes[..3], &[0xCC, 0x02, 0x04]);
        assert!(
            !sqlite.has_context(&second),
            "another file was checkpointed"
        );
    }
    assert_eq!(
        state.llm_text_cache_lock().get(&first_alias),
        Some(&compact_before)
    );

    state.ir_context_lock().remove_file(&first_alias);
    let replayed = dispatch(&state, 4, "replay_history", json!({ "filePath": first }));
    assert!(replayed.get("error").is_none(), "{replayed}");
    assert_eq!(
        state
            .ir_context_read()
            .get_ir(&first_alias)
            .expect("reloaded IR"),
        &expected
    );

    let unchanged = dispatch(&state, 5, "save_context", json!({ "filePath": first }));
    assert_eq!(unchanged["result"]["_meta"]["saved"], 0, "{unchanged}");
    assert_eq!(unchanged["result"]["_meta"]["already_durable"], true);
}

#[test]
fn save_context_rejects_missing_session_and_mismatched_durable_identity() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("temp workspace");
    let path = root.path().join("owned.ts").to_string_lossy().into_owned();
    let other = root.path().join("other.ts").to_string_lossy().into_owned();
    let workspace = root.path().to_string_lossy().into_owned();
    let state = state(&root);

    let missing = dispatch(&state, 10, "save_context", json!({ "filePath": path }));
    assert!(missing.get("error").is_some(), "{missing}");
    std::fs::write(&path, "export interface Owned { run(): void; }\n").expect("source");
    assert!(
        compile(&state, &path, &workspace, 11)
            .get("error")
            .is_none()
    );
    let alias = state.alias_for_path(&path).expect("session alias");
    state.remember_persisted_path(&alias, &other);
    let mismatch = dispatch(&state, 12, "save_context", json!({ "filePath": path }));
    assert!(mismatch.get("error").is_some(), "{mismatch}");
    assert!(
        mismatch["error"]["message"]
            .as_str()
            .is_some_and(|message| message.contains("durable identity")),
        "{mismatch}"
    );
}

#[test]
fn registered_apply_delta_rejects_malformed_canonical_tuple_transactionally() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("temp workspace");
    let path = root.path().join("delta.ts").to_string_lossy().into_owned();
    let workspace = root.path().to_string_lossy().into_owned();
    std::fs::write(&path, "export class DeltaOwner { run(): void {} }\n").expect("source");
    let state = state(&root);
    assert!(
        compile(&state, &path, &workspace, 20)
            .get("error")
            .is_none()
    );
    let alias = state.alias_for_path(&path).expect("session alias");
    let (version, tuples) = {
        let context = state.ir_context_read();
        (
            context.file_version(&alias).expect("version"),
            context.get_ir(&alias).cloned().expect("IR"),
        )
    };
    let baseline = CompiledIR {
        file_id: alias.clone(),
        version,
        instructions: tuples
            .iter()
            .map(|tuple| crate::ir::wire::tuple_to_op(tuple).expect("canonical tuple"))
            .collect(),
    };
    let mut target = baseline.clone();
    target.version += 1;
    target
        .instructions
        .push(CoreOp::TypeAlias("T-new".into(), "value".into()));
    let mut delta = SequenceDeltaComputer::new()
        .compute(&baseline, &target)
        .expect("insert delta");
    let SequenceEdit::Insert { instruction, .. } = &mut delta.edits[0] else {
        panic!("expected insert");
    };
    instruction.pop();

    let rejected = dispatch(
        &state,
        21,
        "apply_delta",
        json!({ "delta": delta, "currentVersion": version }),
    );
    assert!(rejected.get("error").is_some(), "{rejected}");
    assert_eq!(state.ir_context_read().get_ir(&alias), Some(&tuples));
}
