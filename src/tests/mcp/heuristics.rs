// src/tests/mcp/heuristics.rs
//
// Tests for the heuristics engine V2 (src/mcp/heuristics.rs)
// Covers V1 backward compatibility + V2 content classification
// + regression tests for FAANG audit findings

use crate::mcp::heuristics;
use crate::compression::text_delta::TextDeltaComputer;
use crate::compression::Fidelity;
use crate::config::CleanCtxConfig;
use crate::ir::replay::ContextState;

fn empty_source() -> &'static str { "" }

// ── V1 Strategy Tests (unchanged) ──────────────────────────────────

#[test]
fn test_first_call_full_compress() {
    let config = CleanCtxConfig::default();
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    let decision = heuristics::decide(
        "/test/file.ts",
        None,
        None,
        &config,
        &text_delta,
        &ir_ctx,
        empty_source(),
        None,
        None,
    );
    assert_eq!(decision.strategy, heuristics::ContextStrategy::FullCompress);
}

#[test]
fn test_delta_after_baseline() {
    let config = CleanCtxConfig::default();
    let mut text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();

    // Store a baseline first
    text_delta.store_snapshot("alpha1", vec!["line1".to_string()]);

    let decision = heuristics::decide(
        "alpha1",
        None,
        None,
        &config,
        &text_delta,
        &ir_ctx,
        empty_source(),
        Some("alpha1"),
        None,
    );
    assert_eq!(decision.strategy, heuristics::ContextStrategy::DeltaTransport);
}

// ── V1 Fidelity Tests (unchanged) ──────────────────────────────────

