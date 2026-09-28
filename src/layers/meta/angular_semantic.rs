//! Shared non-testing Angular semantic projection.

use crate::compression::Fidelity;
use crate::config::CleanCtxConfig;
use crate::layers::meta::semantic::SemanticEdge;

pub(super) fn extract_non_testing_edges(
    source: &str,
    class_captures: &[String],
    fidelity: Fidelity,
    config: Option<&CleanCtxConfig>,
    lexical_regions: Option<&crate::meta_util::LexicalRegions>,
) -> Vec<SemanticEdge> {
    let meta_config = config.and_then(|value| value.meta_layers.get("angular"));
    let mut edges = Vec::new();

    if crate::angular_meta::detect::is_angular_file(source) {
        let mut decl_types = std::collections::HashMap::new();
        for raw_class in class_captures {
            if let Some((class_name, kind, _, _, _)) =
                crate::angular_meta::decorators::extract_graph_entries(raw_class)
            {
                let entity_type = match kind {
                    crate::angular_meta::decorators::ClassKind::Component => "Component",
                    crate::angular_meta::decorators::ClassKind::Service => "Service",
                    crate::angular_meta::decorators::ClassKind::Directive => "Directive",
                    crate::angular_meta::decorators::ClassKind::Pipe => "Pipe",
                    crate::angular_meta::decorators::ClassKind::Module => "Module",
                };
                decl_types.insert(class_name, entity_type);
            }
        }
        for raw_class in class_captures {
            if let Some((class_name, kind, selector, injects, pipe_name)) =
                crate::angular_meta::decorators::extract_graph_entries(raw_class)
            {
                edges.extend(crate::angular_meta::semantic::class_to_semantic_edges(
                    &class_name,
                    kind,
                    selector.as_deref(),
                    &injects,
                    pipe_name.as_deref(),
                    raw_class,
                    fidelity,
                    &decl_types,
                ));
            }
        }
    }

    if meta_config.map(|value| value.ngrx.enabled).unwrap_or(true)
        && let Some(shape) = lexical_regions.map_or_else(
            || crate::angular_meta::ngrx::extract_ngrx_shape(source, fidelity),
            |regions| {
                crate::angular_meta::ngrx::extract_ngrx_shape_with_regions(
                    source, fidelity, regions,
                )
            },
        )
    {
        edges.extend(shape.to_ngrx_semantic_edges());
    }
    if meta_config
        .map(|value| value.routing.enabled)
        .unwrap_or(true)
        && let Some(shape) = lexical_regions.map_or_else(
            || crate::angular_meta::routing::extract_route_shape(source, fidelity),
            |regions| {
                crate::angular_meta::routing::extract_route_shape_with_regions(
                    source, fidelity, regions,
                )
            },
        )
    {
        edges.extend(shape.to_semantic_edges());
    }

    edges
}
