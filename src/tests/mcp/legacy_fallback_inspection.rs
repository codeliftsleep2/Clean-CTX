use std::path::Path;

use crate::mcp::buffered_store::BufferedStore;
use crate::mcp::sqlite_store::SqliteStore;

#[test]
fn registered_inspection_reports_quarantine_without_mutation() {
    let root = tempfile::tempdir().expect("temporary quarantine");
    let fallback = root.path().join(".clean-ctx").join("fallback");
    std::fs::create_dir_all(&fallback).expect("fallback directory");
    let fixtures = [
        (
            "01-save.json",
            r#"{"type":"save_context","file_path":"/legacy.ts","source_hash":"old"}"#,
        ),
        (
            "02-delta.json",
            r#"{"type":"append_delta","context_id":"ctx-old","delta_payload":"AA=="}"#,
        ),
        (
            "03-clear.json",
            r#"{"type":"clear_file","file_path":"/legacy.ts"}"#,
        ),
        ("04-malformed.json", "{not-json"),
    ];
    for (name, body) in fixtures {
        std::fs::write(fallback.join(name), body).expect("legacy fixture");
    }
    let before = std::fs::read_dir(&fallback)
        .expect("fixture listing")
        .map(|entry| entry.expect("fixture entry").path())
        .collect::<Vec<_>>();

    let mut config = crate::tests::test_config();
    config.persistence.enabled = true;
    config.persistence.db_path = root.path().join("contexts.db").display().to_string();
    let state = crate::mcp::McpState::new(config);
    *state.persistence_store_lock() = Some(BufferedStore::new(
        SqliteStore::open(Path::new(":memory:")).expect("in-memory SQLite"),
        root.path().to_path_buf(),
    ));

    let artifacts = state
        .persistence_store_lock()
        .as_ref()
        .expect("inspection store")
        .inspect_legacy_fallbacks();
    assert_eq!(artifacts.len(), 4);
    assert_eq!(artifacts[0].operation.as_deref(), Some("save_context"));
    assert_eq!(artifacts[0].identity.as_deref(), Some("/legacy.ts"));
    assert_eq!(artifacts[1].operation.as_deref(), Some("append_delta"));
    assert_eq!(artifacts[1].identity.as_deref(), Some("ctx-old"));
    assert_eq!(artifacts[2].operation.as_deref(), Some("clear_file"));
    assert!(artifacts[3].operation.is_none());
    assert_eq!(artifacts[3].available_metadata, serde_json::Value::Null);
    assert!(artifacts.iter().all(|artifact| {
        artifact
            .reason
            .contains("lacks complete aligned semantic authority")
    }));
    let result = super::legacy_fallback_inspection_result(artifacts);
    let rows = result["structuredContent"]["artifacts"]
        .as_array()
        .expect("structured artifact rows");
    assert_eq!(result["structuredContent"]["count"], 4);
    assert_eq!(rows[0]["operation"], "save_context");
    assert_eq!(rows[1]["operation"], "append_delta");
    assert_eq!(rows[2]["operation"], "clear_file");
    assert!(rows[3]["operation"].is_null());
    assert!(rows.iter().all(|row| row["recoverable"] == false));
    assert!(rows.iter().all(|row| {
        row["reason"]
            .as_str()
            .is_some_and(|reason| reason.contains("lacks complete aligned semantic authority"))
    }));

    for id in [27, 28] {
        crate::mcp::tools::dispatch_tools_call(
            &serde_json::json!(id),
            "inspect_legacy_fallbacks",
            &serde_json::json!({ "arguments": {} }),
            &state,
        );
    }

    let after = std::fs::read_dir(&fallback)
        .expect("fixture listing after inspection")
        .map(|entry| entry.expect("fixture entry").path())
        .collect::<Vec<_>>();
    assert_eq!(before, after);
    assert!(
        state
            .persistence_store_lock()
            .as_ref()
            .is_some_and(|store| { store.list_contexts(10).is_ok_and(|rows| rows.is_empty()) })
    );
    assert!(state.ir_context_read().file_ids().is_empty());
    assert_eq!(state.workspace_index_read().edge_count(), 0);
}
