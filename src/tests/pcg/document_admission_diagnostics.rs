//! B11: actionable rejection must locate the offending authored declaration.
use crate::pcg::document_fixture::Fixture;
use serde_json::json;

#[test]
fn b11_invalid_status_diagnostic_identifies_exact_field_and_preserves_accepted_head() {
    let fixture = Fixture::new();
    let declaration = Fixture::declaration("DOC-001", "INVESTIGATING");
    fixture.write("candidate.md", &declaration);
    let accepted = fixture.payload("get", Some("DOC-001"));
    let path = fixture.root.join("docs/candidate.md");
    let original = std::fs::read_to_string(&path).unwrap();
    let candidate = original.replace("status: INVESTIGATING", "status: CERTIFIED");
    std::fs::write(&path, &candidate).unwrap();
    let rejected = fixture.payload("get", Some("DOC-001"));
    assert_eq!(rejected["result_state"], "blocked");
    assert_eq!(rejected["catalog_current"], false);
    assert_eq!(rejected["generation"], accepted["generation"]);
    assert_eq!(rejected["documents"], json!([]));
    let diagnostic = rejected["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|diagnostic| diagnostic["code"] == "E_INVALID_STATUS")
        .unwrap();
    assert_eq!(diagnostic["path"], "docs/candidate.md");
    assert_eq!(diagnostic["node"], "DOC-001");
    let line = candidate
        .lines()
        .position(|line| line == "status: CERTIFIED")
        .unwrap()
        + 1;
    assert_eq!(diagnostic["location"]["line"], line, "{diagnostic}");
    assert_eq!(diagnostic["location"]["field"], "status", "{diagnostic}");
    assert_eq!(diagnostic["observed"], "CERTIFIED", "{diagnostic}");
    assert_eq!(
        diagnostic["admission_disposition"], "rejected",
        "{diagnostic}"
    );
    assert_eq!(diagnostic["accepted_publication"], false, "{diagnostic}");
    let history = fixture.payload("history", Some("DOC-001"));
    assert_eq!(history["historical_documents"].as_array().unwrap().len(), 1);
    std::fs::write(&path, original).unwrap();
    assert_eq!(
        fixture.payload("get", Some("DOC-001"))["generation"],
        accepted["generation"]
    );
}

#[test]
fn b11_duplicate_field_diagnostic_locates_second_occurrence_without_publishing() {
    let fixture = Fixture::new();
    let declaration = Fixture::declaration("DOC-001", "INVESTIGATING");
    fixture.write("candidate.md", &declaration);
    let accepted = fixture.payload("get", Some("DOC-001"));
    let path = fixture.root.join("docs/candidate.md");
    let original = std::fs::read_to_string(&path).unwrap();
    let candidate = original.replace(
        "status: INVESTIGATING",
        "status: INVESTIGATING\nstatus: FROZEN",
    );
    std::fs::write(&path, &candidate).unwrap();
    let rejected = fixture.payload("get", Some("DOC-001"));
    assert_eq!(rejected["generation"], accepted["generation"]);
    assert_eq!(rejected["catalog_current"], false);
    let diagnostic = rejected["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|diagnostic| diagnostic["code"] == "E_MALFORMED_DECLARATION")
        .unwrap();
    let line = candidate
        .lines()
        .position(|line| line == "status: FROZEN")
        .unwrap()
        + 1;
    assert_eq!(diagnostic["location"]["line"], line, "{diagnostic}");
    assert_eq!(diagnostic["location"]["field"], "status", "{diagnostic}");
    assert_eq!(
        diagnostic["admission_disposition"], "rejected",
        "{diagnostic}"
    );
    assert_eq!(diagnostic["accepted_publication"], false, "{diagnostic}");
}
