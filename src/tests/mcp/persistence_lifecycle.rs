use crate::ir::compiler::CompiledIR;
use crate::ir::opcodes::CoreOp;
use crate::mcp::context_store::ContextStore;
use crate::mcp::tool_handlers::control_full_test_support;
use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

fn state_with_persistence(root: &tempfile::TempDir) -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config.persistence.enabled = true;
    config.persistence.db_path = root
        .path()
        .join("phase-8c.db")
        .to_string_lossy()
        .into_owned();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    crate::mcp::McpState::new(config)
}

fn response() -> Value {
    crate::protocol::captured_responses()
        .pop()
        .expect("registered handler response")
}

fn dispatch(state: &crate::mcp::McpState, id: i64, tool: &str, arguments: Value) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&json!(id), tool, &json!({ "arguments": arguments }), state);
    response()
}

fn in_memory_ir(state: &crate::mcp::McpState, alias: &str, file_path: &str) -> CompiledIR {
    let guard = state.ir_context_read();
    let instructions = guard
        .get_ir(alias)
        .expect("session IR")
        .iter()
        .filter_map(|tuple| crate::ir::wire::tuple_to_op(tuple))
        .collect();
    CompiledIR {
        file_id: file_path.to_string(),
        instructions,
        version: guard.file_version(alias).expect("session version"),
    }
}

#[test]
fn registered_dispatch_persists_v04_and_replays_sequence_history_exactly() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("temp workspace");
    let path = root.path().join("worker.ts");
    let file_path = path.to_string_lossy().into_owned();
    std::fs::write(
        &path,
        "export class Worker {\n  run(flag: boolean): number {\n    if (flag) { return 1; }\n    return 1;\n  }\n}\n",
    )
    .expect("baseline source");
    let state = state_with_persistence(&root);
    let workspace_root = root.path().to_string_lossy().into_owned();
    let args = || {
        json!({
            "filePath": file_path.clone(),
            "workspaceRoot": workspace_root.clone(),
            "fidelity": "edit"
        })
    };

    let baseline_response = dispatch(&state, 1, "provide_code_context", args());
    assert!(
        baseline_response.get("error").is_none(),
        "{baseline_response}"
    );

    let target_source = "export class Worker {\n  run(flag: boolean): number {\n    if (flag) { return 2; }\n    return 2;\n  }\n  stop(): void {}\n}\n";
    std::fs::write(&path, target_source).expect("target source");
    state.invalidate_source_cache(&file_path);
    let delta_response = dispatch(&state, 2, "delta_code_context", args());
    let delta = delta_response["result"]["delta"].clone();
    assert_eq!(delta["dv"], 2, "{delta_response}");
    let baseline = {
        let store = state.persistence_store_lock();
        let sqlite = store
            .as_ref()
            .expect("persistence")
            .sqlite()
            .expect("SQLite");
        let bytes = sqlite
            .baseline_binary(&file_path)
            .expect("baseline query")
            .expect("persisted baseline");
        assert!(
            bytes.len() > 3,
            "production must persist a non-empty IR blob"
        );
        assert_eq!(&bytes[..3], &[0xCC, 0x02, 0x04]);
        crate::ir::binary_wire::decode(&bytes).expect("physical binary 0x04")
    };
    assert_eq!(baseline.file_id, file_path);
    assert!(baseline.instructions.iter().any(
        |op| matches!(op, CoreOp::Body(_, text, Some(_), Some(_)) if text.contains("return 1"))
    ));
    let applied = dispatch(
        &state,
        3,
        "apply_delta",
        json!({ "delta": delta, "currentVersion": delta_response["result"]["from_version"] }),
    );
    assert!(applied.get("error").is_none(), "{applied}");
    state.flush_persistence();

    let alias = state.get_or_create_alias(file_path.clone());
    let expected = in_memory_ir(&state, &alias, &file_path);
    let replayed = {
        let store = state.persistence_store_lock();
        let sqlite = store
            .as_ref()
            .expect("persistence")
            .sqlite()
            .expect("SQLite");
        let context_id = sqlite
            .current_context_id(&file_path)
            .expect("owner query")
            .expect("persisted owner");
        assert_eq!(sqlite.delta_count(&context_id), 1);
        sqlite
            .load_context_with_deltas(&file_path, None)
            .expect("durable replay")
            .expect("durable context")
            .0
    };
    assert_eq!(
        replayed, expected,
        "order and duplicate occurrences must be exact"
    );

    state.ir_context_lock().remove_file(&alias);
    let replay_response = dispatch(
        &state,
        4,
        "replay_history",
        json!({ "filePath": file_path.clone() }),
    );
    assert!(replay_response.get("error").is_none(), "{replay_response}");
    assert_eq!(in_memory_ir(&state, &alias, &file_path), expected);

    let edit = dispatch(
        &state,
        5,
        "apply_edit",
        json!({
            "filePath": file_path.clone(),
            "operations": [{
                "type": "replace_body",
                "target": "Worker.stop",
                "expectedOldText": "{}",
                "newText": "{\n    return;\n  }"
            }]
        }),
    );
    assert!(edit.get("error").is_none(), "{edit}");
    let after_edit = std::fs::read_to_string(&path).expect("post-edit bytes");
    assert_eq!(
        after_edit,
        target_source.replace("stop(): void {}", "stop(): void {\n    return;\n  }")
    );
}

