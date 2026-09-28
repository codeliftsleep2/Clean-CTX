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
    precomputed_ngrx: Option<Option<&crate::angular_meta::ngrx::NgRxShape>>,
    precomputed_routing: Option<Option<&crate::angular_meta::routing::RouteShape>>,
    precomputed_angular: Option<bool>,
) -> Vec<SemanticEdge> {
    let meta_config = config.and_then(|value| value.meta_layers.get("angular"));
    let mut edges = Vec::new();

    if precomputed_angular.unwrap_or_else(|| crate::angular_meta::detect::is_angular_file(source)) {
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

    let ngrx_shape = match precomputed_ngrx {
        Some(shape) => shape.cloned(),
        None => lexical_regions.map_or_else(
            || crate::angular_meta::ngrx::extract_ngrx_shape(source, fidelity),
            |regions| {
                crate::angular_meta::ngrx::extract_ngrx_shape_with_regions(
                    source, fidelity, regions,
                )
            },
        ),
    };
    if meta_config.map(|value| value.ngrx.enabled).unwrap_or(true)
        && let Some(shape) = ngrx_shape
    {
        edges.extend(shape.to_ngrx_semantic_edges());
    }
    let routing_shape = match precomputed_routing {
        Some(shape) => shape.cloned(),
        None => lexical_regions.map_or_else(
            || crate::angular_meta::routing::extract_route_shape(source, fidelity),
            |regions| {
                crate::angular_meta::routing::extract_route_shape_with_regions(
                    source, fidelity, regions,
                )
            },
        ),
    };
    if meta_config
        .map(|value| value.routing.enabled)
        .unwrap_or(true)
        && let Some(shape) = routing_shape
    {
        edges.extend(shape.to_semantic_edges());
    }

    edges
}

pub(super) fn evaluate(
    layer_name: &'static str,
    context: &super::MetaLayerContext<'_>,
    precomputed_angular: Option<bool>,
) -> super::MetaLayerEvaluation {
    let meta_config = context
        .config
        .and_then(|config| config.meta_layers.get("angular"));
    let ngrx_enabled = meta_config.map(|config| config.ngrx.enabled).unwrap_or(true);
    let ngrx_shape = ngrx_enabled
        .then(|| crate::angular_meta::ngrx::extract_ngrx_shape_with_regions(
            context.source,
            context.fidelity,
            context.lexical_regions,
        ))
        .flatten();
    let routing_enabled = meta_config
        .map(|config| config.routing.enabled)
        .unwrap_or(true);
    let routing_shape = routing_enabled
        .then(|| crate::angular_meta::routing::extract_route_shape_with_regions(
            context.source,
            context.fidelity,
            context.lexical_regions,
        ))
        .flatten();
    let block = crate::angular_meta::run_meta_layer_with_config_path_regions_and_ngrx(
        context.source,
        context.class_captures,
        context.fidelity,
        meta_config,
        context.path,
        context.lexical_regions,
        Some(ngrx_shape.clone()),
        Some(routing_shape.clone()),
        precomputed_angular,
    );
    let output = block.filter(|block| !block.is_empty()).map(|block| {
        super::MetaLayerOutput {
            layer_name,
            rendered: block.render(),
            angular_block: Some(block),
            ..Default::default()
        }
    });
    let captures: Vec<String> = context
        .paired_class_captures
        .iter()
        .map(|(_, text)| text.clone())
        .collect();
    let mut semantic_edges = extract_non_testing_edges(
        context.source,
        &captures,
        context.fidelity,
        context.config,
        Some(context.lexical_regions),
        Some(ngrx_shape.as_ref()),
        Some(routing_shape.as_ref()),
        precomputed_angular,
    );
    if meta_config.map(|config| config.testing.enabled).unwrap_or(true) {
        semantic_edges.extend(
            crate::angular_meta::testing::extract_testing_semantic_edges_with_regions(
                context.source,
                context.path,
                context.lexical_regions,
            ),
        );
    }
    super::MetaLayerEvaluation {
        output,
        semantic_edges,
    }
}

pub(super) fn evaluate_if_applicable(
    layer_name: &'static str,
    context: &super::MetaLayerContext<'_>,
) -> Option<super::MetaLayerEvaluation> {
    let meta_config = context
        .config
        .and_then(|config| config.meta_layers.get("angular"));
    if meta_config.is_some_and(|config| !config.enabled) {
        return None;
    }
    let is_angular = crate::angular_meta::detect::is_angular_file_with_regions(
        context.source,
        context.lexical_regions,
    );
    let testing_enabled = meta_config.map(|config| config.testing.enabled).unwrap_or(true);
    let reactive_forms_enabled = meta_config
        .map(|config| config.reactive_forms.enabled)
        .unwrap_or(true);
    let formly_enabled = meta_config.map(|config| config.formly.enabled).unwrap_or(true);
    let applicable = is_angular
        || crate::angular_meta::rx::has_rxjs_imports(context.source)
        || crate::angular_meta::ngrx::has_ngrx_imports(context.source)
        || crate::angular_meta::signals::has_signal_imports(context.source)
        || reactive_forms_enabled
            && crate::angular_meta::reactive_forms::has_reactive_forms(context.source)
        || formly_enabled && crate::angular_meta::formly::has_formly(context.source)
        || crate::angular_meta::routing::has_router_imports(context.source)
        || testing_enabled
            && crate::angular_meta::testing::is_testing_source(context.source, context.path);
    applicable.then(|| evaluate(layer_name, context, Some(is_angular)))
}
