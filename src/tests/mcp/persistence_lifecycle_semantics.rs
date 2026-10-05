use super::{dispatch, in_memory_ir, state_with_persistence};
use crate::ir::opcodes::CoreOp;
use crate::mcp::tool_handlers::control_full_test_support;
use serde_json::json;

fn configured_state(
    root: &tempfile::TempDir,
    configure: impl FnOnce(&mut crate::config::CleanCtxConfig),
) -> crate::mcp::McpState {
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
    configure(&mut config);
    crate::mcp::McpState::new(config)
}

#[test]
fn historical_replay_rejects_incompatible_config_before_live_mutation() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("temporary workspace");
    let path = root.path().join("historical-config.ts");
    let file = path.to_string_lossy().into_owned();
    std::fs::write(&path, "export class Historical { value: UserId; }\n").expect("source");

    let producer = configured_state(&root, |_| {});
    let saved = dispatch(
        &producer,
        200,
        "compress_code_context",
        json!({
            "filePath": file,
            "workspaceRoot": root.path(),
            "fidelity": "low"
        }),
    );
    assert!(saved.get("error").is_none(), "{saved}");
    drop(producer);

    let restarted = configured_state(&root, |config| {
        config
            .type_aliases
            .insert("uid".to_string(), "UserId".to_string());
    });
    let rejected = dispatch(
        &restarted,
        201,
        "replay_history",
        json!({ "filePath": file, "targetSequence": 0 }),
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
fn registered_dispatch_exposes_migrated_semantic_families_after_reload() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("temp workspace");
    let path = root.path().join("semantic.ts");
    let file_path = path.to_string_lossy().into_owned();
    let padding = " production semantic-family fixture".repeat(180);
    std::fs::write(
        &path,
        format!(
            "abstract class SemanticService {{\n  constructor(private repo: Repo) {{}}\n  async run(flag: boolean): Promise<number> {{\n    stream.subscribe(value => console.log(value));\n    if (flag) {{ return await Promise.resolve(1); }}\n    return 0;\n  }}\n  values(): Observable<number> {{\n    return stream.pipe();\n  }}\n}}\n/*{padding} */\n"
        ),
    )
    .expect("semantic source");
    let state = state_with_persistence(&root);
    let args = json!({
        "filePath": file_path.clone(),
        "workspaceRoot": root.path().to_string_lossy(),
        "fidelity": "high"
    });
    let produced = dispatch(&state, 30, "compress_code_context", args);
    assert!(produced.get("error").is_none(), "{produced}");
    let compact = produced["result"]["content"][0]["text"]
        .as_str()
        .expect("CONTROL-FULL MCP text");
    assert!(compact.starts_with("// COMPACT-A A2") || compact.contains("SemanticService"));
    if compact.starts_with("// COMPACT-A A2") {
        for marker in ["M[id,name,params", "N.D and N.V index"] {
            assert!(compact.contains(marker), "missing {marker}: {compact}");
        }
        assert!(
            compact.contains("\"g\":{\"K\":"),
            "missing g.K calls: {compact}"
        );
        assert!(compact.contains("[\"M4\",\"cs\"]"));
        assert!(!compact.contains("[\"M4\",\"cs\",[[\"IF\",\"RET\",\"RET\"]]]"));
    }
    let methods = produced["result"]["ir"]["ir"]["c"][0]["m"]
        .as_array()
        .expect("structured MCP methods");
    let run = methods
        .iter()
        .find(|method| method["nm"] == "run")
        .expect("run method hierarchy");
    assert!(
        run.get("se").is_none(),
        "se stripped from reduced result.ir"
    );
    assert!(
        run.get("ec").is_none(),
        "ec stripped from reduced result.ir"
    );
    let persisted = {
        let store = state.persistence_store_lock();
        let sqlite = store.as_ref().unwrap().sqlite().unwrap();
        sqlite
            .load_context_with_deltas(&file_path, None)
            .expect("durable replay")
            .expect("durable semantic context")
            .0
    };
    for (family, present) in [
        (
            "ClassModifiers",
            persisted
                .instructions
                .iter()
                .any(|op| matches!(op, CoreOp::ClassModifiers(..))),
        ),
        (
            "MethodModifiers",
            persisted
                .instructions
                .iter()
                .any(|op| matches!(op, CoreOp::MethodModifiers(..))),
        ),
        (
            "ControlSummary",
            persisted
                .instructions
                .iter()
                .any(|op| matches!(op, CoreOp::ControlSummary(..))),
        ),
        (
            "PatternFacts",
            persisted
                .instructions
                .iter()
                .any(|op| matches!(op, CoreOp::PatternFacts(..))),
        ),
        (
            "SideEffect",
            persisted
                .instructions
                .iter()
                .any(|op| matches!(op, CoreOp::SideEffect(..))),
        ),
        (
            "ExecutionContext",
            persisted
                .instructions
                .iter()
                .any(|op| matches!(op, CoreOp::ExecutionContext(..))),
        ),
        (
            "DataFlow",
            persisted
                .instructions
                .iter()
                .any(|op| matches!(op, CoreOp::DataFlow(..))),
        ),
        (
            "ControlFlow",
            persisted
                .instructions
                .iter()
                .any(|op| matches!(op, CoreOp::ControlFlow(..))),
        ),
    ] {
        assert!(
            present,
            "{family} missing from persisted canonical IR: {:?}",
            persisted.instructions
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
    let source = std::fs::read_to_string(&path).expect("semantic source remains readable");
    assert!(
        replayed_text.starts_with("// SCHEMA vNext")
            || replayed_text.starts_with("// COMPACT-A A2")
            || replayed_text == source,
        "unexpected replay presentation: {replayed_text}"
    );
    if replayed_text.starts_with("// SCHEMA vNext") {
        for marker in [
            "C SemanticService",
            "M run p:flag:boolean → Promise<number>",
            "M values → Observable<number>",
        ] {
            assert!(replayed_text.contains(marker), "missing replayed {marker}");
        }
    }
    if replayed_text.starts_with("// COMPACT-A A2") {
        for marker in ["M[id,name,params", "N.D and N.V index"] {
            assert!(replayed_text.contains(marker), "missing replayed {marker}");
        }
        assert!(
            replayed_text.contains("\"g\":{\"K\":"),
            "missing replayed g.K calls"
        );
        assert!(replayed_text.contains("[\"M4\",\"cs\"]"));
        assert!(!replayed_text.contains("[\"M4\",\"cs\",[[\"IF\",\"RET\",\"RET\"]]]"));
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
