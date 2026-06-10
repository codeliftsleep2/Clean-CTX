// src/tests/mcp/heuristics.rs
//
// Tests for the heuristics engine (src/mcp/heuristics.rs)

use crate::mcp::heuristics;
use crate::compression::text_delta::TextDeltaComputer;
use crate::compression::Fidelity;
use crate::config::CleanCtxConfig;
use crate::ir::replay::ContextState;

fn empty_source() -> &'static str { "" }

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
    );
    assert_eq!(decision.strategy, heuristics::ContextStrategy::FullCompress);
}

#[test]
fn test_delta_after_baseline() {
    let config = CleanCtxConfig::default();
    let mut text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();

    // Store a baseline first
    text_delta.store_snapshot("α1", vec!["line1".to_string()]);

    let decision = heuristics::decide(
        "α1",
        None,
        None,
        &config,
        &text_delta,
        &ir_ctx,
        empty_source(),
    );
    assert_eq!(decision.strategy, heuristics::ContextStrategy::DeltaTransport);
}

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
    );
    assert_eq!(decision.fidelity, Fidelity::Low);
}

#[test]
fn test_large_file_low_fidelity() {
    let config = CleanCtxConfig::default();
    let text_delta = TextDeltaComputer::new();
    let ir_ctx = ContextState::new();
    // Create source with many lines to trigger large-file heuristic
    let large_source: String = (0..500).map(|i| format!("line {}\n", i)).collect();
    let decision = heuristics::decide(
        "/test/file.ts",
        None,
        None,
        &config,
        &text_delta,
        &ir_ctx,
        &large_source,
    );
    assert_eq!(decision.fidelity, Fidelity::Low);
}

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
    );
    let summary = decision.summary();
    assert!(summary.contains("fidelity="));
    assert!(summary.contains("strategy="));
    assert!(summary.contains("angular="));
    assert!(summary.contains("lines="));
}