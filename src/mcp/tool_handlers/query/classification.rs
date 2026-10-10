//! Ephemeral refinement of neutral C# class base-list facts.
//!
//! Durable publication remains Class --HasBaseType--> TypeRef. This query-time
//! view adds Extends/Implements only after completed scoped hydration proves
//! exactly one target kind. Nothing derived here is written to WorkspaceIndex.

use super::{
    csharp_resolution::{Catalog, TypeKind},
    identity::ResolvedIdentity,
};
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};
use crate::workspace::index::WorkspaceIndex;
use crate::workspace::scope::WorkspaceScope;

const BUILTIN: &str = "builtin";
const CLASS: &str = "Class";
const INTERFACE: &str = "Interface";
const TYPE_REF: &str = "TypeRef";

pub(super) fn forward(
    index: &WorkspaceIndex,
    identity: &ResolvedIdentity,
    scope: Option<&WorkspaceScope>,
    classification_complete: bool,
    catalog: Option<&Catalog>,
) -> Vec<SemanticEdge> {
    let mut edges = stored_forward(index, identity, scope);
    if !classification_complete || !is_builtin_class(identity) {
        return edges;
    }
    let derived: Vec<_> = edges
        .iter()
        .filter(|edge| is_neutral_class_base(edge))
        .filter_map(|edge| classify(index, edge, scope, catalog))
        .collect();
    append_distinct(&mut edges, derived);
    edges
}

pub(super) fn reverse(
    index: &WorkspaceIndex,
    identity: &ResolvedIdentity,
    scope: Option<&WorkspaceScope>,
    classification_complete: bool,
    catalog: Option<&Catalog>,
) -> Vec<SemanticEdge> {
    let mut edges = stored_reverse(index, identity, scope);
    if !classification_complete
        || identity.domain != BUILTIN
        || !matches!(identity.entity_type.as_str(), CLASS | INTERFACE)
    {
        return edges;
    }
    let target_kind = if identity.entity_type == CLASS {
        TypeKind::Class
    } else {
        TypeKind::Interface
    };
    let mut written_names = vec![identity.name.clone()];
    if let Some(catalog) = catalog {
        written_names.extend(catalog.written_references_to(target_kind, &identity.name));
    }
    written_names.sort();
    written_names.dedup();
    let derived = written_names
        .iter()
        .flat_map(|written| reverse_by_identity(index, BUILTIN, TYPE_REF, written, scope))
        .filter(|edge| is_neutral_class_base(edge))
        .filter_map(|edge| classify(index, edge, scope, catalog))
        .filter(|edge| edge.object.entity_type == identity.entity_type)
        .collect();
    append_distinct(&mut edges, derived);
    edges
}

pub(super) fn neutral_targets_for_forward(
    index: &WorkspaceIndex,
    identity: &ResolvedIdentity,
    scope: Option<&WorkspaceScope>,
) -> Vec<String> {
    if !is_builtin_class(identity) {
        return Vec::new();
    }
    let mut names: Vec<_> = forward_by_identity(
        index,
        &identity.domain,
        &identity.entity_type,
        &identity.name,
        scope,
    )
    .into_iter()
    .filter(|edge| is_neutral_class_base(edge))
    .map(|edge| edge.object.name.clone())
    .collect();
    names.sort();
    names.dedup();
    names
}

fn classify(
    index: &WorkspaceIndex,
    neutral: &SemanticEdge,
    scope: Option<&WorkspaceScope>,
    catalog: Option<&Catalog>,
) -> Option<SemanticEdge> {
    let resolved = catalog.and_then(|catalog| catalog.resolve_edge(neutral));
    let (target_type, target_name, relation) = if let Some(target) = resolved {
        let target_type = target.kind.entity_type();
        let relation = if target.kind == TypeKind::Class {
            SemanticRelation::Extends
        } else {
            SemanticRelation::Implements
        };
        (target_type, target.name, relation)
    } else {
        if catalog.is_some_and(|catalog| catalog.recognizes_edge(neutral)) {
            return None;
        }
        let target = &neutral.object.name;
        let class_exists = index.has_identity_in_scope(BUILTIN, CLASS, target, scope);
        let interface_exists = index.has_identity_in_scope(BUILTIN, INTERFACE, target, scope);
        match (class_exists, interface_exists) {
            (true, false) => (CLASS, target.clone(), SemanticRelation::Extends),
            (false, true) => (INTERFACE, target.clone(), SemanticRelation::Implements),
            _ => return None,
        }
    };
    let mut object = EntityRef::new(BUILTIN, target_type, target_name);
    object.file = neutral.object.file.clone();
    Some(SemanticEdge {
        relation,
        subject: neutral.subject.clone(),
        object,
        layer: "builtin",
        call_evidence: None,
    })
}

fn is_builtin_class(identity: &ResolvedIdentity) -> bool {
    identity.domain == BUILTIN && identity.entity_type == CLASS
}

fn is_neutral_class_base(edge: &SemanticEdge) -> bool {
    edge.relation == SemanticRelation::HasBaseType
        && edge.subject.domain == BUILTIN
        && edge.subject.entity_type == CLASS
        && edge.object.domain == BUILTIN
        && edge.object.entity_type == TYPE_REF
}

fn append_distinct(edges: &mut Vec<SemanticEdge>, derived: Vec<SemanticEdge>) {
    for candidate in derived {
        if !edges.iter().any(|edge| same_fact(edge, &candidate)) {
            edges.push(candidate);
        }
    }
}

fn same_fact(left: &SemanticEdge, right: &SemanticEdge) -> bool {
    left.relation == right.relation
        && left.subject == right.subject
        && left.object == right.object
        && left.call_evidence == right.call_evidence
}

fn stored_forward(
    index: &WorkspaceIndex,
    identity: &ResolvedIdentity,
    scope: Option<&WorkspaceScope>,
) -> Vec<SemanticEdge> {
    forward_by_identity(
        index,
        &identity.domain,
        &identity.entity_type,
        &identity.name,
        scope,
    )
    .into_iter()
    .cloned()
    .collect()
}

fn stored_reverse(
    index: &WorkspaceIndex,
    identity: &ResolvedIdentity,
    scope: Option<&WorkspaceScope>,
) -> Vec<SemanticEdge> {
    reverse_by_identity(
        index,
        &identity.domain,
        &identity.entity_type,
        &identity.name,
        scope,
    )
    .into_iter()
    .cloned()
    .collect()
}

fn forward_by_identity<'a>(
    index: &'a WorkspaceIndex,
    domain: &str,
    entity_type: &str,
    name: &str,
    scope: Option<&WorkspaceScope>,
) -> Vec<&'a SemanticEdge> {
    match scope {
        Some(scope) => index.forward_edges_by_identity_in_scope(domain, entity_type, name, scope),
        None => index.forward_edges_by_identity(domain, entity_type, name),
    }
}

fn reverse_by_identity<'a>(
    index: &'a WorkspaceIndex,
    domain: &str,
    entity_type: &str,
    name: &str,
    scope: Option<&WorkspaceScope>,
) -> Vec<&'a SemanticEdge> {
    match scope {
        Some(scope) => index.reverse_edges_by_identity_in_scope(domain, entity_type, name, scope),
        None => index.reverse_edges_by_identity(domain, entity_type, name),
    }
}
