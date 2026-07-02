// src/tests/dotnet_meta/footer.rs
//
// Tests for .NET footer rendering.

#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use crate::dotnet_meta::footer::render_dotnet_footer;
    use crate::dotnet_meta::graph::DotnetGraph;
    use crate::dotnet_meta::markers::PhiLineKind;

    #[test]
    fn test_render_footer_empty() {
        let graph = DotnetGraph::new();
        let footer = render_dotnet_footer(&graph);
        assert!(footer.is_empty());
    }

    #[test]
    fn test_render_footer_with_nodes() {
        let mut graph = DotnetGraph::new();
        graph.add_node(crate::dotnet_meta::graph::GraphNode {
            id: "UserController".to_string(),
            kind: PhiLineKind::Controller,
            file: "Controllers/UserController.cs".to_string(),
        });
        graph.add_node(crate::dotnet_meta::graph::GraphNode {
            id: "AppDbContext".to_string(),
            kind: PhiLineKind::DbContext,
            file: "Data/AppDbContext.cs".to_string(),
        });

        let footer = render_dotnet_footer(&graph);
        assert!(footer.contains("§ΦMAP"));
        assert!(footer.contains("Φctrl:"));
        assert!(footer.contains("UserController"));
        assert!(footer.contains("Φef:"));
        assert!(footer.contains("AppDbContext"));
    }
}