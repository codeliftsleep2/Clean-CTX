use super::*;
use crate::pcg::document_fixture::Fixture;
use rusqlite::params;
use serde_json::json;

fn legacy() -> (Fixture, String) {
    let f = Fixture::new();
    f.write("shared.md", &Fixture::declaration("DOC-001", "FROZEN"));
    let trusted = profile::Trusted::load(&f.profile).unwrap();
    let before = scan::scan(&trusted).unwrap();
    let old = &before.nodes["DOC-001"];
    let old_g = identity::hash(&json!({"profile":trusted.fingerprint,"revisions":[old.revision]}));
    let c = rusqlite::Connection::open(f.root.join("documents.sqlite")).unwrap();
    c.execute_batch(include_str!("../../pcg/documents/schema_v1.sql"))
        .unwrap();
    c.execute(
        "INSERT INTO document_schema VALUES(1,?)",
        [&trusted.scope_fingerprint],
    )
    .unwrap();
    c.execute(
        "INSERT INTO document_revision VALUES(?,?,?,?,?,?,?,NULL)",
        params![
            old.revision,
            "DOC-001",
            old.path,
            old.source_sha,
            serde_json::to_string(&old.declaration).unwrap(),
            old.bytes.as_slice(),
            serde_json::to_string(&old.evidence).unwrap()
        ],
    )
    .unwrap();
    c.execute("INSERT INTO document_generation VALUES(?)", [&old_g])
        .unwrap();
    c.execute(
        "INSERT INTO document_member VALUES(?,?,?)",
        params![old_g, "DOC-001", old.revision],
    )
    .unwrap();
    c.execute(
        "INSERT INTO document_publication VALUES(1,?,NULL)",
        [&old_g],
    )
    .unwrap();
    let body = "# Shared historical evidence\n\n```architecture-graph\nnode: DOC-001\nkind: MISSION\nstatus: FROZEN\ntitle: First\ntrack: FOUNDATION\nartifact: docs/shared.md\ndepends_on:\n  - DOC-002\n```\n\
```architecture-graph\nnode: DOC-002\nkind: MISSION\nstatus: FROZEN\ntitle: Second\ntrack: FOUNDATION\nartifact: docs/shared.md\n```\n";
    std::fs::write(f.root.join("docs/shared.md"), body).unwrap();
    let current = scan::scan(&trusted).unwrap();
    let generation = identity::hash(
        &json!({"profile":trusted.fingerprint,"revisions":current.nodes.values().map(|n|&n.revision).collect::<Vec<_>>()}),
    );
    c.execute("INSERT INTO document_generation VALUES(?)", [&generation])
        .unwrap();
    for (id, n) in &current.nodes {
        c.execute(
            "INSERT INTO document_revision VALUES(?,?,?,?,?,?,?,?)",
            params![
                n.revision,
                id,
                n.path,
                n.source_sha,
                serde_json::to_string(&n.declaration).unwrap(),
                n.bytes.as_slice(),
                serde_json::to_string(&n.evidence).unwrap(),
                if id == "DOC-001" {
                    Some(&old.revision)
                } else {
                    None
                }
            ],
        )
        .unwrap();
        c.execute(
            "INSERT INTO document_member VALUES(?,?,?)",
            params![generation, id, n.revision],
        )
        .unwrap();
        c.execute(
            "INSERT INTO document_search VALUES(?,?,?,?)",
            params![id, n.declaration.title, body, n.declaration.track],
        )
        .unwrap();
    }
    c.execute("INSERT INTO document_head VALUES(1,?)", [&generation])
        .unwrap();
    c.execute(
        "INSERT INTO document_publication VALUES(2,?,?)",
        params![generation, old_g],
    )
    .unwrap();
    c.execute(
        "INSERT INTO document_edge VALUES(?,?,?,?,?)",
        params![
            generation,
            "DOC-001",
            "DEPENDS_ON",
            "DOC-002",
            json!({"path":"docs/shared.md","source_sha256":current.nodes["DOC-001"].source_sha})
                .to_string()
        ],
    )
    .unwrap();
    (f, generation)
}

#[test]
fn knowledge_schema_migration_preserves_history_identity_edges_and_legacy_sources() {
    let (f, generation) = legacy();
    let result = f.payload("get", Some("DOC-001"));
    assert_eq!(result["result_state"], "ready");
    assert_eq!(result["generation"], generation);
    let history = f.payload("history", Some("DOC-001"));
    assert_eq!(history["historical_documents"].as_array().unwrap().len(), 2);
    assert_eq!(
        history["historical_documents"][1]["previous_revision"],
        history["historical_documents"][0]["revision"]
    );
    assert_ne!(
        history["historical_documents"][0]["source_text"],
        history["historical_documents"][1]["source_text"]
    );
    let c = rusqlite::Connection::open(f.root.join("documents.sqlite")).unwrap();
    let version: i64 = c
        .query_row("SELECT version FROM document_schema", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, 2);
    let sources: i64 = c
        .query_row("SELECT COUNT(*) FROM document_source", [], |r| r.get(0))
        .unwrap();
    let revisions: i64 = c
        .query_row("SELECT COUNT(*) FROM document_revision", [], |r| r.get(0))
        .unwrap();
    assert_eq!((sources, revisions), (2, 3));
    assert_eq!(
        f.payload("context", Some("DOC-001"))["context"]["relationships"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert!(
        c.prepare("PRAGMA foreign_key_check")
            .unwrap()
            .query([])
            .unwrap()
            .next()
            .unwrap()
            .is_none()
    );
    let q = knowledge::Request {
        workspace_root: f.root.clone(),
        mode: "search".into(),
        ..Default::default()
    };
    assert_eq!(
        Session::new(f.profile.clone()).knowledge(&q).unwrap()["result_state"],
        "incomplete"
    );
}

#[test]
fn knowledge_schema_migration_rolls_back_when_current_candidate_is_invalid() {
    let (f, _) = legacy();
    let path = f.root.join("docs/shared.md");
    let valid = std::fs::read_to_string(&path).unwrap();
    std::fs::write(&path, valid.replace("status: FROZEN", "status: INVENTED")).unwrap();
    assert_eq!(f.payload("search", None)["result_state"], "blocked");
    let c = rusqlite::Connection::open(f.root.join("documents.sqlite")).unwrap();
    assert_eq!(
        c.query_row("SELECT version FROM document_schema", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        c.query_row("SELECT COUNT(*) FROM document_revision", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        3
    );
    std::fs::write(&path, valid).unwrap();
    assert_eq!(f.payload("search", None)["result_state"], "ready");
}

#[test]
fn knowledge_schema_migration_corruption_refuses_without_partial_upgrade() {
    let (f, _) = legacy();
    let c = rusqlite::Connection::open(f.root.join("documents.sqlite")).unwrap();
    c.execute(
        "UPDATE document_revision SET body=X'626164' WHERE id='DOC-001' AND previous IS NULL",
        [],
    )
    .unwrap();
    let error = Session::new(f.profile.clone())
        .query(&f.request("search", None))
        .unwrap_err();
    assert_eq!(error.0, "document_corruption");
    assert_eq!(
        c.query_row("SELECT version FROM document_schema", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        c.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE name='document_source'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );
    assert_eq!(
        c.query_row("SELECT COUNT(*) FROM document_revision", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        3
    );
}
