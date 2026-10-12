//! Shared non-testing Angular semantic projection.

use crate::compression::Fidelity;
use crate::config::CleanCtxConfig;
use crate::layers::meta::semantic::SemanticEdge;

#[derive(Default)]
pub(super) struct ProjectionEvidence<'a> {
    lexical_regions: Option<&'a crate::meta_util::LexicalRegions>,
    ngrx: Option<Option<&'a crate::angular_meta::ngrx::NgRxShape>>,
    routing: Option<Option<&'a crate::angular_meta::routing::RouteShape>>,
    is_angular: Option<bool>,
    initializers: Option<&'a [crate::angular_meta::inject_evidence::ClassInitializerEvidence]>,
}

pub(super) fn extract_non_testing_edges(
    source: &str,
    class_captures: &[String],
    fidelity: Fidelity,
    config: Option<&CleanCtxConfig>,
    declaring_path: Option<&std::path::Path>,
    evidence: ProjectionEvidence<'_>,
) -> Vec<SemanticEdge> {
    let meta_config = config.and_then(|value| value.meta_layers.get("angular"));
    let mut edges = Vec::new();

    if evidence
        .is_angular
        .unwrap_or_else(|| crate::angular_meta::detect::is_angular_file(source))
    {
        let classes: Vec<_> = match evidence.initializers {
            Some(classes) => classes
                .iter()
                .map(|class| {
                    (
                        Some(class.consumer.as_str()),
                        class.raw_class.as_str(),
                        class.admitted_tokens(),
                    )
                })
                .collect(),
            None => class_captures
                .iter()
                .map(|raw| (None, raw.as_str(), Vec::new()))
                .collect(),
        };
        let mut decl_types = std::collections::HashMap::new();
        for (owner, raw_class, _) in &classes {
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
                decl_types.insert(owner.unwrap_or(&class_name).to_string(), entity_type);
            }
        }
        for (owner, raw_class, tokens) in classes {
            if let Some((class_name, kind, selector, mut injects, pipe_name)) =
                crate::angular_meta::decorators::extract_graph_entries(raw_class)
            {
                let class_name = owner.unwrap_or(&class_name);
                injects.extend(tokens);
                edges.extend(crate::angular_meta::semantic::class_to_semantic_edges(
                    class_name,
                    kind,
                    selector.as_deref(),
                    &injects,
                    pipe_name.as_deref(),
                    raw_class,
                    fidelity,
                    &decl_types,
                ));
                if let Some(path) = declaring_path
                    && let Some(edge) = crate::angular_meta::semantic::component_template_edge(
                        class_name, kind, raw_class, path,
                    )
                {
                    edges.push(edge);
                }
            }
        }
    }

    let ngrx_shape = match evidence.ngrx {
        Some(shape) => shape.cloned(),
        None => evidence.lexical_regions.map_or_else(
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
    let routing_shape = match evidence.routing {
        Some(shape) => shape.cloned(),
        None => evidence.lexical_regions.map_or_else(
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
    let ngrx_enabled = meta_config
        .map(|config| config.ngrx.enabled)
        .unwrap_or(true);
    let ngrx_shape = ngrx_enabled
        .then(|| {
            let shape = crate::angular_meta::ngrx::extract_ngrx_shape_with_regions(
                context.source,
                context.fidelity,
                context.lexical_regions,
            );
            crate::angular_meta::ngrx::legacy::include(
                shape,
                context.ngrx_class_actions.unwrap_or(&[]),
            )
        })
        .flatten();
    let routing_enabled = meta_config
        .map(|config| config.routing.enabled)
        .unwrap_or(true);
    let routing_shape = routing_enabled
        .then(|| {
            crate::angular_meta::routing::extract_route_shape_with_regions(
                context.source,
                context.fidelity,
                context.lexical_regions,
            )
        })
        .flatten();
    let block = crate::angular_meta::run_meta_layer_with_config_path_regions_and_evidence(
        context.source,
        context.class_captures,
        context.fidelity,
        meta_config,
        context.path,
        context.lexical_regions,
        crate::angular_meta::PrecomputedAngularEvidence {
            rx: context.invocation_evidence.map(|view| {
                crate::angular_meta::rx::extract_rx_shape_with_evidence(
                    context.source,
                    context.lexical_regions,
                    view,
                )
            }),
            ngrx: Some(ngrx_shape.clone()),
            routing: Some(routing_shape.clone()),
            is_angular: precomputed_angular,
            initializers: context.angular_initializers,
        },
    );
    let output = block
        .filter(|block| !block.is_empty())
        .map(|block| super::MetaLayerOutput {
            layer_name,
            rendered: block.render(),
            angular_block: Some(block),
            ..Default::default()
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
        Some(context.path),
        ProjectionEvidence {
            lexical_regions: Some(context.lexical_regions),
            ngrx: Some(ngrx_shape.as_ref()),
            routing: Some(routing_shape.as_ref()),
            is_angular: precomputed_angular,
            initializers: context.angular_initializers,
        },
    );
    if meta_config
        .map(|config| config.testing.enabled)
        .unwrap_or(true)
    {
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
    let testing_enabled = meta_config
        .map(|config| config.testing.enabled)
        .unwrap_or(true);
    let reactive_forms_enabled = meta_config
        .map(|config| config.reactive_forms.enabled)
        .unwrap_or(true);
    let formly_enabled = meta_config
        .map(|config| config.formly.enabled)
        .unwrap_or(true);
    let applicable = is_angular
        || crate::angular_meta::rx::has_rxjs_imports(context.source)
        || crate::angular_meta::ngrx::has_ngrx_imports(context.source)
        || crate::angular_meta::signals::has_signal_imports(context.source)
        || reactive_forms_enabled
            && crate::angular_meta::reactive_forms::extract_reactive_form_shape_with_regions(
                context.source,
                Fidelity::Low,
                context.lexical_regions,
            )
            .is_some()
        || formly_enabled
            && crate::angular_meta::formly::extract_formly_shape_with_regions(
                context.source,
                Fidelity::Low,
                context.lexical_regions,
            )
            .is_some()
        || crate::angular_meta::routing::has_router_imports(context.source)
        || testing_enabled
            && crate::angular_meta::testing::is_testing_source(context.source, context.path);
    applicable.then(|| evaluate(layer_name, context, Some(is_angular)))
}