#[test]
fn buffered_v04_persistence_preserves_exact_duplicate_operations() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("temp workspace");
    let file_path = root
        .path()
        .join("duplicates.ts")
        .to_string_lossy()
        .into_owned();
    let duplicate = CoreOp::TypeAlias("T1".into(), "Thing<string>".into());
    let expected = CompiledIR {
        file_id: file_path.clone(),
        instructions: vec![duplicate.clone(), duplicate],
        version: 1,
    };
    let state = state_with_persistence(&root);
    let binary = crate::ir::binary_wire::encode(&expected);

    state
        .persistence_store_lock()
        .as_ref()
        .expect("persistence")
        .queue_save_context(
            &file_path,
            crate::compression::Fidelity::Low,
            "",
            &binary,
            "duplicate-contract",
            0,
            0,
        );
    state.flush_persistence();

    let store = state.persistence_store_lock();
    let sqlite = store.as_ref().unwrap().sqlite().unwrap();
    let replayed = sqlite
        .load_context_with_deltas(&file_path, None)
        .expect("durable replay")
        .expect("durable context")
        .0;
    assert_eq!(replayed, expected);
}

#[test]
fn overwrite_and_durable_restore_preserve_persisted_ownership_coherently() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("temp workspace");
    let path = root.path().join("lifecycle.ts");
    let file_path = path.to_string_lossy().into_owned();
    let state = state_with_persistence(&root);
    let workspace_root = root.path().to_string_lossy().into_owned();
    let args = || {
        json!({
            "filePath": file_path.clone(),
            "workspaceRoot": workspace_root.clone(),
            "fidelity": "low"
        })
    };

    std::fs::write(&path, "class First { one(): void {} }\n").unwrap();
    let first = dispatch(&state, 10, "compress_code_context", args());
    let repeated = dispatch(&state, 11, "compress_code_context", args());
    assert!(first.get("error").is_none(), "{first}");
    assert!(repeated.get("error").is_none(), "{repeated}");
    assert_eq!(
        control_full_test_support::payload_without_ir_version(&first),
        control_full_test_support::payload_without_ir_version(&repeated)
    );
    std::fs::write(&path, "class Second { two(): void {} }\n").unwrap();
    state.invalidate_source_cache(&file_path);
    assert!(
        dispatch(&state, 12, "compress_code_context", args())
            .get("error")
            .is_none()
    );

    {
        let store = state.persistence_store_lock();
        let sqlite = store.as_ref().unwrap().sqlite().unwrap();
        let rows = sqlite.list_contexts(10).expect("contexts");
        assert_eq!(
            rows.iter().filter(|row| row.file_path == file_path).count(),
            1
        );
        let latest = sqlite
            .load_context_with_deltas(&file_path, None)
            .unwrap()
            .unwrap()
            .0;
        assert!(
            latest
                .instructions
                .iter()
                .any(|op| matches!(op, CoreOp::DefClass(_, name) if name == "Second"))
        );
    }

    let restored = dispatch(&state, 13, "restore_context", args());
    assert!(restored.get("error").is_none(), "{restored}");
    {
        let store = state.persistence_store_lock();
        let sqlite = store.as_ref().unwrap().sqlite().unwrap();
        assert!(sqlite.has_context(&file_path));
    }

    std::fs::remove_file(&path).expect("delete source");
    state.ir_context_lock().remove_file(
        &state
            .alias_for_path(&file_path)
            .expect("session alias before restart"),
    );
    let restored_without_source = dispatch(&state, 14, "restore_context", args());
    assert!(
        restored_without_source.get("error").is_none(),
        "restore must not recompile or require source: {restored_without_source}"
    );

    state
        .persistence_store_lock()
        .as_ref()
        .unwrap()
        .queue_clear_file(&file_path);
    state.flush_persistence();
    let store = state.persistence_store_lock();
    assert!(
        !store
            .as_ref()
            .unwrap()
            .sqlite()
            .unwrap()
            .has_context(&file_path)
    );
}

