use super::*;
use crate::pcg::document_fixture::Fixture;
use serde_json::json;

#[test]
fn source_owned_lifecycle_restart_and_history() {
    let f = Fixture::new();
    assert_eq!(f.payload("search", None)["documents"], json!([]));
    let mut d = Fixture::declaration("DOC-001", "INVESTIGATING");
    f.write("a.md", &d);
    let first = f.payload("get", Some("DOC-001"));
    assert_eq!(first["result_state"], "ready");
    d["status"] = "FROZEN".into();
    f.write("a.md", &d);
    let current = f.payload("get", Some("DOC-001"));
    assert_ne!(first["generation"], current["generation"]);
    assert_eq!(current["documents"][0]["declaration"]["status"], "FROZEN");
    let history = f.payload("history", Some("DOC-001"));
    assert_eq!(history["historical_documents"].as_array().unwrap().len(), 2);
    assert_eq!(
        history["historical_documents"][0]["declaration"]["status"],
        "INVESTIGATING"
    );
    assert_eq!(history["revision_authorization"], "not_established");
    assert_eq!(
        f.payload("get", Some("DOC-001"))["generation"],
        current["generation"]
    );
}

#[test]
fn gls_candidate_blocker_and_history_not_transition_matrix() {
    let f = Fixture::new();
    let mut a = Fixture::declaration("DOC-001", "BLOCKED");
    let mut b = Fixture::declaration("DOC-002", "INVESTIGATING");
    a["relationships"] = json!([{"kind":"BLOCKED_BY","target":"DOC-002"}]);
    f.write("a.md", &a);
    f.write("b.md", &b);
    assert_eq!(f.payload("get", Some("DOC-001"))["result_state"], "ready");
    b["status"] = "RECLOSED".into();
    f.write("b.md", &b);
    let invalid = f.payload("get", Some("DOC-001"));
    assert_eq!(invalid["result_state"], "blocked");
    assert_eq!(invalid["documents"], json!([]));
    a["status"] = "IMPLEMENTATION_READY".into();
    f.write("a.md", &a);
    let admitted = f.payload("get", Some("DOC-001"));
    assert_eq!(admitted["result_state"], "ready");
    assert_eq!(
        admitted["documents"][0]["declaration"]["relationships"][0]["kind"],
        "BLOCKED_BY"
    );
    // No invented pairwise gate: candidate consistency alone does not authorize a revision.
    a["status"] = "PROPOSED".into();
    f.write("a.md", &a);
    assert_eq!(f.payload("get", Some("DOC-001"))["result_state"], "ready");
    assert_eq!(
        f.payload("history", Some("DOC-001"))["historical_documents"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
}

#[test]
fn malformed_duplicate_missing_endpoint_and_witness_refuse_atomically() {
    let f = Fixture::new();
    let mut d = Fixture::declaration("DOC-001", "FROZEN");
    f.write("a.md", &d);
    let before = f.payload("get", Some("DOC-001"));
    f.write("duplicate.md", &d);
    let bad = f.payload("search", None);
    assert_eq!(bad["result_state"], "blocked");
    assert_eq!(bad["generation"], before["generation"]);
    std::fs::remove_file(f.root.join("docs/duplicate.md")).unwrap();
    d["relationships"] = json!([{"kind":"DEPENDS_ON","target":"MISSING"}]);
    f.write("a.md", &d);
    assert_eq!(f.payload("search", None)["result_state"], "blocked");
    d["relationships"] = json!([]);
    d["artifact"] = "missing.txt".into();
    f.write("a.md", &d);
    assert_eq!(f.payload("search", None)["result_state"], "blocked");
    std::fs::write(
        f.root.join("docs/a.md"),
        "```architecture-graph\n{\"id\":1,\"id\":2}\n```\n",
    )
    .unwrap();
    assert_eq!(f.payload("search", None)["documents"], json!([]));
}

#[test]
fn exact_taxonomy_and_relationship_compatibility() {
    assert_eq!(model::KINDS.len(), 7);
    assert_eq!(model::STATUSES.len(), 12);
    assert_eq!(model::RELATIONS.len(), 13);
    let f = Fixture::new();
    let mut a = Fixture::declaration("DOC-001", "FROZEN");
    let b = Fixture::declaration("DOC-002", "FALSIFIED");
    a["relationships"] = json!([{"kind":"USES_EVIDENCE","target":"DOC-002"}]);
    f.write("a.md", &a);
    f.write("b.md", &b);
    assert_eq!(f.payload("search", None)["result_state"], "ready");
    for kind in ["DEPENDS_ON", "PRODUCES_AUTHORITY", "CONSUMES_AUTHORITY"] {
        a["relationships"][0]["kind"] = kind.into();
        f.write("a.md", &a);
        assert_eq!(f.payload("search", None)["result_state"], "blocked");
    }
    a["relationships"][0]["kind"] = "SUPERSEDES".into();
    f.write("a.md", &a);
    assert_eq!(f.payload("search", None)["result_state"], "blocked");
    let mut b = b;
    b["status"] = "SUPERSEDED".into();
    f.write("b.md", &b);
    assert_eq!(f.payload("search", None)["result_state"], "ready");
}

#[test]
fn deterministic_search_tags_fts_ignore_legacy_token_budget() {
    let f = Fixture::new();
    f.write("a.md", &Fixture::declaration("DOC-001", "FROZEN"));
    let s = Session::new(f.profile.clone());
    let mut q = f.request("search", None);
    q.track = Some("ARCHITECTURE".into());
    q.text = Some("Architectural".into());
    let r = s.query(&q).unwrap();
    let p = Fixture::decode(&r.serialized_result);
    assert_eq!(p["documents"].as_array().unwrap().len(), 1);
    assert_eq!(r.serialized_result, s.query(&q).unwrap().serialized_result);
    q.track = Some("unknown".into());
    assert_eq!(
        Fixture::decode(&s.query(&q).unwrap().serialized_result)["documents"],
        json!([])
    );
    q.track = None;
    let path = f.root.join("docs/a.md");
    let old = std::fs::read_to_string(&path).unwrap();
    std::fs::write(path, format!("{}\n{old}", "x ".repeat(70_000))).unwrap();
    q.token_budget = 1024;
    let response = s.query(&q).unwrap();
    let p = Fixture::decode(&response.serialized_result);
    assert_eq!(p["result_state"], "ready");
    assert_eq!(p["documents"].as_array().unwrap().len(), 1);
    assert!(response.used_tokens > 65_536);
    for hint in [0, 1, u64::MAX] {
        q.token_budget = hint;
        assert_eq!(
            s.query(&q).unwrap().serialized_result,
            response.serialized_result
        );
    }
    assert_eq!(p["budget"]["enforced"], false);
    assert!(p["budget"]["limit"].is_null());
}

#[test]
fn document_profile_and_request_need_no_token_budget_but_keep_context_limits() {
    let f = Fixture::new();
    let mut a = Fixture::declaration("DOC-001", "FROZEN");
    a["relationships"] = json!([{"kind":"DEPENDS_ON","target":"DOC-002"}]);
    f.write("a.md", &a);
    f.write("b.md", &Fixture::declaration("DOC-002", "FROZEN"));
    let mut profile: Value = serde_json::from_slice(&std::fs::read(&f.profile).unwrap()).unwrap();
    profile.as_object_mut().unwrap().remove("token_budget");
    std::fs::write(&f.profile, serde_json::to_vec(&profile).unwrap()).unwrap();
    let mut request = serde_json::to_value(f.request("context", Some("DOC-001"))).unwrap();
    request.as_object_mut().unwrap().remove("tokenBudget");
    let mut request: Request = serde_json::from_value(request).unwrap();
    let session = Session::new(f.profile.clone());
    let complete = Fixture::decode(&session.query(&request).unwrap().serialized_result);
    assert_eq!(complete["context"]["complete"], true);
    assert_eq!(complete["documents"].as_array().unwrap().len(), 2);
    request.max_nodes = Some(1);
    let limited = Fixture::decode(&session.query(&request).unwrap().serialized_result);
    assert_eq!(limited["context"]["complete"], false);
    assert_eq!(limited["documents"], json!([]));
    assert_eq!(limited["diagnostics"][0]["code"], "E_CONTEXT_LIMIT");
}

#[test]
fn complete_transitive_context_exceeds_former_ceiling_in_both_formats() {
    let f = Fixture::new();
    let mut a = Fixture::declaration("DOC-001", "FROZEN");
    a["relationships"] = json!([{"kind":"DEPENDS_ON","target":"DOC-002"}]);
    f.write("a.md", &a);
    f.write("b.md", &Fixture::declaration("DOC-002", "FROZEN"));
    for name in ["a.md", "b.md"] {
        let path = f.root.join("docs").join(name);
        let source = std::fs::read_to_string(&path).unwrap();
        std::fs::write(path, format!("{}\n{source}", "x ".repeat(36_000))).unwrap();
    }
    let session = Session::new(f.profile.clone());
    let mut request = f.request("context", Some("DOC-001"));
    request.token_budget = 1;
    for format in ["inline_v1", "source_refs_v1"] {
        request.response_format = Some(format.into());
        let response = session.query(&request).unwrap();
        assert!(response.used_tokens > 65_536);
        let payload = Fixture::decode(&response.serialized_result);
        assert_eq!(payload["result_state"], "ready");
        assert_eq!(payload["catalog_current"], true);
        assert_eq!(payload["context"]["complete"], true);
        assert_eq!(payload["documents"].as_array().unwrap().len(), 2);
        assert_eq!(
            payload["context"]["relationships"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(payload["context"]["paths"].as_array().unwrap().len(), 2);
        assert_eq!(payload["diagnostics"], json!([]));
    }
}

#[test]
fn nested_examples_and_source_disappearance_do_not_forge_state() {
    let f = Fixture::new();
    let nested = b"````text\n```architecture-graph\n{}\n```\n````\n";
    assert!(
        parse::declarations("example.md", nested)
            .unwrap()
            .is_empty()
    );
    f.write("a.md", &Fixture::declaration("DOC-001", "FROZEN"));
    f.payload("get", Some("DOC-001"));
    std::fs::remove_file(f.root.join("docs/a.md")).unwrap();
    let p = f.payload("search", None);
    assert_eq!(p["result_state"], "blocked");
    assert_eq!(p["documents"], json!([]));
}

#[test]
fn completion_rejects_changed_source_and_profile_scope() {
    let f = Fixture::new();
    f.write("a.md", &Fixture::declaration("DOC-001", "FROZEN"));
    let trusted = profile::Trusted::load(&f.profile).unwrap();
    let mut capture = scan::scan(&trusted).unwrap();
    std::fs::write(f.root.join("evidence.txt"), "changed\n").unwrap();
    assert!(capture.completion(&trusted).is_err());
    let mut q = f.request("search", None);
    q.workspace_root = f.root.join("docs");
    assert_eq!(
        Session::new(f.profile.clone()).query(&q).unwrap_err().0,
        "scope_refused"
    );
}

#[test]
fn shared_source_is_read_once_per_observation_and_corruption_is_not_absence() {
    let f = Fixture::new();
    for id in ["DOC-001", "DOC-002", "DOC-003"] {
        f.write(&format!("{id}.md"), &Fixture::declaration(id, "FROZEN"));
    }
    let trusted = profile::Trusted::load(&f.profile).unwrap();
    let mut capture = scan::scan(&trusted).unwrap();
    let expected = 16
        + std::fs::read_dir(f.root.join("docs"))
            .unwrap()
            .map(|p| p.unwrap().metadata().unwrap().len())
            .sum::<u64>();
    assert_eq!(capture.read_work(), (expected, 4));
    capture.completion(&trusted).unwrap();
    assert_eq!(capture.read_work(), (expected * 2, 4));
    f.payload("search", None);
    let c = rusqlite::Connection::open(f.root.join("documents.sqlite")).unwrap();
    c.execute("DELETE FROM document_search", []).unwrap();
    drop(c);
    assert_eq!(
        Session::new(f.profile.clone())
            .query(&f.request("search", None))
            .unwrap_err()
            .0,
        "document_corruption"
    );
}

#[test]
fn rename_preserves_identity_and_deleted_source_keeps_explicit_history() {
    let f = Fixture::new();
    let d = Fixture::declaration("DOC-001", "FROZEN");
    f.write("a.md", &d);
    let old = f.payload("get", Some("DOC-001"));
    std::fs::rename(f.root.join("docs/a.md"), f.root.join("docs/renamed.md")).unwrap();
    let new = f.payload("get", Some("DOC-001"));
    assert_ne!(old["generation"], new["generation"]);
    assert_eq!(new["documents"][0]["path"], "docs/renamed.md");
    std::fs::remove_file(f.root.join("docs/renamed.md")).unwrap();
    for mode in ["search", "get", "related"] {
        let p = f.payload(
            mode,
            if mode == "search" {
                None
            } else {
                Some("DOC-001")
            },
        );
        assert_eq!(p["result_state"], "blocked");
        assert_eq!(p["documents"], json!([]));
    }
    let history = f.payload("history", Some("DOC-001"));
    assert_eq!(history["catalog_current"], false);
    assert_eq!(history["documents"], json!([]));
    assert_eq!(history["historical_documents"].as_array().unwrap().len(), 2);
    assert_eq!(history["historical_documents"][0]["path"], "docs/a.md");
}

#[test]
fn document_crash_worker() {
    let Some(path) = std::env::var_os("CLEAN_CTX_DOCUMENT_CRASH_PROFILE") else {
        return;
    };
    let trusted = profile::Trusted::load(&std::path::PathBuf::from(path)).unwrap();
    let connection = store::open(&trusted).unwrap();
    connection
        .execute_batch("BEGIN IMMEDIATE; INSERT INTO document_generation VALUES('interrupted');")
        .unwrap();
    std::fs::write(trusted.profile.workspace_root.join("child-ready"), b"ready").unwrap();
    loop {
        std::thread::park_timeout(std::time::Duration::from_secs(1));
    }
}

#[test]
fn interrupted_sqlite_writer_recovers_without_partial_publication() {
    let f = Fixture::new();
    f.write("a.md", &Fixture::declaration("DOC-001", "FROZEN"));
    let first = f.payload("get", Some("DOC-001"));
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "pcg::documents::tests::document_crash_worker",
            "--nocapture",
        ])
        .env("CLEAN_CTX_DOCUMENT_CRASH_PROFILE", &f.profile)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let until = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while !f.root.join("child-ready").exists() && std::time::Instant::now() < until {
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let reached = f.root.join("child-ready").exists();
    child.kill().unwrap();
    child.wait().unwrap();
    assert!(reached, "worker did not enter the fixture transaction");
    assert_eq!(
        f.payload("get", Some("DOC-001"))["generation"],
        first["generation"]
    );
    let c = rusqlite::Connection::open(f.root.join("documents.sqlite")).unwrap();
    assert_eq!(
        c.query_row(
            "SELECT count(*) FROM document_generation WHERE g='interrupted'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );
}

#[test]
fn gls_flat_grammar_multiple_nodes_and_duplicate_keys() {
    let text=b"```architecture-graph\nnode: DOC-081\nkind: MISSION\nstatus: INVESTIGATING\ntitle: NO\ntrack: REPOSITORY\nuses_evidence:\n  - DOC-042\n```\n```architecture-graph\nnode: DOC-042\nkind: INVARIANT\nstatus: PROPOSED\ntitle: ON\ntrack: REPOSITORY\n```\n";
    let declarations = parse::declarations("report.md", text).unwrap();
    assert_eq!(declarations.len(), 2);
    assert_eq!(declarations[0].title, "NO");
    assert_eq!(declarations[1].title, "ON");
    assert_eq!(declarations[0].relationships[0].kind, "USES_EVIDENCE");
    let bad = b"```architecture-graph\nnode: DOC-081\nnode: DOC-042\n```\n";
    assert_eq!(
        parse::declarations("bad.md", bad).unwrap_err().code,
        "E_MALFORMED_DECLARATION"
    );
}

#[test]
fn preauthoring_context_reaches_constraints_without_relation_promotion() {
    let f = Fixture::new();
    let mut work = Fixture::declaration("DOC-081", "INVESTIGATING");
    let mut contract = Fixture::declaration("DOC-068", "FROZEN");
    let mut invariant = Fixture::declaration("DOC-042", "FROZEN");
    invariant["kind"] = "INVARIANT".into();
    work["relationships"] = json!([{"kind":"IMPLEMENTS","target":"DOC-068"},{"kind":"USES_EVIDENCE","target":"DOC-090"}]);
    contract["relationships"] = json!([{"kind":"DEPENDS_ON","target":"DOC-042"}]);
    f.write("work.md", &work);
    f.write("contract.md", &contract);
    f.write("invariant.md", &invariant);
    f.write("failed.md", &Fixture::declaration("DOC-090", "FALSIFIED"));
    let mut q = f.request("context", Some("DOC-081"));
    q.max_nodes = Some(8);
    q.max_edges = Some(8);
    let s = Session::new(f.profile.clone());
    let p = Fixture::decode(&s.query(&q).unwrap().serialized_result);
    assert_eq!(p["context"]["complete"], true);
    assert!(
        p["documents"]
            .as_array()
            .unwrap()
            .iter()
            .any(|n| n["declaration"]["node"] == "DOC-042")
    );
    let path = p["context"]["paths"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["node"] == "DOC-042")
        .unwrap();
    assert_eq!(path["steps"][0]["kind"], "IMPLEMENTS");
    assert_eq!(path["steps"][1]["kind"], "DEPENDS_ON");
    assert_eq!(p["context"]["authorizes_execution"], false);
    assert!(p["context"]["relationships"].as_array().unwrap().iter().any(|e|e["kind"]=="USES_EVIDENCE" && e["meaning"]=="informative_not_prerequisite"));
    q.max_nodes = Some(2);
    let limited = Fixture::decode(&s.query(&q).unwrap().serialized_result);
    assert_eq!(limited["result_state"], "incomplete");
    assert_eq!(limited["documents"], json!([]));
    assert_eq!(limited["context"]["complete"], false);
}
