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
    assert_eq!(result["applied"], 1);
    assert!(!result["fileHash"].as_str().unwrap_or("").is_empty());
    assert_eq!(result["syntaxGated"], true);
    // verify:true echoes the new body as a receipt.
    assert!(
        result["operations"][0]["newText"]
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
