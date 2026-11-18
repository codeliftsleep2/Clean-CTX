//! Candidate refinement for exact direct-relative imported Action references.

use crate::ir::invocation_evidence::SourceSpan;
use crate::layers::meta::semantic::{SemanticEdge, SemanticRelation};
use std::collections::BTreeSet;

#[derive(Default)]
struct ActionBindings {
    resolved: Vec<(SemanticRelation, String, String, String)>,
    imported_names: BTreeSet<String>,
    namespaces: BTreeSet<String>,
}

pub(crate) fn refine_candidates(
    source_id: &str,
    source: &str,
    edges: Vec<SemanticEdge>,
) -> Vec<SemanticEdge> {
    let bindings = action_bindings(source_id, source);
    edges
        .into_iter()
        .filter_map(|mut edge| {
            let action = match edge.relation {
                SemanticRelation::Dispatches | SemanticRelation::HandlesAction => &mut edge.object,
                SemanticRelation::TriggersReducer => &mut edge.subject,
                _ => return Some(edge),
            };
            if let Some((_, _, imported_name, module)) =
                bindings
                    .resolved
                    .iter()
                    .find(|(relation, written_name, _, _)| {
                        *relation == edge.relation && *written_name == action.name
                    })
            {
                action.name.clone_from(imported_name);
                action.file = Some(module.clone());
                edge.layer = super::direct_imports::REFINEMENT_LAYER;
            } else if bindings.imported_names.contains(&action.name)
                || action
                    .name
                    .split_once('.')
                    .is_some_and(|(namespace, _)| bindings.namespaces.contains(namespace))
            {
                return None;
            }
            Some(edge)
        })
        .collect()
}

fn action_bindings(source_id: &str, source: &str) -> ActionBindings {
    let Some(language) = crate::compression::language::safe_typescript_language() else {
        return ActionBindings::default();
    };
    let mut parser = tree_sitter::Parser::new();
    if parser.set_language(&language).is_err() {
        return ActionBindings::default();
    }
    let Some(tree) = parser
        .parse(source, None)
        .filter(|tree| !tree.root_node().has_error())
    else {
        return ActionBindings::default();
    };
    let mut occurrences = Vec::new();
    collect_action_occurrences(tree.root_node(), source.as_bytes(), &mut occurrences);
    let mut result = ActionBindings::default();
    collect_imported_names(
        tree.root_node(),
        source.as_bytes(),
        &mut result.imported_names,
        &mut result.namespaces,
    );
    for (relation, node) in occurrences {
        let Ok(span) = SourceSpan::new(node.start_byte(), node.end_byte()) else {
            continue;
        };
        if let Some(binding) =
            crate::ir::typescript_value_import_binding::bind_direct_named_value_import(
                source_id, source, span,
            )
        {
            if binding.source_id != source_id
                || binding.source_hash != crate::cache::content_hash(source.as_bytes())
                || binding.reference_span != span
            {
                continue;
            }
            let item = (
                relation,
                binding.imported_name.clone(),
                binding.imported_name,
                binding.module_specifier,
            );
            if !result.resolved.contains(&item) {
                result.resolved.push(item);
            }
            continue;
        }
        let Some(binding) =
            crate::ir::typescript_value_import_binding::bind_direct_namespace_value_import(
                source_id, source, span,
            )
        else {
            continue;
        };
        if binding.source_id != source_id
            || binding.source_hash != crate::cache::content_hash(source.as_bytes())
            || binding.reference_span != span
        {
            continue;
        }
        let item = (
            relation,
            format!("{}.{}", binding.namespace_name, binding.imported_name),
            binding.imported_name,
            binding.module_specifier,
        );
        if !result.resolved.contains(&item) {
            result.resolved.push(item);
        }
    }
    result
}

fn collect_imported_names(
    node: tree_sitter::Node<'_>,
    source: &[u8],
    names: &mut BTreeSet<String>,
    namespaces: &mut BTreeSet<String>,
) {
    if node.kind() == "import_specifier" {
        let local = node
            .child_by_field_name("alias")
            .or_else(|| node.child_by_field_name("name"));
        if let Some(name) = local.and_then(|name| name.utf8_text(source).ok()) {
            names.insert(name.to_owned());
        }
        return;
    }
    if node.kind() == "namespace_import" {
        if let Some(name) = node
            .named_child(0)
            .and_then(|name| name.utf8_text(source).ok())
        {
            namespaces.insert(name.to_owned());
        }
        return;
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_imported_names(child, source, names, namespaces);
    }
}

fn collect_action_occurrences<'tree>(
    node: tree_sitter::Node<'tree>,
    source: &[u8],
    output: &mut Vec<(SemanticRelation, tree_sitter::Node<'tree>)>,
) {
    if matches!(node.kind(), "identifier" | "member_expression") {
        if let Some(relation) = relation_for(node, source) {
            output.push((relation, node));
        }
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_action_occurrences(child, source, output);
    }
}

fn relation_for(node: tree_sitter::Node<'_>, source: &[u8]) -> Option<SemanticRelation> {
    let parent = node.parent()?;
    if parent.kind() == "arguments" {
        let call = parent
            .parent()
            .filter(|parent| parent.kind() == "call_expression")?;
        let function = call.child_by_field_name("function")?;
        let written = function.utf8_text(source).ok()?;
        return match written {
            "ofType" => Some(SemanticRelation::HandlesAction),
            "on" if parent
                .named_child_count()
                .checked_sub(1)
                .and_then(|index| u32::try_from(index).ok())
                .and_then(|index| parent.named_child(index))
                != Some(node) =>
            {
                Some(SemanticRelation::TriggersReducer)
            }
            _ => None,
        };
    }
    if parent.kind() == "call_expression" && parent.child_by_field_name("function") == Some(node) {
        let arguments = parent
            .parent()
            .filter(|parent| parent.kind() == "arguments")?;
        if arguments.named_child(0)? != parent {
            return None;
        }
        let outer = arguments
            .parent()
            .filter(|parent| parent.kind() == "call_expression")?;
        let function = outer.child_by_field_name("function")?;
        if function.kind() == "member_expression"
            && function
                .child_by_field_name("property")
                .and_then(|property| property.utf8_text(source).ok())
                == Some("dispatch")
        {
            return Some(SemanticRelation::Dispatches);
        }
    }
    None
}
