//! Accepted governance must permit exact reviewed changes without reusable grants.
use crate::pcg::{document_fixture::Fixture, documents::operator};
use serde_json::Value;

const DIRECTORY: &str = "docs/architecture/knowledge-governance";

fn authorize(fixture: &Fixture) -> Value {
    let description = operator::describe(&fixture.profile).unwrap();
    let digest = description["digest"]
        .as_str()
        .expect("reviewable descriptor");
    assert_eq!(
        operator::authorize(&fixture.profile, digest).unwrap()["authorization_recorded"],
        true
    );
    description
}

fn fixture() -> Fixture {
    let fixture = Fixture::new();
    fixture.write("governed.md", &Fixture::declaration("DOC-001", "PROPOSED"));
    assert_eq!(
        fixture.payload("get", Some("DOC-001"))["result_state"],
        "ready"
    );
    let directory = fixture.root.join(DIRECTORY);
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(
        directory.join("corpus.csv"),
        "source_path,expected_ids\ndocs/governed.md,DOC-001\n",
    )
    .unwrap();
    std::fs::write(directory.join("protected.csv"), "id,role,label,selector\n").unwrap();
    authorize(&fixture);
    assert_eq!(
        fixture.payload("get", Some("DOC-001"))["result_state"],
        "ready"
    );
    fixture
}

#[test]
fn b11_reviewed_membership_removal_preserves_history_without_disappearance_bypass() {
    let fixture = fixture();
    let accepted = fixture.payload("search", None);
    std::fs::remove_file(fixture.root.join("docs/governed.md")).unwrap();
    std::fs::write(
        fixture.root.join(DIRECTORY).join("corpus.csv"),
        "source_path,expected_ids\n",
    )
    .unwrap();
    let blocked = fixture.payload("search", None);
    assert_eq!(blocked["result_state"], "blocked");
    assert_eq!(blocked["generation"], accepted["generation"]);
    let description = operator::describe(&fixture.profile).unwrap();
    assert!(
        description["digest"].is_string(),
        "valid membership removal must be reviewable: {description}"
    );
    authorize(&fixture);
    let published = fixture.payload("search", None);
    assert_eq!(published["result_state"], "ready", "{published}");
    assert_ne!(published["generation"], accepted["generation"]);
    assert!(published["documents"].as_array().unwrap().is_empty());
    let history = fixture.payload("history", Some("DOC-001"));
    assert_eq!(history["historical_documents"].as_array().unwrap().len(), 2);
}

#[test]
fn b11_membership_relocation_requires_exact_review_and_preserves_identity() {
    let fixture = fixture();
    std::fs::rename(
        fixture.root.join("docs/governed.md"),
        fixture.root.join("docs/relocated.md"),
    )
    .unwrap();
    std::fs::write(
        fixture.root.join(DIRECTORY).join("corpus.csv"),
        "source_path,expected_ids\ndocs/relocated.md,DOC-001\n",
    )
    .unwrap();
    assert_eq!(
        fixture.payload("get", Some("DOC-001"))["result_state"],
        "blocked"
    );
    authorize(&fixture);
    let published = fixture.payload("get", Some("DOC-001"));
    assert_eq!(published["result_state"], "ready");
    assert_eq!(published["documents"][0]["path"], "docs/relocated.md");
    assert_eq!(published["documents"][0]["declaration"]["node"], "DOC-001");
}
