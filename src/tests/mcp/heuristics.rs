// src/tests/mcp/heuristics.rs
//
// Tests for the heuristics engine V2 (src/mcp/heuristics.rs)
// Covers V1 backward compatibility + V2 content classification
// + regression tests for FAANG audit findings

use crate::compression::Fidelity;
use crate::compression::text_delta::TextDeltaComputer;
use crate::config::CleanCtxConfig;
use crate::ir::replay::ContextState;
use crate::mcp::heuristics;

#[allow(clippy::too_many_arguments)]
fn decide_ok(
    file_path: &str,
    explicit_fidelity: Option<&str>,
    explicit_intent: Option<&str>,
    config: &CleanCtxConfig,
    text_delta: &TextDeltaComputer,
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
        text_delta,
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

// ── V1 Strategy Tests (unchanged) ──────────────────────────────────

#[test]
fn test_first_call_full_compress() {
    let config = CleanCtxConfig::default();
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    let decision = decide_ok(
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

    let decision = decide_ok(
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
    assert_eq!(
        decision.strategy,
        heuristics::ContextStrategy::DeltaTransport
    );
}

// ── F-32: Delta fidelity-change guard ──────────────────────────────

/// Delta transport must NOT be selected when the caller explicitly
/// changes `fidelity` between calls on the same file. The prior
/// baseline was compiled at a different fidelity; its wire format is
/// incompatible with `apply_delta`, which would produce a bare summary
/// line with no structured delta payload. Force a full compress.
#[test]
fn test_explicit_fidelity_change_forces_full_compress() {
    let config = CleanCtxConfig::default();
    let mut text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();

    // Store a baseline first (as if a prior call compressed this file).
    text_delta.store_snapshot("alpha1", vec!["line1".to_string()]);

    // Explicit fidelity change → delta would be incompatible.
    let decision = decide_ok(
        "alpha1",
        Some("edit"),
        None,
        &config,
        &text_delta,
        &ir_ctx,
        empty_source(),
        Some("alpha1"),
        None,
    );
    assert_eq!(decision.strategy, heuristics::ContextStrategy::FullCompress);
}

/// Same guard for an explicit `intent` change (which maps to a
/// fidelity via `config.smart_defaults`).
#[test]
fn test_explicit_intent_change_forces_full_compress() {
    let config = CleanCtxConfig::default();
    let mut text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();

    // Store a baseline first (as if a baseline was compressed earlier).
    text_delta.store_snapshot("alpha1", vec!["line1".to_string()]);

    let decision = decide_ok(
        "alpha1",
        None,
        Some("edit"),
        &config,
        &text_delta,
        &ir_ctx,
        empty_source(),
        Some("alpha1"),
        None,
    );
    assert_eq!(decision.strategy, heuristics::ContextStrategy::FullCompress);
}

// ── V1 Fidelity Tests (unchanged) ──────────────────────────────────

#[test]
fn test_intent_refactor_high_fidelity() {
    let config = CleanCtxConfig::default();
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    let decision = decide_ok(
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
    let decision = decide_ok(
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
    let decision = decide_ok(
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
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    let small_source: String = (0..150).map(|i| format!("line {}\n", i)).collect();
    let decision = decide_ok(
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
    let text_delta = TextDeltaComputer::new();
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
    let decision = decide_ok(
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
    let decision = decide_ok(
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
    assert!(
        summary.contains("class="),
        "V2: summary should include class= field"
    );
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
    let decision = decide_ok(
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
    assert_eq!(
        decision.fidelity,
        Fidelity::Low,
        "test files should get Low fidelity"
    );
    assert_eq!(decision.file_class, heuristics::FileClass::Test);
}

#[test]
fn test_v2_classify_test_path() {
    let config = CleanCtxConfig::default();
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    let decision = decide_ok(
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
    assert_eq!(
        decision.fidelity,
        Fidelity::Low,
        "test path files should get Low fidelity"
    );
    assert_eq!(decision.file_class, heuristics::FileClass::Test);
}

#[test]
fn test_v2_classify_config_file() {
    let config = CleanCtxConfig::default();
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    let decision = decide_ok(
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
    assert_eq!(
        decision.fidelity,
        Fidelity::Low,
        "config files should get Low fidelity"
    );
    assert_eq!(decision.file_class, heuristics::FileClass::Config);
}

/// M-3 regression: configure.rs should NOT be treated as config (exact path segment match)
#[test]
fn test_v2_m3_configure_not_config() {
    let config = CleanCtxConfig::default();
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    let decision = decide_ok(
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
    assert_ne!(
        decision.file_class,
        heuristics::FileClass::Config,
        "M-3 regression: configure.rs should NOT be classified as config"
    );
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
    let decision = decide_ok(
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
    assert_eq!(
        decision.fidelity,
        Fidelity::Medium,
        "model files should get Medium fidelity"
    );
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
    let decision = decide_ok(
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
    assert_ne!(
        decision.file_class,
        heuristics::FileClass::Model,
        "M-1 regression: file with 1 struct + impl blocks should NOT be classified as Model"
    );
}

/// M-2 regression: fn test_ functions should NOT trigger test classification
#[test]
fn test_v2_m2_test_helper_not_test_file() {
    let config = CleanCtxConfig::default();
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    let source = "fn test_connection() -> bool { true }\npub fn connect() { }";
    let decision = decide_ok(
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
    assert_ne!(
        decision.file_class,
        heuristics::FileClass::Test,
        "M-2 regression: fn test_ helper should NOT trigger test classification"
    );
}

/// C-1 regression: stored_fidelity from DB should be used when no explicit args
#[test]
fn test_v2_c1_stored_fidelity_reused() {
    let config = CleanCtxConfig::default();
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    let source = "pub fn do_stuff() { }";
    let decision = decide_ok(
        "/project/src/utils.rs",
        None,
        None,
        &config,
        &text_delta,
        &ir_ctx,
        source,
        None,
        Some(Fidelity::High), // C-1: DB says this was High before
    );
    assert_eq!(
        decision.fidelity,
        Fidelity::High,
        "C-1 regression: stored_fidelity=High should be reused when no explicit args"
    );
}

/// C-1 regression: explicit fidelity still overrides stored_fidelity
#[test]
fn test_v2_c1_explicit_overrides_stored() {
    let config = CleanCtxConfig::default();
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    let source = "pub fn do_stuff() { }";
    let decision = decide_ok(
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
    assert_eq!(
        decision.fidelity,
        Fidelity::Low,
        "C-1 regression: explicit fidelity=low should override stored_fidelity=High"
    );
}

/// C-1 regression: session_aware_fidelity=false ignores stored_fidelity
#[test]
fn test_v2_c1_disabled_ignores_stored() {
    let mut config = CleanCtxConfig::default();
    config.heuristics.session_aware_fidelity = false;
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    let source = "pub fn do_stuff() { }";
    let decision = decide_ok(
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
    assert_eq!(
        decision.fidelity,
        Fidelity::Low,
        "C-1 regression: stored_fidelity should be ignored when session_aware_fidelity=false"
    );
}

#[test]
fn test_v2_classify_service_file() {
    let mut config = CleanCtxConfig::default();
    // Isolate the classifier's native Service→High mapping from the
    // auto-edit override (which is covered by its own tests).
    config.heuristics.auto_edit_mode = false;
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    let mut source = String::new();
    for i in 0..16 {
        source.push_str(&format!("use crate::module{}::Thing{};\n", i, i));
    }
    for i in 0..11 {
        source.push_str(&format!(
            "pub fn func{}(x: i32) -> i32 {{ x + {} }}\n",
            i, i
        ));
    }
    let decision = decide_ok(
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
    assert_eq!(
        decision.fidelity,
        Fidelity::High,
        "service files should get High fidelity"
    );
    assert_eq!(decision.file_class, heuristics::FileClass::Service);
}

#[test]
fn test_v2_classify_implementation_file() {
    let mut config = CleanCtxConfig::default();
    // Isolate the classifier's native Implementation→Medium mapping from
    // the auto-edit override (which is covered by its own tests).
    config.heuristics.auto_edit_mode = false;
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
    let decision = decide_ok(
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
    assert_eq!(
        decision.fidelity,
        Fidelity::Medium,
        "implementation files should get Medium fidelity"
    );
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
    let decision = decide_ok(
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
    assert_eq!(
        decision.fidelity,
        Fidelity::Low,
        "very small files should get Low"
    );
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
    let decision = decide_ok(
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
    let text_delta = TextDeltaComputer::new();
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
        &text_delta,
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
    let text_delta = TextDeltaComputer::new();
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
        &text_delta,
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
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    let large_source: String = (0..500).map(|i| format!("line {}\n", i)).collect();
    let decision = decide_ok(
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
    assert_eq!(
        decision.fidelity,
        Fidelity::Low,
        "V1 fallback: large file -> Low when auto_classify is disabled"
    );
    assert_eq!(decision.file_class, heuristics::FileClass::General);
}

// ══════════════════════════════════════════════════════════════════
// ANGULAR_HTML_COMPRESSION_PLAN Phase 3: `.component.html` tests
// ══════════════════════════════════════════════════════════════════

#[test]
fn test_v2_classify_component_html_implementation() {
    let mut config = CleanCtxConfig::default();
    // Isolate the classifier's native Implementation→Medium mapping from
    // the auto-edit override.
    config.heuristics.auto_edit_mode = false;
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    let html = r#"<div class="container"><app-card [data]="cardData"></app-card></div>"#;
    let decision = decide_ok(
        "/project/src/app/user-card.component.html",
        None,
        None,
        &config,
        &text_delta,
        &ir_ctx,
        html,
        None,
        None,
    );
    assert_eq!(
        decision.fidelity,
        Fidelity::Medium,
        ".component.html files should get Medium fidelity by default"
    );
    assert_eq!(decision.file_class, heuristics::FileClass::Implementation);
}

#[test]
fn test_v2_component_html_edit_intent_high_fidelity() {
    let config = CleanCtxConfig::default();
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    let html = r#"<div><span>{{ name }}</span></div>"#;
    let decision = decide_ok(
        "/project/src/app/user-card.component.html",
        None,
        Some("edit"),
        &config,
        &text_delta,
        &ir_ctx,
        html,
        None,
        None,
    );
    assert_eq!(
        decision.fidelity,
        Fidelity::High,
        "template editing intent on .component.html should get High fidelity"
    );
    assert_eq!(decision.file_class, heuristics::FileClass::Implementation);
}

// ── Edit Mode tests (Phase 4) ─────────────────────────────────────

/// Gap 2 fix: an invalid explicit fidelity must surface as an error,
/// not silently degrade to the default.
#[test]
fn test_invalid_explicit_fidelity_returns_error() {
    let config = CleanCtxConfig::default();
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    let result = heuristics::decide(
        "/project/src/service.ts",
        Some("full"), // invalid — not a recognized fidelity
        None,
        &config,
        &text_delta,
        &ir_ctx,
        "export class Foo {}",
        None,
        None,
    );
    assert!(
        result.is_err(),
        "invalid explicit fidelity should return an error"
    );
    assert!(
        result.unwrap_err().contains("full"),
        "error should mention the bad value"
    );
}

/// Gap 2 fix: a valid explicit fidelity still succeeds.
#[test]
fn test_valid_explicit_fidelity_succeeds() {
    let config = CleanCtxConfig::default();
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    let result = heuristics::decide(
        "/project/src/service.ts",
        Some("edit"),
        None,
        &config,
        &text_delta,
        &ir_ctx,
        "export class Foo {}",
        None,
        None,
    );
    assert!(result.is_ok(), "valid explicit fidelity should succeed");
    assert_eq!(result.unwrap().fidelity, Fidelity::Edit);
}

/// Gap 2.1 fix: intent="edit" maps to Fidelity::Edit via smart_defaults.
#[test]
fn test_intent_edit_maps_to_edit_fidelity() {
    let config = CleanCtxConfig::default();
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    let decision = decide_ok(
        "/project/src/service.ts",
        None,
        Some("edit"),
        &config,
        &text_delta,
        &ir_ctx,
        "export class Foo {}",
        None,
        None,
    );
    assert_eq!(
        decision.fidelity,
        Fidelity::Edit,
        "intent=edit should map to Edit fidelity via smart_defaults"
    );
}

/// Gap 2.1 fix: auto-edit mode maps Service files to Edit when no
/// explicit intent/fidelity is given.
#[test]
fn test_auto_edit_mode_service_file() {
    let config = CleanCtxConfig::default();
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    let mut source = String::new();
    for i in 0..16 {
        source.push_str(&format!("use crate::module{}::Thing{};\n", i, i));
    }
    for i in 0..11 {
        source.push_str(&format!(
            "pub fn func{}(x: i32) -> i32 {{ x + {} }}\n",
            i, i
        ));
    }
    let decision = decide_ok(
        "/project/src/services/user_service.ts",
        None,
        None,
        &config,
        &text_delta,
        &ir_ctx,
        &source,
        None,
        None,
    );
    assert_eq!(
        decision.fidelity,
        Fidelity::Edit,
        "auto_edit_mode should map Service files to Edit"
    );
}

/// Gap 2.1 fix: auto-edit mode maps Implementation files to Edit.
#[test]
fn test_auto_edit_mode_implementation_file() {
    let config = CleanCtxConfig::default();
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    let impl_source = r#"
use std::collections::HashMap;
use crate::models::User;

pub fn get_user(id: u32) -> Option<User> {
    None
}
"#;
    let decision = decide_ok(
        "/project/src/user.handler.ts",
        None,
        None,
        &config,
        &text_delta,
        &ir_ctx,
        impl_source,
        None,
        None,
    );
    assert_eq!(
        decision.fidelity,
        Fidelity::Edit,
        "auto_edit_mode should map Implementation files to Edit"
    );
}

/// Gap 2.1 fix: disabling auto_edit_mode leaves Service files at High.
#[test]
fn test_auto_edit_mode_disabled_keeps_high() {
    let mut config = CleanCtxConfig::default();
    config.heuristics.auto_edit_mode = false;
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    let mut source = String::new();
    for i in 0..16 {
        source.push_str(&format!("use crate::module{}::Thing{};\n", i, i));
    }
    for i in 0..11 {
        source.push_str(&format!(
            "pub fn func{}(x: i32) -> i32 {{ x + {} }}\n",
            i, i
        ));
    }
    let decision = decide_ok(
        "/project/src/services/user_service.ts",
        None,
        None,
        &config,
        &text_delta,
        &ir_ctx,
        &source,
        None,
        None,
    );
    assert_eq!(
        decision.fidelity,
        Fidelity::High,
        "with auto_edit_mode off, Service files should stay High"
    );
}

/// Gap 2.1 fix: custom edit_auto_classifications can include Model files.
#[test]
fn test_auto_edit_mode_custom_classifications() {
    let mut config = CleanCtxConfig::default();
    config.heuristics.edit_auto_classifications = vec!["model".to_string()];
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    let model_source = r#"
pub struct User { pub name: String, pub age: u32 }
pub struct Post { pub title: String, pub body: String }
pub enum Status { Active, Inactive }
"#;
    let decision = decide_ok(
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
    assert_eq!(
        decision.fidelity,
        Fidelity::Edit,
        "custom edit_auto_classifications should map Model files to Edit"
    );
}

#[test]
fn test_v2_component_html_explicit_fidelity_overrides() {
    let config = CleanCtxConfig::default();
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    let html = r#"<div><span>{{ name }}</span></div>"#;
    let decision = decide_ok(
        "/project/src/app/user-card.component.html",
        Some("low"),
        None,
        &config,
        &text_delta,
        &ir_ctx,
        html,
        None,
        None,
    );
    assert_eq!(
        decision.fidelity,
        Fidelity::Low,
        "explicit fidelity=low should override .component.html default"
    );
}
