// src/tests/mcp/envelope_contract.rs
//
// Dispatched-handler MCP envelope conformance (MCP-001).
//
// Every externally exposed Clean-CTX MCP tool that RETURNS a RESULT must
// conform to the canonical MCP `CallToolResult` envelope at the dispatched
// boundary: `content` (required, non-empty), optional `structuredContent` /
// `_meta`, and NO ad-hoc domain fields directly at the result level.
//
// This suite closes the remaining dispatched-handler coverage gap for tools
// that already emit canonical envelopes but previously had no wire-contract
// assertion at the dispatch boundary:
//     diff_code_context, context_history, context_stats, list_sessions,
//     diff_commits.
//
// compress_code_context / delta_code_context are deliberately EXCLUDED
// here: they are scheduled for the R-46 0.6.0 migration and still emit
// legacy ad-hoc result-level fields (incomplete Phase-3 metadata migration).
//
// Dispatch goes through the real registered path (dispatch_tools_call) and
// responses are observed via the cfg(test)-only `CAPTURED_RESPONSES` sink,
// serialized through `HANDLER_RESPONSE_SERIAL` — mirroring phase3_contract.rs.

use crate::mcp::tools::dispatch_tools_call;
use crate::tests::assert_valid_mcp_envelope;
use serde_json::json;

/// Pop the single response a handler must have emitted.
fn take_response() -> serde_json::Value {
    crate::protocol::captured_responses()
        .pop()
        .expect("handler must have sent exactly one response")
}

/// Dispatch one tool through the real registered path and return its response.
fn dispatch(tool: &str, args: serde_json::Value) -> serde_json::Value {
    let _serial = crate::protocol::handler_response_serial();
    let id = json!(1);
    crate::protocol::captured_responses().clear();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    dispatch_tools_call(&id, tool, &json!({ "arguments": args }), &state);
    take_response()
}

/// Assert a successful response carries a canonical MCP result envelope.
/// Returns the result object (`Map`) for further assertions.
fn assert_canonical_result(
    resp: &serde_json::Value,
    tool: &str,
) -> serde_json::Map<String, serde_json::Value> {
    assert!(
        resp.get("error").is_none(),
        "{tool} must not return a JSON-RPC error: {resp}"
    );
    let result = resp["result"]
        .as_object()
        .unwrap_or_else(|| panic!("{tool}: result must be an object: {resp}"))
        .clone();
    assert_valid_mcp_envelope(&result);
    result
}

// ── context_history ──────────────────────────────────────────────────
// No tracked files yet → readable content path. Must be a canonical envelope.

#[test]
fn context_history_emits_canonical_envelope() {
    let resp = dispatch("context_history", json!({}));
    let result = assert_canonical_result(&resp, "context_history");
    let text = result["content"][0]["text"]
        .as_str()
        .expect("context_history content[0].text");
    assert!(
        !text.trim().is_empty(),
        "context_history content must be readable"
    );
}

// ── context_stats ────────────────────────────────────────────────────
// Default dashboard render → canonical envelope with dashboard text.

#[test]
fn context_stats_emits_canonical_envelope() {
    let resp = dispatch("context_stats", json!({}));
    let result = assert_canonical_result(&resp, "context_stats");
    let text = result["content"][0]["text"]
        .as_str()
        .expect("context_stats content[0].text");
    assert!(
        !text.trim().is_empty(),
        "context_stats dashboard content must be readable"
    );
}

// ── list_sessions ────────────────────────────────────────────────────
// Persistence-disabled → human-readable notice. Still a canonical envelope.

#[test]
fn list_sessions_emits_canonical_envelope() {
    let resp = dispatch("list_sessions", json!({}));
    let result = assert_canonical_result(&resp, "list_sessions");
    let text = result["content"][0]["text"]
        .as_str()
        .expect("list_sessions content[0].text");
    assert!(
        text.contains("Persistence not enabled") || text.contains("Persisted contexts"),
        "list_sessions content must describe persistence state, got: {text}"
    );
}
// ── diff_code_context ────────────────────────────────────────────────
// A real compressible file in a temp workspace → canonical envelope.

#[test]
fn diff_code_context_emits_canonical_envelope() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let file = dir.path().join("svc.ts");
    std::fs::write(&file, "export class Svc { run(): void {} }\n").expect("write fixture");
    let args = json!({
        "filePath": file.to_string_lossy().into_owned(),
        "workspaceRoot": dir.path().to_string_lossy().into_owned(),
        "fidelity": "low",
    });
    let resp = dispatch("diff_code_context", args);
    let result = assert_canonical_result(&resp, "diff_code_context");
    let text = result["content"][0]["text"]
        .as_str()
        .expect("diff_code_context content[0].text");
    assert!(
        !text.trim().is_empty(),
        "diff_code_context content must be readable"
    );
}

// ── diff_commits ─────────────────────────────────────────────────────
// Real temp git repo (two commits) → canonical envelope with _meta.

#[test]
fn diff_commits_emits_canonical_envelope() {
    let dir = init_git_repo();
    let args = json!({
        "workspaceRoot": dir.path().to_string_lossy().into_owned(),
        "fromRef": "HEAD~1",
        "toRef": "HEAD",
    });
    let resp = dispatch("diff_commits", args);
    let result = assert_canonical_result(&resp, "diff_commits");
    let meta = result["_meta"]
        .as_object()
        .expect("diff_commits must carry _meta");
    assert!(
        meta.contains_key("fileCount"),
        "diff_commits _meta must carry fileCount, got: {meta:?}"
    );
}

/// Create a temp git repo with two commits (one modified file).
fn init_git_repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().to_str().expect("utf8 tempdir path");

    let git = |args: &[&str]| {
        let out = std::process::Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
            .output()
            .expect("git command");
        assert!(
            out.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    };

    git(&["init", "-q"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);

    // Commit 1
    std::fs::write(
        dir.path().join("app.ts"),
        "class UserService {\n  getUser(id: string): Promise<User> {\n    return api.get(id);\n  }\n}\n",
    )
    .expect("write app.ts");
    git(&["add", "."]);
    git(&["commit", "-q", "-m", "commit1"]);

    // Commit 2: add a method to app.ts
    std::fs::write(
        dir.path().join("app.ts"),
        "class UserService {\n  getUser(id: string): Promise<User> {\n    return api.get(id);\n  }\n  saveUser(u: User): void {\n    api.post(u);\n  }\n}\n",
    )
    .expect("write app.ts v2");
    git(&["add", "-A"]);
    git(&["commit", "-q", "-m", "commit2"]);

    dir
}
