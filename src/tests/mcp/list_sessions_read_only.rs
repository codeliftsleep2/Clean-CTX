use crate::compression::Fidelity;
use crate::mcp::context_store::ContextStore;

#[test]
fn registered_list_sessions_observes_only_committed_rows() {
    let root = tempfile::tempdir().expect("temporary workspace");
    let mut config = crate::tests::test_config();
    config.persistence.enabled = true;
    config.persistence.db_path = root.path().join("contexts.db").display().to_string();
    let state = crate::mcp::McpState::new(config);

    {
        let guard = state.persistence_store_lock();
        let store = guard.as_ref().expect("persistence store");
        store.queue_save_context(
            "/committed.ts",
            Fidelity::Low,
            "committed",
            b"committed-ir",
            "committed",
            10,
            5,
        );
        assert_eq!(store.flush(), 1);

        store.queue_save_context(
            "/pending.ts",
            Fidelity::Low,
            "pending",
            b"pending-ir",
            "pending",
            10,
            5,
        );
        store.queue_append_delta("ctx-committed", b"pending-delta", Some("edit"));
        store.queue_clear_file("/committed.ts");
        assert_eq!(store.pending_count(), 3);
    }

    crate::mcp::tools::dispatch_tools_call(
        &serde_json::json!(23),
        "list_sessions",
        &serde_json::json!({ "arguments": {} }),
        &state,
    );

    let guard = state.persistence_store_lock();
    let store = guard.as_ref().expect("persistence store");
    assert_eq!(store.pending_count(), 3);
    let sqlite = store.sqlite().expect("SQLite store");
    assert!(sqlite.has_context("/committed.ts"));
    assert!(!sqlite.has_context("/pending.ts"));
    assert_eq!(sqlite.delta_count("ctx-committed"), 0);
}
