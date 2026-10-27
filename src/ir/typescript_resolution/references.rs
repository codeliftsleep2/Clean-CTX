//! Exact value-reference evidence from the consumed source tree.
use super::{DeclarationId, Namespace, Resolution, Resolver};
use crate::ir::typescript_identifier_binding::ValueDeclarationKind;
use std::path::Path;

/// Reuse lexical shadow proof before crossing a source-owned callable scope.
/// This grants no import/API meaning and does not certify arbitrary scopes.
fn callable_import_scope(
    reference: tree_sitter::Node<'_>,
    file: &Path,
    source: &str,
    root: tree_sitter::Node<'_>,
    invocation: bool,
) -> bool {
    let mut receiver = reference;
    while receiver.kind() == "member_expression" {
        let Some(object) = receiver.child_by_field_name("object") else {
            return false;
        };
        receiver = object;
    }
    if receiver.kind() != "identifier" {
        return false;
    }
    let Some(file) = file.to_str() else {
        return false;
    };
    let Ok(span) =
        crate::ir::invocation_evidence::SourceSpan::new(receiver.start_byte(), receiver.end_byte())
    else {
        return false;
    };
    if !matches!(
        crate::ir::typescript_identifier_binding::lookup_value_identifier_in_tree(
            file, source, span, root
        ),
        Ok(None)
    ) {
        return false;
    }
    let Ok(name) = receiver.utf8_text(source.as_bytes()) else {
        return false;
    };
    let mut parent = reference.parent();
    let mut callable = false;
    while let Some(node) = parent {
        match node.kind() {
            "program" => return callable || invocation,
            "arrow_function" | "function_expression" => callable = true,
            "function_declaration" if invocation => {
                if node
                    .child_by_field_name("name")
                    .and_then(|id| id.utf8_text(source.as_bytes()).ok())
                    == Some(name)
                {
                    return false;
                }
                callable = true;
            }
            "class_body"
            | "method_definition"
            | "function_declaration"
            | "generator_function"
            | "generator_function_declaration"
            | "for_statement"
            | "for_in_statement"
            | "catch_clause"
            | "internal_module"
            | "module" => return false,
            "statement_block" => {
                // The shared variable binder does not certify block-scoped
                // class/function/enum declarations; refuse their names here.
                let mut cursor = node.walk();
                if node.named_children(&mut cursor).any(|child| {
                    matches!(
                        child.kind(),
                        "class_declaration"
                            | "abstract_class_declaration"
                            | "function_declaration"
                            | "enum_declaration"
                    ) && child
                        .child_by_field_name("name")
                        .and_then(|id| id.utf8_text(source.as_bytes()).ok())
                        == Some(name)
                }) {
                    return false;
                }
            }
            _ => {}
        }
        parent = node.parent();
    }
    false
}

impl Resolver {
    pub(super) fn value_reference(
        &mut self,
        file: &Path,
        source: &str,
        span: crate::ir::invocation_evidence::SourceSpan,
    ) -> Option<Resolution> {
        self.value_reference_for_query(file, source, span, false)
    }

    pub(super) fn value_reference_for_query(
        &mut self,
        file: &Path,
        source: &str,
        span: crate::ir::invocation_evidence::SourceSpan,
        type_query: bool,
    ) -> Option<Resolution> {
        self.value_reference_with_scope(file, source, span, type_query, false)
    }

    /// Only explicitly demanded native invocation positions use this guarded
    /// scope path. General reference/type-query consumers retain their policy.
    pub(super) fn value_reference_for_invocation(
        &mut self,
        file: &Path,
        source: &str,
        span: crate::ir::invocation_evidence::SourceSpan,
    ) -> Option<Resolution> {
        self.value_reference_with_scope(file, source, span, false, true)
    }

