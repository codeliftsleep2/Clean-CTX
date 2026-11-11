use super::support::Fixture;
use crate::pcg::{
    query::{self, Limits, Mode, Request, State, Subject},
    selectors::{self, Truth},
};
use serde_json::json;
use std::{
    fs,
    process::{Command, Stdio},
};
#[path = "query_pilot.rs"]
mod pilot;

fn request(f: &Fixture) -> Request {
    Request {
        workspace_root: f.root.clone(),
        corpus: f.manifest["corpus_id"].as_str().unwrap().into(),
        generation: f.run(false, false).unwrap().generation,
        mode: Mode::Constraints,
        subject: Subject {
            workspace: f.profile["workspace_id"].as_str().unwrap().into(),
            file: Some("decision.md".into()),
            entity: None,
            operation: Some("edit".into()),
            task: Some("modify".into()),
            at: "2026-11-11T12:00:00Z".into(),
        },
        records: vec![],
        include_history: false,
        limits: Limits {
            records: 4096,
            edges: 8192,
            evidence_bytes: 4194304,
        },
    }
}
fn ids(result: &query::Acquisition) -> Vec<&str> {
    result
        .required_records
        .iter()
        .map(|r| r["id"].as_str().unwrap())
        .collect()
}
fn edge(f: &mut Fixture, id: &str, kind: &str, from: &str, to: &str) {
    f.manifest["edges"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id":id,"kind":kind,"from":from,"to":to,"evidence":"quote"}));
}

#[test]
fn applicability_three_valued() {
    let f = Fixture::new();
    let q = request(&f);
    let selector: crate::pcg::model::Selector =
        serde_json::from_value(f.manifest["selectors"][1].clone()).unwrap();
    assert_eq!(selectors::evaluate(&selector, &q.subject), Truth::True);
    let mut subject = q.subject.clone();
    subject.task = None;
    assert_eq!(selectors::evaluate(&selector, &subject), Truth::Unknown);
    subject.file = Some("other.md".into());
    assert_eq!(selectors::evaluate(&selector, &subject), Truth::False);
    assert_eq!(Truth::Unknown.union(Truth::True), Truth::True);
    let mut bounded = selector;
    bounded.until = Some(q.subject.at.clone());
    assert_eq!(selectors::evaluate(&bounded, &q.subject), Truth::False);
}

#[test]
fn covered_empty_and_unknown_scope() {
    let mut f = Fixture::new();
    let mut q = request(&f);
    q.subject.file = Some("unconstrained.md".into());
    let ready = query::acquire(&f.profile_path, &q).unwrap();
    assert_eq!(ready.state, State::Ready);
    assert!(ready.required_records.is_empty());
    assert!(
        !ready.context_complete
            && !ready.continuous_currentness
            && !ready.atomic_filesystem_snapshot
    );
    let mut peer = f.manifest["records"][0].clone();
    peer["id"] = json!("other-decision");
    peer["disposition"] = json!("REJECT");
    f.manifest["records"].as_array_mut().unwrap().push(peer);
    f.manifest["selectors"].as_array_mut().unwrap().push(json!({"id":"other-binding","purpose":"applicability","workspace":"fixture","file":"other.md","from":null,"until":null}));
    f.manifest["edges"].as_array_mut().unwrap().push(json!({"id":"other-bound","kind":"CONSTRAINS","from":"other-decision","to_selector":"other-binding","evidence":"quote"}));
    f.approve();
    q = request(&f);
    assert_eq!(
        query::acquire(&f.profile_path, &q).unwrap().state,
        State::Ready,
        "disjoint policy peer is not unknown overlap"
    );
    let source_hash = f.manifest["sources"][0]["sha256"].clone();
    f.manifest["selectors"].as_array_mut().unwrap().push(json!({"id":"unknown-entity","purpose":"applicability","workspace":"fixture","file":"decision.md",
        "entity":{"kind":"declaration","file":"decision.md","source_sha256":source_hash,"start":0,"end":1,"name":"x","declaration_kind":"class"},"from":null,"until":null}));
    f.manifest["edges"].as_array_mut().unwrap().push(json!({"id":"unknown-binding","kind":"CONSTRAINS","from":"decision","to_selector":"unknown-entity","evidence":"quote"}));
    f.approve();
    q = request(&f);
    assert_eq!(
        query::acquire(&f.profile_path, &q).unwrap().state,
        State::Ready,
        "TRUE binding dominates UNKNOWN for the same record"
    );
    f.manifest["selectors"][0]["file"] = json!("decision.md");
    f.approve();
    q = request(&f);
    q.subject.file = Some("unconstrained.md".into());
    assert_eq!(
        query::acquire(&f.profile_path, &q).unwrap().state,
        State::Incomplete
    );
    q.subject.file = Some("decision.md".into());
    q.subject.task = None;
    let unknown = query::acquire(&f.profile_path, &q).unwrap();
    assert_eq!(unknown.state, State::Incomplete);
    assert!(unknown.reasons.contains("applicability_unknown"));
    f.manifest["enrollment"]["complete"] = json!(false);
    f.approve();
    q = request(&f);
    assert!(
        !query::acquire(&f.profile_path, &q)
            .unwrap()
            .coverage
            .enrollment
    );
}

#[test]
fn scope_generation_identity_and_record_refusal() {
    let f = Fixture::new();
    let mut q = request(&f);
    q.workspace_root = f.dir.path().to_owned();
    assert!(query::acquire(&f.profile_path, &q).is_err());
    q.workspace_root = f.root.clone();
    q.generation = "0".repeat(64);
    assert_eq!(
        query::acquire(&f.profile_path, &q).unwrap().state,
        State::Unavailable
    );
    q = request(&f);
    q.mode = Mode::Closure;
    q.records = vec!["not-enrolled".into()];
    assert_eq!(
        query::acquire(&f.profile_path, &q).unwrap().state,
        State::Incomplete
    );
    q = request(&f);
    q.subject.entity = Some(query::Declaration {
        kind: "declaration".into(),
        file: "decision.md".into(),
        source_sha256: f.manifest["sources"][0]["sha256"].as_str().unwrap().into(),
        start: 0,
        end: 1,
        name: "x".into(),
        declaration_kind: "class".into(),
    });
    let result = query::acquire(&f.profile_path, &q).unwrap();
    assert_eq!(result.state, State::Incomplete);
    assert!(result.reasons.contains("identity_unavailable"));
}

#[test]
fn mandatory_refs_and_incoming_counterevidence() {
    let mut f = Fixture::new();
    f.add_claim("counterexample");
    f.manifest["records"][1]["role"] = json!("evidence");
    f.manifest["records"][1]["lifecycle"] = json!("retracted");
    f.manifest["records"][1]["verification"]["subject"] = json!("counterexample_detection");
    edge(
        &mut f,
        "refutation",
        "FALSIFIES",
        "counterexample",
        "decision",
    );
    f.approve();
    let q = request(&f);
    let result = query::acquire(&f.profile_path, &q).unwrap();
    assert_eq!(result.state, State::Ready);
    assert_eq!(ids(&result), vec!["counterexample", "decision"]);
    assert_eq!(result.citations.len(), 1);
    assert_eq!(result.counts.evidence_validations, 1);
    assert!(
        result
            .relationships
            .iter()
            .any(|r| r["kind"] == "FALSIFIES")
    );
    f.manifest["records"][1]["lifecycle"] = json!("active");
    f.add_claim("support");
    edge(&mut f, "support-edge", "VALIDATES", "support", "decision");
    f.approve();
    let q = request(&f);
    let result = query::acquire(&f.profile_path, &q).unwrap();
    assert_eq!(result.state, State::Blocked);
    assert!(result.reasons.contains("authority_conflict"));
    assert!(result.required_records.is_empty() && result.citations.is_empty());
}

#[test]
fn descriptive_evidence_never_promotes() {
    let mut f = Fixture::new();
    f.add_claim("optional-evidence");
    f.manifest["records"][1]["role"] = json!("evidence");
    f.manifest["records"][1]["required"] = json!(true);
    edge(
        &mut f,
        "uses",
        "USES_EVIDENCE",
        "decision",
        "optional-evidence",
    );
    f.approve();
    let q = request(&f);
    let result = query::acquire(&f.profile_path, &q).unwrap();
    assert_eq!(result.state, State::Ready);
    assert_eq!(ids(&result), vec!["decision"]);
    assert!(
        result
            .relationships
            .iter()
            .all(|r| r["kind"] != "USES_EVIDENCE")
    );
    assert_eq!(
        result.required_records[0]["authority"]["assurance"],
        "source_attributed"
    );
    assert_eq!(
        result.required_records[0]["verification"]["execution"],
        "report_of_execution"
    );
}

#[test]
fn gates_and_historical_truth() {
    let mut f = Fixture::new();
    f.add_claim("gate");
    f.manifest["edges"].as_array_mut().unwrap().push(json!({"id":"gate-edge","kind":"BLOCKED_BY","from":"decision","to":"gate","evidence":"quote",
        "gate":{"outcome":"supported","subject":"required-subject","authority":"verified_claim"}}));
    f.approve();
    let q = request(&f);
    assert_eq!(
        query::acquire(&f.profile_path, &q).unwrap().state,
        State::Blocked
    );
    f.manifest["records"][1]["verification"]["subject"] = json!("required-subject");
    f.approve();
    let q = request(&f);
    assert_eq!(
        query::acquire(&f.profile_path, &q).unwrap().state,
        State::Ready
    );
    f.manifest["records"][1]["lifecycle"] = json!("falsified");
    f.approve();
    let mut q = request(&f);
    assert_eq!(
        query::acquire(&f.profile_path, &q).unwrap().state,
        State::Blocked
    );
    // A historical proposition queried for explanation is not itself a true premise.
    f.manifest["edges"]
        .as_array_mut()
        .unwrap()
        .retain(|e| e["id"] != "gate-edge");
    f.approve();
    q.generation = f.run(false, false).unwrap().generation;
    q.mode = Mode::Provenance;
    q.records = vec!["gate".into()];
    q.include_history = true;
    assert_eq!(
        query::acquire(&f.profile_path, &q).unwrap().state,
        State::Ready
    );
    f.profile["history_allowed"] = json!(false);
    f.write_profile();
    assert!(query::acquire(&f.profile_path, &q).is_err());
}

#[test]
fn reverse_impact_and_exact_provenance() {
    let mut f = Fixture::new();
    f.add_claim("premise");
    f.add_claim("consumer");
    edge(&mut f, "dependency", "DEPENDS_ON", "decision", "premise");
    edge(&mut f, "second", "DEPENDS_ON", "consumer", "decision");
    f.approve();
    let mut q = request(&f);
    q.mode = Mode::Impact;
    q.records = vec!["premise".into()];
    let impact = query::acquire(&f.profile_path, &q).unwrap();
    assert_eq!(impact.state, State::Ready);
    assert_eq!(ids(&impact), vec!["consumer", "decision", "premise"]);
    q.mode = Mode::Provenance;
    q.records = vec!["decision".into()];
    let provenance = query::acquire(&f.profile_path, &q).unwrap();
    assert_eq!(ids(&provenance), vec!["decision", "premise"]);
    assert_eq!(
        provenance.citations[0]["quote_sha256"],
        f.manifest["evidence"][0]["sha256"]
    );
}

#[test]
fn drift_corruption_and_no_read_side_effects() {
    let f = Fixture::new();
    let q = request(&f);
    let before = fs::read(f.database()).unwrap();
    assert_eq!(
        query::acquire(&f.profile_path, &q).unwrap().state,
        State::Ready
    );
    assert_eq!(before, fs::read(f.database()).unwrap());
    let original = fs::read(f.root.join("decision.md")).unwrap();
    {
        let trusted = f.trusted();
        let connection = crate::pcg::store::reader(&trusted, &q.corpus).unwrap();
        let mut reader = crate::pcg::query_rows::Reader::new(&connection, &trusted, &q);
        let mut work = crate::pcg::profile::Work::new(q.limits.evidence_bytes);
        let witnesses = crate::pcg::query_witnesses::start(
            &mut reader,
            &std::collections::BTreeSet::from(["quote".into()]),
            &mut work,
        )
        .unwrap();
        fs::write(f.root.join("decision.md"), vec![b'x'; original.len()]).unwrap();
        assert_eq!(
            crate::pcg::query_witnesses::complete(&reader, &witnesses, &mut work)
                .unwrap_err()
                .0,
            "source_stale"
        );
        fs::write(f.root.join("decision.md"), &original).unwrap();
    }
    fs::write(f.root.join("decision.md"), vec![b'x'; original.len()]).unwrap();
    let stale = query::acquire(&f.profile_path, &q).unwrap();
    assert_eq!(stale.state, State::Blocked);
    assert!(stale.reasons.contains("source_stale"));
    assert!(stale.required_records.is_empty());
    fs::write(f.root.join("decision.md"), original).unwrap();
    let db = rusqlite::Connection::open(f.database()).unwrap();
    db.execute("UPDATE record_revision SET disposition='CHANGED'", [])
        .unwrap();
    assert_eq!(
        query::acquire(&f.profile_path, &q).unwrap().state,
        State::Unavailable
    );
}

#[test]
fn shared_witness_fanout_and_bounded_work() {
    for n in [4, 8, 16] {
        let mut f = Fixture::new();
        for i in 0..n {
            let id = format!("required-{i}");
            f.add_claim(&id);
            let r = f.manifest["records"]
                .as_array_mut()
                .unwrap()
                .last_mut()
                .unwrap();
            r["role"] = json!("decision");
            r["required"] = json!(true);
            f.manifest["edges"].as_array_mut().unwrap().push(json!({"id":format!("binding-{i}"),"kind":"CONSTRAINS","from":id,"to_selector":"binding","evidence":"quote"}));
        }
        f.approve();
        let mut q = request(&f);
        let result = query::acquire(&f.profile_path, &q).unwrap();
        assert_eq!(result.state, State::Ready);
        assert_eq!(result.required_records.len(), n + 1);
        assert_eq!(result.citations.len(), 1);
        assert_eq!(result.counts.evidence_validations, 1);
        assert_eq!(
            result.counts.witness_bytes,
            4 * fs::metadata(f.root.join("decision.md")).unwrap().len()
        );
        // Ordinary reads rely on the admitted digest, not mutable unapproved sidecar bytes.
        fs::write(f.dir.path().join("manifest.json"), b"unapproved edit").unwrap();
        assert_eq!(
            query::acquire(&f.profile_path, &q).unwrap().state,
            State::Ready
        );
        q.limits.records = 1;
        let limited = query::acquire(&f.profile_path, &q).unwrap();
        assert_eq!(limited.state, State::Incomplete);
        assert!(limited.required_records.is_empty());
    }
}

#[test]
fn query_fixture_worker() {
    let Some(profile) = std::env::var_os("PCG_QUERY_TEST_PROFILE") else {
        return;
    };
    let q: Request = serde_json::from_slice(
        &fs::read(std::env::var_os("PCG_QUERY_TEST_REQUEST").unwrap()).unwrap(),
    )
    .unwrap();
    let result = query::acquire(std::path::Path::new(&profile), &q).unwrap();
    assert_eq!(result.state, State::Ready);
    assert_eq!(ids(&result), vec!["decision"]);
}

#[test]
fn fresh_process_file_acquisition() {
    let f = Fixture::new();
    let q = request(&f);
    let path = f.dir.path().join("query.json");
    fs::write(&path, serde_json::to_vec(&q).unwrap()).unwrap();
    let status = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "pcg::query_tests::query_fixture_worker",
            "--nocapture",
        ])
        .env("PCG_QUERY_TEST_PROFILE", &f.profile_path)
        .env("PCG_QUERY_TEST_REQUEST", path)
        .stdin(Stdio::null())
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
fn af_perf_frozen_dispositions() {
    pilot::exercise();
}
