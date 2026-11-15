//! Operator-owned procedural authorization is simulated only in isolated tests.
use crate::pcg::document_fixture::Fixture;
use crate::pcg::documents::{Session, knowledge, operator};
use serde_json::Value;

const SOURCE: &str = "Outside protected content.\n<!-- pcg:RULE:begin -->\nExact controlling rule.\n<!-- pcg:RULE:end -->\n<!-- pcg:QUAL:begin -->\nA bounded rule is not universal certification.\n<!-- pcg:QUAL:end -->\n```architecture-graph\nnode: DOC-001\nkind: INVARIANT\nstatus: IMPLEMENTED\ntitle: Test invariant\ntrack: FOUNDATION\nartifact: docs/rules.md\nknowledge_scope: invariant\nknowledge_content:\n  - statement self#RULE\n  - qualification self#QUAL\n```\n";

fn query(fixture: &Fixture) -> Value {
    Session::new(fixture.profile.clone())
        .knowledge(&knowledge::Request {
            workspace_root: fixture.root.clone(),
            mode: "get".into(),
            id: Some("DOC-001".into()),
            ..Default::default()
        })
        .unwrap()
}

fn fixture() -> Fixture {
    let fixture = Fixture::new();
    std::fs::write(fixture.root.join("docs/rules.md"), SOURCE).unwrap();
    assert_eq!(query(&fixture)["result_state"], "ready");
    let directory = fixture.root.join("docs/architecture/knowledge-governance");
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(
        directory.join("corpus.csv"),
        "source_path,expected_ids\ndocs/rules.md,DOC-001\n",
    )
    .unwrap();
    std::fs::write(
        directory.join("protected.csv"),
        "id,role,label,selector\nDOC-001,statement,,RULE\n",
    )
    .unwrap();
    fixture
}

fn owner_authorize(fixture: &Fixture) -> Value {
    let descriptor = operator::describe(&fixture.profile).unwrap();
    let report =
        operator::authorize(&fixture.profile, descriptor["digest"].as_str().unwrap()).unwrap();
    assert_eq!(report["authorization_recorded"], true);
    assert_eq!(report["accepted_publication"], false);
    assert_eq!(
        report["authorization_assurance"],
        "procedural_operator_action_not_authenticated_identity"
    );
    descriptor
}

#[test]
fn b11_protected_component_replacement_requires_reviewable_exact_authorization() {
    let fixture = fixture();
    owner_authorize(&fixture);
    let accepted = query(&fixture);
    let replacement = SOURCE.replace("RULE", "REPLACEMENT");
    std::fs::write(fixture.root.join("docs/rules.md"), &replacement).unwrap();
    std::fs::write(
        fixture
            .root
            .join("docs/architecture/knowledge-governance/protected.csv"),
        "id,role,label,selector\nDOC-001,statement,,REPLACEMENT\n",
    )
    .unwrap();
    // Removing an old selector cannot erase its accepted obligation. A valid
    // replacement must nevertheless be describable for explicit owner review.
    let descriptor = operator::describe(&fixture.profile)
        .expect("valid protected replacement must produce a reviewable descriptor");
    assert_eq!(descriptor["accepted_publication"], false);
    assert_eq!(
        descriptor["descriptor"]["predecessor_generation"],
        accepted["generation"]
    );
    assert_eq!(
        descriptor["diagnostics"][0]["code"],
        "E_GOVERNANCE_AUTHORIZATION_REQUIRED"
    );
    let blocked = query(&fixture);
    assert_eq!(blocked["result_state"], "blocked");
    assert_eq!(blocked["generation"], accepted["generation"]);
    owner_authorize(&fixture);
    assert_eq!(query(&fixture)["result_state"], "ready");
    let history = fixture.payload("history", Some("DOC-001"));
    let snapshots = history["historical_documents"].as_array().unwrap();
    assert_eq!(snapshots.len(), 3);
    assert_eq!(snapshots[1]["source_text"], SOURCE);
    assert_eq!(snapshots[2]["source_text"], replacement);
}

#[test]
fn b11_operator_activation_protected_change_and_outside_offsets_preserve_exact_boundaries() {
    let fixture = fixture();
    let blocked = query(&fixture);
    assert_eq!(blocked["result_state"], "blocked");
    owner_authorize(&fixture);
    let activated = query(&fixture);
    assert_eq!(activated["result_state"], "ready");
    let path = fixture.root.join("docs/rules.md");
    std::fs::write(&path, format!("A new ordinary prefix.\n{SOURCE}")).unwrap();
    let ordinary = query(&fixture);
    assert_eq!(ordinary["result_state"], "ready", "{ordinary}");
    assert_ne!(ordinary["generation"], activated["generation"]);
    let changed = SOURCE.replace("rule.", "rule!");
    std::fs::write(&path, &changed).unwrap();
    let rejected = query(&fixture);
    assert_eq!(rejected["result_state"], "blocked");
    assert_eq!(rejected["generation"], ordinary["generation"]);
    assert_eq!(
        rejected["diagnostics"][0]["code"],
        "E_GOVERNANCE_AUTHORIZATION_REQUIRED"
    );
    let approved = owner_authorize(&fixture);
    // A further protected mutation cannot use the previously recorded grant.
    std::fs::write(&path, changed.replace("rule!", "rule? second candidate")).unwrap();
    assert_eq!(query(&fixture)["result_state"], "blocked");
    assert!(operator::authorize(&fixture.profile, approved["digest"].as_str().unwrap()).is_err());
    std::fs::write(&path, &changed).unwrap();
    assert_eq!(query(&fixture)["result_state"], "ready");
    let history = fixture.payload("history", Some("DOC-001"));
    let snapshots = history["historical_documents"].as_array().unwrap();
    // History is publication-oriented: activation adds a publication with the
    // same source revision, followed by ordinary and protected source revisions.
    assert_eq!(snapshots.len(), 4);
    assert_eq!(snapshots[0]["revision"], snapshots[1]["revision"]);
    assert_ne!(
        snapshots[0]["published_generation"],
        snapshots[1]["published_generation"]
    );
    let revisions: std::collections::BTreeSet<_> = snapshots
        .iter()
        .map(|snapshot| snapshot["revision"].as_str().unwrap())
        .collect();
    assert_eq!(revisions.len(), 3);
    assert_eq!(snapshots[0]["source_text"], SOURCE);
    assert_eq!(snapshots[1]["source_text"], SOURCE);
    assert_eq!(
        snapshots[2]["source_text"],
        format!("A new ordinary prefix.\n{SOURCE}")
    );
    assert_eq!(snapshots[3]["source_text"], changed);
}

