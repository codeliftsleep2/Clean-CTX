// src/tests/dotnet_meta/graph.rs
//
// Tests for .NET dependency graph.

#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use crate::dotnet_meta::graph::{DotnetGraph, GraphNode};
    use crate::dotnet_meta::markers::PhiLineKind;

    #[test]
    fn test_graph_creation() {
        let graph = DotnetGraph::new();
        assert!(graph.nodes().is_empty());
        assert!(graph.edges().is_empty());
    }

    #[test]
    fn test_add_node() {
        let mut graph = DotnetGraph::new();
        let node = GraphNode {
            id: "UserController".to_string(),
            kind: PhiLineKind::Controller,
            file: "Controllers/UserController.cs".to_string(),
        };
        graph.add_node(node);
        assert_eq!(graph.nodes().len(), 1);
    }

    #[test]
    fn test_nodes_for_file() {
        let mut graph = DotnetGraph::new();
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
        graph.add_node(node1);
        graph.add_node(node2);

        let user_controller_nodes = graph.nodes_for_file("Controllers/UserController.cs");
        assert_eq!(user_controller_nodes.len(), 1);
        assert_eq!(user_controller_nodes[0].id, "UserController");
    }

    #[test]
    fn test_render_footer_empty() {
        let graph = DotnetGraph::new();
        let footer = graph.render_footer();
        assert!(footer.is_empty());
    }
}