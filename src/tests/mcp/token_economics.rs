// src/tests/mcp/token_economics.rs
//
// Tests for the cheap token-economics gate.
//
// Covers:
// 1. File below threshold -> predicts unfavorable.
// 2. File above threshold -> predicts favorable.
// 3. File near threshold shows conservative bias.
// 4. Structural fidelities bypass the gate.
// 5. Different languages use calibrated parameters.
// 6. Zero-token file at structural still compresses.
// 7. Verbatim returns true from gate.
// 8. Very small file at Edit skips compression.
// 9. Large file at Edit attempts compression.
// 10. Regression: structural threshold is zero.

use crate::compression::Fidelity;
use crate::mcp::token_economics::{compression_threshold, should_attempt_compression};
#[test]
fn small_file_below_threshold_predicts_unfavorable() {
    // .rs at Edit: threshold ~510 (150/0.28*0.85)
    assert!(!should_attempt_compression(100, Fidelity::Edit, "rs"));
}

#[test]
fn large_file_above_threshold_predicts_favorable() {
    assert!(should_attempt_compression(2000, Fidelity::Edit, "rs"));
}

#[test]
fn near_threshold_shows_conservative_bias() {
    // .cs: overhead=180, ratio=0.25, biased threshold = 180/0.25*0.85 = 612
    assert_eq!(compression_threshold(Fidelity::Edit, "cs"), 612);
    assert!(should_attempt_compression(620, Fidelity::Edit, "cs"));
}

#[test]
fn structural_fidelity_always_attempts_compression() {
    for f in &[Fidelity::Low, Fidelity::Medium, Fidelity::High] {
        assert!(should_attempt_compression(1, *f, "rs"));
    }
}

#[test]
fn different_languages_have_different_thresholds() {
    for ext in &["ts", "cs", "rs", "java"] {
        let t = compression_threshold(Fidelity::Edit, ext);
        assert!(t > 0);
        assert!(t >= 400 && t <= 700, "Threshold {} out of range", t);
    }
}

#[test]
fn unknown_extension_uses_conservative_defaults() {
    assert_eq!(compression_threshold(Fidelity::Edit, "unknown"), 765);
}

#[test]
fn extension_with_leading_dot_is_normalized() {
    assert_eq!(
        compression_threshold(Fidelity::Edit, ".rs"),
        compression_threshold(Fidelity::Edit, "rs")
    );
}

#[test]
fn zero_tokens_structural_still_attempts() {
    for f in &[Fidelity::Low, Fidelity::Medium, Fidelity::High] {
        assert!(should_attempt_compression(0, *f, "rs"));
    }
}

#[test]
fn verbatim_returns_true() {
    assert!(should_attempt_compression(0, Fidelity::Verbatim, "rs"));
}

#[test]
fn edit_very_small_file_skips_compression() {
    for ext in &["ts", "cs", "rs", "java"] {
        assert!(!should_attempt_compression(10, Fidelity::Edit, ext));
    }
}

#[test]
fn edit_large_file_attempts_compression() {
    for ext in &["ts", "cs", "rs", "java"] {
        assert!(should_attempt_compression(2000, Fidelity::Edit, ext));
    }
}

#[test]
fn structural_all_languages_always_attempt() {
    for f in &[Fidelity::Low, Fidelity::Medium, Fidelity::High] {
        for ext in &["ts", "cs", "rs", "java", "unknown", ""] {
            assert!(should_attempt_compression(5, *f, ext));
        }
    }
}

#[test]
fn compression_threshold_structural_is_zero() {
    for f in &[Fidelity::Low, Fidelity::Medium, Fidelity::High] {
        assert_eq!(compression_threshold(*f, "rs"), 0);
        assert_eq!(compression_threshold(*f, "cs"), 0);
    }
}
