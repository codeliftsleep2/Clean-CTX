// src/tests/mcp/heuristics_edit_mode.rs
//
// Explicit and automatically inferred Edit-fidelity policy contracts.

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

// ── Edit Mode tests (Phase 4) ─────────────────────────────────────

/// Gap 2 fix: an invalid explicit fidelity must surface as an error,
/// not silently degrade to the default.
#[test]
fn test_invalid_explicit_fidelity_returns_error() {
    let config = CleanCtxConfig::default();
    let ir_ctx = ContextState::new();
    let result = heuristics::decide(
        "/project/src/service.ts",
        Some("full"), // invalid — not a recognized fidelity
        None,
        &config,
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
    let ir_ctx = ContextState::new();
    let result = heuristics::decide(
        "/project/src/service.ts",
        Some("edit"),
        None,
        &config,
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
    let ir_ctx = ContextState::new();
    let decision = decide_ok(
        "/project/src/service.ts",
        None,
        Some("edit"),
        &config,
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
    let ir_ctx = ContextState::new();
    let html = r#"<div><span>{{ name }}</span></div>"#;
    let decision = decide_ok(
        "/project/src/app/user-card.component.html",
        Some("low"),
        None,
        &config,
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
