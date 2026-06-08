// src/tests/angular_meta/graph.rs
//
// Tests for the Angular cross-file dependency graph (Phase 3, Tier 3).

use crate::angular_meta::graph::{AngularGraph, ClassKind, GraphCollector};

#[test]
fn empty_graph_has_no_classes() {
    let graph = AngularGraph::new();
    assert!(graph.is_empty());
    assert_eq!(graph.len(), 0);
    assert!(!graph.is_resolved());
}

#[test]
fn register_and_resolve_single_service() {
    let mut collector = GraphCollector::new();
    collector.push("LoggerService", "α1", ClassKind::Service, None, &[], None);
    let graph = collector.build_graph();

    assert!(!graph.is_empty());
    assert_eq!(graph.len(), 1);
    assert!(graph.is_resolved());

    let entry = graph.get_class("LoggerService");
    assert!(entry.is_some());
    assert_eq!(entry.unwrap().file_alias, "α1");
    assert_eq!(entry.unwrap().kind, ClassKind::Service);
}

#[test]
fn register_component_with_selector() {
    let mut collector = GraphCollector::new();
    collector.push(
        "UserCardComponent",
        "α2",
        ClassKind::Component,
        Some("app-user-card"),
        &[],
        None,
    );
    let graph = collector.build_graph();

    assert_eq!(graph.len(), 1);

    let resolved = graph.resolve_selector("app-user-card");
    assert_eq!(resolved, Some("UserCardComponent@α2".to_string()));
}

#[test]
fn register_unknown_selector_returns_none() {
    let mut collector = GraphCollector::new();
    collector.push("UserCardComponent", "α2", ClassKind::Component, Some("app-user-card"), &[], None);
    let graph = collector.build_graph();

    let resolved = graph.resolve_selector("app-unknown");
    assert!(resolved.is_none());
}

#[test]
fn resolve_inject_type_finds_registered_service() {
    let mut collector = GraphCollector::new();
    collector.push("UserService", "α3", ClassKind::Service, None, &[], None);
    let graph = collector.build_graph();

    let resolved = graph.resolve_inject_type("UserService");
    assert_eq!(resolved, Some("UserService@α3".to_string()));
}

#[test]
fn resolve_inject_type_unknown_returns_none() {
    let mut collector = GraphCollector::new();
    collector.push("UserService", "α3", ClassKind::Service, None, &[], None);
    let graph = collector.build_graph();

    let resolved = graph.resolve_inject_type("UnknownService");
    assert!(resolved.is_none());
}

#[test]
fn di_chain_direct_injection() {
    let mut collector = GraphCollector::new();
    let empty: Vec<String> = Vec::new();
    let injects = vec!["UserService".to_string()];
    collector.push("UserCardComponent", "α1", ClassKind::Component, Some("app-user-card"), &injects, None);
    collector.push("UserService", "α2", ClassKind::Service, None, &empty, None);
    let graph = collector.build_graph();

    assert_eq!(graph.len(), 2);
    assert!(graph.is_resolved());

    // UserCardComponent should show injects=[UserService@α2]
    let graph_line = graph.format_graph_line("UserCardComponent").unwrap();
    assert!(graph_line.contains("UserService@α2"));
    assert!(graph_line.contains("injects="));
}

#[test]
fn di_chain_transitive_injection() {
    let mut collector = GraphCollector::new();
    let empty: Vec<String> = Vec::new();
    let logger_injects = vec!["LoggerService".to_string()];
    let user_svc_injects = vec!["HttpClient".to_string(), "LoggerService".to_string()];
    collector.push("UserPageComponent", "α1", ClassKind::Component, Some("app-user-page"), &logger_injects, None);
    collector.push("UserService", "α2", ClassKind::Service, None, &user_svc_injects, None);
    collector.push("LoggerService", "α3", ClassKind::Service, None, &empty, None);
    collector.push("HttpClient", "α4", ClassKind::Service, None, &empty, None);
    let graph = collector.build_graph();

    assert_eq!(graph.len(), 4);

    // LoggerService should show injected-by=[UserPageComponent, UserService]
    let logger_line = graph.format_graph_line("LoggerService").unwrap();
    assert!(logger_line.contains("injected-by"));
    assert!(logger_line.contains("UserPageComponent@α1") | logger_line.contains("UserService@α2"));

    // UserService should show injects=[HttpClient@α4, LoggerService@α3]
    let user_svc_line = graph.format_graph_line("UserService").unwrap();
    assert!(user_svc_line.contains("HttpClient@α4") | user_svc_line.contains("LoggerService@α3"));
}

#[test]
fn graph_line_injects_unknown_types_with_question_mark() {
    let mut collector = GraphCollector::new();
    let injects = vec!["UnknownService".to_string()];
    collector.push("UserCardComponent", "α1", ClassKind::Component, None, &injects, None);
    let graph = collector.build_graph();

    let graph_line = graph.format_graph_line("UserCardComponent").unwrap();
    // Unknown types should show with ?
    assert!(graph_line.contains("UnknownService?"));
}

#[test]
fn format_graph_footer_empty_for_unresolved() {
    let graph = AngularGraph::new();
    let footer = graph.format_graph_footer();
    assert!(footer.is_empty());
}

#[test]
fn format_graph_footer_non_empty_for_resolved() {
    let mut collector = GraphCollector::new();
    let empty: Vec<String> = Vec::new();
    collector.push("UserCardComponent", "α1", ClassKind::Component, Some("app-user-card"), &empty, None);
    let graph = collector.build_graph();
    let footer = graph.format_graph_footer();
    assert!(!footer.is_empty());
    assert!(footer.contains("§ΦGRAPH"));
    assert!(footer.contains("α1"));
    assert!(footer.contains("selector=\"app-user-card\""));
}

#[test]
fn register_directive_with_selector() {
    let mut collector = GraphCollector::new();
    collector.push("HighlightDirective", "α5", ClassKind::Directive, Some("[appHighlight]"), &[], None);
    let graph = collector.build_graph();

    assert_eq!(graph.len(), 1);
    let resolved = graph.resolve_selector("[appHighlight]");
    assert_eq!(resolved, Some("HighlightDirective@α5".to_string()));
}

#[test]
fn register_pipe() {
    let mut collector = GraphCollector::new();
    collector.push("UpperCasePipe", "α6", ClassKind::Pipe, None, &[], Some("uppercase"));
    let graph = collector.build_graph();

    assert_eq!(graph.len(), 1);
    let entry = graph.get_class("UpperCasePipe").unwrap();
    assert_eq!(entry.pipe_name, Some("uppercase".to_string()));
}

#[test]
fn class_names_by_kind_filters_correctly() {
    let mut collector = GraphCollector::new();
    let empty: Vec<String> = Vec::new();
    collector.push("UserCardComponent", "α1", ClassKind::Component, None, &empty, None);
    collector.push("UserService", "α2", ClassKind::Service, None, &empty, None);
    collector.push("LoggerService", "α3", ClassKind::Service, None, &empty, None);
    let graph = collector.build_graph();

    let services = graph.class_names_by_kind(ClassKind::Service);
    assert_eq!(services.len(), 2);
    assert!(services.contains(&"UserService".to_string()));
    assert!(services.contains(&"LoggerService".to_string()));

    let components = graph.class_names_by_kind(ClassKind::Component);
    assert_eq!(components.len(), 1);
    assert!(components.contains(&"UserCardComponent".to_string()));
}