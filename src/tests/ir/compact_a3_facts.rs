use super::{decode, decode_bodies, encode, encode_body, BodyFrame};
use serde_json::json;

#[test]
fn sparse_facts_roundtrip_calls_behavior_imports_and_aliases() {
    let oracle = json!({
        "mode":{"fidelity":"high"},
        "classes":[{"methods":[{"id":"M1","control_flow":[["if","x|y"]],"data_flow":[],"side_effects":["io"],"execution_contexts":[]}]}],
        "interfaces":[],
        "calls":[
            {"caller_method_id":"M1","callee_written_name":"find|one","explicit_argument_count":1,"has_spread":false,"callee_resolution":"unresolved"},
            {"caller_method_id":"M1","callee_written_name":"audit","explicit_argument_count":1,"has_spread":true,"callee_resolution":"unresolved"}
        ],
        "imports":[{"alias":"Api","module":"./api|v2","named_export":null}],
        "type_aliases":[{"alias":"Id","original_type":"string"}]
    });
    let wire = encode(&oracle).unwrap();
    assert_eq!(wire.matches("K|M1").count(), 1);
    assert!(!wire.contains("unresolved"));
    let decoded = decode(&wire).unwrap();
    assert_eq!(
        decoded["calls"],
        json!([
            {"occurrence":0,"caller_method_id":"M1","callee_written_name":"find|one","explicit_argument_count":1,"has_spread":false,"callee_resolution":"unresolved"},
            {"occurrence":1,"caller_method_id":"M1","callee_written_name":"audit","explicit_argument_count":1,"has_spread":true,"callee_resolution":"unresolved"}
        ])
    );
    assert_eq!(
        decoded["imports"],
        json!([{"occurrence":0,"alias":"Api","module":"./api|v2","named_export":null}])
    );
    assert_eq!(
        decoded["type_aliases"],
        json!([{"occurrence":0,"alias":"Id","original_type":"string"}])
    );
    assert_eq!(decoded["behavior"][0]["method_id"], "M1");
    assert_eq!(decoded["behavior"][0]["value"], json!(["if", "x|y"]));
}

#[test]
fn exact_body_frames_preserve_bytes_and_reject_truncation() {
    let frame = BodyFrame {
        method_id: "M1".into(),
        start: 7,
        end: 45,
        body: "{\r\n  return `B|fake`; // 🦀\r\n}".into(),
    };
    let wire = encode_body(&frame);
    assert_eq!(decode_bodies(&wire).unwrap(), vec![frame]);
    assert!(decode_bodies(&wire[..wire.len() - 2]).is_err());
}

#[test]
fn sparse_facts_reject_scope_and_resolution_corruption() {
    assert!(decode("k|\"call\"|1\n").is_err());
    assert!(decode("fc|0|\"if\"|\"x\"\n").is_err());
    assert!(decode("V|M1\nfc|1|\"if\"|\"x\"\n").is_err());
    let invalid = json!({"mode":{"fidelity":"high"},"classes":[],"interfaces":[],"calls":[{"caller_method_id":"M1","callee_written_name":"x","explicit_argument_count":0,"has_spread":false,"callee_resolution":"resolved"}],"imports":[],"type_aliases":[]});
    assert!(encode(&invalid).is_err());
}
