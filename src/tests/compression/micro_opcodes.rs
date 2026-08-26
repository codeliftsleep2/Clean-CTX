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
fn apply_micro_opcodes_low_fidelity_replaces_guard() {
    let body = "⊕guard foo() ⊕guard bar()";
    let result = apply_micro_opcodes(body, Fidelity::Low);
    assert_eq!(result, "§I foo() §I bar()");
}

#[test]
fn apply_micro_opcodes_low_fidelity_replaces_loop() {
    let body = "⊕loop items ⊕loop users";
    let result = apply_micro_opcodes(body, Fidelity::Low);
    assert_eq!(result, "§L items §L users");
}

#[test]
fn apply_micro_opcodes_low_fidelity_replaces_return() {
    let body = "⊕⇒result ⊕⇒data";
    let result = apply_micro_opcodes(body, Fidelity::Low);
    assert_eq!(result, "§Eresult §Edata");
}

#[test]
fn apply_micro_opcodes_low_fidelity_combined() {
    let body = "Foo{field1;field2};$ctor C1 M1 $s payload;⊕guard check() ⊕⇒result";
    let result = apply_micro_opcodes(body, Fidelity::Low);
    assert_eq!(
        result,
        "Foo§Cfield1;field2§C;§P C1 M1 $s payload;§I check() §Eresult"
    );
}

#[test]
fn apply_micro_opcodes_medium_fidelity_no_change() {
    let body = "Foo{field1;field2}⊕guard⊕loop⊕⇒";
    let result = apply_micro_opcodes(body, Fidelity::Medium);
    assert_eq!(result, "Foo{field1;field2}⊕guard⊕loop⊕⇒");
}

#[test]
fn apply_micro_opcodes_high_fidelity_no_change() {
    let body = "Foo{field1;field2}⊕guard⊕loop⊕⇒";
    let result = apply_micro_opcodes(body, Fidelity::High);
    assert_eq!(result, "Foo{field1;field2}⊕guard⊕loop⊕⇒");
}

#[test]
fn apply_micro_opcodes_empty_body() {
    let result = apply_micro_opcodes("", Fidelity::Low);
    assert_eq!(result, "");
}

#[test]
fn micro_opcode_table_has_entries() {
    let table = micro_opcode_table();
    assert!(!table.is_empty());
    assert!(table.iter().any(|(op, _, _)| *op == "§C"));
    assert!(table.iter().any(|(op, _, _)| *op == "§P"));
    assert!(table.iter().any(|(op, _, _)| *op == "§I"));
    assert!(table.iter().any(|(op, _, _)| *op == "§L"));
    assert!(table.iter().any(|(op, _, _)| *op == "§E"));
}

#[test]
fn micro_opcode_table_has_all_five_entries() {
    assert_eq!(micro_opcode_table().len(), 6);
}
