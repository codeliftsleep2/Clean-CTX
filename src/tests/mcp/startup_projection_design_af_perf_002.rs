//! Startup design: preserve complete query/errors; compare only discarded projection.
use crate::compression::Fidelity;
use crate::mcp::context_store::ContextStore;
use crate::mcp::sqlite_store::SqliteStore;
use serde_json::json;

#[path = "startup_projection_query_af_perf_002.rs"]
mod query;

fn paths(store: &SqliteStore, mode: Option<bool>) -> Result<Vec<String>, String> {
    match mode {
        None => store
            .list_contexts(i64::MAX as usize)
            .map(|v| v.into_iter().map(|v| v.file_path).collect())
            .map_err(|e| format!("{:?}|{e}", e.downcast_ref::<rusqlite::Error>().unwrap())),
        Some(mode) => query::read(store, mode, true)
            .map(|(v, _)| v.paths().into_iter().map(str::to_owned).collect())
            .map_err(|e| format!("{e:?}|{e}")),
    }
}
fn seed(store: &mut SqliteStore, file: &str, id: &str) -> String {
    store
        .save_context(file, Fidelity::High, "fixture", None, id, 17, 9)
        .unwrap()
}
fn equal(store: &SqliteStore) {
    let actual = paths(store, None);
    assert_eq!(actual, paths(store, Some(false)));
    assert_eq!(actual, paths(store, Some(true)));
    if let Ok(expected) = store.list_contexts(i64::MAX as usize) {
        let (copy, _) = query::read(store, false, true).unwrap();
        if let query::Output::Summaries(copy) = copy {
            assert_eq!(format!("{expected:?}"), format!("{copy:?}"));
        } else {
            panic!("Original test form omitted summaries");
        }
    }
}

