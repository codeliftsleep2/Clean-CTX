// src/tests/mcp/apply_edit.rs
//
// Black-box end-to-end tests for the `apply_edit` MCP tool
// (docs/plans/APPLY_EDIT_PLAN.md Phase 4).
//
// The write path performs disk I/O, verifies the tree-sitter syntax gate,
// and refreshes `ContextState` — so it is verified end-to-end rather than
// only exercised for panics. These tests spawn `target/debug/clean-ctx`
// and drive a JSON-RPC session (provide → apply_edit → provide-delta)
// over a single persistent stdin/stdout connection, mirroring how a real
// MCP client holds the session open. Modeled on `e2e_server.rs`; gated
// with `#[ignore]` (requires a prior `cargo build`).

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use serde_json::{Value, json};

use crate::mcp::tools::tool_list;

fn binary_path() -> String {
    let ext = if cfg!(windows) { ".exe" } else { "" };
    if cfg!(debug_assertions) {
        format!("target/debug/clean-ctx{ext}")
    } else {
        format!("target/release/clean-ctx{ext}")
    }
}

/// A live MCP server child process with persistent stdin/stdout.
struct Session {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl Session {
    fn spawn(dir: Option<&std::path::Path>) -> Self {
        let mut cmd = Command::new(binary_path());
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        if let Some(d) = dir {
            cmd.current_dir(d);
        }
        let mut child = cmd
            .spawn()
            .expect("Failed to spawn clean-ctx binary — run `cargo build` first");
        let stdin = child.stdin.take().expect("missing stdin");
        let stdout = BufReader::new(child.stdout.take().expect("missing stdout"));
        Session {
            child,
            stdin,
            stdout,
        }
    }

