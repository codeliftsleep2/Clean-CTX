// src/tests/mcp/phase3_contract.rs
//
// Phase 3 — MCP structured-output clean-up (2026-08-28).
//
// Contract: every response now follows the canonical MCP result envelope:
//
//   result
//   ├── content            ← human/LLM-readable summary (always present)
//   ├── structuredContent  ← primary machine-readable payload (where present)
//   ├── isError            ← MCP-standard error indicator (where appropriate)
//   └── _meta              ← response/telemetry/state metadata
//
// This suite pins the migrated tools to that envelope and proves the moved
// ad-hoc result-level fields now live under `_meta`, that `content` remains
// meaningful, and that the existing JSON-RPC error architecture is untouched.
//
// Phase 3 deliberately introduces NO `structuredContent` and NO `outputSchema`
// for these tools (the audit found no genuine machine-primary payload or
// consumer requiring them). Their migrations are all `content` + `_meta`.
//
// Sink/serial follow the phase_a/phase_b convention: handlers write to stdout,
// so responses are observed via cfg(test)-only `protocol::CAPTURED_RESPONSES`,
// serialized through `protocol::HANDLER_RESPONSE_SERIAL`.

use crate::mcp::tools::dispatch_tools_call;
use serde_json::json;

// Shared Phase 2-promoted envelope helper (crate::tests).
use crate::tests::assert_valid_mcp_envelope;

fn phase3_take_response() -> serde_json::Value {
    let resp = crate::protocol::captured_responses().pop();
    resp.expect("handler must have sent exactly one response")
}

fn phase3_dispatch(tool: &str, args: serde_json::Value) -> serde_json::Value {
    let _serial = crate::protocol::handler_response_serial();
    let id = json!(1);
    crate::protocol::captured_responses().clear();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    dispatch_tools_call(&id, tool, &json!({ "arguments": args }), &state);
    phase3_take_response()
}

/// Extract `content[0].text` from a result object (`Map`), since `Map` has no
/// `.pointer()` (that is a `Value` method).
fn result_content_text(result: &serde_json::Map<String, serde_json::Value>) -> Option<&str> {
    result
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|a| a.first())
        .and_then(|t| t.get("text"))
        .and_then(|t| t.as_str())
}

// ══════════════════════════════════════════════════════════════════════
// save_context — gains `content` (previously message-only) and moves
// ok/saved into `_meta`.
// ══════════════════════════════════════════════════════════════════════

#[test]
fn save_context_has_canonical_envelope_with_content_and_meta() {
    let resp = phase3_dispatch("save_context", json!({ "filePath": "/nonexistent.ts" }));
    assert!(
        resp.get("error").is_none(),
        "save_context must not error: {resp}"
    );

    let result = resp
        .get("result")
        .and_then(|r| r.as_object())
        .expect("result must be an object");
    assert_valid_mcp_envelope(result);

    let text =
        result_content_text(result).unwrap_or_else(|| panic!("missing content[0].text: {resp}"));
    assert!(
        text.contains("Saved") && text.contains("context(s)"),
        "content must summarize the save: {text}"
    );

    let meta = result
        .get("_meta")
        .and_then(|m| m.as_object())
        .unwrap_or_else(|| panic!("missing _meta: {resp}"));
    assert_eq!(meta.get("ok").and_then(|o| o.as_bool()), Some(true));
    // Persistence is disabled in test_config, so exactly 0 contexts are saved;
    // the invariant is that `saved` lives in `_meta` as a number, not its value.
    assert!(
        meta.get("saved").and_then(|s| s.as_i64()).is_some(),
        "_meta.saved must be a number: {resp}"
    );
}

// ══════════════════════════════════════════════════════════════════════
// purge_old_deltas — persistence-disabled path must remain a JSON-RPC error.
// ══════════════════════════════════════════════════════════════════════

#[test]
fn purge_old_deltas_persistence_disabled_keeps_jsonrpc_error() {
    let resp = phase3_dispatch("purge_old_deltas", json!({ "days": 30 }));
    let err = resp
        .get("error")
        .and_then(|e| e.as_object())
        .unwrap_or_else(|| panic!("persistence-disabled purge must error, got: {resp}"));
    assert_eq!(err.get("code").and_then(|c| c.as_i64()), Some(-32603));
}

// ══════════════════════════════════════════════════════════════════════
// replay_history — missing filePath stays a JSON-RPC error.
// ══════════════════════════════════════════════════════════════════════

