//! Required governance association and replay controls, using isolated operator simulation.
use crate::pcg::{document_fixture::Fixture, documents::operator};
use serde_json::Value;

const SOURCE: &str = "<!-- pcg:RULE:begin -->\nExact protected rule.\n<!-- pcg:RULE:end -->\n<!-- pcg:QUAL:begin -->\nBounded applicability only.\n<!-- pcg:QUAL:end -->\n```architecture-graph\nnode: DOC-001\nkind: INVARIANT\nstatus: IMPLEMENTED\ntitle: Protected rule\ntrack: FOUNDATION\nartifact: evidence.txt\nknowledge_scope: invariant\nknowledge_content:\n  - statement self#RULE\n  - qualification self#QUAL\n```\n";
const GOVERNANCE: &str = "docs/architecture/knowledge-governance";

fn authorize(fixture: &Fixture) -> Value {
    let description = operator::describe(&fixture.profile).unwrap();
    assert_eq!(
        operator::authorize(&fixture.profile, description["digest"].as_str().unwrap()).unwrap()["authorization_recorded"],
        true
    );
    description
}

fn fixture() -> Fixture {
    let fixture = Fixture::new();
    std::fs::write(fixture.root.join("docs/rule.md"), SOURCE).unwrap();
    fixture.write("ordinary.md", &Fixture::declaration("DOC-002", "PROPOSED"));
    assert_eq!(
        fixture.payload("get", Some("DOC-001"))["result_state"],
        "ready"
    );
    let directory = fixture.root.join(GOVERNANCE);
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(
        directory.join("corpus.csv"),
        "source_path,expected_ids\ndocs/rule.md,DOC-001\ndocs/ordinary.md,DOC-002\n",
    )
    .unwrap();
    std::fs::write(
        directory.join("protected.csv"),
        "id,role,label,selector\nDOC-001,statement,,RULE\n",
    )
    .unwrap();
    authorize(&fixture);
    assert_eq!(
        fixture.payload("get", Some("DOC-001"))["result_state"],
        "ready"
    );
    fixture
}

fn assert_blocked(fixture: &Fixture, accepted: &Value) {
    let rejected = fixture.payload("get", Some("DOC-001"));
    assert_eq!(rejected["result_state"], "blocked", "{rejected}");
    assert_eq!(rejected["generation"], accepted["generation"]);
    assert_eq!(rejected["catalog_current"], false);
    assert!(
        rejected["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["code"] == "E_GOVERNANCE_AUTHORIZATION_REQUIRED")
    );
}

#[test]
fn b11_outgoing_associations_require_review_while_ordinary_incoming_references_do_not() {
    let fixture = fixture();
    let mut ordinary = Fixture::declaration("DOC-002", "PROPOSED");
    ordinary["relationships"] = serde_json::json!([{"kind":"PRESERVES","target":"DOC-001"}]);
    fixture.write("ordinary.md", &ordinary);
    let incoming = fixture.payload("get", Some("DOC-001"));
    assert_eq!(incoming["result_state"], "ready");
    let path = fixture.root.join("docs/rule.md");
    let changed = SOURCE.replace(
        "knowledge_scope:",
        "depends_on:\n  - DOC-002\nknowledge_scope:",
    );
    std::fs::write(&path, &changed).unwrap();
    assert_blocked(&fixture, &incoming);
    authorize(&fixture);
    let outgoing = fixture.payload("get", Some("DOC-001"));
    assert_eq!(outgoing["result_state"], "ready");
    std::fs::write(&path, changed.replace("depends_on:", "preserves:")).unwrap();
    assert_blocked(&fixture, &outgoing);
}

#[test]
fn b11_external_evidence_and_qualification_membership_are_protected_associations() {
    let fixture = fixture();
    let accepted = fixture.payload("get", Some("DOC-001"));
    std::fs::write(
        fixture.root.join("evidence.txt"),
        "Changed exact governing evidence.\n",
    )
    .unwrap();
    assert_blocked(&fixture, &accepted);
    authorize(&fixture);
    let updated = fixture.payload("get", Some("DOC-001"));
    assert_eq!(updated["result_state"], "ready");
    std::fs::write(
        fixture.root.join("docs/rule.md"),
        SOURCE.replace("  - qualification self#QUAL\n", ""),
    )
    .unwrap();
    assert_blocked(&fixture, &updated);
}

#[test]
fn b11_previous_predecessor_grant_cannot_authorize_the_same_candidate_after_publication() {
    let fixture = fixture();
    let path = fixture.root.join("docs/rule.md");
    let changed = SOURCE.replace("rule.", "rule!");
    std::fs::write(&path, &changed).unwrap();
    let grant = authorize(&fixture);
    std::fs::write(&path, SOURCE).unwrap();
    let mut ordinary = Fixture::declaration("DOC-002", "PROPOSED");
    ordinary["title"] = "New ordinary revision".into();
    fixture.write("ordinary.md", &ordinary);
    let advanced = fixture.payload("get", Some("DOC-001"));
    assert_eq!(advanced["result_state"], "ready");
    assert_ne!(
        advanced["generation"],
        grant["descriptor"]["predecessor_generation"]
    );
    std::fs::write(&path, changed).unwrap();
    assert_blocked(&fixture, &advanced);
    assert!(operator::authorize(&fixture.profile, grant["digest"].as_str().unwrap()).is_err());
    authorize(&fixture);
    assert_eq!(
        fixture.payload("get", Some("DOC-001"))["result_state"],
        "ready"
    );
}

#[test]
fn b11_protection_set_removal_and_source_approval_claim_cannot_self_authorize() {
    let fixture = fixture();
    let accepted = fixture.payload("get", Some("DOC-001"));
    std::fs::write(
        fixture.root.join(GOVERNANCE).join("protected.csv"),
        "id,role,label,selector\n",
    )
    .unwrap();
    let changed = SOURCE.replace("rule.", "rule!").replace(
        "title: Protected rule",
        "note: owner approved every future change\ntitle: Protected rule",
    );
    std::fs::write(fixture.root.join("docs/rule.md"), changed).unwrap();
    assert_blocked(&fixture, &accepted);
    let descriptor = operator::describe(&fixture.profile).unwrap();
    assert!(
        descriptor["descriptor"]["proposed_state"]["protection"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        descriptor["descriptor"]["previously_protected_candidate_view"]["components"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}
