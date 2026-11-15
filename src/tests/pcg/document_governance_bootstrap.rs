//! A registry's existence is not procedural operator authorization to activate it.
use crate::pcg::document_fixture::Fixture;

#[test]
fn b11_valid_candidate_registry_requires_explicit_bound_bootstrap_authorization() {
    let fixture = Fixture::new();
    fixture.write("governed.md", &Fixture::declaration("DOC-001", "PROPOSED"));
    let accepted = fixture.payload("get", Some("DOC-001"));
    assert_eq!(accepted["result_state"], "ready");
    let directory = fixture.root.join("docs/architecture/knowledge-governance");
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(
        directory.join("corpus.csv"),
        "source_path,expected_ids\ndocs/governed.md,DOC-001\n",
    )
    .unwrap();
    std::fs::write(directory.join("protected.csv"), "id,role,label,selector\n").unwrap();
    let blocked = fixture.payload("get", Some("DOC-001"));
    assert_eq!(blocked["result_state"], "blocked", "{blocked}");
    assert_eq!(blocked["catalog_current"], false);
    assert_eq!(blocked["generation"], accepted["generation"]);
    let diagnostic = blocked["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|diagnostic| diagnostic["code"] == "E_GOVERNANCE_BOOTSTRAP_REQUIRED")
        .unwrap();
    assert_eq!(diagnostic["accepted_publication"], false);
    assert_eq!(diagnostic["change_descriptor"]["version"], 1);
    assert_eq!(
        diagnostic["change_descriptor"]["predecessor_generation"],
        accepted["generation"]
    );
    assert_eq!(
        diagnostic["authorization_digest"].as_str().unwrap().len(),
        64
    );
    assert_eq!(
        diagnostic["authorization_assurance"],
        "procedural_operator_action_not_authenticated_identity"
    );
    assert!(diagnostic.get("authorization").is_none());
}
