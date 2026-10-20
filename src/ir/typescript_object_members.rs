//! Neutral TypeScript identity for members of exported object constants.
//!
//! This module proves declarations only. Framework layers may consume the
//! resulting `(file, object.member)` identity, but object shape and literal
//! values never acquire framework meaning here.

use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};

pub(crate) const DOMAIN: &str = "typescript";
pub(crate) const ENTITY_TYPE: &str = "ObjectMember";
pub(crate) const LAYER: &str = "typescript-object-member";

pub(crate) fn project_exported_object_members(source: &str, file: &str) -> Vec<SemanticEdge> {
    let Some(language) = crate::compression::language::safe_typescript_language() else {
        return Vec::new();
    };
    let mut parser = tree_sitter::Parser::new();
    if parser.set_language(&language).is_err() {
        return Vec::new();
    }
    let Some(tree) = parser
        .parse(source, None)
        .filter(|tree| !tree.root_node().has_error())
    else {
        return Vec::new();
    };
    let mut members = Vec::new();
    collect(tree.root_node(), source.as_bytes(), &mut members);
    members.sort();
    members.dedup();
    members
        .into_iter()
        .map(|name| {
            let entity = EntityRef::new(DOMAIN, ENTITY_TYPE, name).with_file(file.to_owned());
            SemanticEdge {
                relation: SemanticRelation::Defines,
                subject: entity.clone(),
                object: entity,
                layer: LAYER,
                call_evidence: None,
            }
        })
        .collect()
}

fn collect(node: tree_sitter::Node<'_>, source: &[u8], output: &mut Vec<String>) {
    if node.kind() == "export_statement"
        && let Some(declaration) = node.named_child(0)
        && declaration.kind() == "lexical_declaration"
    {
        let mut cursor = declaration.walk();
        for declarator in declaration
            .named_children(&mut cursor)
            .filter(|child| child.kind() == "variable_declarator")
        {
            let Some(owner) = declarator
                .child_by_field_name("name")
                .filter(|name| name.kind() == "identifier")
                .and_then(|name| name.utf8_text(source).ok())
            else {
                continue;
            };
            let Some(object) = declarator
                .child_by_field_name("value")
                .filter(|value| value.kind() == "object")
            else {
                continue;
            };
            let mut object_cursor = object.walk();
            for pair in object
                .named_children(&mut object_cursor)
                .filter(|child| child.kind() == "pair")
            {
                let Some(member) = pair
                    .child_by_field_name("key")
                    .filter(|key| matches!(key.kind(), "property_identifier" | "string"))
                    .and_then(|key| key.utf8_text(source).ok())
                    .map(|key| key.trim_matches(['\'', '"']))
                    .filter(|key| is_identifier(key))
                else {
                    continue;
                };
                output.push(format!("{owner}.{member}"));
            }
        }
        return;
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect(child, source, output);
    }
}

fn is_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    chars
        .next()
        .is_some_and(|ch| ch.is_alphabetic() || ch == '_' || ch == '$')
        && chars.all(|ch| ch.is_alphanumeric() || ch == '_' || ch == '$')
}

#[cfg(test)]
#[path = "../tests/ir/typescript_object_members.rs"]
mod tests;
