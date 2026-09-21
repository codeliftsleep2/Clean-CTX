use std::path::{Path, PathBuf};

use crate::compression::Fidelity;
use crate::mcp::buffered_store::BufferedStore;
use crate::mcp::context_store::ContextStore;
use crate::mcp::sqlite_store::SqliteStore;

fn store() -> BufferedStore {
    BufferedStore::new(
        SqliteStore::open(Path::new(":memory:")).expect("in-memory SQLite"),
        PathBuf::from("."),
    )
}

fn commit_context(store: &BufferedStore, path: &str, hash: &str) {
    store.queue_save_context(path, Fidelity::Low, "compact", b"ir", hash, 10, 5);
    assert_eq!(store.flush(), 1);
}

#[test]
fn persisted_reads_observe_only_committed_state() {
    let store = store();
    commit_context(&store, "/committed.ts", "committed");

    store.queue_save_context(
        "/pending.ts",
        Fidelity::Low,
        "pending",
        b"pending-ir",
        "pending",
        20,
        8,
    );
    store.queue_append_delta("ctx-committed", b"pending-delta", Some("edit"));
    store.queue_clear_file("/committed.ts");
    assert_eq!(store.pending_count(), 3);

    let listed = store.list_contexts(20).expect("committed context listing");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].file_path, "/committed.ts");
    assert_eq!(store.pending_count(), 3);

    assert!(
        store
            .load_latest("/pending.ts")
            .expect("observational metadata read")
            .is_none()
    );
    assert_eq!(store.delta_count("ctx-committed"), 0);
    assert_eq!(store.pending_count(), 3);

    let repeated = store.list_contexts(20).expect("repeated committed listing");
    assert_eq!(listed.len(), repeated.len());
    assert_eq!(store.pending_count(), 3);
}

#[test]
fn purge_changes_only_committed_history_and_leaves_queue_pending() {
    let store = store();
    commit_context(&store, "/committed.ts", "committed");
    store.queue_append_delta("ctx-committed", b"pending-delta", Some("edit"));
    store.queue_save_context("/peer.ts", Fidelity::Low, "peer", b"peer-ir", "peer", 10, 5);
    store.queue_clear_file("/committed.ts");

    assert_eq!(
        store.purge_old_deltas(30).expect("committed-history purge"),
        0
    );
    assert_eq!(store.pending_count(), 3);
    assert_eq!(store.delta_count("ctx-committed"), 0);
    assert!(store.has_context("/committed.ts"));
    assert!(!store.has_context("/peer.ts"));
}

#[test]
fn legacy_clear_is_file_scoped_and_does_not_commit_peer_work() {
    let mut store = store();
    commit_context(&store, "/delete.ts", "delete");
    commit_context(&store, "/keep.ts", "keep");
    store.queue_save_context(
        "/pending-peer.ts",
        Fidelity::Low,
        "peer",
        b"peer-ir",
        "pending-peer",
        10,
        5,
    );
    store.queue_append_delta("ctx-keep", b"pending-delta", Some("edit"));
    assert_eq!(store.pending_count(), 2);

    ContextStore::clear_file(&mut store, "/delete.ts");

    assert!(!store.has_context("/delete.ts"));
    assert!(store.has_context("/keep.ts"));
    assert!(!store.has_context("/pending-peer.ts"));
    assert_eq!(store.delta_count("ctx-keep"), 0);
    assert_eq!(store.pending_count(), 2);
}
