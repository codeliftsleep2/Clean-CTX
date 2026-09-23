use super::{decode, decode_cold, encode, encode_cold, target};
use serde_json::json;

fn oracle() -> serde_json::Value {
    json!({
        "schema":"clean-ctx/control-full","schema_version":2,
        "file":{"id":"α1","source_path":"C:\\repo\\complete.ts","ir_version":4},
        "mode":{"fidelity":"edit","exact_body_method_ids":["M1"],"source_requirement":"request edit or verbatim when exact source is required"},
        "classes":[{
            "kind":"class","id":"C1","name":"Complete","synthetic":false,
            "fields":[{"id":"F1","name":"repo","type":"Repository"}],
            "modifier_occurrences":[["EXPORT"]],"class_flag_occurrences":[],
            "extends":null,"implements":["Runner"],"injection_occurrences":[["Repository"]],
            "patterns":[{"name":"SERVICE","args":["C1"]}],
            "methods":[{
                "id":"M1","name":"run","parameters":[{"id":"P1","name":"value","type":"string"}],
                "return_type":"Promise<Result>","modifier_occurrences":[["PUBLIC","ASYNC"]],
                "control_summary_occurrences":[["IF","RET"]],"pattern_fact_occurrences":[[{"k":"OBSERVABLE"}]],
                "legacy_flag_occurrences":[],"patterns":[{"name":"OBSERVABLE","args":["M1"]}],
                "body":"{\r\n  return this.repo.find(value); // 🦀 | Z\r\n}","body_start":40,"body_end":96,
                "control_flow":[["return","Result"]],"data_flow":[["reads","F1"]],
                "side_effects":["io"],"execution_contexts":["async"]
            }]
        }],
        "interfaces":[{"kind":"interface","id":"I1","name":"Runner","methods":[],"fields":[],"modifier_occurrences":[],"extends":[]}],
        "imports":[{"occurrence":0,"alias":"Repository","module":"./repo","named_export":"Repository"}],
        "type_aliases":[{"occurrence":0,"alias":"Result","original_type":"string"}],
        "calls":[
            {"occurrence":0,"caller_method_id":"M1","callee_written_name":"find","explicit_argument_count":1,"has_spread":false,"callee_resolution":"unresolved"},
            {"occurrence":1,"caller_method_id":"M1","callee_written_name":"audit","explicit_argument_count":1,"has_spread":true,"callee_resolution":"unresolved"}
        ],
        "semantic_edges":[],"navigation":{}
    })
}

#[test]
fn phase1c_complete_edit_document_roundtrips_normalized_target() {
    let oracle = oracle();
    let wire = encode(&oracle).expect("encode complete A3 document");
    let decoded = decode(&wire).expect("decode complete A3 document");
    assert_eq!(decoded, target(&oracle).unwrap());
    assert_eq!(
        decoded["classes"][0]["methods"][0]["body"],
        oracle["classes"][0]["methods"][0]["body"]
    );
    assert_eq!(
        wire.windows(b"K|1".len())
            .filter(|window| *window == b"K|1")
            .count(),
        1
    );
    assert!(!String::from_utf8_lossy(&wire).contains("unresolved"));
}

#[test]
fn cold_document_requires_the_exact_versioned_legend() {
    let oracle = oracle();
    let wire = encode_cold(&oracle).unwrap();
    assert_eq!(decode_cold(&wire).unwrap(), target(&oracle).unwrap());
    let corrupted = String::from_utf8(wire)
        .unwrap()
        .replacen("S A3", "S A4", 1)
        .into_bytes();
    assert!(decode_cold(&corrupted).is_err());
}

#[test]
fn phase1c_document_rejects_terminal_body_and_reference_corruption() {
    let wire = encode(&oracle()).unwrap();
    let bad_count = String::from_utf8_lossy(&wire)
        .replace("Z|2|1|2|1", "Z|2|1|3|1")
        .into_bytes();
    assert!(decode(&bad_count).is_err());
    assert!(decode(&wire[..wire.len() - 4]).is_err());
    let bad_call = String::from_utf8_lossy(&wire)
        .replace("K|1", "K|404")
        .into_bytes();
    assert!(decode(&bad_call).is_err());
}

#[test]
fn phase2_low_medium_high_have_distinct_declared_targets() {
    let base = oracle();
    let mut low = base.clone();
    low["mode"]["fidelity"] = json!("low");
    let mut medium = base.clone();
    medium["mode"]["fidelity"] = json!("medium");
    let mut high = base;
    high["mode"]["fidelity"] = json!("high");

    let low_wire = encode(&low).unwrap();
    let medium_wire = encode(&medium).unwrap();
    let high_wire = encode(&high).unwrap();
    let low_decoded = decode(&low_wire).unwrap();
    let medium_decoded = decode(&medium_wire).unwrap();
    let high_decoded = decode(&high_wire).unwrap();

    assert_eq!(low_decoded, target(&low).unwrap());
    assert_eq!(medium_decoded, target(&medium).unwrap());
    assert_eq!(high_decoded, target(&high).unwrap());
    assert_eq!(
        low_decoded["classes"][0]["methods"][0]["parameters"],
        json!([])
    );
    assert_eq!(
        medium_decoded["classes"][0]["methods"][0]["parameters"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        medium_decoded["classes"][0]["methods"][0]["control_flow"],
        json!([])
    );
    assert_eq!(
        high_decoded["classes"][0]["methods"][0]["control_flow"],
        json!([["return", "Result"]])
    );
    assert!(low_wire.len() < medium_wire.len());
    assert!(medium_wire.len() < high_wire.len());
}
