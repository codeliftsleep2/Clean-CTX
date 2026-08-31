// src/angular_meta/semantic.rs
//
// Angular-specific semantic edge construction helpers.
//
// These adapt the existing Angular meta-layer structured data (extracted by
// `extract_decorators`, `extract_graph_entries`, `NgRxShape`, `RouteShape`,
// etc.) into `SemanticEdge` objects.
//
// Phase 1 contract: this module reuses existing extraction helpers and does
// NOT duplicate parsing logic. It is a projection adapter, not a new parser.

use crate::angular_meta::graph::ClassKind;
use crate::compression::Fidelity;
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};

/// Build semantic edges from the structured class metadata returned by
/// `extract_graph_entries` plus the raw class text for input/output fields
/// and NgModule declarations.
///
/// Called once per class capture in `extract_semantic_edges()`.
pub fn class_to_semantic_edges(
    class_name: &str,
    kind: ClassKind,
    selector: Option<&str>,
    injects: &[String],
    raw_class: &str,
    fidelity: Fidelity,
) -> Vec<SemanticEdge> {
    let mut edges: Vec<SemanticEdge> = Vec::new();

    let entity_type = match kind {
        ClassKind::Component => "Component",
        ClassKind::Service => "Service",
        ClassKind::Directive => "Directive",
        ClassKind::Pipe => "Pipe",
        ClassKind::Module => "Module",
    };

    let subject = EntityRef::new("angular", entity_type, class_name);

    // Component → HasSelector → selector-string
    if kind == ClassKind::Component {
        if let Some(sel) = selector {
            edges.push(SemanticEdge {
                relation: SemanticRelation::HasSelector,
                subject: subject.clone(),
                object: EntityRef::new("angular", "Component", format!("[{}]", sel)),
                layer: "angular",
            });
        }
    }

    // Component → Injects → Service for each injected type
    if kind == ClassKind::Component || kind == ClassKind::Service || kind == ClassKind::Directive {
        for injected in injects {
            edges.push(SemanticEdge {
                relation: SemanticRelation::Injects,
                subject: subject.clone(),
                object: EntityRef::new("angular", "Service", injected),
                layer: "angular",
            });
        }
    }

    // Module → DeclaresInModule → Component/Directive/Pipe for each declaration
    if kind == ClassKind::Module {
        let (declarations, imports, _exports) =
            crate::angular_meta::decorators::extract_module_declarations(raw_class);
        for decl in &declarations {
            edges.push(SemanticEdge {
                relation: SemanticRelation::DeclaresInModule,
                subject: subject.clone(),
                object: EntityRef::new("angular", "Component", decl),
                layer: "angular",
            });
        }
        // Module → ImportsModule → Module for each import
        for imp in &imports {
            edges.push(SemanticEdge {
                relation: SemanticRelation::ImportsModule,
                subject: subject.clone(),
                object: EntityRef::new("angular", "Module", imp),
                layer: "angular",
            });
        }
    }

    // Component → HasInput → InputField
    // Component → HasOutput → OutputField
    if kind == ClassKind::Component {
        let io_fields = crate::angular_meta::decorators::extract_io_fields(raw_class, fidelity);
        for (is_input, field_name) in &io_fields {
            let relation = if *is_input {
                SemanticRelation::HasInput
            } else {
                SemanticRelation::HasOutput
            };
            let obj_type = if *is_input {
                "InputField"
            } else {
                "OutputField"
            };
            edges.push(SemanticEdge {
                relation,
                subject: subject.clone(),
                object: EntityRef::new("angular", obj_type, field_name),
                layer: "angular",
            });
        }
    }

    edges
}

/// Build semantic edges from a RouteShape.
pub fn routes_to_semantic_edges(
    routes: &[crate::angular_meta::routing::RouteDecl],
) -> Vec<SemanticEdge> {
    let mut edges = Vec::new();
    for route in routes {
        let route_entity = EntityRef::new("angular", "Route", &route.path);

        // Route → RouteMapsTo → Component
        if let Some(ref comp) = route.component {
            edges.push(SemanticEdge {
                relation: SemanticRelation::RouteMapsTo,
                subject: route_entity.clone(),
                object: EntityRef::new("angular", "Component", comp),
                layer: "angular",
            });
        }

        // Route → GuardedBy → Guard
        for guard in &route.guards {
            edges.push(SemanticEdge {
                relation: SemanticRelation::GuardedBy,
                subject: route_entity.clone(),
                object: EntityRef::new("angular", "Guard", guard),
                layer: "angular",
            });
        }

        // Route → ResolvedBy → Resolver
        for resolver in &route.resolvers {
            edges.push(SemanticEdge {
                relation: SemanticRelation::ResolvedBy,
                subject: route_entity.clone(),
                object: EntityRef::new("angular", "Resolver", resolver),
                layer: "angular",
            });
        }
    }
    edges
}
