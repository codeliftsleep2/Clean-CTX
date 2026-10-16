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

use crate::angular_meta::decorators::ClassKind;
use crate::compression::Fidelity;
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};

/// Build semantic edges from the structured class metadata returned by
/// `extract_graph_entries` plus the raw class text for input/output fields
/// and NgModule declarations.
///
/// Called once per class capture in `extract_semantic_edges()`.
///
/// `pipe_name` is the Angular pipe name (e.g. "uppercase"), used for
/// Pipe entities. NgModule relationships are projected separately from
/// declaration-owned metadata authority.
pub fn class_to_semantic_edges(
    class_name: &str,
    kind: ClassKind,
    selector: Option<&str>,
    injects: &[String],
    pipe_name: Option<&str>,
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

    // The producer has recognized this declaration independently of DI or
    // other relationships. Reuse the index's existing registration carrier;
    // it is normalized into an entity occurrence, never a graph self-edge.
    if kind == ClassKind::Service {
        edges.push(SemanticEdge {
            relation: SemanticRelation::Defines,
            subject: subject.clone(),
            object: subject.clone(),
            layer: "angular",
            call_evidence: None,
        });
    }

    // Pipe → HasPipeName → pipe-name-string
    if kind == ClassKind::Pipe {
        if let Some(pn) = pipe_name {
            edges.push(SemanticEdge {
                relation: SemanticRelation::Defines,
                subject: subject.clone(),
                object: EntityRef::new("angular", "PipeName", pn),
                layer: "angular",
                call_evidence: None,
            });
        }
    }

    // Component → HasSelector → selector-string
    if kind == ClassKind::Component {
        if let Some(sel) = selector {
            edges.push(SemanticEdge {
                relation: SemanticRelation::HasSelector,
                subject: subject.clone(),
                object: EntityRef::new("angular", "Component", sel),
                layer: "angular",
                call_evidence: None,
            });
        }
    }

    // Pipe admission remains constructor-only, as frozen in #138a. Qualified
    // functional tokens are supplied separately by the Angular projection;
    // this guard does not grant functional support to Pipe. Reuse the existing
    // constructor producer, never recover a token from a receiver or marker.
    let pipe_constructor_types = (kind == ClassKind::Pipe).then(|| {
        crate::angular_meta::constructor_injects::extract_constructor_injects(raw_class)
            .unwrap_or_default()
    });
    let admitted = pipe_constructor_types.as_deref().unwrap_or(injects);
    if matches!(
        kind,
        ClassKind::Component | ClassKind::Service | ClassKind::Directive | ClassKind::Pipe
    ) {
        for injected in admitted {
            edges.push(SemanticEdge {
                relation: SemanticRelation::Injects,
                subject: subject.clone(),
                object: EntityRef::new("angular", "Service", injected),
                layer: "angular",
                call_evidence: None,
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
                call_evidence: None,
            });
        }
    }

    edges
}

/// Build the template relationship owned by an Angular Component declaration.
///
/// External templates use their canonical physical identity even when the
/// target is currently missing. Inline templates use a deterministic identity
/// scoped to the declaring source and Component.
pub(crate) fn component_template_edge(
    class_name: &str,
    kind: ClassKind,
    raw_class: &str,
    declaring_file: &std::path::Path,
) -> Option<SemanticEdge> {
    if kind != ClassKind::Component || declaring_file.as_os_str().is_empty() {
        return None;
    }

    let owner = crate::dictionary::path::canonical_identity_key(&declaring_file.to_string_lossy());
    let template_name = if let Some(written) =
        crate::angular_meta::decorators::extract_component_template_url(raw_class)
    {
        let path = crate::angular_meta::template_ownership::resolve_template_path(
            declaring_file,
            &written,
        )?;
        crate::dictionary::path::canonical_identity_key(&path.to_string_lossy())
    } else if crate::angular_meta::decorators::extract_decorators(raw_class, Fidelity::Low)
        .and_then(|result| result.inline_template)
        .is_some()
    {
        format!("inline:{owner}#component={class_name}")
    } else {
        return None;
    };

    Some(SemanticEdge {
        relation: SemanticRelation::HasTemplate,
        subject: EntityRef::new("angular", "Component", class_name),
        object: EntityRef::new("angular", "Template", template_name),
        layer: "angular",
        call_evidence: None,
    })
}

/// Build semantic edges from a RouteShape.
pub fn routes_to_semantic_edges(
    routes: &[crate::angular_meta::routing::RouteDecl],
) -> Vec<SemanticEdge> {
    crate::angular_meta::routing::RouteShape {
        routes: routes.to_vec(),
        ..Default::default()
    }
    .to_semantic_edges()
}
