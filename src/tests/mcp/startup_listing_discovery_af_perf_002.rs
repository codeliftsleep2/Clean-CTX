//! Broad discovery: actual startup/listing plus source-exact SQLite VM observations.
use crate::compression::Fidelity;
use crate::mcp::context_store::ContextStore;
use crate::mcp::sqlite_store::SqliteStore;
use rusqlite::{StatementStatus, params};
use serde_json::json;

fn sql() -> &'static str {
    let source = include_str!("../../mcp/sqlite_store.rs");
    let start = source.find("pub fn list_contexts(").unwrap();
    let start = source[start..].find("\"SELECT ").unwrap() + start + 1;
    let end = source[start..].find("\",").unwrap() + start;
    &source[start..end]
}
fn vm(store: &SqliteStore, limit: usize) -> (usize, i32, i32, Vec<String>) {
    let mut stmt = store.conn.prepare(sql()).unwrap();
    let rows = stmt
        .query_map([limit as i64], |r| {
            Ok(format!(
                "{:?}",
                (
                    r.get::<_, String>(0)?,
                    r.get::<_, i32>(1)?,
                    r.get::<_, i64>(2)?,
                    r.get::<_, i64>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, i64>(5)?
                )
            ))
        })
        .unwrap()
        .collect::<rusqlite::Result<Vec<_>>>()
        .unwrap();
    let plan = store
        .conn
        .prepare(&format!("EXPLAIN QUERY PLAN {}", sql()))
        .unwrap()
        .query_map([limit as i64], |r| r.get::<_, String>(3))
        .unwrap()
        .collect::<rusqlite::Result<Vec<_>>>()
        .unwrap();
    (
        rows.len(),
        stmt.get_status(StatementStatus::VmStep),
        stmt.get_status(StatementStatus::FullscanStep),
        plan,
    )
}
fn config(root: &tempfile::TempDir) -> crate::config::CleanCtxConfig {
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config.persistence.enabled = true;
    config.proxy.port = 0;
    config.persistence.db_path = root
        .path()
        .join("startup.db")
        .to_string_lossy()
        .into_owned();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    config
}

#[test]
fn af002_broad_startup_listing_dimensions() {
    let _serial = crate::protocol::handler_response_serial();
    for (contexts, deltas) in [
        (0, 0),
        (1, 0),
        (16, 0),
        (128, 0),
        (256, 0),
        (1, 16),
        (16, 16),
        (128, 16),
        (256, 16),
        (1, 128),
        (16, 128),
        (128, 128),
    ] {
        let root = tempfile::tempdir().unwrap();
        let cfg = config(&root);
        let mut store = SqliteStore::open(std::path::Path::new(&cfg.persistence.db_path)).unwrap();
        for i in 0..contexts {
            let file = root
                .path()
                .join(format!("f{i:04}.ts"))
                .to_string_lossy()
                .into_owned();
            let id = store
                .save_context(
                    &file,
                    Fidelity::High,
                    "fixture",
                    None,
                    &format!("hash-{i}"),
                    17,
                    9,
                )
                .unwrap();
            // Committed ordinary rows; fixture creation is outside all measurements.
            let tx = store.conn.transaction().unwrap();
            for j in 0..deltas {
                tx.execute(
                    "INSERT INTO deltas(context_id,edit_sequence,delta_payload) VALUES (?1,?2,?3)",
                    params![id, j + 1, b"[]".as_slice()],
                )
                .unwrap();
            }
            tx.execute("UPDATE contexts SET updated_at=datetime('2026-01-01',?1 || ' seconds') WHERE id=?2",params![i,id]).unwrap();
            tx.commit().unwrap();
        }
        let mut query_rows = Vec::new();
        for (consumer, limit) in [("startup", i64::MAX as usize), ("list_sessions", 200)] {
            let actual = store.list_contexts(limit).unwrap();
            assert_eq!(actual.len(), contexts.min(limit));
            for (rank, row) in actual.iter().enumerate() {
                assert_eq!(row.delta_count, deltas as i64);
                assert_eq!((row.raw_tokens, row.compressed_tokens), (17, 9));
                assert_eq!(row.fidelity, "high");
                assert_eq!(
                    row.file_path,
                    root.path()
                        .join(format!("f{:04}.ts", contexts - 1 - rank))
                        .to_string_lossy()
                );
            }
            let (rows, steps, scans, plan) = vm(&store, limit);
            assert_eq!(rows, actual.len());
            query_rows.push(json!({"consumer":consumer,"limit":limit,"rows":rows,"vm_steps":steps,"fullscan_steps":scans,"plan":plan,
                "returned_delta_total":actual.iter().map(|r|r.delta_count).sum::<i64>()}));
        }
        let expected = store.list_contexts(200).unwrap();
        drop(store);
        let started = std::time::Instant::now();
        let state = crate::mcp::McpState::new(cfg);
        let startup_ns = started.elapsed().as_nanos();
        assert_eq!(state.workspace_index_read().edge_count(), 0);
        let started = std::time::Instant::now();
        crate::protocol::captured_responses().clear();
        crate::mcp::tools::dispatch_tools_call(
            &json!(1),
            "list_sessions",
            &json!({"arguments":{}}),
            &state,
        );
        let response = crate::protocol::captured_responses().pop().unwrap();
        let listing_ns = started.elapsed().as_nanos();
        let mut text = format!("Persisted contexts: {}\n", expected.len());
        if expected.is_empty() {
            text.push_str("(none yet — compressed contexts appear here once saved)\n");
        }
        for row in expected {
            text.push_str(&format!(
                "  - {} [{}] deltas={} tokens={}→{} updated={}\n",
                row.file_path,
                row.fidelity,
                row.delta_count,
                row.raw_tokens,
                row.compressed_tokens,
                row.updated_at
            ));
        }
        assert_eq!(response["result"]["content"][0]["text"], text);
        let guard = state.persistence_store_lock();
        let buffered = guard.as_ref().unwrap();
        assert_eq!(buffered.pending_count(), 0);
        assert_eq!(
            buffered.list_contexts(i64::MAX as usize).unwrap().len(),
            contexts
        );
        eprintln!(
            "AF002_BROAD_STARTUP {}",
            json!({"contexts":contexts,"deltas_per_context":deltas,"total_deltas":contexts*deltas,
            "queries":query_rows,"actual_startup_ns":startup_ns,"actual_listing_ns":listing_ns,"response":response})
        );
    }
}

#[test]
fn af002_broad_startup_metadata_failure_boundary() {
    let root = tempfile::tempdir().unwrap();
    let cfg = config(&root);
    let mut store = SqliteStore::open(std::path::Path::new(&cfg.persistence.db_path)).unwrap();
    store
        .save_context(
            "/valid-path.ts",
            Fidelity::High,
            "fixture",
            None,
            "hash",
            17,
            9,
        )
        .unwrap();
    store
        .conn
        .execute_batch("UPDATE contexts SET fidelity=X'00'")
        .unwrap();
    let error = store.list_contexts(i64::MAX as usize).unwrap_err();
    assert!(error.downcast_ref::<rusqlite::Error>().is_some());
    eprintln!(
        "AF002_BROAD_STARTUP_ERROR {}",
        json!({"metadata_error":format!("{error:?}|{error}"),"consumer":"startup source uses if let Ok; skips registration on this failure"})
    );
    drop(store);
    let state = crate::mcp::McpState::new(cfg);
    assert_eq!(state.workspace_index_read().edge_count(), 0);
}
