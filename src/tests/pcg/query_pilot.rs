//! Frozen AF-PERF evidence, read from pinned Git objects into an isolated test root.
//! Fixture enrollment is not production owner enrollment or a performance benchmark.
use super::{Fixture, Mode, State, ids, query, request};
use crate::pcg::identity;
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::Path, process::Command};

const COMMIT: &str = "8ce0a6865ff8a2b55866895c6a239f6bcc3245a3";
fn git(root: &Path, args: &[&str]) -> Vec<u8> {
    let output = Command::new("git")
        .arg("--no-replace-objects")
        .arg("--no-pager")
        .arg("-C")
        .arg(root)
        .args(args)
        .env("GIT_NO_LAZY_FETCH", "1")
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "pinned AF-PERF Git fixture acquisition failed"
    );
    output.stdout
}
fn record(
    id: &str,
    role: &str,
    primary: &str,
    subject: &str,
    disposition: &str,
    required: bool,
) -> Value {
    json!({"id":id,"role":role,"required":required,"primary":primary,
        "authority":{"class":if role == "decision" {"explicitly_approved_decision"} else {"verified_claim"},"attributed_to":"fixture mapping of source-reported outcome"},
        "verification":{"outcome":"supported","subject":subject,"execution":"report_of_execution"},
        "lifecycle":"active","subject":subject,"disposition":disposition,"evidence_refs":[]})
}
fn bind(f: &mut Fixture, id: &str, path: &str, operation: &str) {
    for task in ["modify", "explain"] {
        let selector = format!("{id}-{task}");
        f.manifest["selectors"].as_array_mut().unwrap().push(
            json!({"id":selector,"purpose":"applicability","workspace":"cleanctx",
            "file":path,"operation":operation,"task":task,"from":null,"until":null}),
        );
        f.manifest["selectors"].as_array_mut().unwrap().push(
            json!({"id":format!("coverage-{selector}"),"purpose":"coverage","workspace":"cleanctx",
            "file":path,"operation":operation,"task":task,"from":null,"until":null}),
        );
        f.manifest["edges"].as_array_mut().unwrap().push(json!({"id":format!("bind-{selector}"),"kind":"CONSTRAINS","from":id,"to_selector":selector,"evidence":match id {
            "delta-capture"|"delta-second-read"|"delta-currentness" => "A-final", "replay-reuse" => "C-accepted",
            "replay-later-footer" => "C-reject", _ => "D-disposition" }}));
    }
}

