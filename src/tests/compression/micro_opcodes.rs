use super::*;

#[test]
fn apply_micro_opcodes_low_fidelity_replaces_braces() {
    let body = "Foo{field1;field2}";
    let result = apply_micro_opcodes(body, Fidelity::Low);
    assert_eq!(result, "Foo§Cfield1;field2§C");
}

#[test]
fn apply_micro_opcodes_low_fidelity_replaces_ctor() {
    let body = "$ctor C1 M1 $s payload";
    let result = apply_micro_opcodes(body, Fidelity::Low);
    assert_eq!(result, "§P C1 M1 $s payload");
}

#[test]
fn apply_micro_opcodes_low_fidelity_combined() {
    let body = "Foo{field1;field2};$ctor C1 M1 $s payload";
    let result = apply_micro_opcodes(body, Fidelity::Low);
    assert_eq!(result, "Foo§Cfield1;field2§C;§P C1 M1 $s payload");
}

#[test]
fn apply_micro_opcodes_medium_fidelity_no_change() {
    let body = "Foo{field1;field2}";
    let result = apply_micro_opcodes(body, Fidelity::Medium);
    assert_eq!(result, "Foo{field1;field2}");
}

#[test]
fn apply_micro_opcodes_high_fidelity_no_change() {
    let body = "Foo{field1;field2}";
    let result = apply_micro_opcodes(body, Fidelity::High);
    assert_eq!(result, "Foo{field1;field2}");
}

#[test]
fn apply_micro_opcodes_empty_body() {
    let result = apply_micro_opcodes("", Fidelity::Low);
    assert_eq!(result, "");
}

#[test]
fn expand_micro_opcodes_restores_braces() {
    let body = "Foo§Cfield1;field2§C";
    let result = expand_micro_opcodes(body);
    assert_eq!(result, "Foo{field1;field2}");
}

#[test]
fn expand_micro_opcodes_restores_ctor() {
    let body = "§P C1 M1 $s payload";
    let result = expand_micro_opcodes(body);
    assert_eq!(result, "$ctor C1 M1 $s payload");
}

#[test]
fn expand_micro_opcodes_combined() {
    let body = "Foo§Cfield1;field2§C;§P C1 M1 $s payload";
    let result = expand_micro_opcodes(body);
    assert_eq!(result, "Foo{field1;field2};$ctor C1 M1 $s payload");
}

#[test]
fn round_trip_micro_opcodes() {
    let original = "Foo{field1;field2};$ctor C1 M1 $s payload";
    let compressed = apply_micro_opcodes(original, Fidelity::Low);
    let expanded = expand_micro_opcodes(&compressed);
    assert_eq!(expanded, original);
}

#[test]
fn round_trip_multiple_classes() {
    let original = "Foo{field1;field2};Bar{field3};$ctor C1 M1 $s payload;$ctor C2 M2 $s data";
    let compressed = apply_micro_opcodes(original, Fidelity::Low);
    let expanded = expand_micro_opcodes(&compressed);
    assert_eq!(expanded, original);
}

#[test]
fn round_trip_no_fields() {
    let original = "Foo{}";
    let compressed = apply_micro_opcodes(original, Fidelity::Low);
    let expanded = expand_micro_opcodes(&compressed);
    assert_eq!(expanded, original);
}

#[test]
fn micro_opcode_table_has_entries() {
    let table = micro_opcode_table();
    assert!(!table.is_empty());
    assert!(table.iter().any(|(op, _, _)| *op == "§C"));
    assert!(table.iter().any(|(op, _, _)| *op == "§P"));
}