use super::*;
use super::super::walker::LineKind;

#[test]
fn test_decompress_low() {
    let input = "// --- Compacted Layout (Low Fidelity): α1 ---\n$c SampleService;$ctor();$b isInitialized\n\n§PATHMAP\n  α1 = C:\\project\\Service.ts";
    let mut d = Decompressor::new();
    let result = d.quick_decompress(input);
    assert!(result.contains("class SampleService"));
    assert!(result.contains("constructor()"));
    assert!(result.contains("boolean isInitialized"));
}

#[test]
fn test_line_classification() {
    assert_eq!(classify_line_kind(""), LineKind::Blank);
    assert_eq!(classify_line_kind("   "), LineKind::Blank);
    assert_eq!(classify_line_kind("// --- header"), LineKind::Header);
    assert_eq!(classify_line_kind("§PATHMAP"), LineKind::SectionStart);
    assert_eq!(classify_line_kind("hello world"), LineKind::Body);
}

fn classify_line_kind(line: &str) -> LineKind {
    super::super::walker::classify(line)
}