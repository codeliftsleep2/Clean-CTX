//! AF-PERF-001: operation-local work stays bounded without changing authority.
use crate::pcg::document_fixture::Fixture;
use crate::pcg::documents::{
    admission, governance_state, model::Diagnostic, profile::Trusted, scan,
};
use serde_json::json;

fn fixture() -> Fixture {
    let fixture = Fixture::new();
    let mut source = String::from(
        "<!-- pcg:POLICY:begin -->\nExact policy.\n<!-- pcg:POLICY:end -->\n<!-- pcg:LIMIT:begin -->\nOnly the declared scope.\n<!-- pcg:LIMIT:end -->\n```architecture-graph\nnode: AUTH.PCG_DOCUMENTS\nkind: AUTHORITY\nstatus: IMPLEMENTED\ntitle: Work-count fixture\ntrack: FOUNDATION\nartifact: docs/authority.md\nknowledge_scope: authority\nknowledge_content:\n  - statement self#POLICY\n  - qualification self#LIMIT\n",
    );
    for index in 0..12 {
        source.push_str(&format!("  - constraint:c{index} self#POLICY\n"));
    }
    source.push_str("```\n");
    std::fs::write(fixture.root.join("docs/authority.md"), source).unwrap();
    assert_eq!(
        fixture.payload("get", Some("AUTH.PCG_DOCUMENTS"))["result_state"],
        "ready"
    );
    fixture
}

#[test]
fn b11_governing_reference_loads_are_per_distinct_authority_not_per_diagnostic() {
    let fixture = fixture();
    let connection = rusqlite::Connection::open_with_flags(
        fixture.root.join("documents.sqlite"),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    for count in [2, 200] {
        let mut diagnostics: Vec<_> = (0..count)
            .map(|index| {
                Diagnostic::new(
                    if index % 2 == 0 {
                        "E_MALFORMED_DECLARATION"
                    } else {
                        "E_INVALID_STATUS"
                    },
                    "docs/candidate.md",
                    Some("DOC-001"),
                    "Fixture rejection",
                )
            })
            .collect();
        let work =
            admission::attach_authorities(&connection, &fixture.root, &mut diagnostics).unwrap();
        // Two requested authorities; only the document authority exists here.
        assert_eq!(work, (2, 1));
        for diagnostic in diagnostics {
            assert_eq!(diagnostic.accepted_publication, Some(false));
            let governing = diagnostic.governing_authority.unwrap();
            if diagnostic.code == "E_MALFORMED_DECLARATION" {
                assert_eq!(governing["standing"], "previously_accepted_snapshot");
                assert_eq!(
                    governing["acquire_request"]["references"]
                        .as_array()
                        .unwrap()
                        .len(),
                    14
                );
            } else {
                assert_eq!(governing["standing"], "accepted_reference_unavailable");
            }
        }
    }
}

#[test]
fn b11_protected_associations_derive_once_per_unit_without_repeated_source_reads() {
    let fixture = fixture();
    let trusted = Trusted::load(&fixture.profile).unwrap();
    let capture = scan::scan(&trusted).unwrap();
    let source_work = capture.read_work();
    assert_eq!(source_work.1, 1);
    for count in [1, 12] {
        let selectors: Vec<_> = (0..count).map(|index| json!({
            "id":"AUTH.PCG_DOCUMENTS", "role":"constraint", "label":format!("c{index}"), "selector":"POLICY",
        })).collect();
        let (view, work) =
            governance_state::project_with_work(&capture.nodes, &selectors, false).unwrap();
        assert_eq!(work, 1);
        assert_eq!(view["components"].as_array().unwrap().len(), count);
        assert_eq!(view["units"].as_array().unwrap().len(), 1);
        assert_eq!(
            view["units"][0]["qualifications"].as_array().unwrap().len(),
            1
        );
        assert_eq!(capture.read_work(), source_work);
    }
}
