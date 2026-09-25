#![cfg(feature = "typescript")]

use crate::compression::Fidelity;
use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

fn config(root: &tempfile::TempDir) -> crate::config::CleanCtxConfig {
    let mut config = crate::tests::test_config();
    config.persistence.enabled = true;
    config.persistence.auto_save = false;
    config.persistence.db_path = root
        .path()
        .join("delta-fidelity.db")
        .to_string_lossy()
        .into_owned();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    config
}

fn dispatch(state: &crate::mcp::McpState, id: i64, tool: &str, arguments: Value) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&json!(id), tool, &json!({ "arguments": arguments }), state);
    crate::protocol::captured_responses()
        .pop()
        .expect("registered handler response")
}

fn source(return_value: u8) -> String {
    format!(
        r#"export class Worker {{
  private first: number = 1;
  private revision_{return_value}: number = {return_value};

  async run(flag: boolean): Promise<number> {{
    if (flag) {{
      await fetch('/worker');
      return {return_value};
    }}
    return 0;
  }}
}}
{}"#,
        "// economics-padding-0123456789abcdef\n".repeat(1_000)
    )
}

#[test]
fn apply_delta_rejects_missing_authoritative_fidelity_before_mutation() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("temp workspace");
    let path = root.path().join("missing-fidelity.ts");
    let file = path.to_string_lossy().into_owned();
    std::fs::write(&path, "export class Worker {}\n").expect("source");
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let alias = state.get_or_create_alias(file);
    state.ir_context_lock().load_ir(
        crate::ir::compiler::CompiledIR {
            file_id: alias.clone(),
            instructions: Vec::new(),
            version: 1,
        },
        Some("baseline-hash".to_string()),
    );

    let response = dispatch(
        &state,
        1,
        "apply_delta",
        json!({
            "delta": {
                "file": alias.clone(),
                "from": 1,
                "to": 2,
                "ops": { "+": [], "~": [], "-": [] }
            },
            "currentVersion": 1
        }),
    );
    assert!(
        response
            .to_string()
            .contains("missing authoritative fidelity"),
        "{response}"
    );
    assert_eq!(state.file_version(&alias), Some(1));
}

#[test]
fn auto_delta_missing_baseline_preserves_high_fidelity_across_restart_restore() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("temp workspace");
    let path = root.path().join("worker.ts");
    let file = path.to_string_lossy().into_owned();
    let workspace_root = root.path().to_string_lossy().into_owned();
    std::fs::write(&path, source(1)).expect("baseline source");

    let state = crate::mcp::McpState::new(config(&root));
    let baseline = dispatch(
        &state,
        1,
        "provide_code_context",
        json!({
            "filePath": file.clone(),
            "workspaceRoot": workspace_root.clone(),
            "fidelity": "high"
        }),
    );
    assert!(baseline.get("error").is_none(), "{baseline}");
    assert_eq!(baseline["result"]["_meta"]["strategy"], "full");

    std::fs::write(&path, source(2)).expect("target source");
    state.invalidate_source_cache(&file);
    let generated = dispatch(
        &state,
        2,
        "provide_code_context",
        json!({
            "filePath": file.clone(),
            "workspaceRoot": workspace_root
        }),
    );
    assert!(generated.get("error").is_none(), "{generated}");
    assert_eq!(generated["result"]["_meta"]["strategy"], "delta");
    assert_eq!(generated["result"]["_meta"]["fidelity"], "high");
    let delta = generated["result"]["_meta"]["delta"].clone();
    let from = generated["result"]["_meta"]["from_version"].clone();
    assert!(delta.is_object(), "automatic delta payload: {generated}");

    {
        let store = state.persistence_store_lock();
        let sqlite = store
            .as_ref()
            .expect("persistence")
            .sqlite()
            .expect("SQLite");
        assert!(
            sqlite
                .baseline_binary(&file)
                .expect("baseline lookup")
                .is_none(),
            "the apply path must materialize the missing durable baseline"
        );
    }

    let applied = dispatch(
        &state,
        3,
        "apply_delta",
        json!({ "delta": delta, "currentVersion": from }),
    );
    assert!(applied.get("error").is_none(), "{applied}");

    {
        let store = state.persistence_store_lock();
        let sqlite = store
            .as_ref()
            .expect("persistence")
            .sqlite()
            .expect("SQLite");
        let durable = sqlite
            .load_durable_context(&file, None)
            .expect("durable load")
            .expect("durable context");
        assert_eq!(durable.fidelity, Fidelity::High);
    }
    drop(state);

    let restarted = crate::mcp::McpState::new(config(&root));
    let restored = dispatch(
        &restarted,
        4,
        "restore_context",
        json!({ "filePath": file.clone() }),
    );
    assert!(restored.get("error").is_none(), "{restored}");
    let text = restored["result"]["content"][0]["text"]
        .as_str()
        .expect("restored presentation");
    assert!(text.starts_with("// SCHEMA v5"), "{text}");
    assert!(
        text.contains("ctl:") && text.contains("IF"),
        "High-only control-flow presentation must survive restore: {text}"
    );
    assert!(
        text.contains("revision_2"),
        "restored presentation must contain the applied structural target: {text}"
    );
    let alias = restarted
        .alias_for_path(&file)
        .expect("restore recreates session alias");
    assert_eq!(restarted.context_fidelity(&alias), Some(Fidelity::High));
}
