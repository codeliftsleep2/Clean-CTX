use super::*;
use crate::pcg::document_fixture::Fixture;
use serde_json::{Value, json};

fn source() -> String {
    "<!-- pcg:PIPE-A:begin -->\nPassPipeline has an explicit order.\n<!-- pcg:PIPE-A:end -->\n\
<!-- pcg:QUAL:begin -->\nHistorical validation is not current certification.\n<!-- pcg:QUAL:end -->\n\
<!-- pcg:WIRE-B:begin -->\nWire serialization preserves every instruction.\n<!-- pcg:WIRE-B:end -->\n\
```architecture-graph\nnode: DOC-001\nkind: INVARIANT\nstatus: IMPLEMENTED\ntitle: Pipeline order\n\
track: FOUNDATION\nartifact: docs/shared.md\ndepends_on:\n  - DOC-002\nknowledge_scope: invariant\nknowledge_content:\n  - statement self#PIPE-A\n  - qualification self#QUAL\n```\n\
```architecture-graph\nnode: DOC-002\nkind: INVARIANT\nstatus: IMPLEMENTED\ntitle: Wire preservation\n\
track: FOUNDATION\nartifact: docs/shared.md\nknowledge_scope: invariant\nknowledge_content:\n  - statement self#WIRE-B\n```\n".into()
}
fn fixture() -> Fixture {
    let f = Fixture::new();
    std::fs::write(f.root.join("docs/shared.md"), source()).unwrap();
    f
}
fn request(f: &Fixture, mode: &str, id: Option<&str>) -> knowledge::Request {
    knowledge::Request {
        workspace_root: f.root.clone(),
        mode: mode.into(),
        id: id.map(str::to_owned),
        ..Default::default()
    }
}
fn query(f: &Fixture, q: &knowledge::Request) -> Value {
    Session::new(f.profile.clone()).knowledge(q).unwrap()
}

#[test]
fn knowledge_search_isolates_siblings_and_acquires_exact_components() {
    let f = fixture();
    let mut q = request(&f, "search", None);
    q.text = Some("PassPipeline".into());
    let found = query(&f, &q);
    assert_eq!(found["result_state"], "ready");
    assert_eq!(found["units"].as_array().unwrap().len(), 1);
    assert_eq!(found["units"][0]["id"], "DOC-001");
    assert_eq!(found["units"][0]["matches"][0]["origin"], "statement");
    assert!(found["units"][0].get("components").is_none());
    assert!(!found.to_string().contains("Wire serialization"));
    assert_eq!(found["sources"], json!([]));
    q.text = Some("PassPipeline certification".into());
    let composed = query(&f, &q);
    assert_eq!(
        composed["units"][0]["matches"][0]["origin"],
        "composed_knowledge"
    );
    let unit = query(&f, &request(&f, "get", Some("DOC-001")));
    assert_eq!(
        unit["units"][0]["components"][0]["text"],
        "PassPipeline has an explicit order.\n"
    );
    assert_eq!(
        unit["units"][0]["components"][1]["text"],
        "Historical validation is not current certification.\n"
    );
    assert_eq!(unit["units"][0]["unit_complete"], true);
    assert_eq!(unit["relationships"][0]["target"], "DOC-002");
    assert!(!unit.to_string().contains("Wire serialization"));
    let raw = query(&f, &request(&f, "source", Some("DOC-001")));
    assert_eq!(raw["sources"][0]["text"], source());
    for component in unit["units"][0]["components"].as_array().unwrap() {
        let start = component["start"].as_u64().unwrap() as usize;
        let end = component["end"].as_u64().unwrap() as usize;
        assert_eq!(&source()[start..end], component["text"].as_str().unwrap());
    }
}

