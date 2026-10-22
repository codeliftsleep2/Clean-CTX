//! Type occurrences use the same declaration/export authority and source tree.
use super::{DeclarationId, Namespace, Resolution, Resolver};
use crate::ir::invocation_evidence::SourceSpan;
use std::collections::BTreeSet;
use std::path::Path;

impl Resolver {
    pub(super) fn type_reference(
        &mut self,
        file: &Path,
        source: &str,
        span: SourceSpan,
    ) -> Option<Resolution> {
        let captured = self.host.read(file)?;
        if captured.as_ref() != source {
            return Some(Resolution::Unresolved(
                "Type reference snapshot mismatch".into(),
            ));
        }
        let physical = self.host.normalized(file)?;
        let route = self.resolution_file(file)?;
        let file = route.as_path();
        let tree = self.source_tree(file)?;
        let reference = tree
            .root_node()
            .named_descendant_for_byte_range(span.start_byte(), span.end_byte())?;
        if reference.start_byte() == span.start_byte()
            && reference.end_byte() == span.end_byte()
            && reference.kind() == "type_query"
        {
            return self.type_query_reference(file, source, reference);
        }
        if reference.start_byte() == span.start_byte()
            && reference.end_byte() == span.end_byte()
            && matches!(reference.kind(), "member_expression" | "call_expression")
        {
            return self.import_type_reference(file, source, reference);
        }
        if reference.start_byte() != span.start_byte()
            || reference.end_byte() != span.end_byte()
            || !matches!(
                reference.kind(),
                "type_identifier" | "nested_type_identifier"
            )
        {
            return None;
        }
        let parent = reference.parent()?;
        if parent.kind() == "infer_type" {
            return None;
        }
        if parent.child_by_field_name("name") == Some(reference)
            && matches!(
                parent.kind(),
                "type_parameter"
                    | "type_alias_declaration"
                    | "class_declaration"
                    | "abstract_class_declaration"
                    | "interface_declaration"
                    | "enum_declaration"
                    | "mapped_type_clause"
            )
        {
            return None;
        }
        // Qualified names are resolved as one path, never by their terminal name.
        if matches!(
            parent.kind(),
            "nested_type_identifier" | "nested_identifier"
        ) {
            return None;
        }
        let mut parts = Vec::new();
        if !type_path(reference, source, &mut parts) {
            return None;
        }
        let (name, members) = parts.split_first()?;
        let name = *name;
        let qualified = !members.is_empty();
        let mut ancestor = reference.parent();
        while let Some(scope) = ancestor {
            if let Some(parameters) = scope.child_by_field_name("type_parameters") {
                let mut cursor = parameters.walk();
                let declarations: BTreeSet<_> = parameters
                    .named_children(&mut cursor)
                    .filter(|parameter| parameter.kind() == "type_parameter")
                    .filter(|parameter| {
                        parameter.child_by_field_name("name").is_some_and(|node| {
                            node.utf8_text(source.as_bytes()).ok() == Some(name)
                        })
                    })
                    .map(|node| type_id(&physical, source, node, name))
                    .collect();
                if !declarations.is_empty() {
                    return Some(if qualified {
                        Resolution::Unresolved(
                            "A lexical type declaration is not a module namespace".into(),
                        )
                    } else {
                        self.outcome(declarations, true)
                    });
                }
            }
            if scope.kind() == "statement_block" {
                let mut cursor = scope.walk();
                let declarations: BTreeSet<_> = scope
                    .named_children(&mut cursor)
                    .filter(|node| {
                        matches!(
                            node.kind(),
                            "class_declaration"
                                | "abstract_class_declaration"
                                | "interface_declaration"
                                | "type_alias_declaration"
                                | "enum_declaration"
                        )
                    })
                    .filter(|node| {
                        node.child_by_field_name("name").is_some_and(|node| {
                            node.utf8_text(source.as_bytes()).ok() == Some(name)
                        })
                    })
                    .map(|node| type_id(&physical, source, node, name))
                    .collect();
                if !declarations.is_empty() {
                    return Some(if qualified {
                        Resolution::Unresolved(
                            "A lexical type declaration is not a module namespace".into(),
                        )
                    } else {
                        self.outcome(declarations, true)
                    });
                }
            }
            if scope.kind() == "index_signature" {
                let mut cursor = scope.walk();
                let clause = scope
                    .named_children(&mut cursor)
                    .find(|node| node.kind() == "mapped_type_clause");
                if let Some(clause) = clause {
                    let in_constraint = clause
                        .child_by_field_name("type")
                        .is_some_and(|node| contains(node, reference));
                    if !in_constraint
                        && clause.child_by_field_name("name").is_some_and(|node| {
                            node.utf8_text(source.as_bytes()).ok() == Some(name)
                        })
                    {
                        return Some(if qualified {
                            Resolution::Unresolved("Mapped key is not a namespace".into())
                        } else {
                            Resolution::Resolved(type_id(&physical, source, clause, name))
                        });
                    }
                }
            }
            if scope.kind() == "conditional_type"
                && scope
                    .child_by_field_name("consequence")
                    .is_some_and(|node| contains(node, reference))
            {
                if let Some(pattern) = scope.child_by_field_name("right") {
                    let mut pending = vec![pattern];
                    let mut declarations = BTreeSet::new();
                    while let Some(node) = pending.pop() {
                        if node.kind() == "infer_type" {
                            let mut cursor = node.walk();
                            if node
                                .named_children(&mut cursor)
                                .find(|node| node.kind() == "type_identifier")
                                .is_some_and(|node| {
                                    node.utf8_text(source.as_bytes()).ok() == Some(name)
                                })
                            {
                                declarations.insert(type_id(&physical, source, node, name));
                            }
                        }
                        if node.kind() != "conditional_type" {
                            let mut cursor = node.walk();
                            pending.extend(node.named_children(&mut cursor));
                        }
                    }
                    if !declarations.is_empty() {
                        return Some(if qualified {
                            Resolution::Unresolved("Inferred type is not a namespace".into())
                        } else {
                            self.outcome(declarations, true)
                        });
                    }
                }
            }
            if matches!(scope.kind(), "internal_module" | "module") {
                return Some(Resolution::Unresolved(
                    "Type scope requires additional binding authority".into(),
                ));
            }
            ancestor = scope.parent();
        }
        Some(if qualified {
            self.namespace_chain(file, name, members, Namespace::Type)
        } else {
            self.local(file, name, Namespace::Type)
        })
    }
}

fn type_id(file: &Path, source: &str, node: tree_sitter::Node<'_>, name: &str) -> DeclarationId {
    DeclarationId {
        source: file.to_path_buf(),
        source_hash: crate::cache::content_hash(source.as_bytes()),
        start: node.start_byte(),
        end: node.end_byte(),
        name: name.into(),
        kind: node.kind().into(),
    }
}

fn type_path<'a>(node: tree_sitter::Node<'_>, source: &'a str, parts: &mut Vec<&'a str>) -> bool {
    if matches!(
        node.kind(),
        "identifier" | "type_identifier" | "property_identifier"
    ) {
        let Ok(name) = node.utf8_text(source.as_bytes()) else {
            return false;
        };
        parts.push(name);
        return true;
    }
    if !matches!(node.kind(), "nested_type_identifier" | "nested_identifier") {
        return false;
    }
    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .filter(|node| node.kind() != "comment")
        .all(|child| type_path(child, source, parts))
}

fn contains(owner: tree_sitter::Node<'_>, reference: tree_sitter::Node<'_>) -> bool {
    owner.start_byte() <= reference.start_byte() && owner.end_byte() >= reference.end_byte()
}
