//! Bounded test-only text-conversion comparison. Production remains unchanged.
use crate::compression::Fidelity;
use crate::mcp::context_store::{ContextStore, StoredContextMeta};
use crate::mcp::sqlite_store::SqliteStore;
use rusqlite::types::{FromSql, FromSqlResult, ValueRef};
use serde_json::json;
use std::path::Path;

#[path = "sqlite_text_query_af_perf_002.rs"]
mod comparison;

struct ValidatedText;
impl FromSql for ValidatedText {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        // Same conversion/UTF-8 validation as String, without ToString::to_string.
        value.as_str().map(|_| Self)
    }
}

fn outcome(value: Result<Option<StoredContextMeta>, Box<dyn std::error::Error>>) -> String {
    match value {
        Ok(value) => format!("{value:?}"),
        Err(error) => match error.downcast_ref::<rusqlite::Error>() {
            Some(sqlite) => format!("SQL:{sqlite:?}|{sqlite}"),
            None => format!("OTHER:{error:?}|{error}"),
        },
    }
}

fn seed(store: &mut SqliteStore, text: &str) -> String {
    let id = store
        .save_context("/design.ts", Fidelity::High, text, None, "hash", 17, 9)
        .unwrap();
    store.conn.execute(
        "UPDATE contexts SET created_at='2026-01-02 03:04:05', updated_at='2026-01-02 03:04:05'",
        [],
    ).unwrap();
    id
}

#[test]
fn af002_sqlite_text_design_contracts() {
    let production = include_str!("sqlite_text_before_af_perf_002.txt");
    let begin = production.find("    fn load_latest(").unwrap();
    let end = production[begin..].find("    fn has_context(").unwrap() + begin;
    let mut expected = production[begin..end].to_owned();
    expected = expected
        .replace("fn load_latest(", "fn query(")
        .replace("        &self,", "        conn: &rusqlite::Connection,")
        .replace(
            "        file_path: &str,",
            "        file_path: &str,\n        borrowed: bool,",
        )
        .replace("self.conn.prepare", "conn.prepare")
        .replace(
            "chrono_parse_or_now(",
            "super::super::super::chrono_parse_or_now(",
        );
    for observation in [
        "metadata_query()",
        "metadata_conversion()",
        "metadata_text(_pretty.as_deref())",
    ] {
        expected = expected.replace(&format!("#[cfg(test)]\n        crate::default_operation_observations_af_perf_002::{observation};"), "");
        expected = expected.replace(&format!("#[cfg(test)]\r\n        crate::default_operation_observations_af_perf_002::{observation};"), "");
        expected = expected.replace(&format!("#[cfg(test)]\n            crate::default_operation_observations_af_perf_002::{observation};"), "");
        expected = expected.replace(&format!("#[cfg(test)]\r\n            crate::default_operation_observations_af_perf_002::{observation};"), "");
    }
    let copy = include_str!("sqlite_text_query_af_perf_002.rs");
    let copy = &copy[copy.find("fn query(").unwrap()..];
    let branch = "let _pretty = if borrowed { let _: Option<ValidatedText> = row.get(4)?; None } else { row.get::<_, Option<String>>(4)? };";
    let compact = |s: &str| s.split_whitespace().collect::<String>();
    assert_eq!(
        compact(copy).replace(
            &compact(branch),
            &compact("let _pretty: Option<String> = row.get(4)?;")
        ),
        compact(&expected)
    );

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
            seed(&mut store, "initial");
            store
                .conn
                .execute_batch(&format!("UPDATE contexts SET pretty_text={text}{fault}"))
                .unwrap();
            let actual = outcome(store.load_latest("/design.ts"));
            assert_eq!(
                actual,
                outcome(comparison::query(&store.conn, "/design.ts", false))
            );
            assert_eq!(
                actual,
                outcome(comparison::query(&store.conn, "/design.ts", true)),
                "{text}{fault}"
            );
        }
    }
    let mut store = SqliteStore::open(Path::new(":memory:")).unwrap();
    let old = seed(&mut store, "old");
    store
        .save_context(
            "/design.ts",
            Fidelity::Edit,
            "new",
            None,
            "new-hash",
            30,
            12,
        )
        .unwrap();
    store.conn.execute("UPDATE contexts SET updated_at='2027-01-01 00:00:00', created_at='2026-01-02 03:04:05' WHERE id != ?1", [&old]).unwrap();
    for file in ["/design.ts", "/absent.ts"] {
        let actual = outcome(store.load_latest(file));
        assert_eq!(actual, outcome(comparison::query(&store.conn, file, false)));
        assert_eq!(actual, outcome(comparison::query(&store.conn, file, true)));
    }
    assert_eq!(
        store
            .load_latest("/design.ts")
            .unwrap()
            .unwrap()
            .source_hash,
        "new-hash"
    );
}

fn sample(store: &SqliteStore, borrowed: bool, requests: usize) -> u128 {
    let start = std::time::Instant::now();
    for _ in 0..requests {
        drop(std::hint::black_box(
            comparison::query(&store.conn, "/design.ts", borrowed).unwrap(),
        ));
    }
    start.elapsed().as_nanos()
}

#[test]
fn af002_sqlite_text_design_costs() {
    for disk in [false, true] {
        for (repetitions, requests, deltas) in [
            (0, 1, 0),
            (16, 1, 0),
            (1024, 1, 0),
            (65536, 1, 0),
            (1024, 16, 0),
            (1024, 1, 64),
        ] {
            let root = tempfile::tempdir().unwrap();
            let path = if disk {
                root.path().join("cost.db")
            } else {
                ":memory:".into()
            };
            let mut store = SqliteStore::open(&path).unwrap();
            let text = "共有".repeat(repetitions);
            let id = seed(&mut store, &text);
            for _ in 0..deltas {
                store.append_delta(&id, b"[]", None).unwrap();
            }
            crate::default_operation_observations_af_perf_002::reset();
            let original = outcome(store.load_latest("/design.ts"));
            let actual_work = crate::default_operation_observations_af_perf_002::snapshot();
            assert_eq!(actual_work.metadata_queries, 1);
            assert_eq!(actual_work.metadata_rows, 1);
            assert_eq!(actual_work.validated_pretty_bytes, text.len());
            assert_eq!(
                original,
                outcome(comparison::query(&store.conn, "/design.ts", false))
            );
            assert_eq!(
                original,
                outcome(comparison::query(&store.conn, "/design.ts", true))
            );
            for _ in 0..3 {
                sample(&store, false, requests);
                sample(&store, true, requests);
            }
            let mut owned = Vec::new();
            let mut borrowed = Vec::new();
            for round in 0..24 {
                if round % 2 == 0 {
                    owned.push(sample(&store, false, requests));
                    borrowed.push(sample(&store, true, requests));
                } else {
                    borrowed.push(sample(&store, true, requests));
                    owned.push(sample(&store, false, requests));
                }
            }
            owned.sort_unstable();
            borrowed.sort_unstable();
            eprintln!(
                "AF002_SQLITE_TEXT {}",
                json!({"disk":disk,"requests":requests,"deltas":deltas,
                "text_bytes":text.len(),"owned_text_bytes":text.len()*requests,"borrowed_owned_text_bytes":0,
                "queries_per_mode":requests,"owned_median_ns":owned[12],"borrowed_median_ns":borrowed[12],
                "owned_samples_ns":owned,"borrowed_samples_ns":borrowed})
            );
        }
    }
}
