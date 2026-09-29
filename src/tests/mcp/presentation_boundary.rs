// src/tests/mcp/presentation_boundary.rs
//
// RED regression suite for the model-visible presentation boundary.
//
// The defect: every production content path assembles the model-visible
// `content` from the CONTROL-FULL *codec* (`compact_a::render_file_context`,
// A2) instead of from an LLM-facing presentation, so the model is handed the
// codec's decoder contract — its preamble, its grammar legend, its envelope
// schema id, and its body framing.
//
// That legend is code-side machinery by construction, not a presentation
// key: the A3 decoder requires it as a byte-exact prefix
// (`document::decode_cold` strips COLD_PREAMBLE + COLD_LEGEND before parsing
// anything). A decoder key exists so the codec can round-trip; the model
// only has to read.
//
// These tests pin the BOUNDARY, not the shape: the model must never be handed
// the codec document or its decoder key, and the facts the model genuinely
// needs must survive. They deliberately do NOT pin a specific presentation
// renderer or version, so a legitimate redesign cannot fail them for the
// wrong reason. Raw source satisfies them too — raw source is not the codec.

#![cfg(feature = "typescript")]

use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

/// The production fixture shape: large enough that the economics gate selects
/// a compressed candidate over raw source, so the boundary is genuinely
/// exercised rather than trivially satisfied.
fn fixture() -> String {
    let padding = " production semantic-family fixture".repeat(180);
    format!(
        "abstract class SemanticService {{\n  constructor(private repo: Repo) {{}}\n  async run(flag: boolean): Promise<number> {{\n    stream.subscribe(value => console.log(value));\n    if (flag) {{ return await Promise.resolve(1); }}\n    return 0;\n  }}\n}}\n/*{padding} */\n"
    )
}

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

fn model_text(response: &Value) -> String {
    assert!(response.get("error").is_none(), "{response}");
    response["result"]["content"][0]["text"]
        .as_str()
        .expect("model-visible text")
        .to_string()
}

/// Write the fixture and return everything one `compress_code_context` call
/// needs. The serial guard mirrors the other MCP suites: the captured-response
/// channel is process-wide.
fn compress_text(root: &tempfile::TempDir) -> String {
    let _serial = crate::protocol::handler_response_serial();
    let path = root.path().join("semantic.ts");
    std::fs::write(&path, fixture()).expect("semantic fixture");
    let state = state(root);
    let response = dispatch(
        &state,
        10,
        "compress_code_context",
        json!({
            "filePath": path.to_string_lossy(),
            "workspaceRoot": root.path().to_string_lossy(),
            "fidelity": "low",
        }),
    );
    model_text(&response)
}

fn provide_text(root: &tempfile::TempDir) -> String {
    let _serial = crate::protocol::handler_response_serial();
    let path = root.path().join("semantic.ts");
    std::fs::write(&path, fixture()).expect("semantic fixture");
    let state = state(root);
    let response = dispatch(
        &state,
        11,
        "provide_code_context",
        json!({
            "filePath": path.to_string_lossy(),
            "fidelity": "low",
        }),
    );
    model_text(&response)
}

// ── RED: the model must not be handed the codec document ──────────────────

#[test]
fn red_compress_content_is_not_the_codec_document() {
    let root = tempfile::tempdir().expect("temp workspace");
    let text = compress_text(&root);
    assert!(
        !text.contains("// COMPACT-A A1") && !text.contains("// COMPACT-A A2"),
        "model-visible content must be a presentation, not the COMPACT-A codec \
         document:\n{text}"
    );
}

#[test]
fn red_compress_content_does_not_carry_the_codec_decoder_legend() {
    let root = tempfile::tempdir().expect("temp workspace");
    let text = compress_text(&root);
    assert!(
        !text.contains(crate::ir::compact_a::FILE_CONTEXT_LEGEND),
        "model-visible content must not carry the codec's decoder legend — that \
         legend is a decoder contract required byte-exactly by the codec, not a \
         presentation the model needs:\n{text}"
    );
}

#[test]
fn red_compress_content_does_not_carry_the_codec_schema_id() {
    let root = tempfile::tempdir().expect("temp workspace");
    let text = compress_text(&root);
    assert!(
        !text.contains(crate::ir::compact_a::FILE_CONTEXT_SCHEMA),
        "model-visible content must not carry the codec envelope schema id `{}`:\n{text}",
        crate::ir::compact_a::FILE_CONTEXT_SCHEMA
    );
}

#[test]
fn red_compress_content_does_not_carry_codec_body_framing() {
    let root = tempfile::tempdir().expect("temp workspace");
    let text = compress_text(&root);
    assert!(
        !text.contains("§BODIES"),
        "model-visible content must not carry the codec's body-framing section:\n{text}"
    );
}

#[test]
fn red_provide_content_is_not_the_codec_document() {
    let root = tempfile::tempdir().expect("temp workspace");
    let text = provide_text(&root);
    assert!(
        !text.contains("// COMPACT-A A1") && !text.contains("// COMPACT-A A2"),
        "provide_code_context content must be a presentation, not the COMPACT-A \
         codec document:\n{text}"
    );
    assert!(
        !text.contains(crate::ir::compact_a::FILE_CONTEXT_LEGEND),
        "provide_code_context content must not carry the codec's decoder legend:\n{text}"
    );
}

// ── Guard: facts the model genuinely needs must survive the fix ───────────
//
// Non-RED invariant guard (like `pattern_identity.rs`'s documented guards):
// this is green before the fix and must stay green after it. It exists so the
// boundary fix cannot be "achieved" by shipping something that no longer tells
// the model which type owns the declarations or what the method is called.

#[test]
fn guard_presentation_keeps_typed_owner_and_method_identity() {
    let root = tempfile::tempdir().expect("temp workspace");
    let text = compress_text(&root);
    assert!(
        text.contains("SemanticService"),
        "the model must still be told which type owns the declarations:\n{text}"
    );
    assert!(
        text.contains("run"),
        "the model must still be told the method identity:\n{text}"
    );
}