#[test]
fn af002_startup_projection_contracts() {
    // Relaxed scratch schema permits storage classes forbidden by durable schema.
    // It exercises the exact production reader, not a replacement production store.
    for assignment in [
        "fidelity=NULL",
        "fidelity=X'00'",
        "file_path=NULL",
        "file_path=X'00'",
        "raw_tokens=NULL",
        "compressed_tokens=NULL",
        "raw_tokens=X'00'",
        "compressed_tokens='bad'",
        "updated_at=NULL",
        "updated_at=X'00'",
        "updated_at=CAST(X'80' AS TEXT)",
        "fidelity=99",
        "updated_at=''",
        "updated_at='共有'",
        "raw_tokens=-1",
    ] {
        let mut store = SqliteStore::open(std::path::Path::new(":memory:")).unwrap();
        seed(&mut store, "/root/a.ts", "hash");
        store
            .conn
            .execute_batch(
                "ALTER TABLE contexts RENAME TO original_contexts;
            CREATE TABLE contexts AS SELECT * FROM original_contexts",
            )
            .unwrap();
        store
            .conn
            .execute_batch(&format!("UPDATE contexts SET {assignment}"))
            .unwrap();
        equal(&store);
    }
    for fault in [
        "DROP TABLE deltas",
        "ALTER TABLE deltas RENAME COLUMN context_id TO lost_owner",
        "DROP TABLE contexts",
    ] {
        let mut store = SqliteStore::open(std::path::Path::new(":memory:")).unwrap();
        seed(&mut store, "/root/a.ts", "hash");
        store.conn.execute_batch(fault).unwrap();
        equal(&store);
        if !fault.ends_with("contexts") {
            assert!(paths(&store, None).is_err());
            // Explicitly rejected negative control: dropping COUNT removes errors.
            let accepted = store
                .conn
                .prepare("SELECT file_path FROM contexts ORDER BY updated_at DESC LIMIT ?1")
                .unwrap()
                .query_map([i64::MAX], |r| r.get::<_, String>(0))
                .unwrap()
                .collect::<rusqlite::Result<Vec<_>>>();
            assert!(accepted.is_ok());
            eprintln!(
                "AF002_STARTUP_REJECTED {}",
                json!({"fault":fault,"original":paths(&store,None),"path_only_succeeds":true})
            );
        }
    }
    let mut store = SqliteStore::open(std::path::Path::new(":memory:")).unwrap();
    equal(&store);
    for (i, file) in ["/root-a/a.ts", "/root-b/b.ts", "/root-a/c.ts"]
        .iter()
        .enumerate()
    {
        seed(&mut store, file, &format!("h{i}"));
    }
    // Duplicate paths and tied timestamps remain legal reader inputs, without a new tie policy.
    store.conn.execute_batch("INSERT INTO contexts(id,file_path,content_hash,source_hash,fidelity,ir_binary,updated_at)
        VALUES('duplicate','/root-a/a.ts','duplicate','duplicate',2,X'00','2026-01-01');
        UPDATE contexts SET updated_at='2026-01-01'").unwrap();
    equal(&store);
    let tied = paths(&store, None).unwrap();
    assert_eq!(tied.len(), 4);
    assert_eq!(
        tied.iter().filter(|p| p.as_str() == "/root-a/a.ts").count(),
        2
    );
    store
        .conn
        .execute_batch("UPDATE contexts SET fidelity=X'00',file_path=X'00',raw_tokens=X'00'")
        .unwrap();
    equal(&store);
}

fn config(root: &tempfile::TempDir) -> crate::config::CleanCtxConfig {
    let mut cfg = crate::tests::test_config();
    cfg.cbm.enabled = false;
    cfg.proxy.port = 0;
    cfg.persistence.enabled = true;
    cfg.persistence.db_path = root.path().join("design.db").to_string_lossy().into_owned();
    cfg.additional_roots
        .push(root.path().to_string_lossy().into_owned());
    cfg
}

#[test]
fn af002_startup_projection_physical_order() {
    let root = tempfile::tempdir().unwrap();
    let mut cfg = config(&root);
    let first = root.path().join("root-a");
    let second = root.path().join("root-b");
    std::fs::create_dir(&first).unwrap();
    std::fs::create_dir(&second).unwrap();
    cfg.additional_roots.extend([
        first.to_string_lossy().into_owned(),
        second.to_string_lossy().into_owned(),
    ]);
    let a = first.join("a.ts");
    let b = second.join("b.ts");
    std::fs::write(&a, "export class A {}\n").unwrap();
    std::fs::hard_link(&a, &b).unwrap();
    let a = a.to_string_lossy().into_owned();
    let b = b.to_string_lossy().into_owned();
    for malformed in [false, true] {
        let mut store = SqliteStore::open(std::path::Path::new(&cfg.persistence.db_path)).unwrap();
        seed(&mut store, &a, "a");
        seed(&mut store, &b, "b");
        store
            .conn
            .execute(
                "UPDATE contexts SET updated_at='2026-01-01' WHERE file_path=?1",
                [&a],
            )
            .unwrap();
        store
            .conn
            .execute(
                "UPDATE contexts SET updated_at='2027-01-01' WHERE file_path=?1",
                [&b],
            )
            .unwrap();
        if malformed {
            store
                .conn
                .execute(
                    "UPDATE contexts SET fidelity=X'00' WHERE file_path=?1",
                    [&a],
                )
                .unwrap();
        }
        equal(&store);
        let expected = paths(&store, Some(true));
        drop(store);
        let state = crate::mcp::McpState::new(cfg.clone());
        assert_eq!(state.workspace_index_read().edge_count(), 0);
        assert!(state.alias_for_path(&a).is_none());
        if malformed {
            assert!(expected.is_err());
            // All-or-nothing: even the valid first row was not registered.
            assert_eq!(state.durable_owner_path(&a), a);
        } else {
            assert_eq!(expected.unwrap(), vec![b.clone(), a.clone()]);
            assert_eq!(state.durable_owner_path(&a), b);
        }
    }
}

fn sample(store: &SqliteStore, projected: bool) -> u128 {
    let start = std::time::Instant::now();
    drop(std::hint::black_box(
        query::read(store, projected, false).unwrap(),
    ));
    start.elapsed().as_nanos()
}
#[test]
fn af002_startup_projection_costs() {
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
            let id = seed(
                &mut store,
                &format!("/root-{}/file-{i:04}.ts", i % 2),
                &format!("h{i}"),
            );
            let tx = store.conn.transaction().unwrap();
            for j in 0..deltas {
                tx.execute(
                    "INSERT INTO deltas(context_id,edit_sequence,delta_payload) VALUES(?1,?2,?3)",
                    rusqlite::params![id, j + 1, b"[]".as_slice()],
                )
                .unwrap();
            }
            tx.commit().unwrap();
        }
        equal(&store);
        let (old, ow) = query::read(&store, false, true).unwrap();
        let (new, nw) = query::read(&store, true, true).unwrap();
        assert_eq!(old.paths(), new.paths());
        assert_eq!(
            (
                ow.rows,
                ow.conversions,
                ow.count_results,
                ow.delta_total,
                ow.vm_steps
            ),
            (
                nw.rows,
                nw.conversions,
                nw.count_results,
                nw.delta_total,
                nw.vm_steps
            )
        );
        assert_eq!(nw.rows, contexts);
        assert_eq!(nw.summaries, 0);
        for _ in 0..3 {
            sample(&store, false);
            sample(&store, true);
        }
        let mut before = Vec::new();
        let mut after = Vec::new();
        for round in 0..24 {
            if round % 2 == 0 {
                before.push(sample(&store, false));
                after.push(sample(&store, true));
            } else {
                after.push(sample(&store, true));
                before.push(sample(&store, false));
            }
        }
        before.sort_unstable();
        after.sort_unstable();
        eprintln!(
            "AF002_STARTUP_PROJECTION {}",
            json!({"contexts":contexts,"deltas_per_context":deltas,"original_work":ow,"projection_work":nw,
            "original_median_ns":before[12],"projection_median_ns":after[12],"original_samples_ns":before,"projection_samples_ns":after,
            "summary_header_bytes":std::mem::size_of::<crate::mcp::sqlite_store::PersistedContextSummary>(),"path_header_bytes":std::mem::size_of::<String>()})
        );
    }
}
