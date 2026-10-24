//! Structured Angular member-signal and functional-inject interpretation.

use crate::ir::invocation_evidence::{
    DeclarationContextKind, InvocationEvidenceView, InvocationOccurrence, SourceSpan, WrittenCallee,
};
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};

#[cfg(test)]
pub(crate) fn project(
    source_id: &str,
    source: &str,
    evidence: &InvocationEvidenceView<'_>,
) -> Vec<SemanticEdge> {
    project_with_authority(source_id, source, evidence, None)
}

pub(crate) fn project_with_authority(
    source_id: &str,
    source: &str,
    evidence: &InvocationEvidenceView<'_>,
    authority: Option<&crate::ir::typescript_resolution::TypeScriptAuthority>,
) -> Vec<SemanticEdge> {
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
    let mut edges = Vec::new();
    for call in evidence.occurrences() {
        if let Some((owner_type, owner, member, kind)) =
            signal_call(source_id, source, tree.root_node(), call, authority)
        {
            let member_evidence = if let Some(authority) = authority {
                let Some(context) = call.declaration_context() else {
                    continue;
                };
                let Some(evidence) = authority.member_at(context.declaration(), call.span()) else {
                    continue;
                };
                if evidence.owner.name != owner || evidence.member.name != member {
                    continue;
                }
                Some(evidence)
            } else {
                None
            };
            let first = edges.len();
            let subject = EntityRef::new("angular", owner_type, owner);
            if matches!(kind, SignalApi::Input | SignalApi::Model) {
                edges.push(SemanticEdge {
                    relation: SemanticRelation::HasInput,
                    subject: subject.clone(),
                    object: EntityRef::new("angular", "InputField", &member),
                    layer: "angular",
                    call_evidence: None,
                });
            }
            if matches!(kind, SignalApi::Output | SignalApi::Model) {
                edges.push(SemanticEdge {
                    relation: SemanticRelation::HasOutput,
                    subject,
                    object: EntityRef::new("angular", "OutputField", member),
                    layer: "angular",
                    call_evidence: None,
                });
            }
            if let Some(evidence) = member_evidence {
                use crate::layers::meta::canonical_endpoint::CanonicalEndpoint;
                for edge in &mut edges[first..] {
                    edge.subject.file = Some(evidence.owner.source.to_string_lossy().into_owned());
                    edge.object.file = Some(evidence.member.source.to_string_lossy().into_owned());
                    edge.subject.canonical = Some(CanonicalEndpoint::Declaration {
                        declaration: (&evidence.owner).into(),
                        site: Some((evidence.owner.start, evidence.owner.end)),
                    });
                    edge.object.canonical = Some(CanonicalEndpoint::Declaration {
                        declaration: (&evidence.member).into(),
                        site: Some(evidence.initializer),
                    });
                }
            }
        }
        if let Some(authority) = authority {
            if let Some(edge) =
                super::functional_authority::project(authority, tree.root_node(), call)
            {
                edges.push(edge);
            }
        } else if let Some((owner_type, owner, token)) =
            functional_inject(source_id, source, tree.root_node(), evidence, call)
        {
            edges.push(SemanticEdge {
                relation: SemanticRelation::Injects,
                subject: EntityRef::new("angular", owner_type, owner),
                object: EntityRef::new("angular", "Service", token),
                layer: "angular",
                call_evidence: None,
            });
        }
    }
    edges
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SignalApi {
    Input,
    Output,
    Model,
}

fn signal_call(
    source_id: &str,
    source: &str,
    root: tree_sitter::Node<'_>,
    call: &InvocationOccurrence,
    authority: Option<&crate::ir::typescript_resolution::TypeScriptAuthority>,
) -> Option<(&'static str, String, String, SignalApi)> {
    if call.parent().is_some() {
        return None;
    }
    let kind = if let Some(authority) = authority {
        match super::member_authority::api(authority, source, call)? {
            "input" => SignalApi::Input,
            "output" => SignalApi::Output,
            "model" => SignalApi::Model,
            _ => return None,
        }
    } else {
        let (binding_span, imported_name, kind) = match call.callee() {
            WrittenCallee::Free { terminal } => {
                let name = slice(source, *terminal)?;
                let kind = match name {
                    "input" => SignalApi::Input,
                    "output" => SignalApi::Output,
                    "model" => SignalApi::Model,
                    _ => return None,
                };
                (*terminal, name, kind)
            }
            WrittenCallee::Member { receiver, terminal }
                if slice(source, *receiver)? == "input"
                    && slice(source, *terminal)? == "required" =>
            {
                (*receiver, "input", SignalApi::Input)
            }
            _ => return None,
        };
        let binding = crate::ir::typescript_value_import_binding::bind_named_value_import(
            source_id,
            source,
            binding_span,
        )?;
        if binding.module_specifier != "@angular/core" || binding.imported_name != imported_name {
            return None;
        }
        kind
    };
    let context = call.declaration_context()?;
    if !matches!(
        context.kind(),
        DeclarationContextKind::FieldInitializer | DeclarationContextKind::PropertyInitializer
    ) {
        return None;
    }
    let declaration = exact_node(root, context.declaration())?;
    let initializer = declaration.child_by_field_name("value")?;
    if initializer.start_byte() != call.span().start_byte()
        || initializer.end_byte() != call.span().end_byte()
    {
        return None;
    }
    let member = slice(source, context.written_name()?)?
        .trim_end_matches(['?', '!'])
        .to_owned();
    if !is_identifier(&member) {
        return None;
    }
    let class = ancestor(declaration, "class_declaration")?;
    let owner = class
        .child_by_field_name("name")?
        .utf8_text(source.as_bytes())
        .ok()?
        .to_owned();
    let owner_type = match authority {
        Some(authority) => super::member_authority::owner(authority, source, class)?,
        None => angular_declaration_type(source_id, source, class)?,
    };
    Some((owner_type, owner, member, kind))
}

fn functional_inject(
    source_id: &str,
    source: &str,
    root: tree_sitter::Node<'_>,
    evidence: &InvocationEvidenceView<'_>,
    call: &InvocationOccurrence,
) -> Option<(&'static str, String, String)> {
    let terminal = match call.callee() {
        WrittenCallee::Free { terminal } if slice(source, *terminal)? == "inject" => *terminal,
        _ => return None,
    };
    let inject_import = crate::ir::typescript_value_import_binding::bind_named_value_import(
        source_id, source, terminal,
    )?;
    if inject_import.module_specifier != "@angular/core" || inject_import.imported_name != "inject"
    {
        return None;
    }
    let context = call.declaration_context()?;
    if !matches!(
        context.kind(),
        DeclarationContextKind::VariableInitializer | DeclarationContextKind::Callable
    ) {
        return None;
    }
    let declaration = exact_node(root, context.declaration())?;
    let role_declaration = ancestor(declaration, "variable_declarator").unwrap_or(declaration);
    let (owner_type, _) = functional_role(source_id, source, role_declaration)?;
    let owner = slice(source, context.written_name()?)?.to_owned();
    let argument = call.arguments().first()?;
    let argument_node = exact_node(root, argument.span())?;
    let token = crate::angular_meta::provider_semantics::reference_identity(
        source_id,
        source,
        root,
        argument_node,
    )?;
    // Slice through the evidence view as the final source-bound validation.
    evidence.slice(argument.span()).ok()?;
    Some((owner_type, owner, token))
}

fn functional_role(
    source_id: &str,
    source: &str,
    declaration: tree_sitter::Node<'_>,
) -> Option<(&'static str, String)> {
    let roles = [
        ("CanActivateFn", "@angular/router", "Guard"),
        ("CanMatchFn", "@angular/router", "Guard"),
        ("ResolveFn", "@angular/router", "Resolver"),
        ("HttpInterceptorFn", "@angular/common/http", "Interceptor"),
    ];
    let mut found = Vec::new();
    collect_identifiers(declaration, &mut found);
    for identifier in found {
        let written = identifier.utf8_text(source.as_bytes()).ok()?;
        let Some((_, module, owner_type)) = roles.iter().find(|(role, _, _)| *role == written)
        else {
            continue;
        };
        let span = SourceSpan::new(identifier.start_byte(), identifier.end_byte()).ok()?;
        let binding =
            crate::ir::typescript_type_import_binding::bind_type_reference_to_named_import(
                source_id, source, span,
            )?;
        if binding.module_specifier() == *module && binding.imported_symbol_name() == written {
            return Some((*owner_type, written.to_owned()));
        }
    }
    None
}

fn angular_declaration_type(
    source_id: &str,
    source: &str,
    class: tree_sitter::Node<'_>,
) -> Option<&'static str> {
    let decorator_owner = crate::angular_meta::provider_semantics::decorator_owner(class);
    let mut cursor = decorator_owner.walk();
    decorator_owner
        .named_children(&mut cursor)
        .filter(|node| node.kind() == "decorator")
        .find_map(|decorator| {
            let call = decorator
                .named_child(0)
                .filter(|node| node.kind() == "call_expression")?;
            let function = call.child_by_field_name("function")?;
            let span = SourceSpan::new(function.start_byte(), function.end_byte()).ok()?;
            let binding = crate::ir::typescript_value_import_binding::bind_named_value_import(
                source_id, source, span,
            )?;
            if binding.module_specifier != "@angular/core" {
                return None;
            }
            match binding.imported_name.as_str() {
                "Component" => Some("Component"),
                "Directive" => Some("Directive"),
                _ => None,
            }
        })
}
fn exact_node<'tree>(
    root: tree_sitter::Node<'tree>,
    span: SourceSpan,
) -> Option<tree_sitter::Node<'tree>> {
    let node = root.named_descendant_for_byte_range(span.start_byte(), span.end_byte())?;
    (node.start_byte() == span.start_byte() && node.end_byte() == span.end_byte()).then_some(node)
}

fn ancestor<'tree>(
    mut node: tree_sitter::Node<'tree>,
    kind: &str,
) -> Option<tree_sitter::Node<'tree>> {
    loop {
        if node.kind() == kind {
            return Some(node);
        }
        node = node.parent()?;
    }
}

fn collect_identifiers<'tree>(
    node: tree_sitter::Node<'tree>,
    output: &mut Vec<tree_sitter::Node<'tree>>,
) {
    if node.kind() == "type_identifier" || node.kind() == "identifier" {
        output.push(node);
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_identifiers(child, output);
    }
}

fn is_identifier(value: &str) -> bool {
    let mut characters = value.chars();
    characters
        .next()
        .is_some_and(|character| character.is_alphabetic() || matches!(character, '_' | '$'))
        && characters.all(|character| character.is_alphanumeric() || matches!(character, '_' | '$'))
}

fn slice(source: &str, span: SourceSpan) -> Option<&str> {
    source.get(span.start_byte()..span.end_byte())
}

#[cfg(test)]
#[path = "../tests/angular_meta/member_semantics.rs"]
mod tests;
