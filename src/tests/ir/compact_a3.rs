use super::{decode_declarations, encode_declarations};
use serde_json::{Value, json};

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
    let mut target = json!({
        "schema":"clean-ctx/file-context","schema_version":4,
        "file":value["file"],"mode":{"fidelity":value["mode"]["fidelity"]},
        "classes":value["classes"],"interfaces":value["interfaces"]
    });
    for family in ["classes", "interfaces"] {
        for owner in target[family].as_array_mut().unwrap() {
            owner
                .as_object_mut()
                .unwrap()
                .remove("injection_occurrences");
            owner.as_object_mut().unwrap().remove("patterns");
            for method in owner["methods"].as_array_mut().unwrap() {
                let method_obj = method.as_object_mut().unwrap();
                method_obj.remove("control_summary_occurrences");
                method_obj.remove("pattern_fact_occurrences");
                method_obj.remove("legacy_flag_occurrences");
                method_obj.remove("patterns");
                method_obj.remove("control_flow");
                method_obj.remove("data_flow");
                method_obj.remove("side_effects");
                method_obj.remove("execution_contexts");
            }
        }
    }
    target
}

#[test]
fn phase1a_declarations_roundtrip_typed_identity_groups_and_delimiters() {
    let oracle = oracle();
    let encoded = encode_declarations(&oracle).expect("encode A3 declarations");
    assert!(encoded.starts_with("A3|4|H|"));
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
    assert!(decode_declarations("A3|4|H|f|1|\"p\"\np|P1|\"x\"|\"t\"\nZ|0|0|0|0\n").is_err());
    assert!(decode_declarations(&encoded.replace("Z|2|2|0|0", "Z|2|3|0|0")).is_err());
    assert!(decode_declarations(&encoded.replacen("M|2|", "M|1|", 1)).is_err());
}

#[test]
fn phase3d_single_value_groups_elide_count_and_roundtrip() {
    let oracle = oracle();
    let encoded = encode_declarations(&oracle).unwrap();
    assert!(
        encoded.contains("cm|EXPORT\n"),
        "single class modifier elides count"
    );
    assert!(
        encoded.contains("cf|ABSTRACT\n"),
        "single class flag elides count"
    );
    assert!(
        encoded.contains("mo|PUBLIC\n"),
        "single method modifier elides count"
    );
    assert!(
        encoded.contains("mo|0\n"),
        "empty group keeps explicit zero count"
    );
    assert_eq!(
        decode_declarations(&encoded).expect("decode A3 declarations"),
        declaration_target(&oracle)
    );
}

#[test]
fn phase3d_explicit_and_elided_counts_decode_identically() {
    let explicit = "A3|4|H|f|1|p\nC|1|Owner|0\nM|1|run|0|void\nmo|1|PUBLIC\nZ|1|1|0|0\n";
    let elided = "A3|4|H|f|1|p\nC|1|Owner|0\nM|1|run|0|void\nmo|PUBLIC\nZ|1|1|0|0\n";
    assert_eq!(
        decode_declarations(explicit).unwrap()["classes"][0]["methods"][0]["modifier_occurrences"],
        json!([["PUBLIC"]])
    );
    assert_eq!(
        decode_declarations(elided).unwrap(),
        decode_declarations(explicit).unwrap()
    );
}

#[test]
fn phase3d_numeric_single_value_keeps_explicit_count() {
    let numeric = json!({
        "schema":"clean-ctx/control-full","schema_version":2,
        "file":{"id":"f","source_path":"p","ir_version":1},
        "mode":{"fidelity":"high"},
        "classes":[{
            "kind":"class","id":"C1","name":"N","synthetic":false,
            "methods":[{"id":"M1","name":"run","parameters":[],"return_type":"void",
                "modifier_occurrences":[["42"]],"control_summary_occurrences":[],
                "pattern_fact_occurrences":[],"legacy_flag_occurrences":[],"patterns":[],
                "body":null,"body_start":null,"body_end":null,
                "control_flow":[],"data_flow":[],"side_effects":[],"execution_contexts":[]}],
            "fields":[],"modifier_occurrences":[],"class_flag_occurrences":[],
            "extends":null,"implements":[],"injection_occurrences":[],"patterns":[]
        }],
        "interfaces":[]
    });
    let encoded = encode_declarations(&numeric).unwrap();
    assert!(
        encoded.contains("mo|1|42\n"),
        "numeric single value keeps explicit count"
    );
    let decoded = decode_declarations(&encoded).unwrap();
    assert_eq!(
        decoded["classes"][0]["methods"][0]["modifier_occurrences"],
        json!([["42"]])
    );
}
