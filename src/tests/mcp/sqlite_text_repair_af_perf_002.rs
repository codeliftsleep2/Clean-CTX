//! Original production controls and independent materialization budget.
use crate::compression::Fidelity;
use crate::default_operation_observations_af_perf_002 as work;
use crate::mcp::context_store::{ContextStore, StoredContextMeta};
use crate::mcp::sqlite_store::SqliteStore;
use serde_json::{Value, json};
use std::path::Path;

#[path = "sqlite_text_original_af_perf_002.rs"]
mod original;

fn result(value: Result<Option<StoredContextMeta>, Box<dyn std::error::Error>>) -> String {
    match value {
        Ok(v) => format!("{v:?}"),
        Err(e) => format!("{:?}|{e}", e.downcast_ref::<rusqlite::Error>().unwrap()),
    }
}

#[test]
fn af002_sqlite_text_frozen_semantics() {
    for text in [
        "NULL",
        "''",
        "'共有'",
        "42",
        "3.5",
        "X'4142'",
        "CAST(X'80' AS TEXT)",
    ] {
        for fault in ["", ", compressed_tokens=X'00'", ", fidelity=X'00'"] {
            let mut store = SqliteStore::open(Path::new(":memory:")).unwrap();
            store
                .save_context("/case.ts", Fidelity::High, "initial", None, "hash", 17, 9)
                .unwrap();
            store.conn.execute_batch(&format!("UPDATE contexts SET created_at='2026-01-02 03:04:05', pretty_text={text}{fault}")).unwrap();
            assert_eq!(
                result(store.load_latest("/case.ts")),
                result(original::load_latest(&store, "/case.ts")),
                "{text}{fault}"
            );
            assert_eq!(
                result(store.load_latest("/absent")),
                result(original::load_latest(&store, "/absent"))
            );
        }
    }
    let mut store = SqliteStore::open(Path::new(":memory:")).unwrap();
    let old = store
        .save_context("/case.ts", Fidelity::Low, "old", None, "old", 1, 1)
        .unwrap();
    let new = store
        .save_context("/case.ts", Fidelity::Edit, "new", None, "new", 7, 3)
        .unwrap();
    store.append_delta(&new, b"[]", None).unwrap();
    store.conn.execute("UPDATE contexts SET updated_at='2025-01-01 00:00:00',created_at='2026-01-02 03:04:05' WHERE id=?1", [&old]).unwrap();
    store.conn.execute("UPDATE contexts SET updated_at='2027-01-01 00:00:00',created_at='2026-01-02 03:04:05' WHERE id=?1", [&new]).unwrap();
    let meta = store.load_latest("/case.ts").unwrap().unwrap();
    assert_eq!(meta.source_hash, "new");
    assert_eq!(meta.version, 2);
    assert_eq!(
        result(store.load_latest("/case.ts")),
        result(original::load_latest(&store, "/case.ts"))
    );
    store
        .conn
        .execute_batch("UPDATE contexts SET created_at='invalid'")
        .unwrap();
    let before = std::time::SystemTime::now();
    let actual = store.load_latest("/case.ts").unwrap().unwrap();
    let frozen = original::load_latest(&store, "/case.ts").unwrap().unwrap();
    let after = std::time::SystemTime::now();
    assert!(actual.created_at >= before && actual.created_at <= after);
    assert!(frozen.created_at >= before && frozen.created_at <= after);
    store
        .conn
        .execute_batch("ALTER TABLE contexts RENAME COLUMN pretty_text TO unavailable_text")
        .unwrap();
    assert_eq!(
        result(store.load_latest("/case.ts")),
        result(original::load_latest(&store, "/case.ts"))
    );
}

