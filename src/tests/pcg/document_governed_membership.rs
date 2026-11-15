//! B11 disposition 03: explicit governed membership, not prose classification.
use crate::pcg::document_fixture::Fixture;

fn registry(fixture: &Fixture, rows: &str) {
    let path = fixture.root.join("docs/architecture/knowledge-governance");
    std::fs::create_dir_all(&path).unwrap();
    std::fs::write(
        path.join("corpus.csv"),
        format!("source_path,expected_ids\n{rows}"),
    )
    .unwrap();
}

#[test]
fn b11_registered_source_without_declaration_rejects_without_accepted_publication() {
    let fixture = Fixture::new();
    std::fs::write(
        fixture.root.join("docs/governed.md"),
        "# A governed candidate without a declaration\n",
    )
    .unwrap();
    registry(&fixture, "docs/governed.md,DOC-001\n");
    let rejected = fixture.payload("search", None);
    assert_eq!(rejected["result_state"], "blocked", "{rejected}");
    assert_eq!(rejected["catalog_current"], false);
    assert!(rejected["generation"].is_null());
    let diagnostic = rejected["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|diagnostic| diagnostic["code"] == "E_GOVERNED_DECLARATION_MISSING")
        .unwrap();
    assert_eq!(diagnostic["path"], "docs/governed.md");
    assert_eq!(diagnostic["node"], "DOC-001");
    assert_eq!(diagnostic["accepted_publication"], false);
}

#[test]
fn b11_unregistered_declared_source_rejects_but_ordinary_markdown_remains_outside_admission() {
    let fixture = Fixture::new();
    registry(&fixture, "");
    std::fs::write(
        fixture.root.join("docs/ordinary.md"),
        "# Ordinary documentation\n",
    )
    .unwrap();
    assert_eq!(fixture.payload("search", None)["result_state"], "ready");
    fixture.write(
        "unexpected.md",
        &Fixture::declaration("DOC-001", "PROPOSED"),
    );
    let rejected = fixture.payload("get", Some("DOC-001"));
    assert_eq!(rejected["result_state"], "blocked", "{rejected}");
    let diagnostic = rejected["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|diagnostic| diagnostic["code"] == "E_UNREGISTERED_DECLARED_SOURCE")
        .unwrap();
    assert_eq!(diagnostic["path"], "docs/unexpected.md");
    assert_eq!(diagnostic["node"], "DOC-001");
    assert_eq!(diagnostic["accepted_publication"], false);
}
