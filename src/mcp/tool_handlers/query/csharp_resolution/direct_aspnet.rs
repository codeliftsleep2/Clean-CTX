use super::{AspNetProjection, Catalog, Declaration};

pub(in crate::mcp::tool_handlers::query) fn projection_for_catalog(
    catalog: &Catalog,
    name: &str,
    fidelity: crate::compression::Fidelity,
) -> Option<AspNetProjection> {
    let declaration = catalog.unique_named_class(name)?;
    (declaration.source.contains("[ApiController]") || declaration.source.contains("[Controller]"))
        .then(|| projection(declaration, fidelity))?
}

pub(super) fn projection(
    declaration: &Declaration,
    fidelity: crate::compression::Fidelity,
) -> Option<AspNetProjection> {
    // Direct current declaration evidence must not depend on survival of a
    // derived durable relationship in the workspace index.
    let mut analysis =
        crate::dotnet_meta::aspnet::analyze_authorized_controller(&declaration.source, fidelity)?;
    for edge in &mut analysis.semantic_edges {
        edge.subject.file = Some(declaration.file.clone());
        edge.object.file = Some(declaration.file.clone());
    }
    Some(AspNetProjection {
        file: declaration.file.clone(),
        edges: analysis.semantic_edges,
    })
}
