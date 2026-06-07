use super::*;

#[test]
fn test_symbol_dictionary_basic() {
    let mut sd = SymbolDictionary::new();

    // Built-in primitives
    assert_eq!(sd.get_opcode("async"), Some("$a"));
    assert_eq!(sd.get_opcode("class"), Some("$c"));

    // Auto-register on second occurrence
    sd.register("CustomType");
    assert_eq!(sd.get_opcode("CustomType"), None);
    sd.register("CustomType");
    assert_eq!(sd.get_opcode("CustomType"), Some("$1"));
}

#[test]
fn test_encode() {
    let mut sd = SymbolDictionary::new();
    sd.register("CustomType");
    sd.register("CustomType");

    // `async` and `function` are both built-in primitives; only
    // `CustomType` gets auto-registered on its second occurrence.
    let encoded = sd.encode("async function CustomType");
    assert_eq!(encoded, "$a $fn $1");
}

#[test]
fn default_impl_works() {
    let sd = SymbolDictionary::default();
    assert_eq!(sd.get_opcode("async"), Some("$a"));
}