// src/tests/mcp/heuristics.rs
//
// Tests for the heuristics engine V2 (src/mcp/heuristics.rs)
// Covers V1 backward compatibility + V2 content classification
// + regression tests for FAANG audit findings

use crate::compression::Fidelity;
use crate::config::CleanCtxConfig;
use crate::ir::replay::ContextState;
use crate::mcp::heuristics;

#[allow(clippy::too_many_arguments)]
fn decide_ok(
    file_path: &str,
    explicit_fidelity: Option<&str>,
    explicit_intent: Option<&str>,
    config: &CleanCtxConfig,
    ir_ctx: &ContextState,
    source: &str,
    path_alias: Option<&str>,
    stored_fidelity: Option<Fidelity>,
) -> heuristics::ContextDecision {
    heuristics::decide(
        file_path,
        explicit_fidelity,
        explicit_intent,
        config,
        ir_ctx,
        source,
        path_alias,
        stored_fidelity,
    )
    .expect("decide should succeed")
}

fn empty_source() -> &'static str {
    ""
}

// ── Complete-provider boundary ─────────────────────────────────────

#[test]
fn test_first_call_full_compress() {
    let config = CleanCtxConfig::default();
    let ir_ctx = ContextState::new();
    let decision = decide_ok(
        "/test/file.ts",
        None,
        None,
        &config,
        &ir_ctx,
        empty_source(),
        None,
        None,
    );
    assert!(decision.summary().contains("strategy=full_compress"));
}
#[test]
fn test_ir_baseline_does_not_change_provider_strategy() {
    let config = CleanCtxConfig::default();
    let mut ir_ctx = ContextState::new();

    // Seed an IR baseline, as if a prior call compiled this file.
    ir_ctx.load_ir(
        crate::ir::compiler::CompiledIR {
            file_id: "alpha1".to_string(),
            instructions: vec![],
            version: 1,
        },
        None,
    );

    let decision = decide_ok(
        "alpha1",
        None,
        None,
        &config,
        &ir_ctx,
        empty_source(),
        Some("alpha1"),
        None,
    );
    assert!(decision.summary().contains("strategy=full_compress"));
}

// ── Explicit fidelity and intent ───────────────────────────────────

#[test]
fn test_explicit_fidelity_is_honored_with_an_existing_baseline() {
    let config = CleanCtxConfig::default();
    let mut ir_ctx = ContextState::new();

    // Seed an IR baseline, as if a prior call compressed this file.
    ir_ctx.load_ir(
        crate::ir::compiler::CompiledIR {
            file_id: "alpha1".to_string(),
            instructions: vec![],
            version: 1,
        },
        None,
    );

    let decision = decide_ok(
        "alpha1",
        Some("edit"),
        None,
        &config,
        &ir_ctx,
        empty_source(),
        Some("alpha1"),
        None,
    );
    assert_eq!(decision.fidelity, Fidelity::Edit);
    assert!(decision.summary().contains("strategy=full_compress"));
}

#[test]
fn test_explicit_intent_is_honored_with_an_existing_baseline() {
    let config = CleanCtxConfig::default();
    let mut ir_ctx = ContextState::new();

    // Seed an IR baseline, as if a baseline was compressed earlier.
    ir_ctx.load_ir(
        crate::ir::compiler::CompiledIR {
            file_id: "alpha1".to_string(),
            instructions: vec![],
            version: 1,
        },
        None,
    );

    let decision = decide_ok(
        "alpha1",
        None,
        Some("edit"),
        &config,
        &ir_ctx,
        empty_source(),
        Some("alpha1"),
        None,
    );
    assert_eq!(decision.fidelity, Fidelity::Edit);
    assert!(decision.summary().contains("strategy=full_compress"));
}

// ── V1 Fidelity Tests (unchanged) ──────────────────────────────────

#[test]
fn test_intent_refactor_high_fidelity() {
    let config = CleanCtxConfig::default();
    let ir_ctx = ContextState::new();
    let decision = decide_ok(
        "/test/file.ts",
        None,
        Some("refactor"),
        &config,
        &ir_ctx,
        empty_source(),
        None,
        None,
    );
    assert_eq!(decision.fidelity, Fidelity::High);
}

#[test]
fn test_intent_overview_low_fidelity() {
    let config = CleanCtxConfig::default();
    let ir_ctx = ContextState::new();
    let decision = decide_ok(
        "/test/file.ts",
        None,
        Some("overview"),
        &config,
        &ir_ctx,
        empty_source(),
        None,
        None,
    );
    assert_eq!(decision.fidelity, Fidelity::Low);
}

// ── V2: Large file with no content -> complexity fallback (Medium) ──

#[test]
fn test_large_file_v2_complexity_medium() {
    let config = CleanCtxConfig::default();
    let ir_ctx = ContextState::new();
    let large_source: String = (0..500).map(|i| format!("line {}\n", i)).collect();
    let decision = decide_ok(
        "/project/src/unknown.rs",
        None,
        None,
        &config,
        &ir_ctx,
        &large_source,
        None,
        None,
    );
    assert_eq!(
        decision.fidelity,
        Fidelity::Medium,
        "V2: 500-line file without content patterns -> complexity Medium"
    );
}

// V2: Small files (<=150 lines) still get Low via complexity fallback
#[test]
fn test_small_file_v2_complexity_low() {
    let config = CleanCtxConfig::default();
    let ir_ctx = ContextState::new();
    let small_source: String = (0..150).map(|i| format!("line {}\n", i)).collect();
    let decision = decide_ok(
        "/project/src/unknown.rs",
        None,
        None,
        &config,
        &ir_ctx,
        &small_source,
        None,
        None,
    );
    assert_eq!(
        decision.fidelity,
        Fidelity::Low,
        "V2: 150-line file without content patterns -> complexity Low"
    );
}

