//! Current production source and construction-inclusive query costs.
use crate::compression::Fidelity;
use crate::default_operation_observations_af_perf_002 as work;
use crate::mcp::context_store::ContextStore;
use crate::mcp::sqlite_store::SqliteStore;
use serde_json::json;

#[path = "sqlite_text_cost_original_af_perf_002.rs"]
mod original;

fn compact(s: &str) -> String {
    s.split_whitespace().collect()
}
fn protected(source: &str, helper: &str) -> bool {
    let expected = include_str!("sqlite_text_before_af_perf_002.txt")
        .replace("mod edit_intent;", "mod edit_intent;\nmod metadata_text;")
        .replace(
            "let _pretty: Option<String> = row.get(4)?;",
            "let _pretty: Option<metadata_text::ValidatedText> = row.get(4)?;",
        )
        .replace("metadata_text(_pretty.as_deref())", "metadata_validated()");
    let expected_helper = r#"
        //! Validate discarded presentation text with String's SQL/UTF-8 conversion contract.
        use rusqlite::types::{FromSql, FromSqlResult, ValueRef};
        pub(super) struct ValidatedText;
        impl FromSql for ValidatedText {
            fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
                let text = value.as_str()?;
                #[cfg(test)]
                crate::default_operation_observations_af_perf_002::metadata_validated_bytes(text.len());
                // Retain UTF-8 validation in release builds without retaining text ownership.
                let _ = text;
                Ok(Self)
            }
        }
    "#;
    compact(source) == compact(&expected) && compact(helper) == compact(expected_helper)
}

#[test]
fn afperf_sqlite_text_static() {
    let source = include_str!("../../mcp/sqlite_store.rs");
    let helper = include_str!("../../mcp/sqlite_store/metadata_text.rs");
    assert!(
        protected(source, helper),
        "SQLite validated-text ownership static protection"
    );
    assert!(!protected(
        &source.replace("Option<metadata_text::ValidatedText>", "Option<String>"),
        helper
    ));
    assert!(!protected(
        source,
        &helper.replace("value.as_str()?", "value.as_bytes()?")
    ));
    assert!(!protected(
        &source.replace("row.get(4)?", "row.get(5)?"),
        helper
    ));
    assert!(!protected(
        &source.replace("metadata_conversion();", "metadata_validated();"),
        helper
    ));
    assert_eq!(
        compact(include_str!("sqlite_text_cost_original_af_perf_002.rs")),
        compact(include_str!("sqlite_text_original_af_perf_002.rs"))
    );
}

#[test]
fn afperf_sqlite_text_validation_law() {
    // Exercise the actual type independently of load_latest, including in the
    // isolated original-conversion mutant where this helper is otherwise unused.
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    for expression in [
        "NULL",
        "''",
        "'共有'",
        "42",
        "3.5",
        "X'4142'",
        "CAST(X'80' AS TEXT)",
    ] {
        let sql = format!("SELECT {expression}");
        let owned = conn.query_row(&sql, [], |row| row.get::<_, Option<String>>(0));
        work::reset();
        let borrowed = conn.query_row(&sql, [], |row| {
            let text = row.get::<_, Option<super::super::metadata_text::ValidatedText>>(0)?;
            work::metadata_validated();
            Ok(text.is_some())
        });
        match (owned, borrowed) {
            (Ok(text), Ok(present)) => {
                assert_eq!(text.is_some(), present);
                let counts = work::snapshot();
                assert_eq!(counts.metadata_rows, 1);
                assert_eq!(
                    counts.validated_pretty_bytes,
                    text.as_ref().map_or(0, String::len)
                );
                assert_eq!(
                    (
                        counts.discarded_pretty_strings,
                        counts.discarded_pretty_bytes
                    ),
                    (0, 0)
                );
            }
            (Err(old), Err(new)) => assert_eq!(format!("{old:?}|{old}"), format!("{new:?}|{new}")),
            _ => panic!("Actual validation acceptance differs for {expression}"),
        }
    }
}

fn sample(store: &SqliteStore, old: bool, requests: usize) -> u128 {
    work::unobserved(|| {
        let start = std::time::Instant::now();
        for _ in 0..requests {
            let value = if old {
                original::load_latest(store, "/cost.ts")
            } else {
                store.load_latest("/cost.ts")
            };
            drop(std::hint::black_box(value.unwrap()));
        }
        start.elapsed().as_nanos()
    })
}

#[test]
fn af002_sqlite_text_integrated_cost() {
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
            let id = store
                .save_context("/cost.ts", Fidelity::High, &text, None, "hash", 17, 9)
                .unwrap();
            store
                .conn
                .execute_batch("UPDATE contexts SET created_at='2026-01-02 03:04:05'")
                .unwrap();
            for _ in 0..deltas {
                store.append_delta(&id, b"[]", None).unwrap();
            }
            work::reset();
            let old = original::load_latest(&store, "/cost.ts").unwrap();
            let before = work::snapshot();
            work::reset();
            let new = store.load_latest("/cost.ts").unwrap();
            let after = work::snapshot();
            assert_eq!(format!("{old:?}"), format!("{new:?}"));
            assert_eq!(
                (
                    before.metadata_queries,
                    before.metadata_rows,
                    before.metadata_conversion_attempts,
                    before.validated_pretty_bytes
                ),
                (
                    after.metadata_queries,
                    after.metadata_rows,
                    after.metadata_conversion_attempts,
                    after.validated_pretty_bytes
                )
            );
            assert_eq!(
                (
                    after.metadata_queries,
                    after.metadata_rows,
                    after.metadata_conversion_attempts,
                    after.validated_pretty_bytes
                ),
                (1, 1, 1, text.len())
            );
            assert_eq!(
                (
                    before.discarded_pretty_strings,
                    before.discarded_pretty_bytes
                ),
                (1, text.len())
            );
            assert_eq!(
                (after.discarded_pretty_strings, after.discarded_pretty_bytes),
                (0, 0)
            );
            for _ in 0..3 {
                sample(&store, true, requests);
                sample(&store, false, requests);
            }
            let mut old_samples = Vec::new();
            let mut new_samples = Vec::new();
            for round in 0..24 {
                if round % 2 == 0 {
                    old_samples.push(sample(&store, true, requests));
                    new_samples.push(sample(&store, false, requests));
                } else {
                    new_samples.push(sample(&store, false, requests));
                    old_samples.push(sample(&store, true, requests));
                }
            }
            old_samples.sort_unstable();
            new_samples.sort_unstable();
            eprintln!(
                "AF002_SQLITE_TEXT_INTEGRATED {}",
                json!({"disk":disk,"text_bytes":text.len(),"requests":requests,"deltas":deltas,
                "original_work":before,"production_work":after,"original_median_ns":old_samples[12],"production_median_ns":new_samples[12],
                "original_samples_ns":old_samples,"production_samples_ns":new_samples})
            );
        }
    }
}
