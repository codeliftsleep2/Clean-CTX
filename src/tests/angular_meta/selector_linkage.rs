// src/tests/angular_meta/selector_linkage.rs
//
// Tests for selector linkage resolution in the Angular cross-file
// dependency graph (Phase 3, Tier 3). These tests verify that
// custom-element tag names are correctly resolved to their component
// class aliases, and that selector queries work for components,
// directives, and pipes.

use crate::angular_meta::graph::{ClassKind, GraphCollector};

#[test]
fn component_selector_resolved_to_class_alias() {
    let mut collector = GraphCollector::new();
    let empty: Vec<String> = Vec::new();
    collector.push(
        "UserCardComponent",
        "α5",
        ClassKind::Component,
        Some("app-user-card"),
        &empty,
        None,
    );
    let graph = collector.build_graph();

    let resolved = graph.resolve_selector("app-user-card").unwrap();
    assert_eq!(resolved, "UserCardComponent@α5");
}

#[test]
fn directive_selector_resolved() {
    let mut collector = GraphCollector::new();
    let empty: Vec<String> = Vec::new();
    collector.push(
        "HighlightDirective",
        "α7",
        ClassKind::Directive,
        Some("[appHighlight]"),
        &empty,
        None,
    );
    let graph = collector.build_graph();

    let resolved = graph.resolve_selector("[appHighlight]").unwrap();
    assert_eq!(resolved, "HighlightDirective@α7");
}

#[test]
fn multiple_components_with_different_selectors() {
    let mut collector = GraphCollector::new();
    let empty: Vec<String> = Vec::new();
    collector.push(
        "AComponent",
        "α1",
        ClassKind::Component,
        Some("app-a"),
        &empty,
        None,
    );
    collector.push(
        "BComponent",
        "α2",
        ClassKind::Component,
        Some("app-b"),
        &empty,
        None,
    );
    collector.push(
        "CComponent",
        "α3",
        ClassKind::Component,
        Some("app-c"),
        &empty,
        None,
    );
    let graph = collector.build_graph();

    assert_eq!(graph.resolve_selector("app-a").unwrap(), "AComponent@α1");
    assert_eq!(graph.resolve_selector("app-b").unwrap(), "BComponent@α2");
    assert_eq!(graph.resolve_selector("app-c").unwrap(), "CComponent@α3");
}

#[test]
fn component_without_selector_not_found() {
    let mut collector = GraphCollector::new();
    let empty: Vec<String> = Vec::new();
    collector.push(
        "InternalComponent",
        "α4",
        ClassKind::Component,
        None,
        &empty,
        None,
    );
    let graph = collector.build_graph();

    // No selector was registered, so resolving any selector should fail.
    let resolved = graph.resolve_selector("app-internal");
    assert!(resolved.is_none());
}

#[test]
fn selector_not_found_returns_none() {
    let mut collector = GraphCollector::new();
    let empty: Vec<String> = Vec::new();
    collector.push(
        "AComponent",
        "α1",
        ClassKind::Component,
        Some("app-a"),
        &empty,
        None,
    );
    let graph = collector.build_graph();

    let resolved = graph.resolve_selector("app-nonexistent");
    assert!(resolved.is_none());
}

#[test]
fn format_graph_line_includes_selector() {
    let mut collector = GraphCollector::new();
    let empty: Vec<String> = Vec::new();
    collector.push(
        "UserCardComponent",
        "α1",
        ClassKind::Component,
        Some("app-user-card"),
        &empty,
        None,
    );
    let graph = collector.build_graph();

    let footer = graph.format_graph_footer();
    assert!(footer.contains("app-user-card"));
    assert!(footer.contains("selector=\"app-user-card\""));
}

#[test]
fn format_graph_footer_lists_all_classes() {
    let mut collector = GraphCollector::new();
    let empty: Vec<String> = Vec::new();
    collector.push(
        "AComponent",
        "α1",
        ClassKind::Component,
        Some("app-a"),
        &empty,
        None,
    );
    collector.push("BService", "α2", ClassKind::Service, None, &empty, None);
    let graph = collector.build_graph();

    let footer = graph.format_graph_footer();
    assert!(footer.contains("cmp AComponent@α1"));
    assert!(footer.contains("svc BService@α2"));
}

#[test]
fn selector_resolution_works_with_injection() {
    // A component that both has a selector and injects a service.
    let mut collector = GraphCollector::new();
    let empty: Vec<String> = Vec::new();
    let injects = vec!["UserService".to_string()];
    collector.push(
        "UserCardComponent",
        "α1",
        ClassKind::Component,
        Some("app-user-card"),
        &injects,
        None,
    );
    collector.push("UserService", "α2", ClassKind::Service, None, &empty, None);
    let graph = collector.build_graph();

    // Selector resolution works
    assert_eq!(
        graph.resolve_selector("app-user-card").unwrap(),
        "UserCardComponent@α1"
    );

    // Injection resolution works
    let line = graph.format_graph_line("UserCardComponent").unwrap();
    assert!(line.contains("UserService@α2"));

    // Selector shows up in the graph footer, not in format_graph_line
    let footer = graph.format_graph_footer();
    assert!(footer.contains("app-user-card"));
    assert!(footer.contains("selector=\"app-user-card\""));
}

#[test]
fn selector_of_directive_not_confused_with_component() {
    let mut collector = GraphCollector::new();
    let empty: Vec<String> = Vec::new();
    collector.push(
        "AppCardComponent",
        "α1",
        ClassKind::Component,
        Some("app-card"),
        &empty,
        None,
    );
    collector.push(
        "CardHighlightDirective",
        "α2",
        ClassKind::Directive,
        Some("[appCard]"),
        &empty,
        None,
    );
    let graph = collector.build_graph();

    assert_eq!(
        graph.resolve_selector("app-card").unwrap(),
        "AppCardComponent@α1"
    );
    assert_eq!(
        graph.resolve_selector("[appCard]").unwrap(),
        "CardHighlightDirective@α2"
    );
}

#[test]
fn pipe_does_not_have_selector() {
    let mut collector = GraphCollector::new();
    let empty: Vec<String> = Vec::new();
    collector.push(
        "MyPipe",
        "α3",
        ClassKind::Pipe,
        None,
        &empty,
        Some("myPipe"),
    );
    let graph = collector.build_graph();

    // Pipes don't have selectors, so resolving any selector should return None.
    assert!(graph.resolve_selector("myPipe").is_none());
}
