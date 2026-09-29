//! Regression coverage for delta-handler statistics lifecycle.
//!
//! A handler response and the read-only `context_stats` view must describe the
//! same event. Full fallbacks are full events; dedicated delta generation must
//! not bypass the session accumulator.

use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

fn state(root: &tempfile::TempDir) -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config.auto_delta = true;
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
        .expect("registered handler response")
}

fn source(call: &str) -> String {
    format!(
        "import {{ Injectable }} from '@angular/core';\n\
         @Injectable()\n\
         export class DeltaStats {{ run() {{ {call} }} }}\n{}",
        "// economics-padding-0123456789abcdef\n".repeat(1_000)
    )
}

#[test]
fn dedicated_delta_records_full_baseline_and_generated_delta_stats() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let path = root.path().join("dedicated-delta-stats.ts");
    let file = path.to_string_lossy().into_owned();
    std::fs::write(&path, source("first();")).expect("baseline source");
    let state = state(&root);
    let args = || {
        json!({
            "filePath": file.clone(),
            "workspaceRoot": root.path().to_string_lossy(),
            "fidelity": "high"
        })
    };

    let baseline = dispatch(&state, 1, "delta_code_context", args());
    assert!(baseline.get("error").is_none(), "{baseline}");
    {
        let stats = state.session_stats_lock();
        let file_stats = stats
            .file_stats(&file)
            .expect("dedicated baseline must be visible to context_stats");
        assert_eq!(file_stats.strategy, "full");
        assert_eq!(file_stats.fidelity, "high");
        assert!(file_stats.is_angular);
        assert!(file_stats.raw_tokens > 0);
        assert!(file_stats.compressed_tokens > 0);
        assert_eq!(file_stats.delta_count, 0);
    }

    std::fs::write(&path, source("second();")).expect("changed source");
    state.invalidate_source_cache(&file);
    let delta = dispatch(&state, 2, "delta_code_context", args());
    assert_eq!(delta["result"]["strategy"], "delta", "{delta}");

    let stats = state.session_stats_lock();
    let file_stats = stats
        .file_stats(&file)
        .expect("generated delta must be visible to context_stats");
    assert_eq!(file_stats.strategy, "delta");
    assert_eq!(file_stats.fidelity, "high");
    assert!(file_stats.is_angular);
    assert_eq!(file_stats.delta_count, 1);
}

#[test]
fn unchanged_follow_up_provide_remains_a_full_stats_event() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let path = root.path().join("unchanged-provider.ts");
    let file = path.to_string_lossy().into_owned();
    std::fs::write(&path, source("unchanged();")).expect("source");
    let state = state(&root);
    let args = || {
        json!({
            "filePath": file.clone(),
            "workspaceRoot": root.path().to_string_lossy()
        })
    };

    let baseline = dispatch(&state, 1, "provide_code_context", args());
    assert_eq!(baseline["result"]["_meta"]["strategy"], "full");
    let unchanged = dispatch(&state, 2, "provide_code_context", args());
    assert_eq!(unchanged["result"]["_meta"]["strategy"], "full");

    let stats = state.session_stats_lock();
    let file_stats = stats.file_stats(&file).expect("full stats event");
    assert_eq!(file_stats.strategy, "full");
    assert_eq!(file_stats.delta_count, 0);
    assert_eq!(stats.summary().delta_count, 0);
}
