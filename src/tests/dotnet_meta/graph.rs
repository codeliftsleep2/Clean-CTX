// src/tests/dotnet_meta/graph.rs
//
// Tests for .NET dependency graph.

#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use crate::dotnet_meta::graph::{DotnetGraph, DotnetGraphBuilder, GraphNode, GraphEdge};
    use crate::dotnet_meta::markers::PhiLineKind;

    #[test]
    fn test_graph_creation() {
        let graph = DotnetGraphBuilder::new().build();
        assert!(graph.nodes().is_empty());
        assert!(graph.edges().is_empty());
    }

    #[test]
    fn test_add_node() {
        let mut builder = DotnetGraphBuilder::new();
        let node = GraphNode {
            id: "UserController".to_string(),
            kind: PhiLineKind::Controller,
            file: "Controllers/UserController.cs".to_string(),
        };
        builder.register_node(node);
        let graph = builder.build();
        assert_eq!(graph.nodes().len(), 1);
    }

    #[test]
    fn test_nodes_for_file() {
        let mut builder = DotnetGraphBuilder::new();
        let node1 = GraphNode {
            id: "UserController".to_string(),
            kind: PhiLineKind::Controller,
            file: "Controllers/UserController.cs".to_string(),
        };
        let node2 = GraphNode {
            id: "AppDbContext".to_string(),
            kind: PhiLineKind::DbContext,
            file: "Data/AppDbContext.cs".to_string(),
        };
        builder.register_node(node1);
        builder.register_node(node2);
        let graph = builder.build();

        let user_controller_nodes = graph.nodes_for_file("Controllers/UserController.cs");
        assert_eq!(user_controller_nodes.len(), 1);
        assert_eq!(user_controller_nodes[0].id, "UserController");
    }

    #[test]
    fn test_render_footer_empty() {
        let graph = DotnetGraphBuilder::new().build();
        let footer = graph.render_footer();
        assert!(footer.is_empty());
    }

    #[test]
    fn test_has_cycle_no_cycle() {
        let mut builder = DotnetGraphBuilder::new();
        builder.register_node(GraphNode {
            id: "A".to_string(),
            kind: PhiLineKind::Service,
            file: "A.cs".to_string(),
        });
        builder.register_node(GraphNode {
            id: "B".to_string(),
            kind: PhiLineKind::Service,
            file: "B.cs".to_string(),
        });
        builder.register_node(GraphNode {
            id: "C".to_string(),
            kind: PhiLineKind::Service,
            file: "C.cs".to_string(),
        });
        builder.add_edge(GraphEdge {
            from: "A".to_string(),
            to: "B".to_string(),
            kind: "di".to_string(),
        });
        builder.add_edge(GraphEdge {
            from: "B".to_string(),
            to: "C".to_string(),
            kind: "di".to_string(),
        });
        let graph = builder.build();
        assert!(!graph.has_cycle());
    }

    #[test]
    fn test_has_cycle_detected() {
        let mut builder = DotnetGraphBuilder::new();
        builder.register_node(GraphNode {
            id: "A".to_string(),
            kind: PhiLineKind::Service,
            file: "A.cs".to_string(),
        });
        builder.register_node(GraphNode {
            id: "B".to_string(),
            kind: PhiLineKind::Service,
            file: "B.cs".to_string(),
        });
        builder.register_node(GraphNode {
            id: "C".to_string(),
            kind: PhiLineKind::Service,
            file: "C.cs".to_string(),
        });
        builder.add_edge(GraphEdge {
            from: "A".to_string(),
            to: "B".to_string(),
            kind: "di".to_string(),
        });
        builder.add_edge(GraphEdge {
            from: "B".to_string(),
            to: "C".to_string(),
            kind: "di".to_string(),
        });
        builder.add_edge(GraphEdge {
            from: "C".to_string(),
            to: "A".to_string(),
            kind: "di".to_string(),
        });
        let graph = builder.build();
        assert!(graph.has_cycle());
    }

    #[test]
    fn test_find_cycles() {
        let mut builder = DotnetGraphBuilder::new();
        builder.register_node(GraphNode {
            id: "A".to_string(),
            kind: PhiLineKind::Service,
            file: "A.cs".to_string(),
        });
        builder.register_node(GraphNode {
            id: "B".to_string(),
            kind: PhiLineKind::Service,
            file: "B.cs".to_string(),
        });
        builder.register_node(GraphNode {
            id: "C".to_string(),
            kind: PhiLineKind::Service,
            file: "C.cs".to_string(),
        });
        builder.add_edge(GraphEdge {
            from: "A".to_string(),
            to: "B".to_string(),
            kind: "di".to_string(),
        });
        builder.add_edge(GraphEdge {
            from: "B".to_string(),
            to: "C".to_string(),
            kind: "di".to_string(),
        });
        builder.add_edge(GraphEdge {
            from: "C".to_string(),
            to: "A".to_string(),
            kind: "di".to_string(),
        });
        let graph = builder.build();
        let cycles = graph.find_cycles();
        assert!(!cycles.is_empty());
        // The cycle should be A → B → C → A (or a sub-cycle)
        assert!(cycles.iter().any(|c| c.len() >= 2));
    }

    #[test]
    fn test_transitive_dependencies_depth_1() {
        let mut builder = DotnetGraphBuilder::new();
        builder.register_node(GraphNode {
            id: "A".to_string(),
            kind: PhiLineKind::Service,
            file: "A.cs".to_string(),
        });
        builder.register_node(GraphNode {
            id: "B".to_string(),
            kind: PhiLineKind::Service,
            file: "B.cs".to_string(),
        });
        builder.register_node(GraphNode {
            id: "C".to_string(),
            kind: PhiLineKind::Service,
            file: "C.cs".to_string(),
        });
        builder.add_edge(GraphEdge {
            from: "A".to_string(),
            to: "B".to_string(),
            kind: "di".to_string(),
        });
        builder.add_edge(GraphEdge {
            from: "B".to_string(),
            to: "C".to_string(),
            kind: "di".to_string(),
        });
        let graph = builder.build();
        let deps = graph.transitive_dependencies("A", 1);
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0], "B");
    }

    #[test]
    fn test_transitive_dependencies_depth_2() {
        let mut builder = DotnetGraphBuilder::new();
        builder.register_node(GraphNode {
            id: "A".to_string(),
            kind: PhiLineKind::Service,
            file: "A.cs".to_string(),
        });
        builder.register_node(GraphNode {
            id: "B".to_string(),
            kind: PhiLineKind::Service,
            file: "B.cs".to_string(),
        });
        builder.register_node(GraphNode {
            id: "C".to_string(),
            kind: PhiLineKind::Service,
            file: "C.cs".to_string(),
        });
        builder.add_edge(GraphEdge {
            from: "A".to_string(),
            to: "B".to_string(),
            kind: "di".to_string(),
        });
        builder.add_edge(GraphEdge {
            from: "B".to_string(),
            to: "C".to_string(),
            kind: "di".to_string(),
        });
        let graph = builder.build();
        let deps = graph.transitive_dependencies("A", 2);
        assert_eq!(deps.len(), 2);
        assert!(deps.contains(&"B".to_string()));
        assert!(deps.contains(&"C".to_string()));
    }

    #[test]
    fn test_transitive_dependencies_all() {
        let mut builder = DotnetGraphBuilder::new();
        builder.register_node(GraphNode {
            id: "A".to_string(),
            kind: PhiLineKind::Service,
            file: "A.cs".to_string(),
        });
        builder.register_node(GraphNode {
            id: "B".to_string(),
            kind: PhiLineKind::Service,
            file: "B.cs".to_string(),
        });
        builder.register_node(GraphNode {
            id: "C".to_string(),
            kind: PhiLineKind::Service,
            file: "C.cs".to_string(),
        });
        builder.add_edge(GraphEdge {
            from: "A".to_string(),
            to: "B".to_string(),
            kind: "di".to_string(),
        });
        builder.add_edge(GraphEdge {
            from: "B".to_string(),
            to: "C".to_string(),
            kind: "di".to_string(),
        });
        let graph = builder.build();
        let deps = graph.transitive_dependencies("A", 0);
        assert_eq!(deps.len(), 2);
        assert!(deps.contains(&"B".to_string()));
        assert!(deps.contains(&"C".to_string()));
    }

    #[test]
    fn test_build_from_markers() {
        let markers = vec![
            (
                "Controllers/UserController.cs".to_string(),
                vec!["Φctrl:UserController [api/users]".to_string()],
            ),
            (
                "Services/UserService.cs".to_string(),
                vec!["Φsvc:UserService".to_string()],
            ),
            (
                "Data/AppDbContext.cs".to_string(),
                vec!["Φef:AppDbContext".to_string()],
            ),
        ];
        let graph = DotnetGraph::build_from_markers(&markers);
        assert_eq!(graph.nodes().len(), 3);
        assert!(!graph.render_footer().is_empty());
    }

    #[test]
    fn test_parse_node_from_line() {
        let node = DotnetGraph::parse_node_from_line("Φctrl:UserController [api/users]", "Controllers/UserController.cs");
        assert!(node.is_some());
        let node = node.unwrap();
        assert_eq!(node.id, "UserController");
        assert_eq!(node.kind, PhiLineKind::Controller);
        assert_eq!(node.file, "Controllers/UserController.cs");
    }

    #[test]
    fn test_parse_node_from_line_hub() {
        let node = DotnetGraph::parse_node_from_line("Φhub:NotificationHub [INotificationClient]", "Hubs/NotificationHub.cs");
        assert!(node.is_some());
        let node = node.unwrap();
        assert_eq!(node.id, "NotificationHub");
        assert_eq!(node.kind, PhiLineKind::Hub);
    }

    #[test]
    fn test_parse_node_from_line_ef() {
        let node = DotnetGraph::parse_node_from_line("Φef:AppDbContext", "Data/AppDbContext.cs");
        assert!(node.is_some());
        let node = node.unwrap();
        assert_eq!(node.id, "AppDbContext");
        assert_eq!(node.kind, PhiLineKind::DbContext);
    }

    #[test]
    fn test_render_footer_with_nodes() {
        let mut builder = DotnetGraphBuilder::new();
        builder.register_node(GraphNode {
            id: "UserController".to_string(),
            kind: PhiLineKind::Controller,
            file: "Controllers/UserController.cs".to_string(),
        });
        builder.register_node(GraphNode {
            id: "UserService".to_string(),
            kind: PhiLineKind::Service,
            file: "Services/UserService.cs".to_string(),
        });
        builder.add_edge(GraphEdge {
            from: "UserController".to_string(),
            to: "UserService".to_string(),
            kind: "di".to_string(),
        });
        let graph = builder.build();
        let footer = graph.render_footer();
        assert!(footer.contains("§ΦMAP"));
        assert!(footer.contains("Φctrl:"));
        assert!(footer.contains("Φsvc:"));
        assert!(footer.contains("§EDGES"));
        assert!(footer.contains("UserController → UserService"));
    }
}