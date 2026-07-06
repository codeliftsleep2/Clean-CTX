// src/tests/spring_meta/graph.rs
//
// Tests for the Spring Boot cross-file dependency graph.

#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use crate::spring_meta::graph::{ClassKind, SpringGraphBuilder};

    #[test]
    fn empty_graph_is_empty() {
        let graph = SpringGraphBuilder::new().build();
        assert!(graph.is_empty());
    }

    #[test]
    fn register_single_service() {
        let mut builder = SpringGraphBuilder::new();
        builder.register_class("UserService", "α1", ClassKind::Service, &[], &[]);
        let graph = builder.build();
        assert!(!graph.is_empty());
        assert_eq!(graph.all_classes().len(), 1);

        let entry = graph.get_class("UserService");
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().file_alias, "α1");
    }

    #[test]
    fn resolve_inject_type() {
        let mut builder = SpringGraphBuilder::new();
        builder.register_class("UserService", "α2", ClassKind::Service, &[], &[]);
        let graph = builder.build();
        let resolved = graph.resolve_inject_type("UserService");
        assert_eq!(resolved, Some("UserService@α2".to_string()));
    }

    #[test]
    fn resolve_inject_type_unknown_returns_none() {
        let graph = SpringGraphBuilder::new().build();
        let resolved = graph.resolve_inject_type("UnknownService");
        assert!(resolved.is_none());
    }

    #[test]
    fn resolve_endpoint() {
        let mut builder = SpringGraphBuilder::new();
        let endpoint = crate::spring_meta::markers::RequestMappingMapping {
            method: Some("GET".to_string()),
            path: "/api/users".to_string(),
        };
        builder.register_class("UserController", "α3", ClassKind::Controller, &[endpoint], &[]);
        let graph = builder.build();
        let resolved = graph.resolve_endpoint("/api/users");
        assert_eq!(resolved, Some("UserController@α3".to_string()));
    }

#[test]
fn format_graph_line() {
    let mut builder = SpringGraphBuilder::new();
    builder.register_class("UserService", "α4", ClassKind::Service, &[], &[]);
    let graph = builder.build();
    let line = graph.format_graph_line("UserService");
    assert!(line.is_some());
    let line_str = line.unwrap();
    assert!(line_str.contains("Φgraph:UserService"));
    assert!(line_str.contains("injects=[]"));
    assert!(line_str.contains("← injected-by=[]"));
}

    #[test]
    fn format_graph_footer_empty_for_empty_graph() {
        let graph = SpringGraphBuilder::new().build();
        assert!(graph.format_graph_footer().is_empty());
    }

    #[test]
    fn format_graph_footer_non_empty() {
        let mut builder = SpringGraphBuilder::new();
        builder.register_class("UserController", "α5", ClassKind::Controller, &[], &[]);
        let graph = builder.build();
        let footer = graph.format_graph_footer();
        assert!(!footer.is_empty());
        assert!(footer.contains("§ΦGRAPH"));
        assert!(footer.contains("α5"));
    }

    #[test]
    fn di_chain() {
        let mut builder = SpringGraphBuilder::new();
        builder.register_class("UserController", "α1", ClassKind::Controller, &[], &["UserService".to_string()]);
        builder.register_class("UserService", "α2", ClassKind::Service, &[], &[]);
        let graph = builder.build();
        assert_eq!(graph.all_classes().len(), 2);

        let ctrl_line = graph.format_graph_line("UserController").unwrap();
        assert!(ctrl_line.contains("UserService@α2"));
    }

    // --- Cycle Detection Tests ---

    #[test]
    fn has_cycle_no_cycle() {
        let mut builder = SpringGraphBuilder::new();
        builder.register_class("SvcA", "α1", ClassKind::Service, &[], &[]);
        builder.register_class("SvcB", "α2", ClassKind::Service, &[], &["SvcA".to_string()]);
        let graph = builder.build();
        assert!(!graph.has_cycle());
    }

    #[test]
    fn has_cycle_detected() {
        let mut builder = SpringGraphBuilder::new();
        builder.register_class("SvcA", "α1", ClassKind::Service, &[], &["SvcB".to_string()]);
        builder.register_class("SvcB", "α2", ClassKind::Service, &[], &["SvcC".to_string()]);
        builder.register_class("SvcC", "α3", ClassKind::Service, &[], &["SvcA".to_string()]);
        let graph = builder.build();
        assert!(graph.has_cycle());
    }

    #[test]
    fn find_cycles_returns_empty_for_acyclic() {
        let mut builder = SpringGraphBuilder::new();
        builder.register_class("SvcA", "α1", ClassKind::Service, &[], &[]);
        builder.register_class("SvcB", "α2", ClassKind::Service, &[], &["SvcA".to_string()]);
        let graph = builder.build();
        let cycles = graph.find_cycles();
        assert!(cycles.is_empty());
    }

    #[test]
    fn find_cycles_detects_simple_cycle() {
        let mut builder = SpringGraphBuilder::new();
        builder.register_class("A", "α1", ClassKind::Service, &[], &["B".to_string()]);
        builder.register_class("B", "α2", ClassKind::Service, &[], &["A".to_string()]);
        let graph = builder.build();
        let cycles = graph.find_cycles();
        assert!(!cycles.is_empty());
        assert!(cycles.iter().any(|c| c.len() >= 2));
    }

    #[test]
    fn transitive_dependencies_depth_1() {
        let mut builder = SpringGraphBuilder::new();
        builder.register_class("A", "α1", ClassKind::Service, &[], &["B".to_string()]);
        builder.register_class("B", "α2", ClassKind::Service, &[], &["C".to_string()]);
        builder.register_class("C", "α3", ClassKind::Service, &[], &[]);
        let graph = builder.build();
        let deps = graph.transitive_dependencies("A", 1);
        assert_eq!(deps.len(), 1);
        assert!(deps.contains(&"B".to_string()));
    }

    #[test]
    fn transitive_dependencies_depth_2() {
        let mut builder = SpringGraphBuilder::new();
        builder.register_class("A", "α1", ClassKind::Service, &[], &["B".to_string()]);
        builder.register_class("B", "α2", ClassKind::Service, &[], &["C".to_string()]);
        builder.register_class("C", "α3", ClassKind::Service, &[], &[]);
        let graph = builder.build();
        let deps = graph.transitive_dependencies("A", 2);
        assert_eq!(deps.len(), 2);
        assert!(deps.contains(&"B".to_string()));
        assert!(deps.contains(&"C".to_string()));
    }
}