#[test]
fn knowledge_components_are_typed_queryable_and_match_attributed() {
    let f = fixture();
    let mut q = request(&f, "search", None);
    q.text = Some("certification".into());
    let found = query(&f, &q);
    assert_eq!(found["units"][0]["matches"][0]["origin"], "qualification");
    q.role = Some("statement".into());
    assert_eq!(query(&f, &q)["units"], json!([]));
    let mut q = request(&f, "get", Some("DOC-001"));
    q.role = Some("qualification".into());
    let acquired = query(&f, &q);
    assert_eq!(acquired["complete"], true);
    assert_eq!(acquired["units"][0]["unit_complete"], false);
    assert_eq!(
        acquired["units"][0]["components"].as_array().unwrap().len(),
        1
    );
    let mut q = request(&f, "get", Some("DOC-001"));
    q.role = Some("statement".into());
    let qualified = query(&f, &q);
    assert_eq!(
        qualified["units"][0]["components"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        qualified["units"][0]["components"][1]["role"],
        "qualification"
    );
    assert_eq!(
        qualified["units"][0]["components"][1]["selected_by_predicate"],
        false
    );
    let mut q = request(&f, "search", None);
    q.text = Some("preservation".into());
    assert_eq!(
        query(&f, &q)["units"][0]["matches"][0]["origin"],
        "identity_title"
    );
    q.kind = Some("MISSION".into());
    assert_eq!(query(&f, &q)["units"], json!([]));
}

#[test]
fn knowledge_context_preserves_declared_closure_and_relationship_direction() {
    let f = fixture();
    let ctx = query(&f, &request(&f, "context", Some("DOC-001")));
    assert_eq!(ctx["complete"], true);
    assert_eq!(ctx["units"].as_array().unwrap().len(), 2);
    assert_eq!(ctx["relationships"].as_array().unwrap().len(), 1);
    assert_eq!(ctx["relationships"][0]["meaning"], "necessary_premise");
    assert_eq!(ctx["paths"].as_array().unwrap().len(), 2);
    let mut q = request(&f, "related", Some("DOC-002"));
    q.direction = Some("incoming".into());
    q.edge_kind = Some("DEPENDS_ON".into());
    let related = query(&f, &q);
    assert_eq!(related["relationships"][0]["source"], "DOC-001");
    assert_eq!(related["relationships"][0]["direction"], "incoming");
    assert!(
        related["units"]
            .as_array()
            .unwrap()
            .iter()
            .all(|u| u.get("components").is_none())
    );
    let mut q = request(&f, "context", Some("DOC-001"));
    q.max_nodes = Some(1);
    let incomplete = query(&f, &q);
    assert_eq!(incomplete["complete"], false);
    assert_eq!(incomplete["units"], json!([]));
    assert_eq!(incomplete["relationships"], json!([]));
    assert_eq!(incomplete["diagnostics"][0]["code"], "E_CONTEXT_LIMIT");
}

#[test]
fn knowledge_boundaries_fail_atomically_and_recover_exactly() {
    let f = fixture();
    let q = request(&f, "get", Some("DOC-001"));
    let initial = query(&f, &q);
    for broken in [
        source().replace("statement self#PIPE-A", "statement self#ABSENT"),
        format!(
            "{}<!-- pcg:PIPE-A:begin -->\nforged\n<!-- pcg:PIPE-A:end -->\n",
            source()
        ),
        source().replace("<!-- pcg:PIPE-A:end -->", ""),
        source().replace("<!-- pcg:PIPE-A:end -->", "<!-- pcg:PIPE-A:finish -->"),
    ] {
        std::fs::write(f.root.join("docs/shared.md"), broken).unwrap();
        let blocked = query(&f, &q);
        assert_eq!(blocked["result_state"], "blocked");
        assert_eq!(blocked["complete"], false);
        assert_eq!(blocked["generation"], initial["generation"]);
        assert_eq!(blocked["units"], json!([]));
        assert_eq!(blocked["diagnostics"][0]["code"], "E_KNOWLEDGE_BOUNDARY");
    }
    std::fs::write(f.root.join("docs/shared.md"), source()).unwrap();
    assert_eq!(query(&f, &q), initial);
}

#[test]
fn knowledge_marker_examples_are_inert_and_shared_source_work_is_bounded() {
    let f = fixture();
    let mut text = source();
    text.push_str(
        "\n````example\n<!-- pcg:PIPE-A:begin -->\nforged\n<!-- pcg:PIPE-A:end -->\n````\n",
    );
    std::fs::write(f.root.join("docs/shared.md"), text).unwrap();
    let trusted = profile::Trusted::load(&f.profile).unwrap();
    let capture = scan::scan(&trusted).unwrap();
    assert!(capture.diagnostics.is_empty());
    assert_eq!(capture.boundary_work(), (1, 1, 1, 3, 3));
    assert_eq!(
        query(&f, &request(&f, "get", Some("DOC-001")))["result_state"],
        "ready"
    );
}

#[test]
fn knowledge_external_content_is_revision_bound_and_source_access_is_scoped() {
    let f = fixture();
    std::fs::write(f.root.join("evidence.txt"),"<!-- pcg:REASON:begin -->\nRejected for a registered counterexample.\n<!-- pcg:REASON:end -->\n").unwrap();
    let text = source().replace(
        "  - qualification self#QUAL",
        "  - qualification self#QUAL\n  - rejection_reason:REJECT evidence.txt#REASON",
    );
    std::fs::write(f.root.join("docs/shared.md"), text).unwrap();
    let q = request(&f, "get", Some("DOC-001"));
    let initial = query(&f, &q);
    assert_eq!(initial["units"][0]["components"][2]["label"], "REJECT");
    std::fs::write(f.root.join("evidence.txt"),"<!-- pcg:REASON:begin -->\nRejected; later observations must remain visible.\n<!-- pcg:REASON:end -->\n").unwrap();
    let changed = query(&f, &q);
    assert_ne!(
        changed["units"][0]["revision"],
        initial["units"][0]["revision"]
    );
    assert_eq!(
        changed["units"][0]["source_sha256"],
        initial["units"][0]["source_sha256"]
    );
    let mut q = request(&f, "source", Some("DOC-001"));
    q.path = Some("evidence.txt".into());
    assert!(
        query(&f, &q)["sources"][0]["text"]
            .as_str()
            .unwrap()
            .contains("later observations")
    );
    q.id = Some("DOC-002".into());
    assert_eq!(
        Session::new(f.profile.clone()).knowledge(&q).unwrap_err().0,
        "scope_refused"
    );
}

#[test]
fn knowledge_has_no_token_ceiling_and_unrepresented_search_is_truthfully_incomplete() {
    let f = fixture();
    let large = "x ".repeat(70_000);
    std::fs::write(
        f.root.join("docs/shared.md"),
        source().replace("PassPipeline has an explicit order.", &large),
    )
    .unwrap();
    let result = query(&f, &request(&f, "get", Some("DOC-001")));
    assert_eq!(result["result_state"], "ready");
    assert_eq!(
        result["units"][0]["components"][0]["text"]
            .as_str()
            .unwrap()
            .len(),
        large.len() + 1
    );
    let legacy = Fixture::new();
    legacy.write("a.md", &Fixture::declaration("DOC-001", "FROZEN"));
    let incomplete = query(&legacy, &request(&legacy, "search", None));
    assert_eq!(incomplete["result_state"], "incomplete");
    assert_eq!(incomplete["catalog_coverage"]["represented"], 0);
    assert_eq!(incomplete["units"], json!([]));
}

#[test]
fn knowledge_publication_detects_index_corruption_instead_of_silently_repairing_it() {
    let f = fixture();
    let q = request(&f, "get", Some("DOC-001"));
    query(&f, &q);
    let c = rusqlite::Connection::open(f.root.join("documents.sqlite")).unwrap();
    c.execute(
        "UPDATE knowledge_search SET text='forged' WHERE ordinal=0",
        [],
    )
    .unwrap();
    assert_eq!(
        Session::new(f.profile.clone()).knowledge(&q).unwrap_err().0,
        "knowledge_corruption"
    );
}

#[test]
fn knowledge_integrity_does_not_depend_on_a_legacy_response_projection() {
    let f = fixture();
    let q = request(&f, "get", Some("DOC-001"));
    let initial = query(&f, &q);
    let c = rusqlite::Connection::open(f.root.join("documents.sqlite")).unwrap();
    c.execute(
        "UPDATE document_search SET body='legacy projection damaged'",
        [],
    )
    .unwrap();
    assert_eq!(query(&f, &q), initial);
    assert_eq!(
        Session::new(f.profile.clone())
            .query(&f.request("get", Some("DOC-001")))
            .unwrap_err()
            .0,
        "document_corruption"
    );
    c.execute("UPDATE document_source SET body=X'626164'", [])
        .unwrap();
    assert_eq!(
        Session::new(f.profile.clone()).knowledge(&q).unwrap_err().0,
        "document_corruption"
    );
}

#[test]
fn knowledge_preserves_utf8_bytes_and_binary_original_evidence() {
    let f = fixture();
    let unicode = "PassPipeline λ 日本語 é e\u{301}.";
    let text = source()
        .replace("PassPipeline has an explicit order.", unicode)
        .replace("artifact: docs/shared.md", "artifact: evidence.bin");
    std::fs::write(f.root.join("docs/shared.md"), &text).unwrap();
    std::fs::write(f.root.join("evidence.bin"), [0, 255, 128, 1]).unwrap();
    let result = query(&f, &request(&f, "get", Some("DOC-001")));
    let component = &result["units"][0]["components"][0];
    assert_eq!(component["text"], format!("{unicode}\n"));
    let start = component["start"].as_u64().unwrap() as usize;
    let end = component["end"].as_u64().unwrap() as usize;
    assert_eq!(
        &text.as_bytes()[start..end],
        component["text"].as_str().unwrap().as_bytes()
    );
    let mut q = request(&f, "source", Some("DOC-001"));
    q.path = Some("evidence.bin".into());
    let raw = query(&f, &q);
    assert_eq!(raw["sources"][0]["encoding"], "byte_array");
    assert_eq!(raw["sources"][0]["bytes"], json!([0, 255, 128, 1]));
    assert_eq!(
        raw["completeness_scope"],
        "complete_original_source_artifact"
    );
    let pinned: knowledge::Request =
        serde_json::from_value(component["source_request"].clone()).unwrap();
    std::fs::write(
        f.root.join("docs/shared.md"),
        text.replace(unicode, "Changed current statement."),
    )
    .unwrap();
    assert_eq!(
        Session::new(f.profile.clone())
            .knowledge(&pinned)
            .unwrap_err()
            .0,
        "source_revision_changed"
    );
    let mut denied = vec![255];
    denied.extend_from_slice(format!("sk-{}", "a".repeat(32)).as_bytes());
    std::fs::write(f.root.join("evidence.bin"), denied).unwrap();
    assert_eq!(
        Session::new(f.profile.clone()).knowledge(&q).unwrap_err().0,
        "material_denied"
    );
}

#[test]
fn knowledge_source_and_shared_interval_work_stays_bounded_under_growth_and_refusal() {
    for size in [2, 12, 32] {
        let f = Fixture::new();
        let mut text = String::from(
            "<!-- pcg:COMMON:begin -->\nSource-attributed evidence is not execution.\n<!-- pcg:COMMON:end -->\n",
        );
        for number in 1..=size {
            text.push_str(&format!("<!-- pcg:P{number}:begin -->\nIndependent proposition {number}.\n<!-- pcg:P{number}:end -->\n\
```architecture-graph\nnode: DOC-{number:03}\nkind: MISSION\nstatus: FROZEN\ntitle: Proposition {number}\ntrack: FOUNDATION\nartifact: docs/growth.md\nknowledge_scope: proposition\nknowledge_content:\n  - statement self#P{number}\n  - qualification self#COMMON\n```\n"));
        }
        std::fs::write(f.root.join("docs/growth.md"), &text).unwrap();
        let trusted = profile::Trusted::load(&f.profile).unwrap();
        let capture = scan::scan(&trusted).unwrap();
        assert!(capture.diagnostics.is_empty(), "{:?}", capture.diagnostics);
        assert_eq!(capture.boundary_work(), (1, 1, 1, size + 1, size * 2));
        let first = &capture.nodes.values().next().unwrap().bytes;
        assert!(
            capture
                .nodes
                .values()
                .all(|node| std::sync::Arc::ptr_eq(first, &node.bytes))
        );
        // A malformed shared source is parsed once, while every affected identity refuses.
        text.push_str("<!-- pcg:COMMON:begin -->\nDuplicate\n<!-- pcg:COMMON:end -->\n");
        std::fs::write(f.root.join("docs/growth.md"), text).unwrap();
        let failed = scan::scan(&trusted).unwrap();
        assert_eq!(failed.diagnostics.len(), size);
        assert_eq!(failed.boundary_work(), (1, 1, 0, 0, 0));
    }
}
