use super::super::walker::LineKind;
use super::*;

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

// ---------- word_boundary_replace (F-06) ----------

use super::word_boundary_replace;

#[test]
fn replaces_at_ascii_word_boundary() {
    // The headline positive case: a "$ctor" surrounded by spaces
    // expands, but a "c$ctor" does not.
    assert_eq!(
        word_boundary_replace("a $ctor b", "$ctor", "constructor"),
        "a constructor b"
    );
    assert_eq!(
        word_boundary_replace("a$ctorb", "$ctor", "constructor"),
        "a$ctorb"
    );
}

#[test]
fn replaces_at_unicode_word_boundary() {
    // F-06 regression: the match MUST be replaced when surrounded
    // by non-alphanumeric Unicode (here, a space). The previous
    // ASCII-only check would have produced the same result by
    // coincidence, but the *real* test is the one below
    // (`does_not_replace_when_neighbouring_unicode_alpha`).
    let out = word_boundary_replace("naïve $ctor", "$ctor", "constructor");
    assert_eq!(out, "naïve constructor");
}

#[test]
fn does_not_replace_inside_unicode_word() {
    // The reverse case: the match should NOT be replaced if it's
    // surrounded by Unicode word chars. This is the case the
    // F-06 ASCII-only implementation would have handled
    // correctly (because ASCII 'e' is alphanumeric), so the new
    // char-based check has to keep that behaviour. The previous
    // test ensures the match *is* replaced when the char before
    // is non-alphanumeric (a space).
    let out = word_boundary_replace("naïve$ctor", "$ctor", "constructor");
    assert_eq!(out, "naïve$ctor");
}

#[test]
fn does_not_replace_when_neighbouring_unicode_alpha() {
    // The F-06 headline test: the ASCII-only implementation
    // would have treated 'é' as "not alphanumeric" and replaced
    // the match. The new char-based check sees 'é' as
    // alphanumeric (via `char::is_alphanumeric`) and keeps the
    // match intact.
    let out = word_boundary_replace("café$ctor", "$ctor", "constructor");
    assert_eq!(out, "café$ctor");
}

#[test]
fn respects_multi_byte_char_boundaries() {
    // A 4-byte emoji at the boundary must not cause a panic or an
    // infinite loop. The function walks char-by-char, not
    // byte-by-byte, on the "no match" branch.
    let out = word_boundary_replace("🚀 $ctor 🚀", "$ctor", "constructor");
    assert_eq!(out, "🚀 constructor 🚀");
}

#[test]
fn handles_match_at_start() {
    assert_eq!(
        word_boundary_replace("$ctor abc", "$ctor", "constructor"),
        "constructor abc"
    );
}

#[test]
fn handles_match_at_end() {
    assert_eq!(
        word_boundary_replace("abc $ctor", "$ctor", "constructor"),
        "abc constructor"
    );
}

#[test]
fn handles_empty_replacement() {
    // "$ctor" with empty replacement should still respect boundaries.
    assert_eq!(word_boundary_replace("a $ctor b", "$ctor", ""), "a  b");
}

// ---------- F-15: precomputed sorted opcode list ----------

#[test]
fn parse_builds_sorted_opcodes_longest_first() {
    // After `parse()`, the sorted_opcodes vec should contain both
    // builtin and custom opcodes, sorted longest-first so that a
    // 5-char opcode is tried before a 2-char one.
    let input = "// header\n$foo = bar\n§PATHMAP\n  α1 = /tmp/x.ts";
    let mut d = Decompressor::new();
    d.parse(input);
    // The sorted_opcodes list must be non-empty (contains builtins).
    assert!(!d.sorted_opcodes.is_empty());
    // The first entry should be the longest opcode.
    for w in d.sorted_opcodes.windows(2) {
        assert!(
            w[0].0.len() >= w[1].0.len(),
            "sorted_opcodes must be longest-first: {:?} before {:?}",
            w[0].0,
            w[1].0
        );
    }
}

#[test]
fn decompress_with_precomputed_opcodes_matches_expected() {
    // A compressed block using a builtin opcode ($c = class) should
    // decompress correctly via the precomputed path.
    let input = "// --- Compacted Layout (Low Fidelity): α1 ---\n$c Foo;$a bar\n\n§PATHMAP\n  α1 = /tmp/Foo.ts";
    let mut d = Decompressor::new();
    let result = d.quick_decompress(input);
    assert!(
        result.contains("class Foo"),
        "Expected 'class Foo' in: {}",
        result
    );
    assert!(
        result.contains("async bar"),
        "Expected 'async bar' in: {}",
        result
    );
}
