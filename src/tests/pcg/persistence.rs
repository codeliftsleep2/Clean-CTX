use super::support::{Fixture, fails};
use crate::pcg::{import, store};
use rusqlite::params;
use serde_json::json;
use std::fs;

#[test]
fn atomic_idempotent_audit_rebuild() {
    let f = Fixture::new();
    let a = f.run(false, false).unwrap();
    let b = f.run(false, false).unwrap();
    let c = f.run(true, false).unwrap();
    let d = f.run(false, true).unwrap();
    assert_eq!(a.generation, b.generation);
    assert_eq!(a.generation, c.generation);
    assert_eq!(a.generation, d.generation);
    assert_eq!(b.operation, "idempotent_import");
    assert_eq!(c.operation, "rebuild");
    let connection = store::reader(&f.trusted(), "fixture").unwrap();
    let count: i64 = connection
        .query_row("SELECT count(*) FROM generation", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 1);
    let foreign_keys: i64 = connection
        .query_row("PRAGMA foreign_keys", [], |r| r.get(0))
        .unwrap();
    assert_eq!(foreign_keys, 1);
    assert!(
        connection
            .execute("DELETE FROM record_revision", [])
            .is_err()
    );
    assert_eq!(a.work.current_sources, 1);
    assert_eq!(
        a.work.current_bytes,
        2 * fs::metadata(f.root.join("decision.md")).unwrap().len()
    );
    assert!(!a.continuous_currentness && !a.atomic_filesystem_snapshot);
}

#[test]
fn failed_import_rebuild_preserve_head() {
    let mut f = Fixture::new();
    let initial = f.run(false, false).unwrap();
    f.add_claim("new-claim");
    f.approve();
    for rebuild in [false, true] {
        let prepared = f.prepare().unwrap();
        let original = fs::read(f.root.join("decision.md")).unwrap();
        fs::write(f.root.join("decision.md"), b"changed at completion").unwrap();
        fails(import::finish(prepared, rebuild, false), "source_stale");
        fs::write(f.root.join("decision.md"), original).unwrap();
        let connection = rusqlite::Connection::open(f.database()).unwrap();
        assert_eq!(
            store::head(&connection, &f.trusted(), "fixture").unwrap(),
            Some(initial.generation.clone())
        );
        let count: i64 = connection
            .query_row("SELECT count(*) FROM generation", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }
    f.manifest["records"][0]["disposition"] = json!("FIX");
    f.approve();
    fails(
        f.run(false, false),
        "decision_replacement_requires_successor",
    );
}

#[test]
fn head_cas_and_writer_conflict() {
    let mut f = Fixture::new();
    f.run(false, false).unwrap();
    f.add_claim("next");
    f.approve();
    let stale = f.prepare().unwrap();
    let (mut writer, _) = store::writer(&f.trusted()).unwrap();
    let held = store::begin(&mut writer).unwrap();
    fails(f.run(false, false), "import_busy");
    drop(held);
    drop(writer);
    f.run(false, false).unwrap();
    fails(import::finish(stale, false, false), "import_head_changed");
}

#[test]
fn projection_corruption_detected() {
    let f = Fixture::new();
    let initial = f.run(false, false).unwrap();
    let connection = rusqlite::Connection::open(f.database()).unwrap();
    // Simulate out-of-band storage damage, bypassing admission deliberately.
    // SQLite may enforce FKs by default; only this fixture connection disables
    // them so the inconsistent header can reach the detection assertions.
    connection.pragma_update(None, "foreign_keys", false).unwrap();
    let foreign_keys: i64 = connection
        .query_row("PRAGMA foreign_keys", [], |r| r.get(0))
        .unwrap();
    assert_eq!(foreign_keys, 0);
    connection
        .execute(
            "UPDATE generation SET g=?1 WHERE g=?2",
            params!["0".repeat(64), initial.generation],
        )
        .unwrap();
    fails(f.run(false, false), "idempotency_conflict");
    // Header/active-pointer mismatch is detected without rehashing the corpus.
    assert!(store::reader(&f.trusted(), "fixture").is_err());
    connection
        .execute("UPDATE generation SET g=?1", [&initial.generation])
        .unwrap();
    connection.pragma_update(None, "foreign_keys", true).unwrap();
    connection.execute("DELETE FROM binding", []).unwrap();
    // Targeted metadata-open deliberately does not certify intact membership.
    store::reader(&f.trusted(), "fixture").unwrap();
    fails(f.run(false, true), "projection_corrupt");
    f.run(true, false).unwrap();
    connection
        .execute("UPDATE generation SET fingerprint=?", ["f".repeat(64)])
        .unwrap();
    fails(f.run(false, false), "projection_corrupt");
}

#[test]
fn source_attribution_and_required_refs() {
    let mut f = Fixture::new();
    f.add_claim("historical-detector");
    f.manifest["records"][1]["role"] = json!("evidence");
    f.manifest["records"][1]["required"] = json!(true);
    f.manifest["records"][1]["authority"]["class"] = json!("attributable_observation");
    f.manifest["records"][1]["verification"]["subject"] = json!("counterexample_detection");
    f.manifest["edges"].as_array_mut().unwrap().push(json!({"id":"uses","kind":"USES_EVIDENCE","from":"decision","to":"historical-detector","evidence":"quote"}));
    f.approve();
    let report = f.run(false, false).unwrap();
    assert_eq!(report.authority_assurance, "source_attributed");
    let c = store::reader(&f.trusted(), "fixture").unwrap();
    let record: (String, String) = c
        .query_row(
            "SELECT authority,execution FROM record_revision WHERE rid='decision'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(
        record,
        (
            "explicitly_approved_decision".into(),
            "report_of_execution".into()
        )
    );
    let refs: i64 = c
        .query_row(
            "SELECT count(*) FROM evidence_ref WHERE rid='decision' AND required=1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(refs, 1);
    let kind: String = c
        .query_row("SELECT kind FROM relation WHERE aid='uses'", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(kind, "USES_EVIDENCE");
    let count: i64 = c
        .query_row("SELECT count(*) FROM evidence_revision", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn bounded_work_and_fanout() {
    for n in [4, 8, 16] {
        let mut f = Fixture::new();
        for i in 0..n {
            f.add_claim(&format!("claim-{i}"));
        }
        f.approve();
        let p = f.prepare().unwrap();
        assert_eq!(p.generation.rows.0["record_revision"].len(), n + 1);
        assert_eq!(p.generation.rows.0["evidence_revision"].len(), 1);
        assert_eq!(p.report.work.evidence_validations, 1);
        assert_eq!(
            p.report.work.git_bytes,
            fs::metadata(f.root.join("decision.md")).unwrap().len()
        );
        f.profile["limits"]["records"] = json!(1);
        f.write_profile();
        fails(f.prepare(), "work_limit");
        f.profile["limits"]["records"] = json!(4096);
        f.profile["limits"]["evidence_bytes"] = json!(8);
        f.write_profile();
        fails(f.prepare(), "work_limit");
    }
}