fn state(root: &tempfile::TempDir) -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config.persistence.enabled = true;
    config.persistence.auto_save = true;
    config.persistence.db_path = root.path().join("repair.db").to_string_lossy().into_owned();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    *state.graph_bridge_lock() = None;
    state
}
fn dispatch(state: &crate::mcp::McpState, tool: &str, file: &str, root: &Path) -> Value {
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(
        &json!(1),
        tool,
        &json!({"arguments":{
        "filePath":file,"workspaceRoot":root.to_string_lossy(),"fidelity":"edit"}}),
        state,
    );
    crate::protocol::captured_responses().pop().unwrap()
}
fn durable(store: &SqliteStore, file: &str) -> Value {
    match store.load_durable_context(file, None) {
        Ok(None) => json!(null),
        Ok(Some(v)) => {
            let crate::ir::CompiledIR {
                file_id,
                instructions,
                version,
            } = v.ir;
            json!({"authority":format!("{:?}",v.authority),
            "ir":{"file_id":file_id,"version":version,"instructions":format!("{instructions:?}")},
            "edges":v.semantic_edges,"dependencies":v.semantic_dependencies,"source_hash":v.source_hash,
            "fidelity":format!("{:?}",v.fidelity),"compatibility":format!("{:?}",v.compatibility)})
        }
        Err(e) => json!({"error_debug":format!("{e:?}"),"error":e.to_string()}),
    }
}
fn identity(state: &crate::mcp::McpState, file: &str) -> Value {
    let alias = state.alias_for_path(file);
    let tuples = alias
        .as_ref()
        .and_then(|a| state.ir_context_read().get_ir(a).cloned());
    let hash = alias
        .as_ref()
        .and_then(|a| state.ir_context_read().get_source_hash(a).cloned());
    let cached = alias
        .as_ref()
        .and_then(|a| state.llm_text_cache_lock().get(a).cloned());
    json!({"alias":alias,"tuples":tuples,"hash":hash,"cached":cached,
        "edges":state.workspace_index_read().edge_count(),"source":std::fs::read_to_string(file).unwrap()})
}
fn normalize(value: &mut Value, root: &str) {
    match value {
        Value::String(s) => {
            *s = s
                .replace(&root.replace('\\', "\\\\"), "<ROOT>")
                .replace(root, "<ROOT>")
        }
        Value::Array(items) => items.iter_mut().for_each(|v| normalize(v, root)),
        Value::Object(items) => items.values_mut().for_each(|v| normalize(v, root)),
        _ => {}
    }
}

