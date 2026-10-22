//! Import types retain their complete syntax-owned module/member path.
use super::{Namespace, Resolution, Resolver, module_resolution::ModuleRequestKind};
use std::path::Path;
use tree_sitter::Node;

pub(super) fn in_type_position(mut node: Node<'_>) -> bool {
    while let Some(parent) = node.parent() {
        if parent.kind() == "member_expression"
            && parent.child_by_field_name("object") == Some(node)
        {
            node = parent;
            continue;
        }
        return match parent.kind() {
            "type_alias_declaration" => parent.child_by_field_name("value") == Some(node),
            "type_query" | "type_annotation" | "type_arguments" | "union_type"
            | "intersection_type" | "array_type" | "tuple_type" | "optional_type" | "rest_type"
            | "parenthesized_type" | "generic_type" | "conditional_type" | "lookup_type"
            | "index_type_query" | "type_parameter" => true,
            _ => false,
        };
    }
    false
}

pub(super) fn call_mode(call: Node<'_>, source: &str) -> Result<ModuleRequestKind, String> {
    let arguments = call
        .child_by_field_name("arguments")
        .ok_or("Missing import type arguments")?;
    let mut cursor = arguments.walk();
    let args: Vec<_> = arguments
        .named_children(&mut cursor)
        .filter(|child| child.kind() != "comment")
        .collect();
    if args.len() == 1 {
        return Ok(ModuleRequestKind::StaticImport);
    }
    if args.len() != 2 || args[1].kind() != "object" {
        return Err("Invalid import type options".into());
    }
    let mut cursor = args[1].walk();
    let mut selected = None;
    for pair in args[1]
        .named_children(&mut cursor)
        .filter(|child| child.kind() != "comment")
    {
        if pair.kind() != "pair" {
            return Err("Unproved import type options".into());
        }
        let key = pair
            .child_by_field_name("key")
            .ok_or("Missing import type option")?;
        let text = key
            .utf8_text(source.as_bytes())
            .map_err(|_| "Invalid import type option")?;
        let key = if key.kind() == "string" {
            super::string_value::decode(text).ok_or("Invalid option key")?
        } else if key.kind() == "property_identifier" {
            text.to_string()
        } else {
            return Err("Unproved import type option key".into());
        };
        if key != "with" && key != "assert" {
            return Err("Unknown import type option".into());
        }
        if selected.is_some() {
            return Err("Duplicate import type attributes".into());
        }
        let object = pair
            .child_by_field_name("value")
            .ok_or("Missing import type attributes")?;
        selected = Some(
            super::import_attributes::object_mode(object, source)?
                .unwrap_or(ModuleRequestKind::StaticImport),
        );
    }
    Ok(selected.unwrap_or(ModuleRequestKind::StaticImport))
}

pub(super) fn call_and_members<'a>(
    mut node: Node<'a>,
    source: &str,
) -> Option<(Node<'a>, Vec<String>)> {
    let mut members = Vec::new();
    while node.kind() == "member_expression" {
        // The pinned grammar propagates fields through aliased type-query
        // productions. A field lookup may return an inner qualifier's property.
        // Own each segment through this member node's immediate syntax children.
        let mut cursor = node.walk();
        let children: Vec<_> = node
            .named_children(&mut cursor)
            .filter(|child| child.kind() != "comment")
            .collect();
        let [object, property] = children.as_slice() else {
            return None;
        };
        if property.kind() != "property_identifier" {
            return None;
        }
        members.push(property.utf8_text(source.as_bytes()).ok()?.to_string());
        node = *object;
    }
    if node.kind() != "call_expression" || node.child_by_field_name("function")?.kind() != "import"
    {
        return None;
    }
    members.reverse();
    Some((node, members))
}

impl Resolver {
    pub(super) fn import_type_reference(
        &mut self,
        file: &Path,
        source: &str,
        node: Node<'_>,
    ) -> Option<Resolution> {
        self.import_reference_in_namespace(file, source, node, Namespace::Type)
    }

    pub(super) fn import_reference_in_namespace(
        &mut self,
        file: &Path,
        source: &str,
        node: Node<'_>,
        namespace: Namespace,
    ) -> Option<Resolution> {
        if !in_type_position(node) {
            return None;
        }
        if node.parent().is_some_and(|parent| {
            parent.kind() == "member_expression"
                && parent.child_by_field_name("object") == Some(node)
        }) {
            return None;
        }
        let (call, members) = call_and_members(node, source)?;
        let arguments = call.child_by_field_name("arguments")?;
        let mut cursor = arguments.walk();
        let literal = arguments
            .named_children(&mut cursor)
            .find(|child| child.kind() != "comment")?;
        if literal.kind() != "string" {
            return Some(Resolution::Unresolved("Nonliteral import type".into()));
        }
        let resolved = (|| {
            let specifier = super::string_value::decode(
                literal
                    .utf8_text(source.as_bytes())
                    .map_err(|_| "Invalid import type text")?,
            )
            .ok_or("Invalid import type specifier")?;
            let mode = call_mode(call, source)?;
            self.in_source_project(file, |resolver| {
                let target = resolver.module_scope(file, &specifier, mode)?;
                let Some((first, rest)) = members.split_first() else {
                    return Err("Import type namespace has no declaration leaf".into());
                };
                if rest.is_empty() {
                    Ok(resolver.scope_export(&target, first, namespace))
                } else {
                    let rest: Vec<_> = rest.iter().map(String::as_str).collect();
                    Ok(resolver.namespace_chain_from(&target, first, &rest, namespace, true))
                }
            })
        })();
        Some(resolved.unwrap_or_else(Resolution::Unresolved))
    }
}
