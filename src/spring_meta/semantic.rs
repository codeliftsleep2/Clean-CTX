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
    parse_mapping_paths, parse_request_mappings, unquote,
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
    let mut is_service = false;
    let mut service_name: Option<String> = None;
    let mut is_repository = false;
    let mut repository_name: Option<String> = None;
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
            AnnotationKind::Service => {
                is_service = true;
                let arg = anno.arg.trim();
                if !arg.is_empty() {
                    service_name = Some(unquote(arg).to_string());
                }
            }
            AnnotationKind::Repository => {
                is_repository = true;
                let arg = anno.arg.trim();
                if !arg.is_empty() {
                    repository_name = Some(unquote(arg).to_string());
                }
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

    // Provision: @Service → Binds → Token
    // The implementation identity is the class itself (authoritative from the
    // class capture). The token is the explicit @Service("name") value if
    // present, otherwise the class name (Spring's default bean name key).
    // Phase 23: dual binding for explicit names — an explicitly named
    // component remains injectable by both its explicit name and its class name.
    if is_service {
        if let Some(name) = &service_name {
            // Explicit name: bind to both the class token and the name token.
            if *name != class_name {
                edges.push(SemanticEdge {
                    relation: SemanticRelation::Binds,
                    subject: EntityRef::new("spring", "Service", &class_name),
                    object: EntityRef::new("spring", "Token", &class_name),
                    layer: "spring",
                });
            }
            edges.push(SemanticEdge {
                relation: SemanticRelation::Binds,
                subject: EntityRef::new("spring", "Service", &class_name),
                object: EntityRef::new("spring", "Token", name),
                layer: "spring",
            });
        } else {
            // Default name: bind to class token only.
            edges.push(SemanticEdge {
                relation: SemanticRelation::Binds,
                subject: EntityRef::new("spring", "Service", &class_name),
                object: EntityRef::new("spring", "Token", &class_name),
                layer: "spring",
            });
        }
    }

    // Provision: @Repository → Binds → Token
    // NOTE: spring/Repository is a new semantic role introduced by this phase.
    // It is the data-access analogue of spring/Service. The token follows the
    // same convention: explicit @Repository("name") value or class name.
    // Phase 23: dual binding for explicit names.
    if is_repository {
        if let Some(name) = &repository_name {
            // Explicit name: bind to both the class token and the name token.
            if *name != class_name {
                edges.push(SemanticEdge {
                    relation: SemanticRelation::Binds,
                    subject: EntityRef::new("spring", "Repository", &class_name),
                    object: EntityRef::new("spring", "Token", &class_name),
                    layer: "spring",
                });
            }
            edges.push(SemanticEdge {
                relation: SemanticRelation::Binds,
                subject: EntityRef::new("spring", "Repository", &class_name),
                object: EntityRef::new("spring", "Token", name),
                layer: "spring",
            });
        } else {
            // Default name: bind to class token only.
            edges.push(SemanticEdge {
                relation: SemanticRelation::Binds,
                subject: EntityRef::new("spring", "Repository", &class_name),
                object: EntityRef::new("spring", "Token", &class_name),
                layer: "spring",
            });
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
                for (_method_name, anno_kind, arg, _return_type) in
                    collect_method_annotations(body_inner)
                {
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

    // Spring-role -> Autowired -> Token
    // Phase 28: general DI consumption projection. The historical
    // controller-only gate predates the role-agnostic Phase 19 field parser;
    // every existing authoritative Spring role identity now projects the
    // same source-level declared-dependency fact. A class with multiple
    // roles emits one edge per role identity (parallel-identity model:
    // distinct subjects are distinct edges, not duplicates).
    if fidelity == Fidelity::High {
        let mut subjects: Vec<EntityRef> = Vec::new();
        if is_controller {
            subjects.push(EntityRef::new("spring", "Controller", &class_name));
        }
        if is_service {
            subjects.push(EntityRef::new("spring", "Service", &class_name));
        }
        if is_repository {
            subjects.push(EntityRef::new("spring", "Repository", &class_name));
        }
        if is_configuration {
            subjects.push(EntityRef::new("spring", "Configuration", &class_name));
        }
        if !subjects.is_empty() {
            if let Some(class_body_start) = find_class_body_open(raw_class) {
                if let Some(body_end) =
                    crate::meta_util::find_matching_brace(&raw_class[class_body_start..], '{')
                {
                    let body = &raw_class[class_body_start..];
                    let body_inner = &body[..=body_end.min(body.len().saturating_sub(1))];
                    for fa in collect_field_annotations(body_inner) {
                        // Phase 19: the dependency identity is the DECLARED
                        // FIELD TYPE (exact source spelling). Occurrences whose
                        // declaration cannot be parsed confidently are dropped
                        // by the extractor (fail closed) and emit no edge —
                        // no `?`/modifier-placeholder identities.
                        if fa.kind != AnnotationKind::Autowired {
                            continue;
                        }
                        // Phase 23: qualifier overrides the type-based token.
                        let token = fa.qualifier.as_ref().unwrap_or(&fa.declared_type);
                        for subject in &subjects {
                            edges.push(SemanticEdge {
                                relation: SemanticRelation::Autowired,
                                subject: subject.clone(),
                                object: EntityRef::new("spring", "Token", token),
                                layer: "spring",
                            });
                        }
                    }
                }
            }
        }
    }

    // Configuration -> BeanProduces (and Binds for @Bean methods)
    if is_configuration {
        let config = EntityRef::new("spring", "Configuration", &class_name);
        if let Some(class_body_start) = find_class_body_open(raw_class) {
            if let Some(body_end) =
                crate::meta_util::find_matching_brace(&raw_class[class_body_start..], '{')
            {
                let body = &raw_class[class_body_start..];
                let body_inner = &body[..=body_end.min(body.len().saturating_sub(1))];
                for (method_name, anno_kind, arg, return_type) in
                    collect_method_annotations(body_inner)
                {
                    if matches!(anno_kind, AnnotationKind::Bean) {
                        // BeanProduces: factory declaration fact (preserved)
                        edges.push(SemanticEdge {
                            relation: SemanticRelation::BeanProduces,
                            subject: config.clone(),
                            object: EntityRef::new("spring", "Bean", &method_name),
                            layer: "spring",
                        });
                        // Phase 25: Binds projection when return type is authoritative.
                        // Dual-binding rule: a @Bean method provides both its return
                        // type (for plain @Autowired by type) and its bean name (for
                        // @Autowired @Qualifier by name).
                        // Generic/array return types fail closed: they cannot be
                        // reduced to a safe provider identity in this phase.
                        if let Some(ret) = &return_type {
                            if !ret.contains(['<', '>', '[', ']']) {
                                let provider = EntityRef::new("spring", "Service", ret);
                                // Token for type-based lookup (plain @Autowired)
                                let type_token = EntityRef::new("spring", "Token", ret);
                                if type_token != EntityRef::new("spring", "Token", &method_name) {
                                    edges.push(SemanticEdge {
                                        relation: SemanticRelation::Binds,
                                        subject: provider.clone(),
                                        object: type_token,
                                        layer: "spring",
                                    });
                                }
                                // Token for name-based lookup (@Qualifier).
                                // Phase 25: only accept a simple quoted-string bean
                                // name. Array forms (@Bean({"a","b"})) and other
                                // non-quoted args fail closed (no name-token).
                                let name_token = if !arg.trim().is_empty() {
                                    let trimmed = arg.trim();
                                    match trimmed
                                        .strip_prefix('"')
                                        .and_then(|s| s.strip_suffix('"'))
                                    {
                                        Some(stripped)
                                            if !stripped.is_empty()
                                                && !stripped.contains('{')
                                                && !stripped.contains(',') =>
                                        {
                                            Some(EntityRef::new("spring", "Token", stripped))
                                        }
                                        _ => None,
                                    }
                                } else {
                                    Some(EntityRef::new("spring", "Token", &method_name))
                                };
                                if let Some(name_token) = name_token {
                                    if name_token != EntityRef::new("spring", "Token", ret) {
                                        edges.push(SemanticEdge {
                                            relation: SemanticRelation::Binds,
                                            subject: provider,
                                            object: name_token,
                                            layer: "spring",
                                        });
                                    }
                                }
                            }
                        }
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
