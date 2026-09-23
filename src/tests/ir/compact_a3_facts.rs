use super::{BodyFrame, decode, decode_bodies, encode, encode_body};
use serde_json::json;

#[test]
fn sparse_facts_roundtrip_imports_and_aliases() {
    let oracle = json!({
        "imports":[{"alias":"Api","module":"./api|v2","named_export":null}],
        "type_aliases":[{"alias":"Id","original_type":"string"}]
    });
    let wire = encode(&oracle).unwrap();
    let decoded = decode(&wire).unwrap();
    assert_eq!(
        decoded["imports"],
        json!([{"occurrence":0,"alias":"Api","module":"./api|v2","named_export":null}])
    );
    assert_eq!(
        decoded["type_aliases"],
        json!([{"occurrence":0,"alias":"Id","original_type":"string"}])
    );
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
fn sparse_facts_reject_malformed_rows_and_dropped_families() {
    assert!(decode("$|x\n").is_err());
    assert!(decode("T|x\n").is_err());
    assert!(decode("K|1|2|call|1\n").is_err());
    assert!(decode("fc|\"if\"|\"x\"\n").is_err());
    assert!(decode("V|1\nfc|if\n").is_err());
}