pub(super) fn exercise() {
    let mut f = Fixture::new();
    let origin = Path::new(env!("CARGO_MANIFEST_DIR"));
    let objects = String::from_utf8(git(origin, &["rev-parse", "--git-path", "objects"])).unwrap();
    let objects = origin.join(objects.trim()).canonicalize().unwrap();
    fs::create_dir_all(f.root.join(".git/objects/info")).unwrap();
    fs::write(
        f.root.join(".git/objects/info/alternates"),
        format!("{}\n", objects.to_string_lossy().replace('\\', "/")),
    )
    .unwrap();
    f.profile["workspace_id"] = json!("cleanctx");
    f.manifest["corpus_id"] = json!("af-perf-175");
    f.manifest["source_namespace"] = json!("codeliftsleep2:Clean-CTX");
    for field in ["sources", "selectors", "evidence", "records", "edges"] {
        f.manifest[field] = json!([]);
    }
    let sources = [
        (
            "delta-report",
            "docs/agent/AF_PERF_002_DELTA_PREVIOUS_REPAIR.md",
            false,
        ),
        (
            "delta-design",
            "docs/agent/AF_PERF_002_DELTA_PREVIOUS_DESIGN.md",
            true,
        ),
        (
            "replay-design",
            "docs/agent/AF_PERF_002_REPLAY_RENDER_DESIGN.md",
            false,
        ),
        (
            "git-report",
            "docs/agent/AF_PERF_002_GITDIFF_DISCOVERY.md",
            false,
        ),
        ("delta-owner", "src/mcp/tool_handlers/core/delta.rs", false),
        (
            "replay-test",
            "src/tests/mcp/replay_footer_observation_af_perf_002.rs",
            false,
        ),
        (
            "replay-owner",
            "src/mcp/tool_handlers/persistence/replay.rs",
            false,
        ),
        ("git-body-owner", "src/gitdiff/engine.rs", false),
        ("git-set-owner", "src/gitdiff/workspace.rs", false),
    ];
    let mut bodies = BTreeMap::new();
    for (id, path, historical) in sources {
        let spec = format!("{COMMIT}:{path}");
        let bytes = git(origin, &["cat-file", "blob", &spec]);
        let blob = String::from_utf8(git(origin, &["rev-parse", &spec])).unwrap();
        let destination = f.root.join(path);
        fs::create_dir_all(destination.parent().unwrap()).unwrap();
        fs::write(destination, &bytes).unwrap();
        f.manifest["sources"].as_array_mut().unwrap().push(json!({"id":id,"path":path,"commit":COMMIT,"blob":blob.trim(),"sha256":identity::sha(&bytes),"check":if historical {"historical"} else {"current"}}));
        bodies.insert(id, bytes);
    }
    let evidence = [
        (
            "A-final",
            "delta-report",
            22798,
            25013,
            "3c95b9640d858ac6963dc5dbb910bf0a41c0c61b272dad29a506d75f0c575e0f",
        ),
        (
            "B-rationale",
            "delta-design",
            489,
            1923,
            "20138b0426b4d17904d9a52e87eb9aaa7a0121e1bc2d35d3545d481a4c3e3470",
        ),
        (
            "C-reject",
            "replay-design",
            13420,
            21998,
            "9c55be6ddb1a7bbf45a17e93f76f7a537cbd4e0491695044c78138264a7eca9f",
        ),
        (
            "C-accepted",
            "git-report",
            250,
            2163,
            "d6d9233a049d6d9e3218c8a99e6b3623523629a89cb53423918294521895eb3b",
        ),
        (
            "D-disposition",
            "git-report",
            14045,
            15898,
            "bf8474cf7e697d217a9a96d0fa2e7e2f6a3555f1bea2d3a6f5613337647c6b4c",
        ),
        (
            "D-work",
            "git-report",
            11230,
            14045,
            "63cc6102e6bedb8c063c634ce02e71b2ae9bba3fb93886e3b0940c2847694055",
        ),
        (
            "E-failed",
            "git-report",
            8538,
            9839,
            "b72f6ac369d7547aef5f09b6258bd2568702c54e48c3246ade659fc468e103eb",
        ),
        (
            "E-final",
            "git-report",
            9839,
            10604,
            "10f4a173989666fcc6bcf950d0896702d82d37a09895c72ee3c557e258d5589e",
        ),
        (
            "W-replay",
            "replay-test",
            0,
            5448,
            "6e8cdc00a7d5bef7e23fcf672f45de4520c4e618af41b64074e4b42b0c8e6408",
        ),
    ];
    for (id, source, start, end, sha) in evidence {
        assert_eq!(
            identity::sha(&bodies[source][start..end]),
            sha,
            "frozen evidence changed: {id}"
        );
        f.manifest["evidence"]
            .as_array_mut()
            .unwrap()
            .push(json!({"id":id,"source":source,"start":start,"end":end,"sha256":sha}));
    }
    let mut records = vec![
        record(
            "delta-capture",
            "decision",
            "A-final",
            "delta_previous_authority_capture",
            "PROTECTED",
            true,
        ),
        record(
            "delta-second-read",
            "decision",
            "A-final",
            "delta_second_source_read",
            "DEFER",
            true,
        ),
        record(
            "delta-currentness",
            "constraint",
            "A-final",
            "source_currentness",
            "REQUIRED",
            true,
        ),
        record(
            "replay-reuse",
            "decision",
            "C-accepted",
            "replay_earlier_candidate_reuse",
            "REJECT",
            true,
        ),
        record(
            "replay-later-footer",
            "constraint",
            "C-reject",
            "later_dictionary_observation",
            "REQUIRED",
            true,
        ),
        record(
            "replay-equivalence",
            "claim",
            "C-reject",
            "equivalence",
            "FALSIFIED",
            false,
        ),
        record(
            "replay-counterexample",
            "evidence",
            "C-reject",
            "counterexample_detection",
            "DIVERGENCE",
            false,
        ),
        record(
            "git-body-copy",
            "decision",
            "D-disposition",
            "git_trim_to_owned_body",
            "DEFER",
            true,
        ),
        record(
            "git-duplicate-set",
            "decision",
            "D-disposition",
            "git_worktree_duplicate_set",
            "DEFER",
            true,
        ),
        record(
            "git-first-packet",
            "evidence",
            "E-failed",
            "gitdiff-discovery-20261109-184618",
            "FAILED_REPORTED",
            false,
        ),
        record(
            "git-final-packet",
            "evidence",
            "E-final",
            "gitdiff-discovery-20261109-190157",
            "COMPLETED_REPORTED",
            false,
        ),
        record(
            "git-measured-work",
            "evidence",
            "D-work",
            "logical_work_counts",
            "OBSERVED",
            false,
        ),
        record(
            "replay-test-definition",
            "evidence",
            "W-replay",
            "test_definition",
            "SOURCE_ONLY",
            false,
        ),
        record(
            "delta-rationale",
            "evidence",
            "B-rationale",
            "prior_lock_trace",
            "HISTORICAL_RATIONALE",
            false,
        ),
    ];
    records[1]["evidence_refs"] = json!([{"id":"B-rationale","required":false}]);
    records[3]["evidence_refs"] =
        json!([{"id":"C-reject","required":true},{"id":"W-replay","required":true}]);
    records[5]["authority"]["class"] = json!("hypothesis");
    records[5]["verification"]["outcome"] = json!("contradicted");
    records[5]["lifecycle"] = json!("falsified");
    for index in [6, 9, 10, 11, 12, 13] {
        records[index]["authority"]["class"] = json!("attributable_observation");
    }
    for index in [12, 13] {
        records[index]["verification"]["outcome"] = json!("unknown");
        records[index]["verification"]["execution"] = json!("not_run");
    }
    for (index, subject) in [
        (0, "delayed_capture_contract"),
        (1, "retain_currentness_boundary"),
        (3, "reuse_rejection"),
        (7, "measured_deferral"),
        (8, "measured_deferral"),
        (9, "fixture_failure_report"),
        (10, "completed_discovery_report"),
    ] {
        records[index]["verification"]["subject"] = json!(subject);
    }
    for index in [7, 8] {
        records[index]["evidence_refs"] = json!([{"id":"D-work","required":true}]);
    }
    f.manifest["records"] = json!(records);
    let delta = "src/mcp/tool_handlers/core/delta.rs";
    let replay = "src/mcp/tool_handlers/persistence/replay.rs";
    let body = "src/gitdiff/engine.rs";
    let set = "src/gitdiff/workspace.rs";
    for id in ["delta-capture", "delta-second-read", "delta-currentness"] {
        bind(&mut f, id, delta, "delta_code_context");
    }
    for id in ["replay-reuse", "replay-later-footer"] {
        bind(&mut f, id, replay, "replay_history");
    }
    bind(&mut f, "git-body-copy", body, "diff_commits");
    bind(&mut f, "git-duplicate-set", set, "diff_commits");
    for (id, kind, from, to) in [
        (
            "delta-dep",
            "DEPENDS_ON",
            "delta-capture",
            "delta-currentness",
        ),
        (
            "read-dep",
            "DEPENDS_ON",
            "delta-second-read",
            "delta-currentness",
        ),
        (
            "replay-dep",
            "DEPENDS_ON",
            "replay-reuse",
            "replay-later-footer",
        ),
        (
            "replay-refutation",
            "FALSIFIES",
            "replay-counterexample",
            "replay-equivalence",
        ),
        (
            "descriptive-counterexample",
            "USES_EVIDENCE",
            "replay-reuse",
            "replay-counterexample",
        ),
        (
            "descriptive-delta-rationale",
            "USES_EVIDENCE",
            "delta-second-read",
            "delta-rationale",
        ),
        (
            "descriptive-body-work",
            "USES_EVIDENCE",
            "git-body-copy",
            "git-measured-work",
        ),
        (
            "descriptive-set-work",
            "USES_EVIDENCE",
            "git-duplicate-set",
            "git-measured-work",
        ),
    ] {
        f.manifest["edges"].as_array_mut().unwrap().push(json!({"id":id,"kind":kind,"from":from,"to":to,"evidence":if id.starts_with("delta")||id.starts_with("read")||id == "descriptive-delta-rationale" {"A-final"} else if from.starts_with("git-") {"D-work"} else {"C-reject"}}));
    }
    f.approve();
    let mut q = request(&f);
    q.subject.file = Some(delta.into());
    q.subject.operation = Some("delta_code_context".into());
    let a = query::acquire(&f.profile_path, &q).unwrap();
    assert_eq!(a.state, State::Ready);
    assert_eq!(
        ids(&a),
        vec!["delta-capture", "delta-currentness", "delta-second-read"]
    );
    assert_eq!(a.citations.len(), 1);
    assert_eq!(a.required_records[0]["disposition"], "PROTECTED");
    assert_eq!(a.required_records[2]["disposition"], "DEFER");
    q.subject.file = Some(replay.into());
    q.subject.operation = Some("replay_history".into());
    let c = query::acquire(&f.profile_path, &q).unwrap();
    assert_eq!(c.state, State::Ready);
    assert!(!ids(&c).contains(&"replay-counterexample"));
    for witness in ["C-accepted", "C-reject", "W-replay"] {
        assert!(c.citations.iter().any(|v| v["id"] == witness));
    }
    q.mode = Mode::Provenance;
    q.records = vec!["replay-equivalence".into()];
    q.include_history = true;
    let refutation = query::acquire(&f.profile_path, &q).unwrap();
    assert_eq!(refutation.state, State::Ready);
    assert!(
        refutation
            .required_records
            .iter()
            .any(|r| r["id"] == "replay-equivalence"
                && r["verification"]["outcome"] == "contradicted")
    );
    for (path, id) in [(body, "git-body-copy"), (set, "git-duplicate-set")] {
        q.mode = Mode::Constraints;
        q.records.clear();
        q.subject.file = Some(path.into());
        q.subject.operation = Some("diff_commits".into());
        let d = query::acquire(&f.profile_path, &q).unwrap();
        assert_eq!(d.state, State::Ready);
        assert_eq!(ids(&d), vec![id]);
        assert_eq!(d.required_records[0]["disposition"], "DEFER");
    }
    q.mode = Mode::Provenance;
    q.records = vec!["git-first-packet".into(), "git-final-packet".into()];
    let e = query::acquire(&f.profile_path, &q).unwrap();
    assert_eq!(e.state, State::Ready);
    for (id, outcome) in [
        ("git-first-packet", "FAILED_REPORTED"),
        ("git-final-packet", "COMPLETED_REPORTED"),
    ] {
        assert!(e.required_records.iter().any(|r| r["id"] == id
            && r["disposition"] == outcome
            && r["verification"]["execution"] == "report_of_execution"));
    }
}