    /// Send one `tools/call` and read the JSON-RPC response (skipping any
    /// non-JSON log lines the server emits to stdout).
    fn call(&mut self, id: u64, tool: &str, args: Value) -> Value {
        let req = json!({
            "jsonrpc": "2.0", "id": id, "method": "tools/call",
            "params": { "name": tool, "arguments": args }
        });
        writeln!(self.stdin, "{}", req).unwrap();
        self.stdin.flush().unwrap();
        loop {
            let mut line = String::new();
            if self.stdout.read_line(&mut line).unwrap_or(0) == 0 {
                panic!("server closed stdout before responding");
            }
            if let Ok(v) = serde_json::from_str::<Value>(&line) {
                if v["jsonrpc"] == "2.0" {
                    return v;
                }
            }
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.child.wait();
    }
}

// ── Schema / surface tests (no spawn) ───────────────────────────────

/// The apply_edit tool must be advertised in the static tool list with the
/// required operation shape and the `verify` escape hatch.
#[test]
fn tool_list_advertises_apply_edit() {
    let tools = tool_list();
    let entry = tools
        .iter()
        .find(|t| t["name"] == "apply_edit")
        .expect("apply_edit must be listed");
    assert_eq!(entry["inputSchema"]["type"], "object");
    assert_eq!(entry["inputSchema"]["required"][0], "filePath");
    assert_eq!(entry["inputSchema"]["required"][1], "operations");
    assert_eq!(
        entry["inputSchema"]["properties"]["verify"]["type"],
        "boolean"
    );

    // outputSchema must describe structuredContent.operations.
    let os = entry
        .get("outputSchema")
        .expect("apply_edit must declare outputSchema");
    assert_eq!(os["type"], "object");
    assert_eq!(os["required"][0], "operations");
    let items = &os["properties"]["operations"]["items"];
    assert_eq!(items["type"], "object");
    assert_eq!(
        items["required"],
        json!(["kind", "target", "startByte", "endByte", "byteDelta"])
    );
    // newText is optional — must NOT appear in required.
    assert!(
        !items["required"]
            .as_array()
            .unwrap_or(&vec![])
            .contains(&json!("newText")),
        "newText must be optional"
    );
    // kind is a closed enum.
    assert_eq!(
        items["properties"]["kind"]["enum"],
        json!(["replace_body", "insert_after", "insert_before", "delete"])
    );
}

/// Regression: small Edit files must survive the token-economics gate with
/// tracked IR state so that apply_edit works after raw_passthrough.
#[test]
fn apply_edit_works_after_raw_passthrough_on_small_file() {
    let dir = tempfile::TempDir::new().unwrap();
    let root_str = dir.path().to_string_lossy().into_owned();
    let path = dir.path().join("tiny.ts");
    // 52 bytes -- well below the .ts Edit threshold of ~544 tokens.
    std::fs::write(&path, "class T {\n  m() {\n    return 1;\n  }\n}\n").unwrap();
    let mut config = crate::tests::test_config();
    config.additional_roots.push(root_str);
    let state = crate::mcp::McpState::new(config);
    let file_path = path.to_string_lossy().into_owned();

    // Step 1: provide_code_context at Edit fidelity.
    crate::mcp::tool_handlers::core::handle_provide_code_context(
        &json!(1),
        &json!({ "arguments": { "filePath": file_path.clone(), "fidelity": "edit" } }),
        &state,
    );

    // Step 2: apply_edit must succeed (IR state was established despite
    // raw_passthrough). Prior to the fix, this would fail with
    // "no prior tracked state".
    crate::mcp::tool_handlers::edit::handle_apply_edit(
        &json!(2),
        &json!({
            "arguments": {
                "filePath": file_path,
                "operations": [{
                    "type": "replace_body",
                    "target": "T.m",
                    "expectedOldText": "{\n    return 1;\n  }",
                    "newText": "{\n    return 2;\n  }"
                }]
            }
        }),
        &state,
    );
    let on_disk = std::fs::read_to_string(&path).unwrap();
    assert!(
        on_disk.contains("return 2;"),
        "edit after raw_passthrough must have written new body to disk:\n{on_disk}"
    );
}
/// apply_edit requires a prior provide and refuses to act on state-less
/// files (policy from Open Question 2). In-process (no spawn): the file on
/// disk must remain untouched.
#[test]
fn apply_edit_requires_prior_tracked_state() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("unseen.ts");
    std::fs::write(
        &path,
        "export class G {\n  run() {\n    return 1;\n  }\n}\n",
    )
    .unwrap();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = json!(1);
    let params = json!({
        "arguments": {
            "filePath": path.to_string_lossy().into_owned(),
            "operations": [{
                "type": "replace_body",
                "target": "G.run",
                "expectedOldText": "{\n    return 1;\n  }",
                "newText": "{\n    return 2;\n  }"
            }]
        }
    });
    crate::mcp::tool_handlers::edit::handle_apply_edit(&id, &params, &state);
    let on_disk = std::fs::read_to_string(&path).unwrap();
    assert!(on_disk.contains("return 1;"));
}

// ── End-to-end (requires `cargo build`) ─────────────────────────────

/// Fixture written into a temp dir for a real server session.
fn round_trip_fixture(dir: &std::path::Path) -> std::path::PathBuf {
    let path = dir.join("apply_edit_fixture.ts");
    std::fs::write(
        &path,
        "export class UserService {\n  processOrder(order: string) {\n    return order.trim();\n  }\n\n  count() {\n    return 1;\n  }\n}\n",
    )
    .unwrap();
    path
}

/// Full round trip: provide → apply_edit → provide(delta).
/// Asserts the write lands, the response is minimal, and the follow-up
/// provide_code_context reports a delta (state refreshed).
#[ignore]
#[test]
fn e2e_apply_edit_round_trip_then_delta() {
    let dir = tempfile::tempdir().unwrap();
    let path = round_trip_fixture(dir.path());
    let file_path = path.to_string_lossy().into_owned();

    let mut session = Session::spawn(Some(dir.path()));

    // 1) Establish state (byte-exact bodies at Edit fidelity).
    let prov = session.call(
        1,
        "provide_code_context",
        json!({ "filePath": file_path, "fidelity": "edit" }),
    );
    assert!(prov.get("error").is_none(), "provide failed: {prov:?}");

    // 2) apply_edit: replace processOrder body.
    let app = session.call(
        2,
        "apply_edit",
        json!({
            "filePath": file_path,
            "verify": true,
            "operations": [{
                "type": "replace_body",
                "target": "UserService.processOrder",
                "expectedOldText": "{\n    return order.trim();\n  }",
                "newText": "{\n    return order.trim().toLowerCase();\n  }"
            }]
        }),
    );
    assert!(app.get("error").is_none(), "apply_edit failed: {app:?}");
    let result = &app["result"];
    assert_eq!(result["_meta"]["applied"], 1);
    assert!(
        !result["_meta"]["fileHash"]
            .as_str()
            .unwrap_or("")
            .is_empty()
    );
    assert_eq!(result["_meta"]["syntaxGated"], true);
    // verify:true echoes the new body as a receipt.
    assert!(
        result["structuredContent"]["operations"][0]["newText"]
            .as_str()
            .unwrap_or("")
            .contains("toLowerCase")
    );

    // 3) On-disk bytes actually changed.
    let on_disk = std::fs::read_to_string(&path).unwrap();
    assert!(on_disk.contains("toLowerCase"));
    assert!(!on_disk.contains("return order.trim();"));

    // 4) Follow-up provide_code_context should now be a DELTA (baseline
    //    refreshed by apply_edit, not a full recompress).
    let prov2 = session.call(
        3,
        "provide_code_context",
        json!({ "filePath": file_path, "fidelity": "edit" }),
    );
    assert!(prov2.get("error").is_none(), "provide2 failed: {prov2:?}");
    assert_eq!(prov2["result"]["strategy"], "delta");
}

/// Adversarial: two edits to the SAME unit — the second is rejected with a
/// mismatch error (never silently overwritten).
#[ignore]
#[test]
fn e2e_apply_edit_same_unit_twice_rejects_second() {
    let dir = tempfile::tempdir().unwrap();
    let path = round_trip_fixture(dir.path());
    let file_path = path.to_string_lossy().into_owned();

    let mut session = Session::spawn(Some(dir.path()));
    session.call(
        1,
        "provide_code_context",
        json!({ "filePath": file_path, "fidelity": "edit" }),
    );
    let op = |new: &str| {
        json!({
            "type": "replace_body",
            "target": "UserService.processOrder",
            "expectedOldText": "{\n    return order.trim();\n  }",
            "newText": new
        })
    };
    // First edit succeeds.
    let a = session.call(
        2,
        "apply_edit",
        json!({ "filePath": file_path, "operations": [op("{\n    return order.trim().trim();\n  }")] }),
    );
    assert!(a.get("error").is_none(), "first apply failed: {a:?}");
    // Second edit to the SAME unit with the ORIGINAL expectation must be
    // rejected (its current text no longer matches), not overwritten.
    let b = session.call(
        3,
        "apply_edit",
        json!({ "filePath": file_path, "operations": [op("{\n    return order.trim().repeat(2);\n  }")] }),
    );
    let err = b.get("error").expect("second apply must be rejected");
    assert_eq!(err["code"], -32602);
    assert!(err["data"]["kind"].is_string());
    // File still holds the FIRST edit's bytes (repeat(2) never landed).
    let on_disk = std::fs::read_to_string(&path).unwrap();
    assert!(on_disk.contains("trim()"));
    assert!(!on_disk.contains("repeat(2)"));
}

/// Adversarial: two apply_edit calls targeting DIFFERENT units in the
/// same file both succeed without serializing — the narrow-guarantee win
/// over the client host's whole-file staleness precondition.
#[ignore]
#[test]
fn e2e_apply_edit_different_units_both_succeed() {
    let dir = tempfile::tempdir().unwrap();
    let path = round_trip_fixture(dir.path());
    let file_path = path.to_string_lossy().into_owned();

    let mut session = Session::spawn(Some(dir.path()));
    session.call(
        1,
        "provide_code_context",
        json!({ "filePath": file_path, "fidelity": "edit" }),
    );

    // processOrder → toUpperCase().
    let a = session.call(
        2,
        "apply_edit",
        json!({ "filePath": file_path, "operations": [{
            "type": "replace_body",
            "target": "UserService.processOrder",
            "expectedOldText": "{\n    return order.trim();\n  }",
            "newText": "{\n    return order.trim().toUpperCase();\n  }"
        }] }),
    );
    assert!(a.get("error").is_none(), "first apply failed: {a:?}");

    // count → return 42 (different unit, same file) — must also succeed.
    let b = session.call(
        3,
        "apply_edit",
        json!({ "filePath": file_path, "operations": [{
            "type": "replace_body",
            "target": "UserService.count",
            "expectedOldText": "{\n    return 1;\n  }",
            "newText": "{\n    return 42;\n  }"
        }] }),
    );
    assert!(b.get("error").is_none(), "second apply failed: {b:?}");

    let on_disk = std::fs::read_to_string(&path).unwrap();
    assert!(on_disk.contains("toUpperCase"));
    assert!(on_disk.contains("return 42;"));
}

// ── Phase 2 structured-output contract tests ───────────────────────

/// Constructed-JSON contract: the success envelope must carry operations in
/// structuredContent, metadata in _meta, and no ad-hoc result-level fields.
#[test]
fn apply_edit_success_response_has_correct_mcp_shape() {
    use crate::tests::{assert_structured_content_has, assert_valid_mcp_envelope};

    let response = serde_json::json!({
        "jsonrpc": "2.0", "id": 1,
        "result": {
            "content": [{ "type": "text", "text": "applied 1 operation(s) to /path/file.ts (v3)" }],
            "structuredContent": {
                "operations": [{
                    "kind": "replace_body",
                    "target": "UserService.processOrder",
                    "startByte": 41,
                    "endByte": 77,
                    "byteDelta": 12,
                    "newText": "{ return 1; }"
                }]
            },
            "_meta": {
                "filePath": "/path/file.ts",
                "fileHash": "e414e1bd",
                "version": 3,
                "applied": 1,
                "syntaxGated": true
            }
        }
    });

    let result = response["result"].as_object().expect("result object");
    assert_valid_mcp_envelope(result);

    let sc = result["structuredContent"]
        .as_object()
        .expect("structuredContent object");
    assert_structured_content_has(sc, &["operations"]);

    let op = &sc["operations"][0];
    assert_eq!(op["kind"], "replace_body");
    assert_eq!(op["target"], "UserService.processOrder");
    assert_eq!(op["startByte"], 41);
    assert_eq!(op["endByte"], 77);
    assert_eq!(op["byteDelta"], 12);

    let meta = result["_meta"].as_object().expect("_meta object");
    assert!(meta.contains_key("filePath"));
    assert!(meta.contains_key("fileHash"));
    assert!(meta.contains_key("version"));
    assert!(meta.contains_key("applied"));
    assert!(meta.contains_key("syntaxGated"));

    // No ad-hoc result-level fields.
    assert!(
        !result.contains_key("operations"),
        "result.operations must not exist"
    );
    assert!(
        !result.contains_key("filePath"),
        "result.filePath must not exist"
    );
    assert!(
        !result.contains_key("fileHash"),
        "result.fileHash must not exist"
    );
    assert!(
        !result.contains_key("version"),
        "result.version must not exist"
    );
    assert!(
        !result.contains_key("applied"),
        "result.applied must not exist"
    );
    assert!(
        !result.contains_key("syntaxGated"),
        "result.syntaxGated must not exist"
    );

    let text = result["content"][0]["text"].as_str().expect("content text");
    assert!(
        text.contains("applied"),
        "content should describe the operation"
    );
}

/// Live in-process dispatch: the real handler must emit the canonical envelope
/// (content + structuredContent.operations + _meta, nothing ad-hoc).
#[cfg(all(test, feature = "rust"))]
#[test]
fn apply_edit_live_dispatch_emits_canonical_envelope() {
    use crate::tests::{assert_structured_content_has, assert_valid_mcp_envelope};

    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("svc.ts");
    std::fs::write(
        &path,
        "export class UserService {\n  processOrder(order: string) {\n    return order.trim();\n  }\n}\n",
    )
    .unwrap();

    let mut config = crate::tests::test_config();
    let root_str = dir.path().to_string_lossy().into_owned();
    config.additional_roots.push(root_str);
    let state = crate::mcp::McpState::new(config);
    let file_path = path.to_string_lossy().into_owned();

    // Establish tracked state.
    crate::mcp::tool_handlers::core::handle_provide_code_context(
        &json!(1),
        &json!({ "arguments": { "filePath": file_path.clone(), "fidelity": "edit" } }),
        &state,
    );

    // Serialize access to the shared CAPTURED_RESPONSES sink.
    let _serial = crate::protocol::handler_response_serial();
    crate::protocol::captured_responses().clear();

    crate::mcp::tool_handlers::edit::handle_apply_edit(
        &json!(2),
        &json!({
            "arguments": {
                "filePath": file_path,
                "verify": true,
                "operations": [{
                    "type": "replace_body",
                    "target": "UserService.processOrder",
                    "expectedOldText": "{\n    return order.trim();\n  }",
                    "newText": "{\n    return order.trim().toLowerCase();\n  }"
                }]
            }
        }),
        &state,
    );

    let response = crate::protocol::captured_responses()
        .pop()
        .expect("handler must have sent exactly one response");
    let result = response["result"].as_object().expect("result object");

    assert_valid_mcp_envelope(result);

    let sc = result["structuredContent"]
        .as_object()
        .expect("structuredContent object");
    assert_structured_content_has(sc, &["operations"]);
    assert_eq!(sc["operations"].as_array().unwrap().len(), 1);

    let op = &sc["operations"][0];
    assert_eq!(op["kind"], "replace_body");
    assert_eq!(op["target"], "UserService.processOrder");
    assert!(op["startByte"].is_number());
    assert!(op["endByte"].is_number());
    assert!(op["byteDelta"].is_number());
    // verify:true → newText present.
    assert!(op["newText"].as_str().unwrap_or("").contains("toLowerCase"));

    let meta = result["_meta"].as_object().expect("_meta object");
    assert!(meta.contains_key("filePath"));
    assert!(meta.contains_key("fileHash"));
    assert!(meta.contains_key("version"));
    assert_eq!(meta["applied"], 1);
    assert_eq!(meta["syntaxGated"], true);

    // No ad-hoc result-level fields.
    assert!(!result.contains_key("operations"));
    assert!(!result.contains_key("filePath"));
    assert!(!result.contains_key("fileHash"));
    assert!(!result.contains_key("version"));
    assert!(!result.contains_key("applied"));
    assert!(!result.contains_key("syntaxGated"));
}

/// Error-path regression: the existing JSON-RPC error shape must survive the
/// migration unchanged (-32602 + structured error.data.kind).
#[cfg(all(test, feature = "rust"))]
#[test]
fn apply_edit_error_path_unchanged_after_migration() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("err.ts");
    std::fs::write(
        &path,
        "export class G {\n  run() {\n    return 1;\n  }\n}\n",
    )
    .unwrap();