// ── V1 Angular Detection (unchanged) ───────────────────────────────

#[test]
fn test_angular_detection() {
    let config = CleanCtxConfig::default();
    let ir_ctx = ContextState::new();
    let angular_source = r#"
        import { Component } from '@angular/core';
        @Component({ selector: 'app-test' })
        export class TestComponent {}
    "#;
    let decision = decide_ok(
        "/test/test.component.ts",
        None,
        None,
        &config,
        &ir_ctx,
        angular_source,
        None,
        None,
    );
    assert!(decision.is_angular);
}

#[test]
fn test_non_angular_not_detected() {
    let config = CleanCtxConfig::default();
    let ir_ctx = ContextState::new();
    let decision = decide_ok(
        "/test/file.ts",
        None,
        None,
        &config,
        &ir_ctx,
        "export class PlainClass {}",
        None,
        None,
    );
    assert!(!decision.is_angular);
}

#[test]
fn test_decision_summary_includes_details() {
    let config = CleanCtxConfig::default();
    let ir_ctx = ContextState::new();
    let decision = decide_ok(
        "/test/file.ts",
        None,
        Some("edit"),
        &config,
        &ir_ctx,
        empty_source(),
        None,
        None,
    );
    let summary = decision.summary();
    assert!(summary.contains("fidelity="));
    assert!(summary.contains("strategy="));
    assert!(summary.contains("angular="));
    assert!(summary.contains("lines="));
    assert!(
        summary.contains("class="),
        "V2: summary should include class= field"
    );
}

// ══════════════════════════════════════════════════════════════════
// V2: Complexity Fallback Tests (reversed from V1)
// ══════════════════════════════════════════════════════════════════

#[test]
fn test_v2_complexity_very_small_low() {
    let config = CleanCtxConfig::default();
    let ir_ctx = ContextState::new();
    let decision = decide_ok(
        "/project/src/lib.rs",
        None,
        None,
        &config,
        &ir_ctx,
        "pub fn add(a: i32, b: i32) -> i32 { a + b }",
        None,
        None,
    );
    assert_eq!(
        decision.fidelity,
        Fidelity::Low,
        "very small files should get Low"
    );
}

#[test]
fn test_v2_complexity_medium_imports() {
    let config = CleanCtxConfig::default();
    let ir_ctx = ContextState::new();
    let mut source = String::new();
    for i in 0..12 {
        source.push_str(&format!("use crate::module{}::Thing{};\n", i, i));
    }
    source.push_str("pub fn process() -> bool { true }\n");
    let decision = decide_ok(
        "/project/src/processor.rs",
        None,
        None,
        &config,
        &ir_ctx,
        &source,
        None,
        None,
    );
    assert_eq!(
        decision.fidelity,
        Fidelity::Medium,
        "12 imports + 1 function should get Medium via complexity"
    );
}

#[test]
fn test_v2_complexity_high() {
    let mut config = CleanCtxConfig::default();
    // Isolate the complexity classifier's native High mapping from
    // the auto-edit override.
    config.heuristics.auto_edit_mode = false;
    let ir_ctx = ContextState::new();
    let mut source = String::new();
    for i in 0..25 {
        source.push_str(&format!("use crate::module{}::Thing{};\n", i, i));
    }
    for i in 0..20 {
        source.push_str(&format!(
            "pub fn func{}(x: i32) -> i32 {{ x + {} }}\n",
            i, i
        ));
    }
    for i in 0..500 {
        source.push_str(&format!("// line {}\n", i));
    }
    let decision = decide_ok(
        "/project/src/massive.rs",
        None,
        None,
        &config,
        &ir_ctx,
        &source,
        None,
        None,
    );
    assert_eq!(
        decision.fidelity,
        Fidelity::High,
        "25 imports + 20 functions should get High via service classifier"
    );
}

#[test]
fn test_v2_explicit_fidelity_overrides_classifier() {
    let config = CleanCtxConfig::default();
    let ir_ctx = ContextState::new();
    let mut source = String::new();
    for i in 0..20 {
        source.push_str(&format!("use crate::mod{}::X{};\n", i, i));
    }
    for i in 0..15 {
        source.push_str(&format!("pub fn f{}(x: i32) -> i32 {{ x + {} }}\n", i, i));
    }
    let decision = decide_ok(
        "/project/src/service.rs",
        Some("low"),
        None,
        &config,
        &ir_ctx,
        &source,
        None,
        None,
    );
    assert_eq!(
        decision.fidelity,
        Fidelity::Low,
        "explicit fidelity=low should override service classification"
    );
}

#[test]
fn test_v2_auto_classify_disabled_v1_fallback() {
    let mut config = CleanCtxConfig::default();
    config.heuristics.auto_classify = false;
    let ir_ctx = ContextState::new();
    let large_source: String = (0..500).map(|i| format!("line {}\n", i)).collect();
    let decision = decide_ok(
        "/project/src/unknown.rs",
        None,
        None,
        &config,
        &ir_ctx,
        &large_source,
        None,
        None,
    );
    assert_eq!(
        decision.fidelity,
        Fidelity::Low,
        "V1 fallback: large file -> Low when auto_classify is disabled"
    );
    assert_eq!(decision.file_class, heuristics::FileClass::General);
}
