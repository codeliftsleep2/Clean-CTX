// src/spring_meta/semantic.rs
//
// Spring-specific semantic edge construction helpers.
//
// These follow the same scanning patterns as the existing Spring meta-layer
// extractors (annotations) but produce SemanticEdge objects instead of
// Phi marker strings.
//
// Phase 3 contract: zero duplication of existing Phi output. Semantic edges
// are a separate projection of the same framework information.

use crate::compression::Fidelity;
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};
use crate::spring_meta::annotations::{
    AnnotationKind, annotation_kind_to_http_method, collect_annotations, collect_field_annotations,
    collect_method_annotations, extract_class_name, find_class_body_open, find_class_head_end,
    parse_mapping_paths, parse_request_mappings,
};
use crate::spring_meta::markers::RequestMappingMapping;

/// Extract Spring semantic edges from a single class capture.
/// Reuses the same string-scanning patterns as the existing Spring
/// annotations extractor.
pub fn extract_spring_semantic_edges(raw_class: &str, fidelity: Fidelity) -> Vec<SemanticEdge> {
    let mut edges: Vec<SemanticEdge> = Vec::new();
    let class_name = match extract_class_name(raw_class) {
        Some(n) => n,
        None => return edges,
    };

    let head_end = match find_class_head_end(raw_class) {
        Some(e) => e,
        None => return edges,
    };
    let head = &raw_class[..head_end];
    let annotations = collect_annotations(head);

    let mut is_controller = false;
    let mut is_configuration = false;
    let mut request_mappings: Vec<RequestMappingMapping> = Vec::new();
    let mut bean_methods: Vec<String> = Vec::new();
    let mut has_config_props = false;

    for anno in &annotations {
        match anno.kind {
            AnnotationKind::RestController | AnnotationKind::Controller => {
                is_controller = true;
                let submappings = parse_request_mappings(&anno.arg);
                for mapping in &submappings {
                    request_mappings.push(mapping.clone());
                }
            }
            AnnotationKind::RequestMapping => {
                let submappings = parse_request_mappings(&anno.arg);
                for mapping in &submappings {
                    request_mappings.push(mapping.clone());
                }
            }
            AnnotationKind::Configuration => {
                is_configuration = true;
            }
            AnnotationKind::Bean => {
                bean_methods.push(anno.arg.trim().to_string());
            }
            AnnotationKind::ConfigurationProperties => {
                has_config_props = true;
            }
            _ => {}
        }
    }

    // Method-level mappings: scan the class body for @GetMapping, @PostMapping,
    // @PutMapping, @DeleteMapping, @PatchMapping (same pattern as extract_annotations).
    if fidelity != Fidelity::Low {
        if let Some(class_body_start) = find_class_body_open(raw_class) {
            if let Some(body_end) =
                crate::meta_util::find_matching_brace(&raw_class[class_body_start..], '{')
            {
                let body = &raw_class[class_body_start..];
                let body_inner = &body[..=body_end.min(body.len().saturating_sub(1))];
                for (_method_name, anno_kind, arg) in collect_method_annotations(body_inner) {
                    if matches!(
                        anno_kind,
                        AnnotationKind::GetMapping
                            | AnnotationKind::PostMapping
                            | AnnotationKind::PutMapping
                            | AnnotationKind::DeleteMapping
                            | AnnotationKind::PatchMapping
                    ) {
                        let method = annotation_kind_to_http_method(anno_kind);
                        let paths = parse_mapping_paths(&arg);
                        for path in paths {
                            request_mappings.push(RequestMappingMapping {
                                method: Some(method.clone()),
                                path,
                            });
                        }
                    }
                }
            }
        }
    }

    // Controller -> EndpointMapsTo -> handler
    if is_controller && fidelity != Fidelity::Low {
        let controller = EntityRef::new("spring", "Controller", &class_name);
        for mapping in &request_mappings {
            let endpoint_str = if let Some(ref method) = mapping.method {
                format!("{} {}", method, mapping.path)
            } else {
                mapping.path.clone()
            };
            edges.push(SemanticEdge {
                relation: SemanticRelation::EndpointMapsTo,
                subject: controller.clone(),
                object: EntityRef::new("spring", "Endpoint", &endpoint_str),
                layer: "spring",
            });
        }
    }

    // Controller -> Autowired -> Service
    if is_controller && fidelity == Fidelity::High {
        if let Some(class_body_start) = find_class_body_open(raw_class) {
            if let Some(body_end) =
                crate::meta_util::find_matching_brace(&raw_class[class_body_start..], '{')
            {
                let body = &raw_class[class_body_start..];
                let body_inner = &body[..=body_end.min(body.len().saturating_sub(1))];
                let controller = EntityRef::new("spring", "Controller", &class_name);
                for (field_name, _) in collect_field_annotations(body_inner) {
                    edges.push(SemanticEdge {
                        relation: SemanticRelation::Autowired,
                        subject: controller.clone(),
                        object: EntityRef::new("spring", "Service", &field_name),
                        layer: "spring",
                    });
                }
            }
        }
    }

    // Configuration -> BeanProduces
    if is_configuration {
        let config = EntityRef::new("spring", "Configuration", &class_name);
        if let Some(class_body_start) = find_class_body_open(raw_class) {
            if let Some(body_end) =
                crate::meta_util::find_matching_brace(&raw_class[class_body_start..], '{')
            {
                let body = &raw_class[class_body_start..];
                let body_inner = &body[..=body_end.min(body.len().saturating_sub(1))];
                for (method_name, anno_kind, _arg) in collect_method_annotations(body_inner) {
                    if matches!(anno_kind, AnnotationKind::Bean) {
                        edges.push(SemanticEdge {
                            relation: SemanticRelation::BeanProduces,
                            subject: config.clone(),
                            object: EntityRef::new("spring", "Bean", &method_name),
                            layer: "spring",
                        });
                    }
                }
            }
        }
        for method in &bean_methods {
            edges.push(SemanticEdge {
                relation: SemanticRelation::BeanProduces,
                subject: config.clone(),
                object: EntityRef::new("spring", "Bean", method),
                layer: "spring",
            });
        }
    }

    // ConfigurationProperties
    if has_config_props {
        edges.push(SemanticEdge {
            relation: SemanticRelation::ConfigurationProperties,
            subject: EntityRef::new("spring", "Configuration", &class_name),
            object: EntityRef::new("spring", "Properties", &class_name),
            layer: "spring",
        });
    }

    edges
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(all(test, feature = "spring_boot"))]
#[path = "../tests/spring_meta/semantic.rs"]
mod tests;