#[test]
fn b11_registry_removal_and_qualification_change_preserve_accepted_governance() {
    let fixture = fixture();
    owner_authorize(&fixture);
    let accepted = query(&fixture);
    let registry = fixture
        .root
        .join("docs/architecture/knowledge-governance/corpus.csv");
    let bytes = std::fs::read(&registry).unwrap();
    std::fs::remove_file(&registry).unwrap();
    let rejected = query(&fixture);
    assert_eq!(rejected["result_state"], "blocked");
    assert_eq!(rejected["generation"], accepted["generation"]);
    assert_eq!(
        rejected["diagnostics"][0]["code"],
        "E_ACCEPTED_GOVERNANCE_REMOVED"
    );
    std::fs::write(registry, bytes).unwrap();
    std::fs::write(
        fixture.root.join("docs/rules.md"),
        SOURCE.replace("bounded rule", "unbounded rule"),
    )
    .unwrap();
    let qualification = query(&fixture);
    assert_eq!(qualification["result_state"], "blocked");
    assert_eq!(qualification["generation"], accepted["generation"]);
    std::fs::write(fixture.root.join("docs/rules.md"), SOURCE).unwrap();
    assert_eq!(query(&fixture)["generation"], accepted["generation"]);
}

#[test]
fn b11_authorization_descriptor_cannot_be_reused_in_another_workspace() {
    let first = fixture();
    let second = fixture();
    let descriptor = operator::describe(&first.profile).unwrap();
    let expected = descriptor["digest"].as_str().unwrap();
    let error = operator::authorize(&second.profile, expected).unwrap_err();
    assert_eq!(error.0, "governance_authorization_descriptor_mismatch");
    assert_eq!(query(&first)["result_state"], "blocked");
    assert_eq!(query(&second)["result_state"], "blocked");
}

#[test]
fn b11_v2_migration_rolls_back_for_unapproved_bootstrap_then_preserves_history_on_authorization() {
    let fixture = fixture();
    let database = fixture.root.join("documents.sqlite");
    let connection = rusqlite::Connection::open(&database).unwrap();
    let scope: String = connection
        .query_row("SELECT profile FROM document_schema", [], |row| row.get(0))
        .unwrap();
    connection.execute_batch("DROP TABLE document_governance_publication; DROP TABLE document_governance_authorization; DROP TABLE document_schema;").unwrap();
    let old_schema = include_str!("../../pcg/documents/schema_v2.sql");
    connection
        .execute_batch(&format!("{};", old_schema.split(';').next().unwrap()))
        .unwrap();
    connection
        .execute("INSERT INTO document_schema VALUES(2,?)", [&scope])
        .unwrap();
    let predecessor: String = connection
        .query_row("SELECT g FROM document_head", [], |row| row.get(0))
        .unwrap();
    drop(connection);
    assert_eq!(query(&fixture)["result_state"], "blocked");
    let descriptor = operator::describe(&fixture.profile).unwrap();
    let connection = rusqlite::Connection::open(&database).unwrap();
    let version: i64 = connection
        .query_row("SELECT version FROM document_schema", [], |row| row.get(0))
        .unwrap();
    assert_eq!(
        version, 2,
        "Rejected/describe-only migration must not persist"
    );
    assert_eq!(
        descriptor["descriptor"]["predecessor_generation"],
        predecessor
    );
    drop(connection);
    operator::authorize(&fixture.profile, descriptor["digest"].as_str().unwrap()).unwrap();
    let accepted = query(&fixture);
    assert_eq!(accepted["result_state"], "ready");
    assert_ne!(accepted["generation"], predecessor);
    let connection = rusqlite::Connection::open(&database).unwrap();
    let version: i64 = connection
        .query_row("SELECT version FROM document_schema", [], |row| row.get(0))
        .unwrap();
    assert_eq!(version, 3);
    let retained: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM document_generation WHERE g=?",
            [predecessor],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(retained, 1);
    let snapshots: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM document_governance_publication",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(snapshots, 1);
    assert_eq!(
        connection
            .query_row("PRAGMA integrity_check", [], |row| row.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
}
