// src/tests/mcp/heuristics_classification.rs
//
// Content-shape and path classification contracts for the heuristics engine.

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

// ══════════════════════════════════════════════════════════════════
// V2: Content-Based Classification Tests
// ══════════════════════════════════════════════════════════════════

#[test]
fn test_v2_classify_test_file() {
    let config = CleanCtxConfig::default();
    let ir_ctx = ContextState::new();
    let test_source = "#[test]\nfn test_foo() { assert!(true); }";
    let decision = decide_ok(
        "/test/file.rs",
        None,
        None,
        &config,
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
    let ir_ctx = ContextState::new();
    let decision = decide_ok(
        "/src/__tests__/utils.ts",
        None,
        None,
        &config,
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
    let ir_ctx = ContextState::new();
    let decision = decide_ok(
        "/project/src/config.rs",
        None,
        None,
        &config,
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
    let ir_ctx = ContextState::new();
    let decision = decide_ok(
        "/project/src/configure.rs",
        None,
        None,
        &config,
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
    let ir_ctx = ContextState::new();
    let source = "fn test_connection() -> bool { true }\npub fn connect() { }";
    let decision = decide_ok(
        "/project/src/db.rs",
        None,
        None,
        &config,
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
    let ir_ctx = ContextState::new();
    let source = "pub fn do_stuff() { }";
    let decision = decide_ok(
        "/project/src/utils.rs",
        None,
        None,
        &config,
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
    let ir_ctx = ContextState::new();
    let source = "pub fn do_stuff() { }";
    let decision = decide_ok(
        "/project/src/utils.rs",
        Some("low"),
        None,
        &config,
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
    let ir_ctx = ContextState::new();
    let source = "pub fn do_stuff() { }";
    let decision = decide_ok(
        "/project/src/utils.rs",
        None,
        None,
        &config,
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
// ANGULAR_HTML_COMPRESSION_PLAN Phase 3: `.component.html` tests
// ══════════════════════════════════════════════════════════════════

#[test]
fn test_v2_classify_component_html_implementation() {
    let mut config = CleanCtxConfig::default();
    // Isolate the classifier's native Implementation→Medium mapping from
    // the auto-edit override.
    config.heuristics.auto_edit_mode = false;
    let ir_ctx = ContextState::new();
    let html = r#"<div class="container"><app-card [data]="cardData"></app-card></div>"#;
    let decision = decide_ok(
        "/project/src/app/user-card.component.html",
        None,
        None,
        &config,
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
    let ir_ctx = ContextState::new();
    let html = r#"<div><span>{{ name }}</span></div>"#;
    let decision = decide_ok(
        "/project/src/app/user-card.component.html",
        None,
        Some("edit"),
        &config,
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