#[test]
fn replay_history_missing_file_keeps_jsonrpc_error() {
    let resp = phase3_dispatch("replay_history", json!({}));
    let err = resp
        .get("error")
        .and_then(|e| e.as_object())
        .expect("replay without filePath must error");
    assert_eq!(err.get("code").and_then(|c| c.as_i64()), Some(-32602));
}

// ══════════════════════════════════════════════════════════════════════
// get_cbm_status — cbm_status/graph_version (+ conditional indexing,
// freshness, checked_paths) move to `_meta`. Content stays the report.
// (CBM disabled → unavailable branch, which also carries checked_paths.)
// ══════════════════════════════════════════════════════════════════════

#[test]
fn get_cbm_status_moves_status_fields_into_meta() {
    let resp = phase3_dispatch("get_cbm_status", json!({}));
    assert!(
        resp.get("error").is_none(),
        "get_cbm_status must not error: {resp}"
    );

    let result = resp
        .get("result")
        .and_then(|r| r.as_object())
        .expect("result must be an object");
    assert_valid_mcp_envelope(result);

    for banned in [
        "cbm_status",
        "graph_version",
        "indexing",
        "freshness",
        "checked_paths",
    ] {
        assert!(
            !result.contains_key(banned),
            "result must not carry ad-hoc field '{banned}': {resp}"
        );
    }

    let meta = result
        .get("_meta")
        .and_then(|m| m.as_object())
        .unwrap_or_else(|| panic!("missing _meta: {resp}"));
    assert!(
        meta.contains_key("cbm_status"),
        "_meta must carry cbm_status: {resp}"
    );
    assert!(
        meta.contains_key("graph_version"),
        "_meta must carry graph_version: {resp}"
    );

    let text = result_content_text(result).unwrap();
    assert!(
        !text.is_empty(),
        "content must be the human-readable status report"
    );
}

// ══════════════════════════════════════════════════════════════════════
// provide_code_context — all seven success sites now use `_meta` for the
// metadata. This exercises a real success path (full compression) against a
// fixture and asserts envelope + _meta placement + meaningful content.
// ══════════════════════════════════════════════════════════════════════

const TS_FIXTURE: &str = "export class Greeter {\n    private prefix: string;\n    constructor(prefix: string) {\n        this.prefix = prefix;\n    }\n    greet(name: string): string {\n        return this.prefix + ', ' + name;\n    }\n}\n";

struct P3Fixture {
    _dir: tempfile::TempDir,
    path: String,
    root: String,
}

fn p3_temp_fixture() -> P3Fixture {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let file = dir.path().join("greeter.ts");
    std::fs::write(&file, TS_FIXTURE).expect("write fixture");
    let root = dir.path().to_string_lossy().to_string();
    let path = file.to_string_lossy().to_string();
    P3Fixture {
        _dir: dir,
        path,
        root,
    }
}

#[test]
fn provide_code_context_uses_meta_not_ad_hoc_fields() {
    let fx = p3_temp_fixture();
    let _serial = crate::protocol::handler_response_serial();
    let id = json!(1);
    crate::protocol::captured_responses().clear();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    dispatch_tools_call(
        &id,
        "provide_code_context",
        &json!({ "arguments": { "filePath": fx.path, "fidelity": "medium", "workspaceRoot": fx.root } }),
        &state,
    );
    let resp = phase3_take_response();
    assert!(
        resp.get("error").is_none(),
        "provide must not error: {resp}"
    );

    let result = resp
        .get("result")
        .and_then(|r| r.as_object())
        .expect("result must be an object");
    assert_valid_mcp_envelope(result);

    let text = result_content_text(result).unwrap();

    // Token-economics gate may select raw_passthrough when the
    // compressed representation costs more tokens than the raw
    // source (tiny files at structural fidelities). Both outcomes
    // are valid — SCHEMA v2 when compression is economical,
    // raw_passthrough when it is not.
    let content_kind = result
        .get("_meta")
        .and_then(|m| m.get("content_kind"))
        .and_then(|k| k.as_str());

    if content_kind == Some("raw_passthrough") {
        // raw source returned verbatim — no SCHEMA v2 expected.
        // Verbatim document means the raw fixture content.
        assert!(
            text.contains("class Greeter"),
            "raw_passthrough must contain the class: {text}"
        );
    } else {
        assert!(
            text.contains("// SCHEMA v2"),
            "content must be SCHEMA v2: {text}"
        );
    }

    for banned in [
        "strategy",
        "fidelity",
        "decision_summary",
        "content_kind",
        "byte_exact",
        "degradation",
        "is_angular",
        "version",
        "verbatim",
    ] {
        assert!(
            !result.contains_key(banned),
            "result must not carry ad-hoc field '{banned}': {resp}"
        );
    }

    let meta = result
        .get("_meta")
        .and_then(|m| m.as_object())
        .unwrap_or_else(|| panic!("missing _meta: {resp}"));
    for key in [
        "strategy",
        "fidelity",
        "decision_summary",
        "content_kind",
        "byte_exact",
        "degradation",
    ] {
        assert!(meta.contains_key(key), "_meta must carry '{key}': {resp}");
    }
}

