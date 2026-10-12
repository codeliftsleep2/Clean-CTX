//! Service ownership of already-qualified constructor dispatches, compilation-local.
use super::constructors::ConstructorDispatch;
use crate::angular_meta::inject_evidence::ClassInitializerEvidence;
use crate::compression::capture_pipeline::CapturedNode;
use crate::ir::invocation_evidence::{DeclarationContextKind, InvocationOccurrence, SourceSpan};
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};
use std::collections::BTreeMap;

pub(crate) const QUERY: &str = r#"
(class_declaration body: (class_body (method_definition
    body: (statement_block (expression_statement (call_expression) @ngrx.service.call)))
    @ngrx.service.method)) @ngrx.service.owner
"#;

#[derive(Debug, Clone)]
pub(crate) struct ServiceDispatch {
    pub start_byte: usize,
    pub service: String,
    pub action: String,
    pub file: String,
    pub source_hash: String,
    #[allow(dead_code)]
    pub class_span: SourceSpan,
    #[allow(dead_code)]
    pub method_span: SourceSpan,
}
fn span(node: &CapturedNode) -> SourceSpan {
    SourceSpan::new(node.start_byte, node.end_byte).expect("ordered parser span")
}

/// Same-match class -> own method -> immediate statement -> invocation, joined
/// to both frozen endpoint evidence and independent canonical Service authority.
pub(crate) fn collect(
    source: &str,
    file: Option<&str>,
    captures: &[CapturedNode],
    invocations: &[InvocationOccurrence],
    classes: &[ClassInitializerEvidence],
    constructors: &[ConstructorDispatch],
) -> Vec<ServiceDispatch> {
    let Some(file) = file else {
        return Vec::new();
    };
    let hash = crate::cache::content_hash(source.as_bytes());
    let mut matches: BTreeMap<usize, BTreeMap<&str, &CapturedNode>> = BTreeMap::new();
    for c in captures
        .iter()
        .filter(|c| c.name.starts_with("ngrx.service."))
    {
        matches.entry(c.match_index).or_default().insert(&c.name, c);
    }
    matches
        .values()
        .filter_map(|m| {
            let call = m.get("ngrx.service.call")?;
            let method = m.get("ngrx.service.method")?;
            let owner = m.get("ngrx.service.owner")?;
            let endpoint = constructors.iter().find(|r| {
                r.start_byte == call.start_byte
                    && r.file.as_deref() == Some(file)
                    && r.source_hash == hash
            })?;
            let action = endpoint.action_name.as_ref()?;
            let canonical = endpoint.owner_name.as_ref()?;
            let class = classes
                .iter()
                .find(|c| c.class_span == span(owner) && &c.consumer == canonical)?;
            if crate::angular_meta::decorators::extract_graph_entries(&class.raw_class)?.1
                != crate::angular_meta::decorators::ClassKind::Service
            {
                return None;
            }
            let invocation = invocations.iter().find(|c| c.span() == span(call))?;
            let context = invocation.declaration_context()?;
            if context.kind() != DeclarationContextKind::Callable
                || context.declaration() != span(method)
            {
                return None;
            }
            Some(ServiceDispatch {
                start_byte: call.start_byte,
                service: class.consumer.clone(),
                action: action.clone(),
                file: file.into(),
                source_hash: hash.clone(),
                class_span: class.class_span,
                method_span: span(method),
            })
        })
        .collect()
}

/// Project precise Service occurrences, leaving all other existing occurrences
/// to the frozen Component/fallback producer. Never rewrite aggregate edges by
/// target spelling: one Action can also have an honest non-Service dispatch.
pub(crate) fn project(
    raw: super::NgRxShape,
    starts: &[usize],
    constructors: &[ConstructorDispatch],
    owners: &[ServiceDispatch],
    source: &str,
    file: &str,
) -> Vec<SemanticEdge> {
    let hash = crate::cache::content_hash(source.as_bytes());
    let admitted: Vec<_> = owners
        .iter()
        .filter(|o| {
            o.source_hash == hash
                && o.file == file
                && constructors.iter().any(|r| {
                    r.start_byte == o.start_byte
                        && r.source_hash == hash
                        && r.file.as_deref() == Some(file)
                        && r.owner_name.as_deref() == Some(&o.service)
                        && r.action_name.as_deref() == Some(&o.action)
                })
        })
        .collect();
    let mut remaining = constructors.to_vec();
    for r in &mut remaining {
        if admitted.iter().any(|o| o.start_byte == r.start_byte) {
            // Keep the suppression record; this occurrence is projected below.
            r.action_name = None;
        }
    }
    let fallback = super::constructors::apply(raw, starts, &remaining, source, file);
    let mut edges = fallback.to_ngrx_semantic_edges();
    edges.extend(admitted.into_iter().map(|o| SemanticEdge {
        subject: EntityRef::new("angular", "Service", &o.service),
        relation: SemanticRelation::Dispatches,
        object: EntityRef::new("ngrx", "Action", &o.action),
        layer: "ngrx",
        call_evidence: None,
    }));
    edges
}