    let mut config = crate::tests::test_config();
    let root_str = dir.path().to_string_lossy().into_owned();
    config.additional_roots.push(root_str);
    let state = crate::mcp::McpState::new(config);
    let file_path = path.to_string_lossy().into_owned();

    // Establish tracked state.
    crate::mcp::tool_handlers::core::handle_provide_code_context(
        &json!(1),
        &json!({ "arguments": { "filePath": file_path.clone(), "fidelity": "edit" } }),
        &state,
    );

    let _serial = crate::protocol::handler_response_serial();
    crate::protocol::captured_responses().clear();

    // Stale expectedOldText → unit_mismatch rejection.
    crate::mcp::tool_handlers::edit::handle_apply_edit(
        &json!(2),
        &json!({
            "arguments": {
                "filePath": file_path,
                "operations": [{
                    "type": "replace_body",
                    "target": "G.run",
                    "expectedOldText": "{ wrong text }",
                    "newText": "{ return 2; }"
                }]
            }
        }),
        &state,
    );

    let response = crate::protocol::captured_responses()
        .pop()
        .expect("handler must have sent exactly one response");

    // JSON-RPC error (NOT result.isError).
    assert!(
        response.get("error").is_some(),
        "error path must produce JSON-RPC error, got: {response}"
    );
    let err = &response["error"];
    assert_eq!(err["code"], -32602);
    assert!(err["data"]["kind"].is_string());
    assert_eq!(err["data"]["kind"], "unit_mismatch");
    // No result envelope at all on error.
    assert!(response.get("result").is_none());
}

