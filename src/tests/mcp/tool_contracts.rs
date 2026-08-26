// src/tests/mcp/tool_contracts.rs
//
// Tool-contract pins from the Non-CBM Tool Audit 2026-08-25.
//
// These tests encode the INTENDED contract of tools whose name/description
// had drifted from their actual behavior, so the drift cannot silently
// return:
//   - `list_sessions` enumerates persisted CONTEXTS (the only real rows
//     in the persistence model — there is no session concept); the
//     description must not promise session objects (audit finding #6).
//
// (`delta_text_context` audit pin #5 was removed with the tool itself in
// the Phase B legacy-transport retirement.)

use crate::mcp::tools::tool_list;
use serde_json::Value;

fn description_of(tool_name: &str) -> String {
    tool_list()
        .into_iter()
        .find(|t| t["name"] == tool_name)
        .unwrap_or_else(|| panic!("{tool_name} should be registered in tool_list()"))["description"]
        .as_str()
        .unwrap_or_default()
        .to_string()
}

/// Audit #6: list_sessions lists persisted CONTEXTS (no session concept
/// exists in the persistence model).
#[test]
fn list_sessions_description_matches_persistence_model() {
    let desc = description_of("list_sessions");
    assert!(
        desc.contains("persisted contexts"),
        "description must describe context enumeration, got: {desc}"
    );
    assert!(
        !desc.to_lowercase().contains("sessions stored"),
        "description must not promise session objects, got: {desc}"
    );
}

/// Audit #6 behavior: with persistence enabled and at least one saved
/// context, `list_sessions` returns an enumeration containing that file —
/// never the old static "Persistence DB active." status line.
///
/// Fixture notes: the sample lives inside a tempdir workspace passed via
/// `workspaceRoot` (the XPIA boundary check requires the file to sit
/// under the trusted root), and it is `.ts` because `.rs` is only
/// compressible when the `rust` feature is enabled.
#[test]
fn list_sessions_enumerates_persisted_contexts() {
    use tempfile::TempDir;

    let tmp = TempDir::new().expect("tempdir");
    let ws = tmp.path().join("ws");
    std::fs::create_dir_all(&ws).expect("ws dir");
    let sample = ws.join("sample.ts");
    std::fs::write(
        &sample,
        "export class SampleService {\n  ping(name: string): string { return name; }\n}\n",
    )
    .expect("write sample");

    let mut config = crate::config::CleanCtxConfig::default();
    config.cbm.enabled = false;
    config.persistence.enabled = true;
    config.persistence.db_path = tmp
        .path()
        .join("list_sessions.db")
        .to_string_lossy()
        .to_string();
    let state = crate::mcp::McpState::new(config);

    let params = serde_json::json!({
        "arguments": {
            "filePath": "sample.ts",
            "workspaceRoot": ws.to_string_lossy(),
            "fidelity": "low"
        }
    });
    crate::mcp::tools::setup_handler_registry_for_tests();
    crate::mcp::tools::dispatch_tools_call(&Value::Null, "compress_code_context", &params, &state);
    state.flush_persistence();

    // The BufferedStore now holds the context; list_contexts must see it.
    let guard = state.persistence_store_lock();
    let store = guard.as_ref().expect("persistence enabled");
    let rows = store.list_contexts(100).expect("list_contexts");
    drop(guard);

    assert!(
        rows.iter().any(|r| r.file_path.contains("sample.ts")),
        "persisted sample.ts context should be enumerated, got: {rows:?}"
    );
}
