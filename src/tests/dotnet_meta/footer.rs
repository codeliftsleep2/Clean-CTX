// src/tests/dotnet_meta/footer.rs
//
// Tests for .NET footer rendering.

#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use crate::dotnet_meta::footer::render_dotnet_footer;
    use crate::dotnet_meta::graph::{DotnetGraphBuilder, GraphNode};
    use crate::dotnet_meta::markers::PhiLineKind;

    #[test]
    fn test_render_footer_empty() {
        let graph = DotnetGraphBuilder::new().build();
        let footer = render_dotnet_footer(&graph);
        assert!(footer.is_empty());
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
            id: "AppDbContext".to_string(),
            kind: PhiLineKind::DbContext,
            file: "Data/AppDbContext.cs".to_string(),
        });
        let graph = builder.build();

        let footer = render_dotnet_footer(&graph);
        assert!(footer.contains("§ΦMAP"));
        assert!(footer.contains("Φctrl:"));
        assert!(footer.contains("UserController"));
        assert!(footer.contains("Φef:"));
        assert!(footer.contains("AppDbContext"));
    }
}
