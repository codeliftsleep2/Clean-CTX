use super::*;

#[test]
fn low_fidelity_suppresses_fields() {
    assert_eq!(extract_field("private readonly userId: string = '';", Fidelity::Low), "");
}

#[test]
fn medium_fidelity_keeps_name_and_type() {
    let out = extract_field("private readonly userId: string = '';", Fidelity::Medium);
    assert_eq!(out, "userId:string");
}

#[test]
fn high_fidelity_strips_only_initialiser() {
    let out = extract_field("private readonly userId: string = '';", Fidelity::High);
    assert!(out.starts_with("private readonly userId"));
    assert!(!out.contains("''"));
}