#[test]
fn test_intent_refactor_high_fidelity() {
    let config = CleanCtxConfig::default();
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    let decision = heuristics::decide(
        "/test/file.ts",
        None,
        Some("refactor"),
        &config,
        &text_delta,
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
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    let decision = heuristics::decide(
        "/test/file.ts",
        None,
        Some("overview"),
        &config,
        &text_delta,
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
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    let large_source: String = (0..500).map(|i| format!("line {}\n", i)).collect();
    let decision = heuristics::decide(
        "/project/src/unknown.rs",
        None,
        None,
        &config,
        &text_delta,
        &ir_ctx,
        &large_source,
        None,
        None,
    );
    assert_eq!(decision.fidelity, Fidelity::Medium,
        "V2: 500-line file without content patterns -> complexity Medium");
}

// V2: Small files (<=150 lines) still get Low via complexity fallback
#[test]
fn test_small_file_v2_complexity_low() {
    let config = CleanCtxConfig::default();
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    let small_source: String = (0..150).map(|i| format!("line {}\n", i)).collect();
    let decision = heuristics::decide(
        "/project/src/unknown.rs",
        None,
        None,
        &config,
        &text_delta,
        &ir_ctx,
        &small_source,
        None,
        None,
    );
    assert_eq!(decision.fidelity, Fidelity::Low,
        "V2: 150-line file without content patterns -> complexity Low");
}

// ── V1 Angular Detection (unchanged) ───────────────────────────────

#[test]
fn test_angular_detection() {
    let config = CleanCtxConfig::default();
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    let angular_source = r#"
        import { Component } from '@angular/core';
        @Component({ selector: 'app-test' })
        export class TestComponent {}
    "#;
    let decision = heuristics::decide(
        "/test/test.component.ts",
        None,
        None,
        &config,
        &text_delta,
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
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    let decision = heuristics::decide(
        "/test/file.ts",
        None,
        None,
        &config,
        &text_delta,
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
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    let decision = heuristics::decide(
        "/test/file.ts",
        None,
        Some("edit"),
        &config,
        &text_delta,
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
    assert!(summary.contains("class="), "V2: summary should include class= field");
}

// ══════════════════════════════════════════════════════════════════
// V2: Content-Based Classification Tests
// ══════════════════════════════════════════════════════════════════

#[test]
fn test_v2_classify_test_file() {
    let config = CleanCtxConfig::default();
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    let test_source = "#[test]\nfn test_foo() { assert!(true); }";
    let decision = heuristics::decide(
        "/test/file.rs",
        None,
        None,
        &config,
        &text_delta,
        &ir_ctx,
        test_source,
        None,
        None,
    );
    assert_eq!(decision.fidelity, Fidelity::Low, "test files should get Low fidelity");
    assert_eq!(decision.file_class, heuristics::FileClass::Test);
}

#[test]
fn test_v2_classify_test_path() {
    let config = CleanCtxConfig::default();
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    let decision = heuristics::decide(
        "/src/__tests__/utils.ts",
        None,
        None,
        &config,
        &text_delta,
        &ir_ctx,
        "export function add(a: number, b: number): number { return a + b; }",
        None,
        None,
    );
    assert_eq!(decision.fidelity, Fidelity::Low, "test path files should get Low fidelity");
    assert_eq!(decision.file_class, heuristics::FileClass::Test);
}

#[test]
fn test_v2_classify_config_file() {
    let config = CleanCtxConfig::default();
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    let decision = heuristics::decide(
        "/project/src/config.rs",
        None,
        None,
        &config,
        &text_delta,
        &ir_ctx,
        "pub struct Config { pub db_path: String }",
        None,
        None,
    );
    assert_eq!(decision.fidelity, Fidelity::Low, "config files should get Low fidelity");
    assert_eq!(decision.file_class, heuristics::FileClass::Config);
}

/// M-3 regression: configure.rs should NOT be treated as config (exact path segment match)
#[test]
fn test_v2_m3_configure_not_config() {
    let config = CleanCtxConfig::default();
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    let decision = heuristics::decide(
        "/project/src/configure.rs",
        None,
        None,
        &config,
        &text_delta,
        &ir_ctx,
        "pub fn configure() {}",
        None,
        None,
    );
    // configure.rs does NOT match "config" as a path segment, so it should NOT be FileClass::Config
    assert_ne!(decision.file_class, heuristics::FileClass::Config,
        "M-3 regression: configure.rs should NOT be classified as config");
}

#[test]
fn test_v2_classify_model_file() {
    let config = CleanCtxConfig::default();
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    let model_source = r#"
pub struct User { pub name: String, pub age: u32 }
pub struct Post { pub title: String, pub body: String }
pub enum Status { Active, Inactive }
pub trait Displayable { fn display(&self) -> String; }
"#;
    let decision = heuristics::decide(
        "/project/src/models.rs",
        None,
        None,
        &config,
        &text_delta,
        &ir_ctx,
        model_source,
        None,
        None,
    );
    assert_eq!(decision.fidelity, Fidelity::Medium, "model files should get Medium fidelity");
    assert_eq!(decision.file_class, heuristics::FileClass::Model);
}

/// M-1 regression: impl blocks should NOT inflate struct count
#[test]
fn test_v2_m1_impl_blocks_not_structs() {
    let config = CleanCtxConfig::default();
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    // File with 1 struct and 5 impl blocks (method implementations)
    // Before M-1 fix: struct_count = 6 (1 struct + 5 impl) -> model classification
    // After M-1 fix: struct_count = 1 -> not model, falls through
    let source = r#"
pub struct User { pub name: String }
impl User {
    pub fn new() -> Self { User { name: String::new() } }
    pub fn name(&self) -> &str { &self.name }
    pub fn set_name(&mut self, n: String) { self.name = n; }
    pub fn validate(&self) -> bool { !self.name.is_empty() }
}
impl std::fmt::Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
"#;
    let decision = heuristics::decide(
        "/project/src/user.rs",
        None,
        None,
        &config,
        &text_delta,
        &ir_ctx,
        source,
        None,
        None,
    );
    // Should NOT be Model (1 struct with 5 fn is not > 3:1 ratio with fns present)
    assert_ne!(decision.file_class, heuristics::FileClass::Model,
        "M-1 regression: file with 1 struct + impl blocks should NOT be classified as Model");
}

/// M-2 regression: fn test_ functions should NOT trigger test classification
#[test]
fn test_v2_m2_test_helper_not_test_file() {
    let config = CleanCtxConfig::default();
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    let source = "fn test_connection() -> bool { true }\npub fn connect() { }";
    let decision = heuristics::decide(
        "/project/src/db.rs",
        None,
        None,
        &config,
        &text_delta,
        &ir_ctx,
        source,
        None,
        None,
    );
    // fn test_connection is a test helper, not a test file
    assert_ne!(decision.file_class, heuristics::FileClass::Test,
        "M-2 regression: fn test_ helper should NOT trigger test classification");
}

/// C-1 regression: stored_fidelity from DB should be used when no explicit args
#[test]
fn test_v2_c1_stored_fidelity_reused() {
    let config = CleanCtxConfig::default();
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    let source = "pub fn do_stuff() { }";
    let decision = heuristics::decide(
        "/project/src/utils.rs",
        None,
        None,
        &config,
        &text_delta,
        &ir_ctx,
        source,
        None,
        Some(Fidelity::High),  // C-1: DB says this was High before
    );
    assert_eq!(decision.fidelity, Fidelity::High,
        "C-1 regression: stored_fidelity=High should be reused when no explicit args");
}

/// C-1 regression: explicit fidelity still overrides stored_fidelity
#[test]
fn test_v2_c1_explicit_overrides_stored() {
    let config = CleanCtxConfig::default();
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    let source = "pub fn do_stuff() { }";
    let decision = heuristics::decide(
        "/project/src/utils.rs",
        Some("low"),
        None,
        &config,
        &text_delta,
        &ir_ctx,
        source,
        None,
        Some(Fidelity::High),
    );
    assert_eq!(decision.fidelity, Fidelity::Low,
        "C-1 regression: explicit fidelity=low should override stored_fidelity=High");
}

/// C-1 regression: session_aware_fidelity=false ignores stored_fidelity
#[test]
fn test_v2_c1_disabled_ignores_stored() {
    let mut config = CleanCtxConfig::default();
    config.heuristics.session_aware_fidelity = false;
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    let source = "pub fn do_stuff() { }";
    let decision = heuristics::decide(
        "/project/src/utils.rs",
        None,
        None,
        &config,
        &text_delta,
        &ir_ctx,
        source,
        None,
        Some(Fidelity::High),
    );
    // With session_aware_fidelity off, stored_fidelity is ignored.
    // The file is small (1 fn, 1 line) -> config default Low
    assert_eq!(decision.fidelity, Fidelity::Low,
        "C-1 regression: stored_fidelity should be ignored when session_aware_fidelity=false");
}

#[test]
fn test_v2_classify_service_file() {
    let config = CleanCtxConfig::default();
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    let mut source = String::new();
    for i in 0..16 {
        source.push_str(&format!("use crate::module{}::Thing{};\n", i, i));
    }
    for i in 0..11 {
        source.push_str(&format!("pub fn func{}(x: i32) -> i32 {{ x + {} }}\n", i, i));
    }
    let decision = heuristics::decide(
        "/project/src/services/user_service.rs",
        None,
        None,
        &config,
        &text_delta,
        &ir_ctx,
        &source,
        None,
        None,
    );
    assert_eq!(decision.fidelity, Fidelity::High, "service files should get High fidelity");
    assert_eq!(decision.file_class, heuristics::FileClass::Service);
}

#[test]
fn test_v2_classify_implementation_file() {
    let config = CleanCtxConfig::default();
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    let impl_source = r#"
use std::collections::HashMap;
use crate::models::User;

pub fn get_user(id: u32) -> Option<User> {
    None
}

pub fn list_users() -> Vec<User> {
    vec![]
}
"#;
    let decision = heuristics::decide(
        "/project/src/user.handler.rs",
        None,
        None,
        &config,
        &text_delta,
        &ir_ctx,
        impl_source,
        None,
        None,
    );
    assert_eq!(decision.fidelity, Fidelity::Medium, "implementation files should get Medium fidelity");
    assert_eq!(decision.file_class, heuristics::FileClass::Implementation);
}

// ══════════════════════════════════════════════════════════════════
// V2: Complexity Fallback Tests (reversed from V1)
// ══════════════════════════════════════════════════════════════════

#[test]
fn test_v2_complexity_very_small_low() {
    let config = CleanCtxConfig::default();
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    let decision = heuristics::decide(
        "/project/src/lib.rs",
        None,
        None,
        &config,
        &text_delta,
        &ir_ctx,
        "pub fn add(a: i32, b: i32) -> i32 { a + b }",
        None,
        None,
    );
    assert_eq!(decision.fidelity, Fidelity::Low, "very small files should get Low");
}

#[test]
fn test_v2_complexity_medium_imports() {
    let config = CleanCtxConfig::default();
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    let mut source = String::new();
    for i in 0..12 {
        source.push_str(&format!("use crate::module{}::Thing{};\n", i, i));
    }
    source.push_str("pub fn process() -> bool { true }\n");
    let decision = heuristics::decide(
        "/project/src/processor.rs",
        None,
        None,
        &config,
        &text_delta,
        &ir_ctx,
        &source,
        None,
        None,
    );
    assert_eq!(decision.fidelity, Fidelity::Medium,
        "12 imports + 1 function should get Medium via complexity");
}

#[test]
fn test_v2_complexity_high() {
    let config = CleanCtxConfig::default();
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    let mut source = String::new();
    for i in 0..25 {
        source.push_str(&format!("use crate::module{}::Thing{};\n", i, i));
    }
    for i in 0..20 {
        source.push_str(&format!("pub fn func{}(x: i32) -> i32 {{ x + {} }}\n", i, i));
    }
    for i in 0..500 {
        source.push_str(&format!("// line {}\n", i));
    }
    let decision = heuristics::decide(
        "/project/src/massive.rs",
        None,
        None,
        &config,
        &text_delta,
        &ir_ctx,
        &source,
        None,
        None,
    );
    assert_eq!(decision.fidelity, Fidelity::High,
        "25 imports + 20 functions should get High via service classifier");
}

#[test]
fn test_v2_explicit_fidelity_overrides_classifier() {
    let config = CleanCtxConfig::default();
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    let mut source = String::new();
    for i in 0..20 {
        source.push_str(&format!("use crate::mod{}::X{};\n", i, i));
    }
    for i in 0..15 {
        source.push_str(&format!("pub fn f{}(x: i32) -> i32 {{ x + {} }}\n", i, i));
    }
    let decision = heuristics::decide(
        "/project/src/service.rs",
        Some("low"),
        None,
        &config,
        &text_delta,
        &ir_ctx,
        &source,
        None,
        None,
    );
    assert_eq!(decision.fidelity, Fidelity::Low,
        "explicit fidelity=low should override service classification");
}

#[test]
fn test_v2_auto_classify_disabled_v1_fallback() {
    let mut config = CleanCtxConfig::default();
    config.heuristics.auto_classify = false;
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    let large_source: String = (0..500).map(|i| format!("line {}\n", i)).collect();
    let decision = heuristics::decide(
        "/project/src/unknown.rs",
        None,
        None,
        &config,
        &text_delta,
        &ir_ctx,
        &large_source,
        None,
        None,
    );
    assert_eq!(decision.fidelity, Fidelity::Low,
        "V1 fallback: large file -> Low when auto_classify is disabled");
    assert_eq!(decision.file_class, heuristics::FileClass::General);
}