// ══════════════════════════════════════════════════════════════════════
// restore_context — version/restored move to _meta; content preserved.
// ══════════════════════════════════════════════════════════════════════

#[test]
fn restore_context_moves_version_and_restored_to_meta() {
    let fx = p3_temp_fixture();
    let _serial = crate::protocol::handler_response_serial();
    let id = json!(1);
    crate::protocol::captured_responses().clear();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    dispatch_tools_call(
        &id,
        "restore_context",
        &json!({ "arguments": { "filePath": fx.path, "fidelity": "medium", "workspaceRoot": fx.root } }),
        &state,
    );
    let resp = phase3_take_response();
    assert!(
        resp.get("error").is_none(),
        "restore must not error: {resp}"
    );

    let result = resp
        .get("result")
        .and_then(|r| r.as_object())
        .expect("result must be an object");
    assert_valid_mcp_envelope(result);

    for banned in ["version", "restored"] {
        assert!(
            !result.contains_key(banned),
            "result must not carry '{banned}': {resp}"
        );
    }
    let meta = result.get("_meta").and_then(|m| m.as_object()).unwrap();
    assert!(meta.contains_key("version"));
    assert_eq!(meta.get("restored").and_then(|r| r.as_bool()), Some(true));

    let text = result_content_text(result).unwrap();
    assert!(
        text.contains("// SCHEMA v2"),
        "restore content must be SCHEMA v2: {text}"
    );
}

// ══════════════════════════════════════════════════════════════════════
// apply_delta — version moves to _meta; error paths unchanged.
// ══════════════════════════════════════════════════════════════════════

#[test]
fn apply_delta_version_mismatch_keeps_jsonrpc_error() {
    let resp = phase3_dispatch(
        "apply_delta",
        json!({
            "delta": { "file": "α1", "from": 1, "to": 2, "ops": { "+": [], "~": [], "-": [] } },
            "currentVersion": 99
        }),
    );
    let err = resp
        .get("error")
        .and_then(|e| e.as_object())
        .unwrap_or_else(|| panic!("version mismatch must error, got: {resp}"));
    assert_eq!(err.get("code").and_then(|c| c.as_i64()), Some(-32602));
}

#[test]
fn apply_delta_success_moves_version_to_meta() {
    let fx = p3_temp_fixture();
    let _serial = crate::protocol::handler_response_serial();
    let id = json!(1);
    let state = crate::mcp::McpState::new(crate::tests::test_config());

    // Establish a baseline via compress_code_context (version 1), then apply
    // an empty-ops delta to bump to version 2 (deterministic).
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &id,
        "compress_code_context",
        &json!({ "arguments": { "filePath": fx.path, "fidelity": "medium", "workspaceRoot": fx.root } }),
        &state,
    );
    let compressed = phase3_take_response();
    assert!(
        compressed.get("error").is_none(),
        "baseline failed: {compressed}"
    );
    let file_id = compressed
        .pointer("/result/file")
        .and_then(|f| f.as_str())
        .expect("compress must expose file id")
        .to_string();

    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &id,
        "apply_delta",
        &json!({
            "arguments": {
                "delta": { "file": file_id, "from": 1, "to": 2, "ops": { "+": [], "~": [], "-": [] } },
                "currentVersion": 1,
            }
        }),
        &state,
    );
    let resp = phase3_take_response();
    assert!(
        resp.get("error").is_none(),
        "apply_delta must succeed: {resp}"
    );

    let result = resp
        .get("result")
        .and_then(|r| r.as_object())
        .expect("result must be an object");
    assert_valid_mcp_envelope(result);
    assert!(
        !result.contains_key("version"),
        "result must not carry 'version': {resp}"
    );
    let meta = result.get("_meta").and_then(|m| m.as_object()).unwrap();
    assert_eq!(meta.get("version").and_then(|v| v.as_i64()), Some(2));

    let text = result_content_text(result).unwrap();
    assert!(!text.is_empty(), "apply_delta content must be non-empty");
}
