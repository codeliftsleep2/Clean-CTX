// src/tests/dotnet_meta/graph_state.rs
//
// Tests for .NET graph state integration.

#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use crate::dotnet_meta::graph_state::DotnetGraphHandle;

    #[test]
    fn test_graph_handle_creation() {
        let handle = DotnetGraphHandle::new();
        assert!(handle.graph().nodes().is_empty());
    }

    #[test]
    fn test_add_file_markers() {
        let mut handle = DotnetGraphHandle::new();
        handle.add_file_markers(
            "Controllers/UserController.cs",
            &["Φctrl:UserController [api/users]".to_string()],
        );
        assert_eq!(handle.graph().nodes().len(), 1);
    }

    #[test]
    fn test_render_footer() {
        let mut handle = DotnetGraphHandle::new();
        handle.add_file_markers(
            "Controllers/UserController.cs",
            &["Φctrl:UserController [api/users]".to_string()],
        );
        let footer = handle.render_footer();
        assert!(footer.contains("§ΦMAP"));
        assert!(footer.contains("Φctrl:"));
        assert!(footer.contains("UserController"));
    }
}