// src/tests/mcp/phase_b_retirement.rs
//
// Phase B — `delta_text_context` / `TextDelta` / `§Δ` transport removal
// (2026-08-25).
//
// New contract:
//   1. `delta_text_context` is NOT a registered MCP tool.
//   2. No registered tool schema advertises the legacy transport.
//   3. The IR-native flow is untouched: provide/compress populate the
//      IR context; `apply_delta` accepts an IRDelta envelope against
//      that baseline (empty-ops probe keeps the test deterministic).
//
// Tests 1/2/4 are RED against the pre-Phase-B tree (the tool exists and
// its schema is advertised). Test 3 is an invariant guard that must pass
// both before and after — it proves the surviving architecture was never
// dependent on the removed transport.

use crate::mcp::tools::{dispatch_tools_call, tool_list};
use serde_json::json;

fn phase_b_fixture() -> (tempfile::TempDir, String, String) {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let file = dir.path().join("svc.ts");
    std::fs::write(
        &file,
        "export class Svc {\n    ping(): string {\n        return 'pong';\n    }\n}\n",
    )
    .expect("write fixture");
    let root = dir.path().to_string_lossy().to_string();
    let path = file.to_string_lossy().to_string();
    (dir, path, root)
}

#[test]
fn phase_b_delta_text_context_is_no_longer_a_registered_tool() {
    let _serial = crate::protocol::handler_response_serial();
    let (_dir, path, root) = phase_b_fixture();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = json!(91);
    let params =
        json!({ "arguments": { "filePath": path, "fidelity": "medium", "workspaceRoot": root } });
    crate::protocol::captured_responses().clear();

    dispatch_tools_call(&id, "delta_text_context", &params, &state);
    let resp = crate::protocol::captured_responses().pop();
    let resp = resp.unwrap();

    let err = resp
        .get("error")
        .and_then(|e| e.as_object())
        .unwrap_or_else(|| {
            panic!("delta_text_context must be unregistered (expected JSON-RPC error), got: {resp}")
        });
    assert_eq!(err.get("code").and_then(|c| c.as_i64()), Some(-32601));
    let message = err.get("message").and_then(|m| m.as_str()).unwrap_or("");
    assert!(
        message.contains("delta_text_context"),
        "error must name the unknown tool: {message}"
    );
}

#[test]
fn phase_b_tool_catalog_omits_the_legacy_transport() {
    let names: Vec<String> = tool_list()
        .iter()
        .filter_map(|t| t.get("name").and_then(|n| n.as_str()).map(String::from))
        .collect();
    assert!(
        !names.iter().any(|n| n.contains("delta_text_context")),
        "legacy transport still advertised in tool catalog: {names:?}"
    );
    // The surviving IR-native delta surface stays registered.
    assert!(
        names.iter().any(|n| n == "delta_code_context"),
        "delta_code_context must remain registered"
    );
    assert!(
        names.iter().any(|n| n == "apply_delta"),
        "apply_delta must remain registered"
    );
}

#[test]
fn phase_b_registered_schemas_carry_no_sd_marker() {
    let wire = serde_json::to_string(&tool_list()).expect("serialize tool catalog");
    assert!(
        !wire.contains("§Δ"),
        "legacy §Δ transport marker leaked into the tool catalog"
    );
    assert!(
        !wire.contains("delta_text_context"),
        "legacy transport name leaked into the tool catalog"
    );
}

#[test]
fn phase_b_ir_native_delta_flow_end_to_end_still_works() {
    let _serial = crate::protocol::handler_response_serial();
    let (_dir, path, root) = phase_b_fixture();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = json!(92);

    // Step 1: full compression populates the IR context (version 1).
    let compress_params =
        json!({ "arguments": { "filePath": path, "fidelity": "medium", "workspaceRoot": root } });
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&id, "compress_code_context", &compress_params, &state);
    let compressed = crate::protocol::captured_responses().pop();
    let compressed = compressed.unwrap();
    assert!(
        compressed.get("error").is_none(),
        "baseline compression failed: {compressed}"
    );
    let file_id = compressed
        .pointer("/result/file")
        .and_then(|f| f.as_str())
        .expect("compress response must expose the IR file id")
        .to_string();

    // Step 2: apply_delta consumes an IRDelta envelope against that
    // baseline. Empty ops keep the assertion deterministic while still
    // exercising deserialize → ContextState::apply → version bump.
    let delta_params = json!({
        "arguments": {
            "delta": { "file": file_id, "from": 1, "to": 2, "ops": { "+": [], "~": [], "-": [] } },
            "currentVersion": 1,
        }
    });
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&id, "apply_delta", &delta_params, &state);
    let applied = crate::protocol::captured_responses().pop();
    let applied = applied.unwrap();

    assert!(
        applied.get("error").is_none(),
        "IR-native apply_delta must succeed against the populated baseline: {applied}"
    );
}
