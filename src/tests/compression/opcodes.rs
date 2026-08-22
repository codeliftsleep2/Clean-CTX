use super::*;

#[test]
fn table_has_34_entries() {
    assert_eq!(PRIMITIVE_OPCODES.len(), 34);
}

#[test]
fn no_duplicate_opcodes() {
    let mut seen = std::collections::BTreeSet::new();
    for (op, _) in PRIMITIVE_OPCODES {
        assert!(
            seen.insert(*op),
            "duplicate opcode in PRIMITIVE_OPCODES: {}",
            op
        );
    }
}

#[test]
fn no_duplicate_tokens() {
    let mut seen = std::collections::BTreeSet::new();
    for (_, token) in PRIMITIVE_OPCODES {
        assert!(
            seen.insert(*token),
            "duplicate token in PRIMITIVE_OPCODES: {}",
            token
        );
    }
}

#[test]
fn is_primitive_opcode_works() {
    assert!(is_primitive_opcode("$a"));
    assert!(is_primitive_opcode("$ctor"));
    assert!(!is_primitive_opcode("$1"));
    assert!(!is_primitive_opcode("$99"));
    assert!(!is_primitive_opcode(""));
}

#[test]
fn builtin_opcode_map_is_consistent() {
    let map = builtin_opcode_map();
    for (op, tok) in PRIMITIVE_OPCODES {
        assert_eq!(map.get(op), Some(tok));
    }
    assert_eq!(map.len(), PRIMITIVE_OPCODES.len());
}
