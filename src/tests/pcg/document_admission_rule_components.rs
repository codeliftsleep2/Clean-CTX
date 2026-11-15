//! A diagnostic must retain the accepted rule after source-owned boundary splitting.
use crate::pcg::{
    document_fixture::Fixture,
    documents::{Session, knowledge},
};
use serde_json::{Value, json};

const AUTHORITY: &str = "<!-- pcg:SUMMARY:begin -->\nDeclaration authoring authority.\n<!-- pcg:SUMMARY:end -->\n<!-- pcg:RULE:begin -->\nReject duplicate declaration fields.\n<!-- pcg:RULE:end -->\n<!-- pcg:LIMIT:begin -->\nGrammar admission is not authenticated owner identity.\n<!-- pcg:LIMIT:end -->\n<!-- pcg:RECEIPT:begin -->\nHistorical fixture-only observation, not governing policy.\n<!-- pcg:RECEIPT:end -->\n```architecture-graph\nnode: AUTH.PCG_DOCUMENTS\nkind: AUTHORITY\nstatus: IMPLEMENTED\ntitle: Source-owned declaration contract\ntrack: FOUNDATION\nartifact: docs/authority.md\nknowledge_scope: authority\nknowledge_content:\n  - statement self#SUMMARY\n  - constraint:duplicate_fields self#RULE\n  - qualification self#LIMIT\n  - observation:receipt self#RECEIPT\n```\n";

fn query(fixture: &Fixture, mut args: Value) -> Value {
    args["workspaceRoot"] = json!(fixture.root);
    let request: knowledge::Request = serde_json::from_value(args).unwrap();
    Session::new(fixture.profile.clone())
        .knowledge(&request)
        .unwrap()
}

#[test]
fn b11_diagnostic_references_include_accepted_controlling_constraints_not_only_summary() {
    let fixture = Fixture::new();
    let authority = fixture.root.join("docs/authority.md");
    std::fs::write(&authority, AUTHORITY).unwrap();
    fixture.write("candidate.md", &Fixture::declaration("DOC-001", "PROPOSED"));
    let accepted = fixture.payload("get", Some("AUTH.PCG_DOCUMENTS"));
    assert_eq!(accepted["catalog_current"], true);
    let revision = accepted["documents"][0]["revision"].clone();
    let candidate = fixture.root.join("docs/candidate.md");
    let text = std::fs::read_to_string(&candidate).unwrap();
    std::fs::write(
        &candidate,
        text.replace(
            "track: ARCHITECTURE",
            "title: Duplicate\ntrack: ARCHITECTURE",
        ),
    )
    .unwrap();
    std::fs::write(
        &authority,
        AUTHORITY.replace(
            "Reject duplicate declaration fields.",
            "Duplicates are now allowed by the candidate.",
        ),
    )
    .unwrap();
    let rejected = fixture.payload("get", Some("DOC-001"));
    assert_eq!(rejected["result_state"], "blocked");
    assert_eq!(rejected["catalog_current"], false);
    assert_eq!(rejected["generation"], accepted["generation"]);
    assert!(rejected["documents"].as_array().unwrap().is_empty());
    let diagnostic = rejected["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|diagnostic| diagnostic["code"] == "E_MALFORMED_DECLARATION")
        .unwrap();
    let request = diagnostic["governing_authority"]["acquire_request"].clone();
    let references = request["references"].as_array().unwrap();
    assert!(
        references
            .iter()
            .any(|reference| reference["role"] == "constraint" && reference["selector"] == "RULE"),
        "A summary-only reference omits the independently owned governing rule: {diagnostic}"
    );
    assert!(
        references
            .iter()
            .all(|reference| reference["revision"] == revision && reference["historical"] == true)
    );
    assert!(
        !references
            .iter()
            .any(|reference| reference["role"] == "observation")
    );
    let acquired = query(&fixture, request);
    assert_eq!(acquired["catalog_current"], false);
    assert_eq!(acquired["freshness"], "historical_snapshot");
    let components: Vec<_> = acquired["acquisitions"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|item| item["components"].as_array().unwrap())
        .collect();
    assert!(
        components
            .iter()
            .any(|component| component["text"] == "Reject duplicate declaration fields.\n")
    );
    assert!(
        components.iter().any(|component| component["text"]
            == "Grammar admission is not authenticated owner identity.\n")
    );
    assert!(!components.iter().any(|component| {
        component["text"]
            .as_str()
            .unwrap()
            .contains("Duplicates are now allowed")
    }));
}
