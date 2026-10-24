//! Functional DI role interpretation over retained native type/value evidence.
use super::class_role_authority::public_api;
use crate::ir::invocation_evidence::{InvocationOccurrence, SourceSpan, WrittenCallee};
use crate::ir::typescript_resolution::{Resolution, TypeScriptAuthority};

fn role_kind(
    authority: &TypeScriptAuthority,
    role: &crate::ir::typescript_resolution::DeclarationId,
) -> Option<&'static str> {
    let (package, kind) = match role.name.as_str() {
        "CanActivateFn" | "CanMatchFn" => ("@angular/router", "Guard"),
        "ResolveFn" => ("@angular/router", "Resolver"),
        "HttpInterceptorFn" => {
            return super::class_role_authority::public_entry_api(
                authority,
                role,
                "@angular/common",
                "@angular/common/http",
                "HttpInterceptorFn",
                crate::ir::typescript_resolution::Namespace::Type,
            )
            .then_some("Interceptor");
        }
        _ => return None,
    };
    public_api(authority, role, package, &role.name).then_some(kind)
}

/// Producer applicability only; exact owner/call admission remains in project.
pub(crate) fn has_role(authority: &TypeScriptAuthority) -> bool {
    authority.type_reference_sites.iter().any(|(_,_,resolution)|
        matches!(resolution, Resolution::Resolved(role) if role_kind(authority, role).is_some()))
}

pub(super) fn project(
    authority: &TypeScriptAuthority,
    root: tree_sitter::Node<'_>,
    call: &InvocationOccurrence,
) -> Option<crate::layers::meta::semantic::SemanticEdge> {
    let callee = match call.callee() {
        WrittenCallee::Free { terminal } => *terminal,
        WrittenCallee::Member { receiver, terminal } => {
            SourceSpan::new(receiver.start_byte(), terminal.end_byte()).ok()?
        }
        _ => return None,
    };
    let api = authority.value_reference_at(callee)?;
    if !public_api(authority, api, "@angular/core", "inject") {
        return None;
    }
    let context = call.declaration_context()?;
    let span = context.declaration();
    let mut node = root.named_descendant_for_byte_range(span.start_byte(), span.end_byte())?;
    if node.start_byte() != span.start_byte() || node.end_byte() != span.end_byte() {
        return None;
    }
    while node.kind() != "variable_declarator" {
        node = node.parent()?;
    }
    let value = node.child_by_field_name("value")?;
    if !(matches!(value.kind(), "arrow_function" | "function_expression")
        && value.start_byte() <= call.span().start_byte()
        && call.span().end_byte() <= value.end_byte())
    {
        return None;
    }
    let annotation = node.child_by_field_name("type")?;
    let role_type = annotation.named_child(0)?;
    let role_type = if role_type.kind() == "generic_type" {
        role_type.child_by_field_name("name")?
    } else {
        role_type
    };
    let mut sites = authority
        .type_reference_sites
        .iter()
        .filter(|(start, end, _)| *start == role_type.start_byte() && *end == role_type.end_byte());
    let (_, _, Resolution::Resolved(role)) = sites.next()? else {
        return None;
    };
    if sites.next().is_some() {
        return None;
    }
    let kind = role_kind(authority, role)?;
    let mut owners = authority.declarations.iter().filter(|id| {
        id.source == authority.source
            && id.start == node.start_byte()
            && id.end == node.end_byte()
            && id.kind == "variable_declarator"
            && id.source_hash == authority.source_hash
    });
    let owner = owners.next()?;
    if owners.next().is_some() {
        return None;
    }
    let mut facets = authority
        .declaration_facets
        .iter()
        .filter(|facet| facet.declaration == *owner);
    facets.next()?;
    if facets.next().is_some() {
        return None;
    }
    let token = authority.value_reference_at(call.arguments().first()?.span())?;
    Some(super::inject_authority::canonical_edge(owner, kind, token))
}
