// src/tests/mcp/phase_a_retirement.rs
//
// Phase A — legacy `$`/`⊕`/`§` retirement (2026-08-25).
//
// Contract: when PRIMARY IR compilation fails, the three interactive
// tools (`compress_code_context`, `provide_code_context`,
// `restore_context`) return a STRUCTURED `ir_unavailable` error instead
// of silently degrading to legacy text-compressor notation.
//
// Natural `CompileError` paths are unreachable with valid grammars, so
// failures are driven via the cfg(test)-only TEST_INJECTED_IR_FAILURE
// hook; payloads are observed via cfg(test)-only
// protocol::CAPTURED_RESPONSES (handlers write to stdout).

use crate::mcp::tools::dispatch_tools_call;
use serde_json::json;
use std::sync::{Mutex, MutexGuard};

/// Serializes every test here so the global injection flag can never
/// leak into a concurrently running sibling test.
static PHASE_A_SERIAL: Mutex<()> = Mutex::new(());

fn phase_a_serial() -> MutexGuard<'static, ()> {
    // Poison-tolerant: a sibling panic must not cascade into this suite.
    match PHASE_A_SERIAL.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    }
}

const TS_FIXTURE: &str = "export class Greeter {\n    private prefix: string;\n    constructor(prefix: string) {\n        this.prefix = prefix;\n    }\n    greet(name: string): string {\n        return this.prefix + ', ' + name;\n    }\n}\n";

struct PhaseAFixture {
    _dir: tempfile::TempDir,
    /// Absolute path of the .ts fixture on disk.
    path: String,
    /// Workspace root the handler is allowed to resolve against.
    root: String,
}

fn phase_a_temp_fixture() -> PhaseAFixture {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let file = dir.path().join("greeter.ts");
    std::fs::write(&file, TS_FIXTURE).expect("write fixture");
    let root = dir.path().to_string_lossy().to_string();
    let path = file.to_string_lossy().to_string();
    PhaseAFixture {
        _dir: dir,
        path,
        root,
    }
}

fn phase_a_set_injection(reason: Option<&str>) {
    let mut slot = crate::mcp::tool_helpers::TEST_INJECTED_IR_FAILURE
        .lock()
        .unwrap();
    *slot = reason.map(str::to_string);
}

fn phase_a_take_response() -> serde_json::Value {
    crate::protocol::CAPTURED_RESPONSES
        .lock()
        .unwrap()
        .pop()
        .expect("handler must have sent exactly one response")
}

#[test]
fn phase_a_fallbacks_return_structured_ir_unavailable_not_legacy_text() {
    let _serial = phase_a_serial();
    let fx = phase_a_temp_fixture();
    let id = json!(77);

    let mk = |root: &str, p: &str| json!({ "arguments": { "filePath": p, "fidelity": "medium", "workspaceRoot": root } });
    let cases: [(&str, serde_json::Value); 3] = [
        ("compress_code_context", mk(&fx.root, &fx.path)),
        ("provide_code_context", mk(&fx.root, &fx.path)),
        ("restore_context", mk(&fx.root, &fx.path)),
    ];

    for (tool, params) in cases {
        let state = crate::mcp::McpState::new(crate::tests::test_config());
        phase_a_set_injection(Some("unit-test"));
        crate::protocol::CAPTURED_RESPONSES.lock().unwrap().clear();

        dispatch_tools_call(&id, tool, &params, &state);
        let resp = phase_a_take_response();
        phase_a_set_injection(None);

        // Structured JSON-RPC error — no result payload at all.
        let err = resp
            .get("error")
            .and_then(|e| e.as_object())
            .unwrap_or_else(|| panic!("[{tool}] expected structured error response, got: {resp}"));
        assert_eq!(
            err.get("code").and_then(|c| c.as_i64()),
            Some(-32603),
            "[{tool}] wrong error code"
        );
        let message = err.get("message").and_then(|m| m.as_str()).unwrap_or("");
        assert!(
            message.contains("IR compilation unavailable"),
            "[{tool}] message must name ir_unavailable: {message}"
        );
        assert!(
            message.contains(fx.path.as_str()),
            "[{tool}] message must identify the file: {message}"
        );
        assert_eq!(
            err.get("data")
                .and_then(|d| d.get("reason"))
                .and_then(|r| r.as_str()),
            Some("ir_unavailable"),
            "[{tool}] machine-readable reason missing"
        );
        assert!(
            resp.get("result").is_none(),
            "[{tool}] must not carry a result alongside the error"
        );

        // Zero retired vocabulary anywhere in the wire payload.
        let wire = resp.to_string();
        for banned in ["Compacted Layout", "// SCHEMA v2", "$c ", "⊕guard"] {
            assert!(
                !wire.contains(banned),
                "[{tool}] retired notation `{banned}` leaked: {wire}"
            );
        }
    }
}

#[test]
fn phase_a_success_paths_still_render_schema_v2() {
    let _serial = phase_a_serial();
    let fx = phase_a_temp_fixture();
    let id = json!(78);

    let mk = |root: &str, p: &str| json!({ "arguments": { "filePath": p, "fidelity": "medium", "workspaceRoot": root } });
    let cases: [(&str, serde_json::Value); 3] = [
        ("compress_code_context", mk(&fx.root, &fx.path)),
        ("provide_code_context", mk(&fx.root, &fx.path)),
        ("restore_context", mk(&fx.root, &fx.path)),
    ];

    for (tool, params) in cases {
        let state = crate::mcp::McpState::new(crate::tests::test_config());
        phase_a_set_injection(None);
        crate::protocol::CAPTURED_RESPONSES.lock().unwrap().clear();

        dispatch_tools_call(&id, tool, &params, &state);
        let resp = phase_a_take_response();

        assert!(
            resp.get("error").is_none(),
            "[{tool}] healthy compilation must not error: {resp}"
        );
        let text = resp
            .pointer("/result/content/0/text")
            .and_then(|t| t.as_str())
            .unwrap_or_else(|| panic!("[{tool}] missing result.content[0].text: {resp}"));
        assert!(
            text.contains("// SCHEMA v2"),
            "[{tool}] successful output must be SCHEMA v2"
        );
        assert!(
            text.contains("Greeter"),
            "[{tool}] successful output must contain the compiled class"
        );
        assert!(
            !text.contains("Compacted Layout"),
            "[{tool}] successful output must never be legacy text"
        );
    }
}
