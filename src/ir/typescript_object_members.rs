//! Neutral TypeScript identity for members of exported object constants.
//!
//! This module proves declarations only. Framework layers may consume the
//! resulting `(file, object.member)` identity, but object shape and literal
//! values never acquire framework meaning here.

use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};

pub(crate) const DOMAIN: &str = "typescript";
pub(crate) const ENTITY_TYPE: &str = "ObjectMember";
pub(crate) const VALUE_ENTITY_TYPE: &str = "ValueDeclaration";
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
    let mut projection = Projection::default();
    collect_top_level_string_values(tree.root_node(), source.as_bytes(), file, &mut projection);
    collect(tree.root_node(), source, file, &mut projection);
    projection.members.sort();
    projection.members.dedup();
    projection.values.sort();
    projection.values.dedup();
    projection.references.sort();
    projection.references.dedup();
    let mut edges: Vec<_> = projection
        .members
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
        .collect();
    edges.extend(projection.values.into_iter().map(|(name, target)| {
        let entity = EntityRef::new(DOMAIN, VALUE_ENTITY_TYPE, name).with_file(target);
        SemanticEdge {
            relation: SemanticRelation::Defines,
            subject: entity.clone(),
            object: entity,
            layer: LAYER,
            call_evidence: None,
        }
    }));
    edges.extend(
        projection
            .references
            .into_iter()
            .map(|(member, value, target)| SemanticEdge {
                relation: SemanticRelation::ReferencesValue,
                subject: EntityRef::new(DOMAIN, ENTITY_TYPE, member).with_file(file.to_owned()),
                object: EntityRef::new(DOMAIN, VALUE_ENTITY_TYPE, value).with_file(target),
                layer: LAYER,
                call_evidence: None,
            }),
    );
    edges
}

pub(crate) fn referenced_value_names(source: &str) -> Vec<String> {
    let Some(language) = crate::compression::language::safe_typescript_language() else {
        return Vec::new();
    };
    let mut parser = tree_sitter::Parser::new();
    if parser.set_language(&language).is_err() {
        return Vec::new();
    }
    let Some(tree) = parser.parse(source, None) else {
        return Vec::new();
    };
    let mut projection = Projection::default();
    collect(tree.root_node(), source, "<discovery>", &mut projection);
    let mut names: Vec<_> = projection
        .references
        .into_iter()
        .map(|(_, name, _)| name)
        .collect();
    names.sort();
    names.dedup();
    names
}

#[derive(Default)]
struct Projection {
    members: Vec<String>,
    values: Vec<(String, String)>,
    references: Vec<(String, String, String)>,
}

fn collect_top_level_string_values(
    root: tree_sitter::Node<'_>,
    source: &[u8],
    file: &str,
    output: &mut Projection,
) {
    let mut cursor = root.walk();
    for child in root.named_children(&mut cursor) {
        let declaration = if child.kind() == "export_statement" {
            child.named_child(0)
        } else {
            Some(child)
        };
        let Some(declaration) = declaration.filter(|node| node.kind() == "lexical_declaration")
        else {
            continue;
        };
        let mut declarations = declaration.walk();
        for declarator in declaration
            .named_children(&mut declarations)
            .filter(|node| node.kind() == "variable_declarator")
        {
            let (Some(name), Some(value)) = (
                declarator.child_by_field_name("name"),
                declarator.child_by_field_name("value"),
            ) else {
                continue;
            };
            if value.kind() == "string"
                && let Ok(name) = name.utf8_text(source)
            {
                output.values.push((name.to_owned(), file.to_owned()));
            }
        }
    }
}

fn collect(node: tree_sitter::Node<'_>, source: &str, file: &str, output: &mut Projection) {
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
                .and_then(|name| name.utf8_text(source.as_bytes()).ok())
            else {
                continue;
            };
            let Some(value) = declarator.child_by_field_name("value") else {
                continue;
            };
            if value.kind() == "string" {
                output.values.push((owner.to_owned(), file.to_owned()));
                continue;
            }
            let Some(object) = (value.kind() == "object").then_some(value) else {
                continue;
            };
            let mut object_cursor = object.walk();
            for property in object.named_children(&mut object_cursor) {
                let (member, value) = if property.kind() == "pair" {
                    let Some(key) = property
                        .child_by_field_name("key")
                        .filter(|key| matches!(key.kind(), "property_identifier" | "string"))
                        .and_then(|key| key.utf8_text(source.as_bytes()).ok())
                        .map(|key| key.trim_matches(['\'', '"']))
                        .filter(|key| is_identifier(key))
                    else {
                        continue;
                    };
                    (key, property.child_by_field_name("value"))
                } else if property.kind() == "shorthand_property_identifier" {
                    let Ok(name) = property.utf8_text(source.as_bytes()) else {
                        continue;
                    };
                    (name, Some(property))
                } else {
                    continue;
                };
                let qualified = format!("{owner}.{member}");
                output.members.push(qualified.clone());
                if let Some((name, target)) = value_reference(file, source, value) {
                    output.references.push((qualified, name, target));
                }
            }
        }
        return;
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect(child, source, file, output);
    }
}

fn value_reference(
    file: &str,
    source: &str,
    value: Option<tree_sitter::Node<'_>>,
) -> Option<(String, String)> {
    let value = value?;
    if !matches!(value.kind(), "identifier" | "shorthand_property_identifier") {
        return None;
    }
    let span =
        crate::ir::invocation_evidence::SourceSpan::new(value.start_byte(), value.end_byte())
            .ok()?;
    if crate::ir::typescript_identifier_binding::bind_direct_value_identifier(file, source, span)
        .is_some()
    {
        return Some((
            value.utf8_text(source.as_bytes()).ok()?.to_owned(),
            file.to_owned(),
        ));
    }
    let binding =
        crate::ir::typescript_value_import_binding::bind_direct_named_value_import_allow_alias(
            file, source, span,
        )?;
    Some((binding.imported_name, binding.module_specifier))
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
