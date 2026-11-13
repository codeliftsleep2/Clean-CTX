use super::*;

fn assert_structural_unit(unit: &Value) {
    let components = unit["components"].as_array().unwrap();
    assert!(
        !components.is_empty(),
        "component references must remain available"
    );
    for component in components {
        assert!(
            component.get("text").is_none(),
            "graph retrieval must not acquire component text"
        );
        for field in [
            "ordinal",
            "role",
            "label",
            "path",
            "source_sha256",
            "start",
            "end",
            "content_sha256",
            "selector",
        ] {
            assert!(
                component.get(field).is_some(),
                "missing exact component descriptor: {field}"
            );
        }
        assert!(
            component["source_sha256"]
                .as_str()
                .is_some_and(|s| s.len() == 64)
        );
        assert!(
            component["content_sha256"]
                .as_str()
                .is_some_and(|s| s.len() == 64)
        );
    }
    for field in [
        "id",
        "revision",
        "path",
        "source_sha256",
        "scope",
        "kind",
        "title",
        "status",
        "evidence",
    ] {
        assert!(
            unit.get(field).is_some(),
            "missing structural unit field: {field}"
        );
    }
    assert!(unit.get("source_text").is_none());
    for association in unit["evidence"].as_array().unwrap() {
        assert!(association.get("text").is_none());
        assert!(association.get("source_request").is_some());
    }
}

#[test]
fn reference_first_get_preserves_descriptors_without_acquiring_content() {
    let f = fixture();
    let result = query(&f, &request(&f, "get", Some("DOC-001")));
    assert_eq!(result["result_state"], "ready");
    assert_eq!(result["catalog_current"], true);
    assert_eq!(result["complete"], true);
    assert_eq!(result["sources"], json!([]));
    assert_eq!(result["units"][0]["id"], "DOC-001");
    assert_eq!(result["relationships"][0]["source"], "DOC-001");
    assert_eq!(result["relationships"][0]["kind"], "DEPENDS_ON");
    assert_eq!(result["relationships"][0]["target"], "DOC-002");
    assert_eq!(
        result["units"][0]["components"].as_array().unwrap().len(),
        2
    );
    assert_eq!(result["units"][0]["components"][1]["role"], "qualification");
    assert_structural_unit(&result["units"][0]);
    assert!(
        !result
            .to_string()
            .contains("PassPipeline has an explicit order.")
    );
    assert!(
        !result
            .to_string()
            .contains("Historical validation is not current certification.")
    );
}

#[test]
fn reference_first_context_preserves_graph_without_acquiring_content() {
    let f = fixture();
    let result = query(&f, &request(&f, "context", Some("DOC-001")));
    assert_eq!(result["result_state"], "ready");
    assert_eq!(result["catalog_current"], true);
    assert_eq!(result["complete"], true);
    assert_eq!(result["sources"], json!([]));
    let units = result["units"].as_array().unwrap();
    assert_eq!(
        units
            .iter()
            .map(|u| u["id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["DOC-001", "DOC-002"]
    );
    assert_eq!(result["relationships"].as_array().unwrap().len(), 1);
    assert_eq!(result["relationships"][0]["source"], "DOC-001");
    assert_eq!(result["relationships"][0]["target"], "DOC-002");
    assert_eq!(result["relationships"][0]["kind"], "DEPENDS_ON");
    assert_eq!(result["paths"].as_array().unwrap().len(), 2);
    for unit in units {
        assert_structural_unit(unit);
    }
    assert!(
        !result
            .to_string()
            .contains("Wire serialization preserves every instruction.")
    );
    // Explicit full-source acquisition remains distinct and available.
    let source = query(&f, &request(&f, "source", Some("DOC-001")));
    assert_eq!(source["sources"][0]["text"], super::source());
}
