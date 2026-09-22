use crate::compression::Fidelity;
use crate::ir::compact_a3::document::{decode, encode, target};
use crate::ir::normalize_control_full;

#[test]
fn real_checked_edit_fixture_roundtrips_overloads_duplicate_di_calls_and_body() {
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
        decoded["classes"][0]["injection_occurrences"],
        serde_json::json!([["Repo", "Repo", "Clock"], ["Repo"]])
    );
    assert_eq!(decoded["calls"][0]["occurrence"], 0);
    assert_eq!(decoded["calls"][1]["occurrence"], 1);
    assert_eq!(decoded["calls"][2]["has_spread"], true);
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
fn caller_run_encoding_preserves_global_switch_and_duplicate_order() {
    let (ir, hierarchy, edges) = super::tests::fixture();
    let mut normalized = normalize_control_full(
        &ir.file_id,
        "C:/repo/owner.ts",
        ir.version,
        Fidelity::High,
        &hierarchy,
        &edges,
    );
    let first = normalized["calls"][0].clone();
    normalized["calls"].as_array_mut().unwrap().push(first);
    normalized["calls"][3]["occurrence"] = serde_json::json!(3);
    let wire = encode(&normalized).unwrap();
    assert_eq!(String::from_utf8_lossy(&wire).matches("K|M1").count(), 2);
    let decoded = decode(&wire).unwrap();
    assert_eq!(decoded["calls"], normalized["calls"]);
}
