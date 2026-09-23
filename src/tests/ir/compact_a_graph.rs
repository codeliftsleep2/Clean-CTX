use crate::compression::Fidelity;
use crate::ir::normalize_control_full;
use crate::layers::meta::semantic::{CallEvidence, EntityRef, SemanticEdge, SemanticRelation};
use serde_json::{Value, json};

fn entity_record(entity: &Value) -> Value {
    json!([
        entity["domain"],
        entity["entity_type"],
        entity["name"],
        entity["file"]
    ])
}

fn decode_entity(record: &Value) -> Value {
    json!({
        "domain": record[0], "entity_type": record[1],
        "name": record[2], "file": record[3]
    })
}

pub(super) fn encode_graph(oracle: &Value) -> Value {
    let calls = oracle["calls"]
        .as_array()
        .unwrap()
        .iter()
        .map(|call| {
            json!([
                call["occurrence"],
                call["caller_method_id"],
                call["callee_written_name"],
                call["explicit_argument_count"],
                call["has_spread"],
                call["callee_resolution"]
            ])
        })
        .collect::<Vec<_>>();
    let edges = oracle["semantic_edges"]
        .as_array()
        .unwrap()
        .iter()
        .map(|edge| {
            json!([
                edge["occurrence"],
                edge["relation"],
                entity_record(&edge["subject"]),
                entity_record(&edge["object"]),
                edge["layer"],
                edge["call_evidence"]
            ])
        })
        .collect::<Vec<_>>();
    json!({ "K": calls, "E": edges })
}

pub(super) fn decode_graph(encoded: &Value) -> Value {
    let calls = encoded["K"]
        .as_array()
        .unwrap()
        .iter()
        .map(|record| {
            json!({
                "occurrence": record[0], "caller_method_id": record[1],
                "callee_written_name": record[2], "explicit_argument_count": record[3],
                "has_spread": record[4], "callee_resolution": record[5]
            })
        })
        .collect::<Vec<_>>();
    let edges = encoded["E"]
        .as_array()
        .unwrap()
        .iter()
        .map(|record| {
            json!({
                "occurrence": record[0], "relation": record[1],
                "subject": decode_entity(&record[2]), "object": decode_entity(&record[3]),
                "layer": record[4], "call_evidence": record[5]
            })
        })
        .collect::<Vec<_>>();
    json!({ "calls": calls, "semantic_edges": edges })
}

#[test]
fn scoped_a1_graph_roundtrips_calls_edges_provenance_and_uncertainty() {
    let (ir, hierarchy, mut edges) = super::tests::fixture();
    edges.push(SemanticEdge {
        relation: SemanticRelation::Calls,
        subject: EntityRef::new("core", "Method", "M1").with_file("C:/repo/owner.ts".into()),
        object: EntityRef::new("core", "Function", "lookup").with_file("C:/repo/lookup.ts".into()),
        layer: "core",
        call_evidence: Some(CallEvidence::new(1, true)),
    });
    let oracle = normalize_control_full(
        &ir.file_id,
        "C:/repo/owner.ts",
        ir.version,
        Fidelity::High,
        &hierarchy,
        &edges,
    );
    let wire = serde_json::to_string(&encode_graph(&oracle)).unwrap();
    let decoded = decode_graph(&serde_json::from_str(&wire).unwrap());
    let expected = json!({ "calls": oracle["calls"], "semantic_edges": oracle["semantic_edges"] });

    assert_eq!(decoded, expected);
    assert_eq!(decoded["calls"][0]["caller_method_id"], "M1");
    assert_eq!(decoded["calls"][0]["callee_resolution"], "unresolved");
    assert_eq!(decoded["calls"][1]["occurrence"], 1);
    assert_eq!(decoded["calls"][2]["has_spread"], true);
    assert_eq!(
        decoded["semantic_edges"][0]["subject"]["file"],
        "C:/repo/owner.ts"
    );
    assert_eq!(
        decoded["semantic_edges"][0]["object"]["file"],
        "C:/repo/repo.ts"
    );
    assert_eq!(decoded["semantic_edges"][1]["relation"], "Calls");
    assert_eq!(
        decoded["semantic_edges"][1]["subject"]["file"],
        "C:/repo/owner.ts"
    );
    assert_eq!(
        decoded["semantic_edges"][1]["object"]["file"],
        "C:/repo/lookup.ts"
    );
    assert_eq!(
        decoded["semantic_edges"][1]["call_evidence"]["explicit_arg_count"],
        1
    );
    assert_eq!(
        decoded["semantic_edges"][1]["call_evidence"]["has_spread"],
        true
    );
}
