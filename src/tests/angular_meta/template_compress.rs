// src/tests/angular_meta/template_compress.rs
//
// Tests for the fidelity-gated Angular template compression
// (ANGULAR_HTML_COMPRESSION_PLAN Phase 1 + Phase 4).

use crate::angular_meta::template::extract_template_shape;
use crate::angular_meta::template_compress::{
    compress_template, compress_template_to_string, compress_template_with_prime_ng,
    extract_prime_ng_markers, is_prime_ng_component,
};
use crate::compression::Fidelity;

// ── Low fidelity: single-line shape summary (current behavior) ─────

#[test]
fn low_fidelity_single_line() {
    let html = r#"<div><span>{{ name }}</span></div>"#;
    let lines = compress_template(html, Fidelity::Low);
    assert_eq!(lines.len(), 1, "Low fidelity should produce a single line");
    assert!(lines[0].starts_with("Φtpl:"));
    assert!(lines[0].contains("div"));
    assert!(lines[0].contains("span"));
}

#[test]
fn low_fidelity_byte_identical_to_marker_line() {
    let html =
        r#"<div *ngIf="show"><app-card [title]="name" (click)="handler()"></app-card></div>"#;
    let shape = extract_template_shape(html);
    let lines = compress_template(html, Fidelity::Low);
    assert_eq!(
        lines[0],
        shape.to_marker_line(),
        "Low fidelity must match to_marker_line"
    );
}

// ── Medium fidelity: multi-line structural Angular semantics ───────

#[test]
fn medium_fidelity_multi_line() {
    let html = r#"<div><span>{{ name }}</span></div>"#;
    let lines = compress_template(html, Fidelity::Medium);
    assert!(
        lines.len() > 1,
        "Medium fidelity should produce multiple lines"
    );
    assert!(lines[0].starts_with("Φtpl:"));
}

#[test]
fn medium_fidelity_preserves_if_condition() {
    let html = r#"@if (isLoggedIn) { <span>Hello</span> }"#;
    let lines = compress_template(html, Fidelity::Medium);
    assert!(
        lines.iter().any(|l| l.contains("@if(isLoggedIn)")),
        "Medium fidelity should preserve @if condition, got: {:?}",
        lines
    );
}

#[test]
fn medium_fidelity_preserves_for_loop() {
    let html = r#"@for (item of items; track item.id) { <li>{{ item.name }}</li> }"#;
    let lines = compress_template(html, Fidelity::Medium);
    assert!(
        lines.iter().any(|l| l.contains("@for(item of items)")),
        "Medium fidelity should preserve @for loop, got: {:?}",
        lines
    );
}

#[test]
fn medium_fidelity_preserves_custom_element_bindings() {
    let html = r#"<app-user-card [user]="user" (select)="onSelect($event)"></app-user-card>"#;
    let lines = compress_template(html, Fidelity::Medium);
    assert!(
        lines
            .iter()
            .any(|l| l.contains("app-user-card") && l.contains("[user]=\"user\"")),
        "Medium fidelity should preserve custom element bindings, got: {:?}",
        lines
    );
}

#[test]
fn medium_fidelity_preserves_ng_if_condition() {
    let html = r#"<div *ngIf="showCard"><span>Card</span></div>"#;
    let lines = compress_template(html, Fidelity::Medium);
    assert!(
        lines.iter().any(|l| l.contains("@if(showCard)")),
        "Medium fidelity should preserve *ngIf condition, got: {:?}",
        lines
    );
}

#[test]
fn medium_fidelity_preserves_ng_for_loop() {
    let html = r#"<li *ngFor="let item of items">{{ item.name }}</li>"#;
    let lines = compress_template(html, Fidelity::Medium);
    assert!(
        lines.iter().any(|l| l.contains("@for(item of items)")),
        "Medium fidelity should preserve *ngFor loop, got: {:?}",
        lines
    );
}

// ── High fidelity: near-full template with scaffolding stripped ────

#[test]
fn high_fidelity_preserves_all_elements() {
    let html = r#"<div class="container"><h1>{{ title }}</h1><app-card [data]="cardData"></app-card></div>"#;
    let lines = compress_template(html, Fidelity::High);
    assert!(
        lines.len() > 1,
        "High fidelity should produce multiple lines"
    );
    assert!(
        lines
            .iter()
            .any(|l| l.contains("app-card") && l.contains("[data]=\"cardData\"")),
        "High fidelity should preserve all custom elements, got: {:?}",
        lines
    );
}

#[test]
fn high_fidelity_preserves_interpolation_count() {
    let html = r#"<div>{{ name }} has {{ count }} items</div>"#;
    let lines = compress_template(html, Fidelity::High);
    assert!(
        lines.iter().any(|l| l.contains("{{}}x2")),
        "High fidelity should preserve interpolation count, got: {:?}",
        lines
    );
}

// ── PrimeNG pattern recognition (Phase 4) ──────────────────────────

#[test]
fn is_prime_ng_component_detects_p_prefix() {
    assert!(is_prime_ng_component("p-table"));
    assert!(is_prime_ng_component("p-card"));
    assert!(is_prime_ng_component("p-inputtext"));
    assert!(!is_prime_ng_component("app-user-card"));
    assert!(!is_prime_ng_component("div"));
}

#[test]
fn extract_prime_ng_markers_finds_components() {
    let html = r#"<p-table [value]="rows"><p-card><p-inputtext /></p-card></p-table>"#;
    let shape = extract_template_shape(html);
    let markers = extract_prime_ng_markers(&shape);
    assert!(markers.contains(&"Φp-table:".to_string()));
    assert!(markers.contains(&"Φp-card:".to_string()));
    assert!(markers.contains(&"Φp-inputtext:".to_string()));
}

#[test]
fn compress_template_with_prime_ng_appends_markers() {
    let html = r#"<p-table [value]="rows"></p-table>"#;
    let lines = compress_template_with_prime_ng(html, Fidelity::Low);
    assert!(
        lines.iter().any(|l| l.contains("Φp-table:")),
        "PrimeNG markers should be appended, got: {:?}",
        lines
    );
}

#[test]
fn compress_template_to_string_joins_lines() {
    let html = r#"@if (cond) { <span>Hello</span> }"#;
    let s = compress_template_to_string(html, Fidelity::Medium);
    assert!(s.contains('\n'), "Medium fidelity should be multi-line");
    assert!(s.starts_with("Φtpl:"));
}

// ── Empty / edge cases ─────────────────────────────────────────────

#[test]
fn empty_template_low_fidelity() {
    let lines = compress_template("", Fidelity::Low);
    assert_eq!(lines, vec!["Φtpl:empty".to_string()]);
}

#[test]
fn empty_template_medium_fidelity() {
    let lines = compress_template("", Fidelity::Medium);
    assert_eq!(lines, vec!["Φtpl:empty".to_string()]);
}

#[test]
fn empty_template_high_fidelity() {
    let lines = compress_template("", Fidelity::High);
    assert_eq!(lines, vec!["Φtpl:empty".to_string()]);
}
