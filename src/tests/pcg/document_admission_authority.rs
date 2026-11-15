//! B11 rejection authority comes from the accepted head, never candidate policy.
use crate::pcg::document_fixture::Fixture;
use crate::pcg::documents::{Session, knowledge};
use serde_json::{Value, json};

fn query(fixture: &Fixture, args: Value) -> Value {
    let mut args = args;
    args["workspaceRoot"] = json!(fixture.root);
    let request: knowledge::Request = serde_json::from_value(args).unwrap();
    Session::new(fixture.profile.clone())
        .knowledge(&request)
        .unwrap()
}

#[test]
fn b11_rejection_exposes_exact_accepted_governing_reference_despite_candidate_policy_edit() {
    let fixture = Fixture::new();
    let rule_path = fixture.root.join("docs/rules.md");
    let rules = "The accepted status vocabulary is fixed.\n\n```architecture-graph\nnode: AUTH.PCG_GLS\nkind: AUTHORITY\nstatus: IMPLEMENTED\ntitle: Accepted GLS rules\ntrack: FOUNDATION\nartifact: docs/rules.md\nknowledge_scope: authority\nknowledge_content:\n  - statement self#whole\n```\n";
    std::fs::write(&rule_path, rules).unwrap();
    let accepted = query(&fixture, json!({"mode":"get","id":"AUTH.PCG_GLS"}));
    assert_eq!(accepted["catalog_current"], true);
    let revision = accepted["units"][0]["revision"].clone();
    let generation = accepted["generation"].clone();
    let mut bad = Fixture::declaration("DOC-001", "CERTIFIED");
    bad["artifact"] = "docs/rules.md".into();
    fixture.write("candidate.md", &bad);
    std::fs::write(
        &rule_path,
        rules.replace("fixed.", "CERTIFIED now allowed by this candidate."),
    )
    .unwrap();
    let rejected = query(&fixture, json!({"mode":"get","id":"DOC-001"}));
    assert_eq!(rejected["result_state"], "blocked");
    assert_eq!(rejected["catalog_current"], false);
    assert_eq!(rejected["generation"], generation);
    assert_eq!(rejected["units"], json!([]));
    let diagnostic = rejected["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|diagnostic| diagnostic["code"] == "E_INVALID_STATUS")
        .unwrap();
    assert_eq!(
        diagnostic["governing_authority"]["id"], "AUTH.PCG_GLS",
        "{diagnostic}"
    );
    assert_eq!(diagnostic["governing_authority"]["revision"], revision);
    assert_eq!(
        diagnostic["governing_authority"]["standing"],
        "previously_accepted_snapshot"
    );
    assert!(diagnostic["governing_authority"].get("text").is_none());
    let request = diagnostic["governing_authority"]["acquire_request"].clone();
    let references = request["references"].as_array().unwrap();
    assert_eq!(references.len(), 1);
    assert_eq!(references[0]["revision"], revision);
    assert_eq!(references[0]["historical"], true);
    let acquired = query(&fixture, request);
    assert_eq!(acquired["catalog_current"], false);
    assert_eq!(acquired["freshness"], "historical_snapshot");
    assert_eq!(acquired["acquisitions"][0]["result_state"], "ready");
    assert_eq!(acquired["acquisitions"][0]["components"][0]["text"], rules);
    std::fs::write(&rule_path, rules).unwrap();
    fixture.write("candidate.md", &Fixture::declaration("DOC-001", "PROPOSED"));
    let corrected = fixture.payload("get", Some("DOC-001"));
    assert_eq!(corrected["result_state"], "ready");
    assert_ne!(corrected["generation"], generation);
}

#[test]
fn b11_missing_accepted_rule_is_disclosed_instead_of_fabricating_a_reference() {
    let fixture = Fixture::new();
    fixture.write(
        "candidate.md",
        &Fixture::declaration("DOC-001", "CERTIFIED"),
    );
    let rejected = query(&fixture, json!({"mode":"get","id":"DOC-001"}));
    assert_eq!(rejected["catalog_current"], false);
    let diagnostic = rejected["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|diagnostic| diagnostic["code"] == "E_INVALID_STATUS")
        .unwrap();
    assert_eq!(
        diagnostic["governing_authority"]["id"], "AUTH.PCG_GLS",
        "{diagnostic}"
    );
    assert_eq!(
        diagnostic["governing_authority"]["standing"],
        "accepted_reference_unavailable"
    );
    assert!(
        diagnostic["governing_authority"]
            .get("acquire_request")
            .is_none()
    );
}
