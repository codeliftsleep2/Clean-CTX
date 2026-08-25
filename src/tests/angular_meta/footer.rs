// src/tests/angular_meta/footer.rs
//
// Tests for the §ΦMAP footer formatter.

use crate::angular_meta::footer::{BundleEntry, FooterBuilder, format_bundle_footer};

#[test]
fn format_bundle_footer_returns_empty_for_no_entries() {
    let footer = format_bundle_footer(&[]);
    assert!(footer.is_empty());
}

#[test]
fn format_bundle_footer_contains_header() {
    let entries = vec![BundleEntry {
        alias: "Φ1".to_string(),
        name: "user-card.component".to_string(),
        file_aliases: vec!["α1".to_string(), "α2".to_string()],
        template_summary: None,
        style_summary: None,
    }];
    let footer = format_bundle_footer(&entries);
    assert!(footer.contains("§ΦMAP"));
    assert!(footer.contains("Φ1 = user-card.component"));
    assert!(footer.contains("[α1, α2]"));
}

#[test]
fn format_bundle_footer_includes_template_summary() {
    let entries = vec![BundleEntry {
        alias: "Φ1".to_string(),
        name: "user-card.component".to_string(),
        file_aliases: vec!["α1".to_string()],
        template_summary: Some("Φtpl:div,span".to_string()),
        style_summary: None,
    }];
    let footer = format_bundle_footer(&entries);
    assert!(footer.contains("Φtpl:div,span"));
}

#[test]
fn format_bundle_footer_includes_style_summary() {
    let entries = vec![BundleEntry {
        alias: "Φ1".to_string(),
        name: "user-card.component".to_string(),
        file_aliases: vec!["α1".to_string()],
        template_summary: None,
        style_summary: Some("Φsty:.card $primary".to_string()),
    }];
    let footer = format_bundle_footer(&entries);
    assert!(footer.contains("Φsty:.card $primary"));
}

#[test]
fn footer_builder_registers_bundles() {
    let mut builder = FooterBuilder::new();
    let alias1 = builder.register_bundle(
        "user-card.component".to_string(),
        vec!["α1".to_string(), "α2".to_string()],
        None,
        None,
    );
    assert_eq!(alias1, "Φ1");

    let alias2 = builder.register_bundle(
        "user-page.component".to_string(),
        vec!["α3".to_string(), "α4".to_string()],
        None,
        None,
    );
    assert_eq!(alias2, "Φ2");

    assert_eq!(builder.len(), 2);
    assert!(!builder.is_empty());
}

#[test]
fn footer_builder_build_produces_correct_output() {
    let mut builder = FooterBuilder::new();
    builder.register_bundle(
        "user-card.component".to_string(),
        vec!["α1".to_string(), "α2".to_string()],
        Some("Φtpl:div".to_string()),
        Some("Φsty:.card".to_string()),
    );

    let footer = builder.build();
    assert!(footer.contains("§ΦMAP"));
    assert!(footer.contains("Φ1 = user-card.component"));
    assert!(footer.contains("[α1, α2]"));
    assert!(footer.contains("Φtpl:div"));
    assert!(footer.contains("Φsty:.card"));
}

#[test]
fn footer_builder_find_by_name() {
    let mut builder = FooterBuilder::new();
    builder.register_bundle(
        "my-component".to_string(),
        vec!["α1".to_string()],
        None,
        None,
    );

    assert!(builder.find_by_name("my-component").is_some());
    assert!(builder.find_by_name("other-component").is_none());
}

#[test]
fn footer_builder_empty_is_empty() {
    let builder = FooterBuilder::new();
    assert!(builder.is_empty());
    assert_eq!(builder.len(), 0);
    assert!(builder.build().is_empty());
}