/// newText must be absent when verify=false (receipt not requested).
#[cfg(all(test, feature = "rust"))]
#[test]
fn apply_edit_newtext_absent_when_verify_false() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("novf.ts");
    std::fs::write(
        &path,
        "export class UserService {\n  processOrder(order: string) {\n    return order.trim();\n  }\n}\n",
    )
    .unwrap();

    let mut config = crate::tests::test_config();
    let root_str = dir.path().to_string_lossy().into_owned();
    config.additional_roots.push(root_str);
    let state = crate::mcp::McpState::new(config);
    let file_path = path.to_string_lossy().into_owned();

    crate::mcp::tool_handlers::core::handle_provide_code_context(
        &json!(1),
        &json!({ "arguments": { "filePath": file_path.clone(), "fidelity": "edit" } }),
        &state,
    );

    let _serial = crate::protocol::handler_response_serial();
    crate::protocol::captured_responses().clear();

    crate::mcp::tool_handlers::edit::handle_apply_edit(
        &json!(2),
        &json!({
            "arguments": {
                "filePath": file_path,
                "verify": false,
                "operations": [{
                    "type": "replace_body",
                    "target": "UserService.processOrder",
                    "expectedOldText": "{\n    return order.trim();\n  }",
                    "newText": "{\n    return order.trim().toLowerCase();\n  }"
                }]
            }
        }),
        &state,
    );

    let response = crate::protocol::captured_responses()
        .pop()
        .expect("handler must have sent exactly one response");
    let result = response["result"].as_object().expect("result object");
    let sc = result["structuredContent"]
        .as_object()
        .expect("structuredContent object");
    let op = &sc["operations"][0];

    // newText must NOT be present when verify=false.
    assert!(
        op.get("newText").is_none(),
        "newText must be absent when verify=false, got: {op}"
    );
    // The operation record is still well-formed.
    assert_eq!(op["kind"], "replace_body");
    assert!(op["byteDelta"].is_number());
}