#[test]
fn registered_replay_rejects_a_mismatched_persisted_file_identity() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("temp workspace");
    let path = root.path().join("identity.ts");
    let file_path = path.to_string_lossy().into_owned();
    std::fs::write(&path, "class Identity { check(): void {} }\n").unwrap();
    let state = state_with_persistence(&root);
    let compressed = dispatch(
        &state,
        20,
        "compress_code_context",
        json!({
            "filePath": file_path.clone(),
            "workspaceRoot": root.path().to_string_lossy(),
            "fidelity": "low"
        }),
    );
    assert!(compressed.get("error").is_none(), "{compressed}");

    {
        let store = state.persistence_store_lock();
        let mut sqlite = store.as_ref().unwrap().sqlite().unwrap();
        let bytes = sqlite.baseline_binary(&file_path).unwrap().unwrap();
        let mut ir = crate::ir::binary_wire::decode(&bytes).unwrap();
        ir.file_id = "different-file.ts".to_string();
        let mismatched = crate::ir::binary_wire::encode(&ir);
        sqlite
            .save_context(
                &file_path,
                crate::compression::Fidelity::Low,
                "unchanged compact output",
                Some(&mismatched),
                "mismatched-owner",
                0,
                0,
            )
            .unwrap();
    }

    let replay = dispatch(
        &state,
        21,
        "replay_history",
        json!({ "filePath": file_path }),
    );
    assert!(
        replay["error"]["message"]
            .as_str()
            .is_some_and(|message| message.contains("persisted binary file identity mismatch"))
    );
}

#[test]
fn registered_dispatch_exposes_migrated_semantic_families_after_reload() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("temp workspace");
    let path = root.path().join("semantic.ts");
    let file_path = path.to_string_lossy().into_owned();
    let padding = " production semantic-family fixture".repeat(180);
    std::fs::write(
        &path,
        format!(
            "abstract class SemanticService {{\n  constructor(private repo: Repo) {{}}\n  async run(flag: boolean): Promise<number> {{\n    stream.subscribe(value => console.log(value));\n    if (flag) {{ return await Promise.resolve(1); }}\n    return 0;\n  }}\n}}\n/*{padding} */\n"
        ),
    )
    .expect("semantic source");
    let state = state_with_persistence(&root);
    let args = json!({
        "filePath": file_path.clone(),
        "workspaceRoot": root.path().to_string_lossy(),
        "fidelity": "low"
    });

    let produced = dispatch(&state, 30, "compress_code_context", args);
    assert!(produced.get("error").is_none(), "{produced}");
    let compact = produced["result"]["content"][0]["text"]
        .as_str()
        .expect("CONTROL-FULL MCP text");
    assert!(compact.starts_with("// CONTROL-FULL v1"));
    for marker in [
        "modifier_occurrences",
        "control_summary_occurrences",
        "pattern_fact_occurrences",
    ] {
        assert!(compact.contains(marker), "missing {marker}: {compact}");
    }
    let methods = produced["result"]["ir"]["ir"]["c"][0]["m"]
        .as_array()
        .expect("structured MCP methods");
    let run = methods
        .iter()
        .find(|method| method["nm"] == "run")
        .expect("run method hierarchy");
    assert_eq!(run["se"], json!(["async"]));
    assert_eq!(run["ec"], json!(["async"]));

    let persisted = {
        let store = state.persistence_store_lock();
        let sqlite = store.as_ref().unwrap().sqlite().unwrap();
        sqlite
            .load_context_with_deltas(&file_path, None)
            .expect("durable replay")
            .expect("durable semantic context")
            .0
    };
    for present in [
        persisted
            .instructions
            .iter()
            .any(|op| matches!(op, CoreOp::ClassModifiers(..))),
        persisted
            .instructions
            .iter()
            .any(|op| matches!(op, CoreOp::MethodModifiers(..))),
        persisted
            .instructions
            .iter()
            .any(|op| matches!(op, CoreOp::ControlSummary(..))),
        persisted
            .instructions
            .iter()
            .any(|op| matches!(op, CoreOp::PatternFacts(..))),
        persisted
            .instructions
            .iter()
            .any(|op| matches!(op, CoreOp::SideEffect(..))),
        persisted
            .instructions
            .iter()
            .any(|op| matches!(op, CoreOp::ExecutionContext(..))),
        persisted
            .instructions
            .iter()
            .any(|op| matches!(op, CoreOp::DataFlow(..))),
        persisted
            .instructions
            .iter()
            .any(|op| matches!(op, CoreOp::ControlFlow(..))),
    ] {
        assert!(
            present,
            "migrated family missing from persisted canonical IR"
        );
    }

    let replayed = dispatch(
        &state,
        31,
        "replay_history",
        json!({ "filePath": file_path }),
    );
    assert!(replayed.get("error").is_none(), "{replayed}");
    let replayed_text = replayed["result"]["content"][0]["text"]
        .as_str()
        .expect("replayed MCP text");
    assert!(replayed_text.starts_with("// CONTROL-FULL v1"));
    for marker in [
        "modifier_occurrences",
        "control_summary_occurrences",
        "pattern_fact_occurrences",
    ] {
        assert!(replayed_text.contains(marker), "missing replayed {marker}");
    }
}

