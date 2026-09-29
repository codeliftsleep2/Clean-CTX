use crate::compression::Fidelity;
use crate::ir::opcodes::CoreOp;
use crate::ir::{
    CompiledIR, normalize_control_full, render_control_full, semantic_edge_navigation,
};
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};

pub(super) fn fixture() -> (CompiledIR, crate::ir::HierarchicalIR, Vec<SemanticEdge>) {
    let ir = CompiledIR {
        file_id: "alpha".into(),
        version: 7,
        instructions: vec![
            CoreOp::DefClass("C1".into(), "Owner".into()),
            CoreOp::Injects(
                "C1".into(),
                vec!["Repo".into(), "Repo".into(), "Clock".into()],
            ),
            CoreOp::Injects("C1".into(), vec!["Repo".into()]),
            CoreOp::DefMethod("C1".into(), "M1".into(), "find".into()),
            CoreOp::Param("M1".into(), "P1".into(), "string".into(), "key".into()),
            CoreOp::Body(
                "M1".into(),
                "{ return lookup(key); }".into(),
                Some(41),
                Some(64),
            ),
            CoreOp::DefMethod("C1".into(), "M2".into(), "find".into()),
            CoreOp::Param("M2".into(), "P2".into(), "string".into(), "key".into()),
            CoreOp::Call("M1".into(), "lookup".into(), 1, false),
            CoreOp::Call("M1".into(), "lookup".into(), 1, false),
            CoreOp::Call("M2".into(), "external".into(), 1, true),
        ],
    };
    let hierarchy = crate::ir::hierarchical::try_ir_to_hierarchical(&ir).unwrap();
    let edges = vec![SemanticEdge {
        relation: SemanticRelation::Injects,
        subject: EntityRef::new("angular", "Service", "Owner").with_file("C:/repo/owner.ts".into()),
        object: EntityRef::new("angular", "Service", "Repo").with_file("C:/repo/repo.ts".into()),
        layer: "angular",
        call_evidence: None,
    }];
    (ir, hierarchy, edges)
}

#[test]
fn normalizer_preserves_ids_occurrences_unresolved_calls_and_provenance() {
    let (ir, hierarchy, edges) = fixture();
    let normalized = normalize_control_full(
        &ir.file_id,
        "C:/repo/owner.ts",
        ir.version,
        Fidelity::Edit,
        &hierarchy,
        &edges,
    );

    assert_eq!(normalized["classes"][0]["id"], "C1");
    assert_eq!(normalized["classes"][0]["methods"][0]["id"], "M1");
    assert_eq!(normalized["classes"][0]["methods"][1]["id"], "M2");
    assert_eq!(
        normalized["classes"][0]["injection_occurrences"],
        serde_json::json!([["Repo", "Repo", "Clock"], ["Repo"]])
    );
    assert_eq!(normalized["calls"].as_array().unwrap().len(), 3);
    assert_eq!(normalized["calls"][0]["occurrence"], 0);
    assert_eq!(normalized["calls"][1]["occurrence"], 1);
    assert_eq!(normalized["calls"][2]["has_spread"], true);
    assert_eq!(normalized["calls"][2]["callee_resolution"], "unresolved");
    assert_eq!(
        normalized["semantic_edges"][0]["subject"]["file"],
        "C:/repo/owner.ts"
    );
    assert_eq!(
        normalized["semantic_edges"][0]["object"]["file"],
        "C:/repo/repo.ts"
    );
    assert_eq!(
        normalized["classes"][0]["methods"][0]["body"],
        "{ return lookup(key); }"
    );
    assert_eq!(normalized["classes"][0]["methods"][0]["body_start"], 41);
    assert_eq!(normalized["classes"][0]["methods"][0]["body_end"], 64);
}

#[test]
fn rendered_control_full_is_the_pretty_form_of_the_exact_oracle() {
    let (ir, hierarchy, edges) = fixture();
    let normalized = normalize_control_full(
        &ir.file_id,
        "C:/repo/owner.ts",
        ir.version,
        Fidelity::High,
        &hierarchy,
        &edges,
    );
    let rendered = render_control_full(
        &ir.file_id,
        "C:/repo/owner.ts",
        ir.version,
        Fidelity::High,
        &hierarchy,
        &edges,
    );
    let (_, json) = rendered.split_once('\n').unwrap();
    let reparsed: serde_json::Value = serde_json::from_str(json).unwrap();
    assert_eq!(reparsed, normalized);
}

#[test]
fn navigation_uses_typed_identity_locators_without_array_indices() {
    let (ir, hierarchy, edges) = fixture();
    let normalized = normalize_control_full(
        &ir.file_id,
        "C:/repo/owner.ts",
        ir.version,
        Fidelity::High,
        &hierarchy,
        &edges,
    );

    assert_eq!(normalized["schema_version"], 2);
    assert_eq!(
        normalized["navigation"]["schema"],
        "clean-ctx/control-full-navigation"
    );
    let groups = normalized["navigation"]["occurrence_groups"]
        .as_array()
        .unwrap();
    let method = groups
        .iter()
        .find(|entry| {
            entry["locator"]["member"]["id"] == "M1"
                && entry["locator"]["field"]["name"] == "control_summary_occurrences"
        })
        .unwrap();
    assert_eq!(method["locator"]["owner"]["id"], "C1");
    assert_eq!(method["locator"]["owner"]["kind"], "class");
    assert_eq!(method["semantics"]["outer_array"], "ordered_occurrences");

    let edge = &normalized["navigation"]["semantic_edge_provenance"][0];
    assert_eq!(edge["locator"]["collection"], "semantic_edges");
    assert_eq!(edge["locator"]["relation"], "Injects");
    assert_eq!(edge["locator"]["subject"]["name"], "Owner");
    assert_eq!(edge["locator"]["object"]["name"], "Repo");
    assert_eq!(
        edge["endpoint_fields"]["subject_file"],
        serde_json::json!({ "endpoint": "subject", "field": "file" })
    );
    assert_eq!(
        edge["endpoint_fields"]["object_file"],
        serde_json::json!({ "endpoint": "object", "field": "file" })
    );
    let navigation = serde_json::to_string(&normalized["navigation"]).unwrap();
    assert!(!navigation.contains("classes["));
    assert!(!navigation.contains("semantic_edges["));
}

#[test]
fn navigation_is_stable_across_fidelity_and_edge_collection_names() {
    let (ir, hierarchy, edges) = fixture();
    let expected = normalize_control_full(
        &ir.file_id,
        "C:/repo/owner.ts",
        ir.version,
        Fidelity::Low,
        &hierarchy,
        &edges,
    )["navigation"]
        .clone();

    for fidelity in [Fidelity::Medium, Fidelity::High, Fidelity::Edit] {
        let normalized = normalize_control_full(
            &ir.file_id,
            "C:/repo/owner.ts",
            ir.version,
            fidelity,
            &hierarchy,
            &edges,
        );
        assert_eq!(normalized["navigation"], expected);
    }

    let after_apply = semantic_edge_navigation(&edges, "semantic_edges_after_apply");
    assert_eq!(
        after_apply[0]["locator"]["collection"],
        "semantic_edges_after_apply"
    );
    assert_eq!(after_apply[0]["locator"]["occurrence"], 0);
    assert_eq!(after_apply[0]["locator"]["subject"]["name"], "Owner");
    assert_eq!(after_apply[0]["locator"]["object"]["name"], "Repo");
}
