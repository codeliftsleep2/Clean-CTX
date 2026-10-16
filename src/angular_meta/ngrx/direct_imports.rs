//! File-local evidence for bounded direct-relative object-dispatch imports.

use super::{DispatchSite, NgRxShape};
use crate::ir::invocation_evidence::SourceSpan;
use crate::ir::typescript_identifier_binding::bind_direct_value_identifier;
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};

pub(crate) const REFINEMENT_LAYER: &str = "ngrx-direct-import-refinement";

pub(crate) fn candidates(source_id: &str, source: &str, shape: &NgRxShape) -> Vec<SemanticEdge> {
    shape
        .dispatch_sites
        .iter()
        .filter(|site| site.object_literal)
        .filter_map(|site| candidate(source_id, source, shape, site))
        .collect()
}

fn candidate(
    source_id: &str,
    source: &str,
    shape: &NgRxShape,
    site: &DispatchSite,
) -> Option<SemanticEdge> {
    let reference_span = reference_span(source, site)?;
    if bind_direct_value_identifier(source_id, source, reference_span).is_some() {
        return None;
    }
    let module = exact_import(source, &site.action_name)?;
    let subject = shape.component_name.as_ref().map_or_else(
        || EntityRef::new("angular", "DispatchSite", "store"),
        |owner| EntityRef::new("angular", "Component", owner),
    );
    let mut object = EntityRef::new("ngrx", "Action", &site.action_name);
    // The WorkspaceIndex intercepts this carrier before query visibility.
    object.file = Some(module);
    Some(SemanticEdge {
        relation: SemanticRelation::Dispatches,
        subject,
        object,
        layer: REFINEMENT_LAYER,
        call_evidence: None,
    })
}

fn reference_span(source: &str, site: &DispatchSite) -> Option<SourceSpan> {
    let tail = source.get(site.call_start..)?;
    let type_at = tail.find("type")?;
    let after_type = tail.get(type_at + 4..)?;
    let colon = after_type.find(':')?;
    let value = after_type.get(colon + 1..)?;
    let leading = value.len() - value.trim_start().len();
    let start = site.call_start + type_at + 4 + colon + 1 + leading;
    let end = start.checked_add(site.action_name.len())?;
    if source.get(start..end) != Some(site.action_name.as_str()) {
        return None;
    }
    SourceSpan::new(start, end).ok()
}

fn exact_import(source: &str, name: &str) -> Option<String> {
    let mut matches = Vec::new();
    for statement in source.split(';') {
        let text = statement.trim();
        if !text.starts_with("import ") || text.starts_with("import type ") {
            continue;
        }
        let Some((clause, module)) = text[7..].split_once(" from ") else {
            continue;
        };
        let clause = clause.trim();
        if !clause.starts_with('{') || !clause.ends_with('}') {
            continue;
        }
        let imports = &clause[1..clause.len() - 1];
        if !imports.split(',').any(|item| item.trim() == name) {
            continue;
        }
        let module = module.trim().trim_matches(['\'', '"']);
        if module.starts_with("./") || module.starts_with("../") {
            matches.push(module.to_owned());
        }
    }
    (matches.len() == 1).then(|| matches.remove(0))
}
