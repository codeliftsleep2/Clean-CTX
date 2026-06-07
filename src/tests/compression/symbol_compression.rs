use super::*;

#[test]
fn apply_symbol_compression_skips_at_medium() {
    let (body, footer) = apply_symbol_compression("hello world", Fidelity::Medium);
    assert_eq!(body, "hello world");
    assert!(footer.is_empty());
}

#[test]
fn apply_symbol_compression_skips_at_high() {
    let (body, footer) = apply_symbol_compression("hello world", Fidelity::High);
    assert_eq!(body, "hello world");
    assert!(footer.is_empty());
}

#[test]
fn apply_symbol_compression_encodes_at_low() {
    let (body, _footer) = apply_symbol_compression("hello hello world", Fidelity::Low);
    assert!(!body.contains("hello"), "expected 'hello' to be encoded, got: {}", body);
    assert!(body.contains("world"), "expected 'world' to remain, got: {}", body);
    assert!(body.starts_with("$1 "), "expected encoded body to start with $1, got: {}", body);
}