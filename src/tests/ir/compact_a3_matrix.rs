use crate::compression::Fidelity;
use crate::ir::compact_a3::document::{decode, encode, target};
use crate::ir::normalize_control_full;

#[test]
fn real_checked_edit_fixture_roundtrips_overloads_and_body() {
    let (ir, hierarchy, edges) = super::tests::fixture();
    let normalized = normalize_control_full(
        &ir.file_id,
        "C:/repo/owner.ts",
        ir.version,
        Fidelity::Edit,
        &hierarchy,
        &edges,
    );
    let wire = encode(&normalized).expect("encode real normalized Edit fixture");
    let decoded = decode(&wire).expect("decode real normalized Edit fixture");
    assert_eq!(decoded, target(&normalized).unwrap());
    assert_eq!(decoded["classes"][0]["methods"][0]["id"], "M1");
    assert_eq!(decoded["classes"][0]["methods"][1]["id"], "M2");
    assert_eq!(
        decoded["mode"]["exact_body_method_ids"],
        serde_json::json!(["M1"])
    );
}

#[test]
fn high_projection_never_leaks_canonical_body_facts() {
    let (ir, hierarchy, edges) = super::tests::fixture();
    let normalized = normalize_control_full(
        &ir.file_id,
        "C:/repo/owner.ts",
        ir.version,
        Fidelity::High,
        &hierarchy,
        &edges,
    );
    assert!(!normalized["classes"][0]["methods"][0]["body"].is_null());
    let wire = encode(&normalized).expect("encode real normalized High fixture");
    let decoded = decode(&wire).expect("decode real normalized High fixture");
    assert_eq!(decoded, target(&normalized).unwrap());
    assert!(decoded["classes"][0]["methods"][0]["body"].is_null());
    assert_eq!(
        decoded["mode"]["exact_body_method_ids"],
        serde_json::json!([])
    );
    assert!(!String::from_utf8_lossy(&wire).contains("{ return lookup(key); }"));
}

#[test]
fn low_retains_parameters_for_same_owner_overload_family() {
    let (ir, hierarchy, edges) = super::tests::fixture();
    let normalized = normalize_control_full(
        &ir.file_id,
        "C:/repo/owner.ts",
        ir.version,
        Fidelity::Low,
        &hierarchy,
        &edges,
    );
    let decoded = decode(&encode(&normalized).unwrap()).unwrap();
    assert_eq!(decoded, target(&normalized).unwrap());
    assert_eq!(
        decoded["classes"][0]["methods"][0]["parameters"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        decoded["classes"][0]["methods"][1]["parameters"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}
