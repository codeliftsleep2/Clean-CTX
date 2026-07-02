// src/dotnet_meta/footer.rs
//
// `§ΦMAP` workspace footer for .NET bundles.
//
// Mirrors `angular_meta::footer` and `spring_meta::footer` patterns.
// Emits a workspace-level summary of all .NET framework constructs
// detected across the entire workspace.

use super::graph::DotnetGraph;
use super::markers::PhiLineKind;

/// Render the .NET workspace footer (`§ΦMAP` section).
///
/// This footer is appended to the end of a workspace bundle and
/// provides a quick index of all .NET framework constructs found
/// across all files in the workspace.
///
/// # Format
///
/// ```text
/// §ΦMAP
/// Φctrl: UserController, ProductController
/// Φef: AppDbContext
/// Φhub: NotificationHub
/// Φmap: UserProfile
/// ...
/// ```
pub fn render_dotnet_footer(graph: &DotnetGraph) -> String {
    if graph.nodes().is_empty() {
        return String::new();
    }

    let mut footer = String::new();
    footer.push_str("§ΦMAP\n");

    // Group nodes by kind
    let mut by_kind: std::collections::HashMap<PhiLineKind, Vec<String>> =
        std::collections::HashMap::new();

    for node in graph.nodes() {
        by_kind.entry(node.kind).or_default().push(node.id.clone());
    }

    // Render each group in canonical order
    for kind in PhiLineKind::all_in_expand_order() {
        if let Some(ids) = by_kind.get(kind) {
            let prefix = kind.marker_prefix();
            footer.push_str(&format!("{} {}\n", prefix, ids.join(", ")));
        }
    }

    // Render edges if present
    let edges: Vec<_> = graph.edges().to_vec();
    if !edges.is_empty() {
        footer.push_str("§EDGES\n");
        for edge in &edges {
            footer.push_str(&format!("{} → {}\n", edge.from, edge.to));
        }
    }

    footer
}

#[cfg(test)]
#[path = "../tests/dotnet_meta/footer.rs"]
mod tests;