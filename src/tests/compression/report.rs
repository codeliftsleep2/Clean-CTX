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

// --- Phase III (Idea #8): Progressive Header Elision tests ---------------

#[test]
fn format_final_output_uses_compact_header_for_low() {
    let source = "class Foo { bar() {} }";
    let body = "// --- Compacted Layout (Low Fidelity): α1 ---\n$c C1";
    let out = format_final_output(source, body, Fidelity::Low, 1, 1, 0);
    // Compact format: §raw:compressed:pct|L|classes:methods:imports§
    assert!(out.starts_with('§'), "Low fidelity output should start with §");
    assert!(out.contains("|L|"), "Low fidelity output should contain |L|");
    assert!(out.contains("|1:1:0§"), "Compact header should contain |1:1:0§");
    assert!(out.contains(body), "Compact header should be followed by body");
}

#[test]
fn format_final_output_uses_verbose_header_for_medium() {
    let source = "class Foo { bar() {} }";
    let body = "// --- Enhanced Layout (Medium Fidelity): α1 ---\nclass C1";
    let out = format_final_output(source, body, Fidelity::Medium, 1, 1, 0);
    assert!(out.starts_with("// --- Token Optimization Report"),
            "Medium fidelity should use verbose header");
    assert!(out.contains("// Fidelity: Medium"));
}

#[test]
fn format_final_output_uses_verbose_header_for_high() {
    let source = "class Foo { bar() {} }";
    let body = "// --- Full Layout (High Fidelity): α1 ---\nclass C1";
    let out = format_final_output(source, body, Fidelity::High, 1, 1, 0);
    assert!(out.starts_with("// --- Token Optimization Report"),
            "High fidelity should use verbose header");
    assert!(out.contains("// Fidelity: High"));
}

#[test]
fn parse_compact_header_valid() {
    let header = "§245:67:72.6|L|3:12:5§";
    let parsed = parse_compact_header(header);
    assert!(parsed.is_some(), "Valid compact header should parse");
    let (raw, compressed, pct, classes, methods, imports) = parsed.unwrap();
    assert_eq!(raw, 245);
    assert_eq!(compressed, 67);
    assert!((pct - 72.6).abs() < 0.01);
    assert_eq!(classes, 3);
    assert_eq!(methods, 12);
    assert_eq!(imports, 5);
}

#[test]
fn parse_compact_header_integer_percentage() {
    let header = "§100:50:50|L|1:0:0§";
    let parsed = parse_compact_header(header);
    assert!(parsed.is_some(), "Integer percentage header should parse");
    let (_, _, pct, ..) = parsed.unwrap();
    assert!((pct - 50.0).abs() < 0.01);
}

#[test]
fn parse_compact_header_zero_percentage() {
    let header = "§245:245:0|L|0:0:0§";
    let parsed = parse_compact_header(header);
    assert!(parsed.is_some(), "Zero percentage header should parse");
    let (raw, compressed, pct, classes, methods, imports) = parsed.unwrap();
    assert_eq!(raw, 245);
    assert_eq!(compressed, 245);
    assert!((pct - 0.0).abs() < 0.01);
    assert_eq!(classes, 0);
    assert_eq!(methods, 0);
    assert_eq!(imports, 0);
}

#[test]
fn parse_compact_header_missing_delimiters() {
    assert!(parse_compact_header("245:67:72.6|L|3:12:5").is_none(), "Missing § should fail");
    assert!(parse_compact_header("§245:67:72.6|L|3:12:5").is_none(), "Missing trailing § should fail");
}

#[test]
fn parse_compact_header_wrong_part_count() {
    assert!(parse_compact_header("§245:67:72.6|L|3:12:5|extra§").is_none(), "Extra part should fail");
    assert!(parse_compact_header("§245:67:72.6|L§").is_none(), "Missing counts should fail");
}

#[test]
fn parse_compact_header_non_numeric_fields() {
    assert!(parse_compact_header("§abc:67:72.6|L|3:12:5§").is_none(), "Non-numeric raw should fail");
    assert!(parse_compact_header("§245:67:72.6|L|a:12:5§").is_none(), "Non-numeric class should fail");
}

#[test]
fn format_savings_pct_integer() {
    assert_eq!(format_savings_pct(100.0), "100");
    assert_eq!(format_savings_pct(0.0), "0");
    assert_eq!(format_savings_pct(50.0), "50");
}

#[test]
fn format_savings_pct_fractional() {
    assert_eq!(format_savings_pct(72.65), "72.7");
    assert_eq!(format_savings_pct(33.333), "33.3");
    assert_eq!(format_savings_pct(99.99), "100.0");
}

#[test]
fn format_compact_cache_hit_format() {
    let out = format_compact_cache_hit(245, 67, 72.6, "α1");
    assert!(out.starts_with('§'), "Compact cache hit should start with §");
    assert!(out.contains("|C|"), "Compact cache hit should contain |C|");
    assert!(out.contains("α1§"), "Compact cache hit should contain alias");
    assert!(out.contains("// [CACHE_HIT]"), "Should contain cache hit notice");
}

#[test]
fn format_compact_cache_hit_integer_percentage() {
    let out = format_compact_cache_hit(100, 50, 50.0, "α2");
    assert!(out.contains("§100:50:50|C|α2§"), "Integer percentage format incorrect");
}

#[test]
fn format_compact_header_counts_roundtrip() {
    // Build a compact header from known values, then parse it back
    let source = "class A { } class B { } class C { }";
    let body = "// --- Compacted Layout (Low): α1 ---\n$c A;$c B;$c C";
    let out = format_final_output(source, body, Fidelity::Low, 3, 0, 0);
    // Extract the first line (the header)
    let first_line = out.lines().next().unwrap();
    let parsed = parse_compact_header(first_line);
    assert!(parsed.is_some(), "Should round-trip through parse_compact_header");
    let (raw, compressed, _pct, classes, methods, imports) = parsed.unwrap();
    assert_eq!(classes, 3);
    assert_eq!(methods, 0);
    assert_eq!(imports, 0);
    assert!(raw > 0, "Raw token count should be > 0");
    assert!(compressed > 0, "Compressed token count should be > 0");
}