fn cases() -> (Vec<Value>, usize, usize) {
    let mut rows = Vec::new();
    let mut strings = 0;
    let mut bytes = 0;
    for expression in [
        "NULL",
        "''",
        "'共有'",
        "'共有' || replace(hex(zeroblob(65536)),'0','a')",
        "X'4142'",
        "CAST(X'80' AS TEXT)",
    ] {
        let root = tempfile::tempdir().unwrap();
        let file = root
            .path()
            .join("subject.ts")
            .to_string_lossy()
            .into_owned();
        std::fs::write(
            &file,
            "export class Subject { run(): number { return 1; } }\n",
        )
        .unwrap();
        let live = state(&root);
        let compiled = dispatch(&live, "compress_code_context", &file, root.path());
        assert!(compiled.get("error").is_none(), "{compiled}");
        {
            let guard = live.persistence_store_lock();
            let store = guard.as_ref().unwrap().sqlite().unwrap();
            store.conn.execute_batch(&format!("UPDATE contexts SET pretty_text={expression},created_at='2026-01-02 03:04:05',updated_at='2026-01-02 03:04:05' ")).unwrap();
        }
        for phase in ["live", "restarted"] {
            let restarted;
            let state = if phase == "live" {
                &live
            } else {
                restarted = state(&root);
                &restarted
            };
            let before = identity(state, &file);
            work::reset();
            let history = dispatch(state, "context_history", &file, root.path());
            let (metadata, restored, database_before, database_after) = {
                let guard = state.persistence_store_lock();
                let store = guard.as_ref().unwrap().sqlite().unwrap();
                let fingerprint = || {
                    store.conn.query_row(
                    "SELECT hex(pretty_text),typeof(pretty_text),source_hash,content_hash,created_at,updated_at FROM contexts WHERE file_path=?1", [&file],
                    |row| Ok((row.get::<_,String>(0)?,row.get::<_,String>(1)?,row.get::<_,String>(2)?,row.get::<_,String>(3)?,row.get::<_,String>(4)?,row.get::<_,String>(5)?))).unwrap()
                };
                let db = fingerprint();
                let actual = result(store.load_latest(&file));
                let binary: Vec<u8> = store
                    .conn
                    .query_row(
                        "SELECT ir_binary FROM contexts WHERE file_path=?1",
                        [&file],
                        |row| row.get(0),
                    )
                    .unwrap();
                let mut decoded = crate::ir::binary_wire::decode(&binary).unwrap();
                decoded.file_id = "<FILE>".into();
                let restored = durable(&store, &file);
                if !expression.starts_with("X'") && !expression.starts_with("CAST") {
                    assert!(
                        restored.get("error").is_none() && !restored.is_null(),
                        "{restored}"
                    );
                }
                let restored = json!({"result":restored,"normalized_binary":crate::ir::binary_wire::encode(&decoded)});
                let after = fingerprint();
                (actual, restored, db, after)
            };
            let observed = work::snapshot();
            // Original-reader comparison is outside the work interval.
            {
                let guard = state.persistence_store_lock();
                let store = guard.as_ref().unwrap().sqlite().unwrap();
                assert_eq!(metadata, result(original::load_latest(&store, &file)));
            }
            assert_eq!(before, identity(state, &file));
            assert_eq!(database_before, database_after);
            assert_eq!(
                database_before.3,
                format!(
                    "{}:{}",
                    database_before.2,
                    super::super::short_path_hash(&file)
                )
            );
            assert!(observed.metadata_queries >= 3);
            assert!(observed.metadata_conversion_attempts >= 3);
            strings += observed.discarded_pretty_strings;
            bytes += observed.discarded_pretty_bytes;
            let mut row = json!({"expression":expression,"phase":phase,"compiled":compiled,"history":history,
                "identity":before,"metadata":metadata,"durable":restored,"database":database_before});
            // Raw storage key is checked above; only its temporary-path hash varies.
            row["database"][3] = json!(format!("{}:<TEMP_PATH_HASH>", database_before.2));
            // Normalize only the temporary canonical root, preserving every field.
            normalize(&mut row, &root.path().to_string_lossy());
            eprintln!(
                "AF002_SQLITE_TEXT_WORK {}",
                json!({"case":rows.len(),"work":observed})
            );
            rows.push(row);
        }
    }
    (rows, strings, bytes)
}
fn compare(rows: &[Value]) {
    assert_eq!(rows.len(), 12);
    if let Ok(path) = std::env::var("AF002_SQLITE_TEXT_ORACLE") {
        let expected: Vec<Value> = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        assert_eq!(rows.len(), expected.len(), "Frozen public row count");
        for (index, (actual, original)) in rows.iter().zip(&expected).enumerate() {
            assert!(
                actual == original,
                "Frozen SQLite public identity mismatch at case {index}"
            );
        }
    }
    for row in rows {
        eprintln!("AF002_SQLITE_TEXT_PUBLIC {row}");
    }
}
#[test]
fn af002_sqlite_text_public_identity() {
    let _serial = crate::protocol::handler_response_serial();
    let (rows, _, _) = cases();
    compare(&rows);
}
#[test]
fn af002_sqlite_text_public_budget() {
    let _serial = crate::protocol::handler_response_serial();
    let (rows, strings, bytes) = cases();
    compare(&rows);
    assert_eq!(
        [strings, bytes],
        [0, 0],
        "SQLite discarded owned-text materialization budget"
    );
}