    fn value_reference_with_scope(
        &mut self,
        file: &Path,
        source: &str,
        span: crate::ir::invocation_evidence::SourceSpan,
        type_query: bool,
        invocation: bool,
    ) -> Option<Resolution> {
        let captured = self.host.read(file)?;
        if captured.as_ref() != source {
            return Some(Resolution::Unresolved(
                "Reference text differs from the consumed source snapshot".into(),
            ));
        }
        let tree = self.source_tree(file)?;
        if tree.root_node().has_error() {
            return None;
        }
        let reference = tree
            .root_node()
            .named_descendant_for_byte_range(span.start_byte(), span.end_byte())?;
        if reference.start_byte() != span.start_byte()
            || reference.end_byte() != span.end_byte()
            || !matches!(
                reference.kind(),
                "identifier" | "shorthand_property_identifier" | "member_expression"
            )
        {
            return None;
        }
        if !type_query && super::import_types::in_type_position(reference) {
            return None;
        }
        if crate::ir::typescript_binding_pattern::is_binding_name(reference) {
            return None;
        }
        if let Some(parent) = reference.parent()
            && matches!(
                parent.kind(),
                "variable_declarator" | "required_parameter" | "optional_parameter"
            )
            && (parent.child_by_field_name("name") == Some(reference)
                || parent.child_by_field_name("pattern") == Some(reference))
        {
            return None;
        }
        if reference.parent().is_some_and(|parent| {
            parent.kind() == "catch_clause"
                && parent.child_by_field_name("parameter") == Some(reference)
        }) {
            return None;
        }
        let constructor_decorator =
            super::parameter_decorators::is_constructor_decorator_site(reference, source);
        let lookup = if reference.kind() == "member_expression" {
            Ok(None)
        } else if constructor_decorator {
            crate::ir::typescript_identifier_binding::lookup_constructor_decorator_value_in_tree(
                file.to_str()?,
                source,
                span,
                tree.root_node(),
            )
        } else {
            crate::ir::typescript_identifier_binding::lookup_value_identifier_in_tree(
                file.to_str()?,
                source,
                span,
                tree.root_node(),
            )
        };
        let binding = match lookup {
            Ok(binding) => binding,
            Err(()) => {
                return Some(Resolution::Unresolved(
                    "Lexical binding is ambiguous or unproved".into(),
                ));
            }
        };
        if let Some(binding) = binding {
            let declared = binding.declaration_span();
            let arrow_parameter =
                binding.declaration_kind() == ValueDeclarationKind::ArrowParameter;
            let function_self = binding.declaration_kind() == ValueDeclarationKind::FunctionSelf;
            let loop_binding = matches!(
                binding.declaration_kind(),
                ValueDeclarationKind::LoopLet
                    | ValueDeclarationKind::LoopConst
                    | ValueDeclarationKind::LoopVar
            );
            let pattern_binding = matches!(
                binding.declaration_kind(),
                ValueDeclarationKind::PatternConst
                    | ValueDeclarationKind::PatternLet
                    | ValueDeclarationKind::PatternVar
                    | ValueDeclarationKind::PatternParameter
                    | ValueDeclarationKind::PatternCatch
            );
            let name = tree
                .root_node()
                .named_descendant_for_byte_range(declared.start_byte(), declared.end_byte())?;
            let mut declaration = name.parent()?;
            while matches!(
                declaration.kind(),
                "object_pattern"
                    | "array_pattern"
                    | "pair_pattern"
                    | "assignment_pattern"
                    | "object_assignment_pattern"
                    | "rest_pattern"
            ) {
                declaration = declaration.parent()?;
            }
            if !matches!(
                declaration.kind(),
                "variable_declarator"
                    | "required_parameter"
                    | "optional_parameter"
                    | "catch_clause"
                    | "for_in_statement"
                    | "arrow_function"
                    | "function_expression"
                    | "generator_function"
            ) {
                return Some(Resolution::Unresolved(
                    "Lexical declaration form requires authority".into(),
                ));
            }
            let catch_parameter = declaration.kind() == "catch_clause";
            let mut owner = if catch_parameter || loop_binding || arrow_parameter || function_self {
                Some(declaration)
            } else {
                declaration.parent()
            };
            while let Some(scope) = owner {
                let parameter = matches!(
                    declaration.kind(),
                    "required_parameter" | "optional_parameter"
                );
                let function_var = matches!(
                    binding.declaration_kind(),
                    ValueDeclarationKind::FunctionVar
                        | ValueDeclarationKind::PatternVar
                        | ValueDeclarationKind::LoopVar
                );
                let boundary = if arrow_parameter || function_self {
                    scope == declaration
                } else if catch_parameter {
                    scope.kind() == "catch_clause"
                } else if parameter || function_var {
                    matches!(
                        scope.kind(),
                        "function_declaration"
                            | "function_expression"
                            | "arrow_function"
                            | "method_definition"
                            | "generator_function"
                            | "generator_function_declaration"
                    ) || (function_var && scope.kind() == "program")
                } else {
                    matches!(
                        scope.kind(),
                        "program" | "statement_block" | "for_statement" | "for_in_statement"
                    )
                };
                if boundary {
                    if scope.start_byte() > reference.start_byte()
                        || scope.end_byte() < reference.end_byte()
                    {
                        return Some(Resolution::Unresolved(
                            "Lexical candidate is outside the reference scope".into(),
                        ));
                    }
                    return Some(Resolution::Resolved(DeclarationId {
                        source: self.host.normalized(file)?,
                        source_hash: binding.source_hash().into(),
                        start: if pattern_binding || loop_binding || arrow_parameter {
                            name.start_byte()
                        } else {
                            declaration.start_byte()
                        },
                        end: if pattern_binding || loop_binding || arrow_parameter {
                            name.end_byte()
                        } else {
                            declaration.end_byte()
                        },
                        name: name.utf8_text(source.as_bytes()).ok()?.into(),
                        kind: if arrow_parameter {
                            "arrow_parameter"
                        } else if loop_binding {
                            "loop_binding"
                        } else if pattern_binding {
                            "binding_pattern"
                        } else {
                            declaration.kind()
                        }
                        .into(),
                    }));
                }
                owner = scope.parent();
            }
            return Some(Resolution::Unresolved("Lexical scope is unresolved".into()));
        }
        let callable_closed =
            callable_import_scope(reference, file, source, tree.root_node(), invocation);
        let mut ancestor = reference.parent();
        let mut direct_field_initializer = false;
        let mut parameter_decorator = constructor_decorator;
        let mut namespace_owners = Vec::new();
        while let Some(node) = ancestor {
            if node.kind() == "internal_module" {
                if let Some(name) = node.child_by_field_name("name")
                    && name.kind() == "nested_identifier"
                {
                    let mut pending = vec![name];
                    let mut segments = Vec::new();
                    while let Some(part) = pending.pop() {
                        if matches!(part.kind(), "identifier" | "property_identifier") {
                            segments.push((part.start_byte(), part.end_byte()));
                        } else if part.kind() == "nested_identifier" {
                            let mut cursor = part.walk();
                            pending.extend(part.named_children(&mut cursor));
                        }
                    }
                    segments.sort();
                    namespace_owners.extend(segments.into_iter().skip(1).rev());
                }
                namespace_owners.push((node.start_byte(), node.end_byte()));
                ancestor = node.parent();
                continue;
            }
            if node.kind() == "statement_block"
                && node
                    .parent()
                    .is_some_and(|parent| parent.kind() == "internal_module")
            {
                ancestor = node.parent();
                continue;
            }
            // A direct class-field value is evaluated in the enclosing lexical
            // environment. Class members do not introduce unqualified value
            // bindings. The tree-taking lexical binder above has already checked
            // local candidates; method/function/static-block guards still apply.
            if node.kind() == "public_field_definition" {
                direct_field_initializer = node.child_by_field_name("value").is_some_and(|value| {
                    value.start_byte() <= reference.start_byte()
                        && reference.end_byte() <= value.end_byte()
                });
            }
            if node.kind() == "formal_parameters" && parameter_decorator {
                ancestor = node.parent();
                continue;
            }
            if node.kind() == "class_body"
                && (direct_field_initializer || parameter_decorator)
                && node.parent().is_some_and(|owner| {
                    matches!(
                        owner.kind(),
                        "class_declaration" | "abstract_class_declaration"
                    )
                })
            {
                direct_field_initializer = false;
                parameter_decorator = false;
                ancestor = node.parent();
                continue;
            }
            if callable_closed
                && (matches!(
                    node.kind(),
                    "statement_block" | "arrow_function" | "function_expression"
                ) || (invocation && node.kind() == "function_declaration"))
            {
                ancestor = node.parent();
                continue;
            }
            // Other nested lexical environments are not certified by this
            // occurrence adapter. Do not mistake an unsupported shadow for
            // the absence of a shadow and fall back to a module import.
            if matches!(
                node.kind(),
                "statement_block"
                    | "class_body"
                    | "formal_parameters"
                    | "arrow_function"
                    | "function_expression"
                    | "function_declaration"
                    | "for_statement"
                    | "for_in_statement"
                    | "catch_clause"
                    | "internal_module"
                    | "module"
            ) {
                return None;
            }
            if node.kind() == "import_statement" {
                return None;
            }
            ancestor = node.parent();
        }
        if !namespace_owners.is_empty() && reference.kind() != "member_expression" {
            return Some(self.namespace_lexical_reference(
                file,
                &namespace_owners,
                reference.utf8_text(source.as_bytes()).ok()?,
                &[],
            ));
        }
        if reference.kind() == "member_expression" {
            let mut object = reference;
            let mut members = Vec::new();
            while object.kind() == "member_expression" {
                let mut cursor = object.walk();
                let children: Vec<_> = object
                    .named_children(&mut cursor)
                    .filter(|child| child.kind() != "comment")
                    .collect();
                let [owner, member] = children.as_slice() else {
                    return None;
                };
                if member.kind() != "property_identifier" {
                    return None;
                }
                members.push(member.utf8_text(source.as_bytes()).ok()?);
                object = *owner;
            }
            if object.kind() != "identifier" {
                return None;
            }
            members.reverse();
            let object = object.utf8_text(source.as_bytes()).ok()?;
            if !namespace_owners.is_empty() {
                return Some(self.namespace_lexical_reference(
                    file,
                    &namespace_owners,
                    object,
                    &members,
                ));
            }
            if type_query {
                return Some(self.query_value_binding(file, object, &members));
            }
            return Some(if members.len() == 1 {
                self.namespace_member(file, object, members[0], Namespace::Value)
            } else {
                self.namespace_chain(file, object, &members, Namespace::Value)
            });
        }
        // The exact occurrence and scope guards establish which source-owned
        // lexical namespace applies. Canonical resolution records absent or
        // competing bindings rather than guessing another declaration.
        let name = reference.utf8_text(source.as_bytes()).ok()?;
        Some(if type_query {
            self.query_value_binding(file, name, &[])
        } else {
            // Record bare absence only after exact lexical/scope checks above.
            // An ineligible import or type declaration is still a binding and
            // must never become a symbolic runtime token by this route.
            self.in_source_project(file, |resolver| {
                if resolver
                    .unit(file)
                    .is_some_and(|unit| !unit.locals.contains_key(name))
                {
                    let (globals, complete) = resolver.global_bindings(
                        file,
                        name,
                        Namespace::Value,
                        &mut std::collections::BTreeSet::new(),
                    );
                    let (types, types_complete) = resolver.global_bindings(
                        file,
                        name,
                        Namespace::Type,
                        &mut std::collections::BTreeSet::new(),
                    );
                    if complete
                        && types_complete
                        && types.is_empty()
                        && globals.is_empty()
                        && let Some(owner) = resolver.host.normalized(file)
                    {
                        resolver.unbound_value_sites.insert((
                            owner,
                            span.start_byte(),
                            span.end_byte(),
                            name.to_owned(),
                        ));
                    }
                }
                resolver.local(file, name, Namespace::Value)
            })
        })
    }
}
