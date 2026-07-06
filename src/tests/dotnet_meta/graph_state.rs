// src/tests/dotnet_meta/graph_state.rs
//
// Tests for .NET graph state integration.

#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use crate::dotnet_meta::graph_state::DotnetGraphHandle;
    use crate::dotnet_meta::graph::{DotnetGraphBuilder, GraphNode};
    use crate::dotnet_meta::markers::PhiLineKind;

    #[test]
    fn test_graph_handle_creation() {
        let handle = DotnetGraphHandle::new();
        assert!(!handle.is_present());
    }

    #[test]
    fn test_set_and_with_graph() {
        let handle = DotnetGraphHandle::new();
        let mut builder = DotnetGraphBuilder::new();
        builder.register_node(GraphNode {
            id: "UserController".to_string(),
            kind: PhiLineKind::Controller,
            file: "Controllers/UserController.cs".to_string(),
        });
        let graph = builder.build();
        handle.set(graph);
        assert!(handle.is_present());

        let count = handle.with_graph(|g| g.nodes().len());
        assert_eq!(count, Some(1));
    }

    #[test]
    fn test_clear() {
        let handle = DotnetGraphHandle::new();
        let mut builder = DotnetGraphBuilder::new();
        builder.register_node(GraphNode {
            id: "Test".to_string(),
            kind: PhiLineKind::Service,
            file: "Test.cs".to_string(),
        });
        handle.set(builder.build());
        assert!(handle.is_present());
        handle.clear();
        assert!(!handle.is_present());
    }

    #[test]
    fn test_with_graph_mut() {
        let handle = DotnetGraphHandle::new();
        let mut builder = DotnetGraphBuilder::new();
        builder.register_node(GraphNode {
            id: "Svc".to_string(),
            kind: PhiLineKind::Service,
            file: "Svc.cs".to_string(),
        });
        handle.set(builder.build());

        // Test mutable access
        let _ = handle.with_graph_mut(|g| {
            // Verify the graph is accessible and resolved
            assert!(g.is_resolved());
            assert!(!g.is_empty());
        });
    }

    #[test]
    fn test_with_graph_none_when_empty() {
        let handle = DotnetGraphHandle::new();
        let result = handle.with_graph(|g| g.nodes().len());
        assert_eq!(result, None);
    }

    #[test]
    fn test_lifecycle() {
        let handle = DotnetGraphHandle::new();
        assert!(!handle.is_present());

        // Build a graph via builder
        let mut builder = DotnetGraphBuilder::new();
        builder.register_node(GraphNode {
            id: "A".to_string(),
            kind: PhiLineKind::Service,
            file: "A.cs".to_string(),
        });
        builder.register_node(GraphNode {
            id: "B".to_string(),
            kind: PhiLineKind::Controller,
            file: "B.cs".to_string(),
        });
        let graph = builder.build();

        handle.set(graph);
        assert!(handle.is_present());

        handle.clear();
        assert!(!handle.is_present());
    }
}