#[test]
fn registered_interface_producers_persist_reload_and_expose_interface_semantics() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("temp workspace");
    let state = state_with_persistence(&root);
    let workspace_root = root.path().to_string_lossy().into_owned();
    let fixtures = [
        (
            "api.ts",
            "export interface WorkerApi extends BaseApi, Audited { version: number; /* production interface persistence fixture deliberately carries enough source context to keep the compact semantic response cheaper than raw passthrough while exercising the registered dispatch path */ }\n",
            "WorkerApi",
            false,
            true,
        ),
        (
            "WorkerApi.java",
            "public interface WorkerApi extends BaseApi, Audited { int VERSION = 1; void run(String value); /* production interface persistence fixture deliberately carries enough source context to keep the compact semantic response cheaper than raw passthrough while exercising the registered dispatch path */ }\n",
            "WorkerApi",
            true,
            true,
        ),
        (
            "WorkerApi.cs",
            "public interface WorkerApi : BaseApi, Audited { int Version { get; } void Run(string value); /* production interface persistence fixture deliberately carries enough source context to keep the compact semantic response cheaper than raw passthrough while exercising the registered dispatch path */ }\n",
            "WorkerApi",
            true,
            true,
        ),
    ];

    for (offset, &(name, source, interface_name, has_method, has_field)) in
        fixtures.iter().enumerate()
    {
        let path = root.path().join(name);
        let source = format!(
            "{source}// {}\n",
            "production-interface-context ".repeat(256)
        );
        std::fs::write(&path, source).expect("interface source");
        let file_path = path.to_string_lossy().into_owned();
        let arguments = || {
            json!({
                "filePath": file_path.clone(),
                "workspaceRoot": workspace_root.clone(),
                "fidelity": "low"
            })
        };

        let first = dispatch(
            &state,
            100 + offset as i64 * 3,
            "compress_code_context",
            arguments(),
        );
        assert!(first.get("error").is_none(), "{first}");
        assert!(
            control_full_test_support::has_interface(&first, interface_name),
            "{first}"
        );

        state.flush_persistence();
        let persisted = {
            let store = state.persistence_store_lock();
            let sqlite = store.as_ref().unwrap().sqlite().unwrap();
            let bytes = sqlite
                .baseline_binary(&file_path)
                .unwrap()
                .expect("interface baseline");
            crate::ir::binary_wire::decode(&bytes).expect("interface binary")
        };
        assert!(
            persisted
                .instructions
                .iter()
                .any(|op| matches!(op, CoreOp::DefInterface(_, value) if value == interface_name))
        );
        assert!(
            !persisted
                .instructions
                .iter()
                .any(|op| matches!(op, CoreOp::DefClass(_, value) if value == interface_name))
        );
        let interface_id = persisted
            .instructions
            .iter()
            .find_map(|op| match op {
                CoreOp::DefInterface(id, value) if value == interface_name => Some(id),
                _ => None,
            })
            .unwrap();
        assert_eq!(
            persisted.instructions.iter().any(
                |op| matches!(op, CoreOp::DefInterfaceMethod(owner, ..) if owner == interface_id)
            ),
            has_method
        );
        assert_eq!(
            persisted.instructions.iter().any(
                |op| matches!(op, CoreOp::DefInterfaceField(owner, ..) if owner == interface_id)
            ),
            has_field
        );
        assert_eq!(
            persisted
                .instructions
                .iter()
                .filter(
                    |op| matches!(op, CoreOp::InterfaceExtends(owner, _) if owner == interface_id)
                )
                .count(),
            2
        );

        let alias = state.get_or_create_alias(file_path.clone());
        state.ir_context_lock().remove_file(&alias);
        let replay = dispatch(
            &state,
            101 + offset as i64 * 3,
            "replay_history",
            json!({ "filePath": file_path.clone() }),
        );
        assert!(replay.get("error").is_none(), "{replay}");
        let reloaded = in_memory_ir(&state, &alias, &file_path);
        assert_eq!(reloaded, persisted);
        let hierarchy = crate::ir::hierarchical::try_ir_to_hierarchical(&reloaded).unwrap();
        assert_eq!(hierarchy.interfaces.len(), 1);
        assert_eq!(hierarchy.interfaces[0].name, interface_name);

        let after_reload = dispatch(
            &state,
            102 + offset as i64 * 3,
            "compress_code_context",
            arguments(),
        );
        assert!(
            control_full_test_support::has_interface(&after_reload, interface_name),
            "{after_reload}"
        );
    }
}
