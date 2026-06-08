// src/tests/angular_meta/di_resolution.rs
//
// Tests for DI resolution in the Angular cross-file dependency graph
// (Phase 3, Tier 3). These tests verify that constructor-injected
// types are correctly resolved to their file-aliased forms.

use crate::angular_meta::graph::{ClassKind, GraphCollector};

#[test]
fn resolve_direct_injection_dependency() {
    // Component A injects Service B. Both are registered.
    let mut collector = GraphCollector::new();
    let empty: Vec<String> = Vec::new();
    let injects = vec!["BService".to_string()];
    collector.push("AComponent", "α1", ClassKind::Component, Some("app-a"), &injects, None);
    collector.push("BService", "α2", ClassKind::Service, None, &empty, None);
    let graph = collector.build_graph();

    let resolved = graph.resolve_inject_type("BService");
    assert_eq!(resolved, Some("BService@α2".to_string()));

    let graph_line = graph.format_graph_line("AComponent").unwrap();
    assert!(graph_line.contains("BService@α2"));
    assert!(graph_line.contains("injects=[BService@α2]"));
}

#[test]
fn resolve_transitive_dependency() {
    // A → B → C (A injects B, B injects C).
    let mut collector = GraphCollector::new();
    let empty: Vec<String> = Vec::new();
    let a_injects = vec!["BService".to_string()];
    let b_injects = vec!["CService".to_string()];
    collector.push("AComponent", "α1", ClassKind::Component, None, &a_injects, None);
    collector.push("BService", "α2", ClassKind::Service, None, &b_injects, None);
    collector.push("CService", "α3", ClassKind::Service, None, &empty, None);
    let graph = collector.build_graph();

    // A should show injects=[BService@α2]
    let a_line = graph.format_graph_line("AComponent").unwrap();
    assert!(a_line.contains("BService@α2"));
    assert!(!a_line.contains("CService")); // A does not directly inject C

    // B should show injects=[CService@α3]
    let b_line = graph.format_graph_line("BService").unwrap();
    assert!(b_line.contains("CService@α3"));
}

#[test]
fn injected_by_reverse_edges() {
    // A and B both inject C.
    let mut collector = GraphCollector::new();
    let empty: Vec<String> = Vec::new();
    let a_injects = vec!["CService".to_string()];
    let b_injects = vec!["CService".to_string()];
    collector.push("AComponent", "α1", ClassKind::Component, None, &a_injects, None);
    collector.push("BComponent", "α2", ClassKind::Component, None, &b_injects, None);
    collector.push("CService", "α3", ClassKind::Service, None, &empty, None);
    let graph = collector.build_graph();

    // C should show injected-by=[AComponent@α1, BComponent@α2]
    let c_line = graph.format_graph_line("CService").unwrap();
    assert!(c_line.contains("AComponent@α1"));
    assert!(c_line.contains("BComponent@α2"));
    assert!(c_line.contains("injected-by=["));
}

#[test]
fn no_false_positive_for_duplicate_class_name() {
    // Two services with the same name in different files.
    // The last registered wins, which is an acceptable limitation.
    let mut collector = GraphCollector::new();
    let empty: Vec<String> = Vec::new();
    collector.push("ConfigService", "α1", ClassKind::Service, None, &empty, None);
    collector.push("ConfigService", "α2", ClassKind::Service, None, &empty, None);
    let graph = collector.build_graph();

    // Only one should be tracked (last wins)
    assert_eq!(graph.len(), 1);

    // The resolved alias should be the last one registered
    let resolved = graph.resolve_inject_type("ConfigService").unwrap();
    assert_eq!(resolved, "ConfigService@α2");
}

#[test]
fn resolution_failure_silently_dropped() {
    // Class injects a type that is not registered anywhere in the graph.
    let mut collector = GraphCollector::new();
    let injects = vec!["UnknownType".to_string()];
    collector.push("AComponent", "α1", ClassKind::Component, None, &injects, None);
    let graph = collector.build_graph();

    // Should not panic or error — unknown types get `?` suffix.
    let line = graph.format_graph_line("AComponent").unwrap();
    assert!(line.contains("UnknownType?"));
    assert!(!line.contains("ERROR")); // no error messages
}

#[test]
fn external_dependency_not_in_graph() {
    // Class injects an external dependency like `HttpClient` that is
    // not part of the workspace — should show as unresolved `?`.
    let mut collector = GraphCollector::new();
    let injects = vec!["HttpClient".to_string()];
    collector.push("UserService", "α1", ClassKind::Service, None, &injects, None);
    let graph = collector.build_graph();

    let line = graph.format_graph_line("UserService").unwrap();
    assert!(line.contains("HttpClient?"));
}

#[test]
fn service_with_no_dependencies_has_empty_injects() {
    let mut collector = GraphCollector::new();
    let empty: Vec<String> = Vec::new();
    collector.push("StandaloneService", "α1", ClassKind::Service, None, &empty, None);
    let graph = collector.build_graph();

    let line = graph.format_graph_line("StandaloneService").unwrap();
    assert!(line.contains("injects=[]"));
}

#[test]
fn module_class_resolution() {
    let mut collector = GraphCollector::new();
    let empty: Vec<String> = Vec::new();
    collector.push("AppModule", "α1", ClassKind::Module, None, &empty, None);
    let graph = collector.build_graph();

    let entry = graph.get_class("AppModule").unwrap();
    assert_eq!(entry.kind, ClassKind::Module);
}

#[test]
fn multiple_injects_in_single_class() {
    let mut collector = GraphCollector::new();
    let empty: Vec<String> = Vec::new();
    let injects = vec!["ServiceA".to_string(), "ServiceB".to_string(), "ServiceC".to_string()];
    collector.push("BigComponent", "α1", ClassKind::Component, None, &injects, None);
    collector.push("ServiceA", "α2", ClassKind::Service, None, &empty, None);
    collector.push("ServiceB", "α3", ClassKind::Service, None, &empty, None);
    collector.push("ServiceC", "α4", ClassKind::Service, None, &empty, None);
    let graph = collector.build_graph();

    let line = graph.format_graph_line("BigComponent").unwrap();
    assert!(line.contains("ServiceA@α2"));
    assert!(line.contains("ServiceB@α3"));
    assert!(line.contains("ServiceC@α4"));
}