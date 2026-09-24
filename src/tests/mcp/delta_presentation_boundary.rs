// src/tests/mcp/delta_presentation_boundary.rs
//
// RED regression suite for the delta-transport content boundary.
//
// The delta is code-side ONLY: it carries the changed operations as an op
// list (`result.delta`) for the server's stateful cache, and its model-visible
// `content` is a minimal summary (adds/mods/dels counts) matching main's
// `handle_delta_code_context`. It must never ship a full presentation (the
// stateless LLM gets that from the full path) nor a `// FILE-CONTEXT-DELTA v1`
// envelope (a this-branch invention main never had).
//
// These tests pin the BOUNDARY: content must be a minimal summary — not an
// envelope, not a presentation.

#![cfg(feature = "typescript")]

use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

fn state(root: &tempfile::TempDir) -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
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

/// Write the fixture, establish a baseline, mutate the file, and return the
/// model-visible `content` of the resulting delta response.
fn delta_content(root: &tempfile::TempDir) -> String {
    let path = root.path().join("delta.ts");
    let args = || {
        json!({
            "filePath": path.to_string_lossy(),
            "workspaceRoot": root.path().to_string_lossy(),
            "fidelity": "high"
        })
    };
    let body = |call: &str| {
        format!(
            "export class Delta {{\n  run() {{ {call} }}\n}}\n{}",
            "// economics-padding-0123456789abcdef\n".repeat(1_000)
        )
    };
    std::fs::write(&path, body("first();")).expect("fixture");
    let state = state(root);
    let baseline = dispatch(&state, 1, "delta_code_context", args());
    assert!(baseline.get("error").is_none(), "{baseline}");
    // Mutate the file; invalidate the source cache so the delta is non-empty.
    std::fs::write(&path, body("second();")).expect("modified fixture");
    state.invalidate_source_cache(path.to_string_lossy().as_ref());
    let delta = dispatch(&state, 2, "delta_code_context", args());
    assert!(delta.get("error").is_none(), "{delta}");
    delta["result"]["content"][0]["text"]
        .as_str()
        .expect("delta content")
        .to_string()
}

#[test]
fn red_delta_content_is_not_the_delta_envelope() {
    let root = tempfile::tempdir().expect("temp workspace");
    let text = delta_content(&root);
    assert!(
        !text.starts_with("// FILE-CONTEXT-DELTA v1"),
        "delta content must be the full presentation, not the FILE-CONTEXT-DELTA \
         envelope:\n{text}"
    );
}

#[test]
fn red_delta_content_is_not_the_presentation() {
    let root = tempfile::tempdir().expect("temp workspace");
    let text = delta_content(&root);
    assert!(
        !text.contains("// SCHEMA v5"),
        "delta content must NOT be the full presentation — the delta is code-side \
         only and must not spend LLM tokens rendering a presentation:\n{text}"
    );
}

#[test]
fn red_delta_content_is_a_minimal_summary() {
    let root = tempfile::tempdir().expect("temp workspace");
    let text = delta_content(&root);
    assert!(
        text.starts_with("Δ delta for"),
        "delta content must be a minimal summary of the adds/mods/dels counts, \
         matching main's `handle_delta_code_context`:\n{text}"
    );
}
