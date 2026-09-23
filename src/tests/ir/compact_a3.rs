use super::{decode_declarations, encode_declarations};
use serde_json::{json, Value};

fn method(id: &str, name: &str, parameters: Value) -> Value {
    json!({
        "id":id,"name":name,"parameters":parameters,"return_type":"Result|Value",
        "modifier_occurrences":[["PUBLIC"],[]],"control_summary_occurrences":[["IF","RET"]],
        "pattern_fact_occurrences":[[{"k":"OBSERVABLE"}]],"legacy_flag_occurrences":[],
        "patterns":[{"name":"PIPE|SAFE","args":[id]}],"body":null,"body_start":null,
        "body_end":null,"control_flow":[],"data_flow":[],"side_effects":[],
        "execution_contexts":[]
    })
}

fn oracle() -> Value {
    json!({
        "schema":"clean-ctx/control-full","schema_version":2,
        "file":{"id":"α1","source_path":"C:\\repo\\pipe|fixture.ts","ir_version":7},
        "mode":{"fidelity":"high"},
        "classes":[{
            "kind":"class","id":"C1","name":"Pipe|Owner","synthetic":false,
            "methods":[method("M1","run|first",json!([{"id":"P1","name":"input|one","type":"string"}])),method("M2","run|second",json!([{"id":"P2","name":"input","type":null}]))],
            "fields":[{"id":"F1","name":"value|field","type":"Value"}],
            "modifier_occurrences":[["EXPORT"],[]],"class_flag_occurrences":[["ABSTRACT"]],
            "extends":"Base|Owner","implements":["Runner"],
            "injection_occurrences":[["Repository"],["Repository"]],
            "patterns":[{"name":"SERVICE","args":["C1"]}]
        }],
        "interfaces":[{
            "kind":"interface","id":"I1","name":"Runner|Api","methods":[],"fields":[],
            "modifier_occurrences":[["EXPORT"]],"extends":["BaseApi"]
        }]
    })
}

fn declaration_target(value: &Value) -> Value {
    json!({
        "schema":"clean-ctx/file-context","schema_version":3,
        "file":value["file"],"mode":{"fidelity":value["mode"]["fidelity"]},
        "classes":value["classes"],"interfaces":value["interfaces"]
    })
}

#[test]
fn phase1a_declarations_roundtrip_typed_identity_groups_and_delimiters() {
    let oracle = oracle();
    let encoded = encode_declarations(&oracle).expect("encode A3 declarations");
    assert!(encoded.starts_with("A3|3|H|"));
    assert!(!encoded.contains("\"parameters\""));
    assert_eq!(
        decode_declarations(&encoded).expect("decode A3 declarations"),
        declaration_target(&oracle)
    );
}

#[test]
fn phase1a_declarations_reject_truncation_scope_and_count_corruption() {
    let encoded = encode_declarations(&oracle()).unwrap();
    assert!(decode_declarations(encoded.trim_end_matches("Z|2|2|0|0\n")).is_err());
    assert!(decode_declarations("A3|3|H|f|1|\"p\"\np|P1|\"x\"|\"t\"\nZ|0|0|0|0\n").is_err());
    assert!(decode_declarations(&encoded.replace("Z|2|2|0|0", "Z|2|3|0|0")).is_err());
    assert!(decode_declarations(&encoded.replacen("M|2|", "M|1|", 1)).is_err());
}
