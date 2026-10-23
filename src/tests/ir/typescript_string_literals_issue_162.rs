use super::string_literal::decode;

#[test]
fn issue_162_literal_units_escape_equivalence_and_distinction() {
    let expected: Vec<u16> = "audit.api".encode_utf16().collect();
    for raw in [r#"'audit.api'"#, r#""audit\u002eapi""#, r#"'audit\x2eapi'"#] {
        assert_eq!(decode(raw), Some(expected.clone()), "{raw}");
    }
    assert_ne!(decode(r#"'Audit.api'"#), Some(expected));
    assert_ne!(decode(r#"'\u00e9'"#), decode(r#"'e\u0301'"#));
    assert_eq!(decode(r#"'\u{1f680}'"#), decode(r#"'\ud83d\ude80'"#));
    assert_eq!(decode(r#"'\ud800'"#), Some(vec![0xd800]));
    assert_eq!(decode(r#"'\u{d800}'"#), Some(vec![0xd800]));
    assert_eq!(decode(r#"'\udfff'"#), Some(vec![0xdfff]));
    assert_eq!(
        decode(r#"'\0\b\f\n\r\t\v'"#),
        Some(vec![0, 8, 12, 10, 13, 9, 11])
    );
    assert_eq!(decode("'a\\\r\nb'"), Some(vec![97, 98]));
    assert_eq!(decode("'a\\\nb'"), Some(vec![97, 98]));
}

#[test]
fn issue_162_literal_units_refuse_invalid_or_unsupported_spellings() {
    for raw in [
        "",
        "'",
        "abc",
        "`abc`",
        "'a\nb'",
        "'a'b'",
        r#"'\u{}'"#,
        r#"'\u{110000}'"#,
        r#"'\u{ffffffffffffffffffff}'"#,
        r#"'\u12'"#,
        r#"'\x0z'"#,
        r#"'\01'"#,
        r#"'\8'"#,
        "'a\\'",
        "'abc\"",
    ] {
        assert_eq!(decode(raw), None, "{raw:?}");
    }
    assert_eq!(super::string_value::decode(r#"'\ud800'"#), None);
    assert_eq!(super::string_value::decode(r#"'\u{d800}'"#), None);
}
