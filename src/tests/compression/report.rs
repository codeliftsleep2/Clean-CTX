use super::*;

#[test]
fn format_compacted_body_omits_footer_when_empty() {
    let out = format_compacted_body("BODY", "", "a1", Fidelity::Low);
    assert!(out.contains("Compacted Layout (Low Fidelity): a1"));
    assert!(out.contains("BODY"));
    assert!(!out.contains("§SYM"));
}

#[test]
fn format_compacted_body_includes_footer_when_present() {
    let out = format_compacted_body("BODY", "§SYM\n  $1 = Foo", "a1", Fidelity::Low);
    assert!(out.contains("§SYM"));
}

#[test]
fn format_compacted_body_labels_medium() {
    let out = format_compacted_body("BODY", "", "b2", Fidelity::Medium);
    assert!(out.contains("Enhanced Layout (Medium Fidelity): b2"));
}

#[test]
fn format_compacted_body_labels_high() {
    let out = format_compacted_body("BODY", "", "c3", Fidelity::High);
    assert!(out.contains("Full Layout (High Fidelity): c3"));
}