//! Angular-owned role admission from exact neutral TypeScript endpoints.
use super::decorators::ClassKind;
use crate::ir::typescript_resolution::{Resolution, TypeScriptAuthority, Witness};

pub(crate) struct ClassRole {
    pub kind: ClassKind,
    pub selector: Option<String>,
    pub pipe_name: Option<String>,
}

fn api_role(name: &str) -> Option<ClassKind> {
    match name {
        "Component" => Some(ClassKind::Component),
        "Injectable" => Some(ClassKind::Service),
        "NgModule" => Some(ClassKind::Module),
        "Directive" => Some(ClassKind::Directive),
        "Pipe" => Some(ClassKind::Pipe),
        _ => None,
    }
}

pub(crate) fn public_api(
    authority: &TypeScriptAuthority,
    endpoint: &crate::ir::typescript_resolution::DeclarationId,
    package_name: &str,
    export_name: &str,
) -> bool {
    endpoint.name == export_name && authority.package_exports.iter().any(|package| {
        package.declaration == *endpoint && package.name == package_name
            && package.version.as_ref().is_some_and(|version| !version.is_empty())
            && package.public_export == Resolution::Resolved(endpoint.clone())
            && authority.witnesses.iter().any(|witness| matches!(witness,
                Witness::Content {path, hash} if path == &package.manifest && hash == &package.manifest_hash))
    })
}

pub(crate) fn decorator_role(
    authority: &TypeScriptAuthority,
    callee: &Resolution,
) -> Option<ClassKind> {
    let Resolution::Resolved(endpoint) = callee else {
        return None;
    };
    public_api(authority, endpoint, "@angular/core", &endpoint.name)
        .then(|| api_role(&endpoint.name))
        .flatten()
}

pub(crate) fn declaration_role(
    authority: &TypeScriptAuthority,
    id: &crate::ir::typescript_resolution::DeclarationId,
) -> Option<ClassKind> {
    use crate::ir::typescript_resolution::declaration_facets::ExpressionValue;
    let facet = authority
        .declaration_facets
        .iter()
        .find(|facet| facet.declaration == *id)?;
    let mut roles = facet.decorators.iter().filter_map(|decorator| {
        if let ExpressionValue::Reference(callee) = &decorator.callee.value {
            decorator_role(authority, callee)
        } else {
            None
        }
    });
    let role = roles.next()?;
    // Conflicting framework decorators cannot supply a unique class role.
    roles.all(|other| other == role).then_some(role)
}

/// Conservative producer applicability only; this grants no framework role.
pub(crate) fn has_decorated_class(source: &str) -> bool {
    let Some(language) = crate::compression::language::safe_typescript_language() else {
        return false;
    };
    let mut parser = tree_sitter::Parser::new();
    if parser.set_language(&language).is_err() {
        return false;
    }
    let Some(tree) = parser.parse(source, None) else {
        return false;
    };
    let mut pending = vec![tree.root_node()];
    while let Some(node) = pending.pop() {
        if node.kind() == "decorator"
            && node.parent().is_some_and(|parent| {
                matches!(
                    parent.kind(),
                    "class_declaration" | "abstract_class_declaration" | "export_statement"
                )
            })
        {
            return true;
        }
        if matches!(
            node.kind(),
            "class_declaration" | "abstract_class_declaration"
        ) {
            let mut cursor = node.walk();
            if node
                .named_children(&mut cursor)
                .any(|child| child.kind() == "decorator")
            {
                return true;
            }
        }
        let mut cursor = node.walk();
        pending.extend(node.named_children(&mut cursor));
    }
    false
}

pub(crate) fn has_class_role(authority: &TypeScriptAuthority) -> bool {
    authority.package_exports.iter().any(|package| {
        package.name == "@angular/core"
            && api_role(&package.declaration.name).is_some()
            && package
                .version
                .as_ref()
                .is_some_and(|version| !version.is_empty())
            && package.public_export == Resolution::Resolved(package.declaration.clone())
    })
}

/// `start` is owned by the parser's class capture, never found by text search.
pub(crate) fn class_role(
    source: &str,
    raw_class: &str,
    start: usize,
    authority: &TypeScriptAuthority,
) -> Option<ClassRole> {
    if authority.source_hash != crate::cache::content_hash(source.as_bytes())
        || !authority.capture_current
        || authority.disposition.is_some()
        || !authority.witnesses.iter().all(Witness::current)
        || source.get(start..start.checked_add(raw_class.len())?) != Some(raw_class)
    {
        return None;
    }
    use crate::ir::typescript_resolution::declaration_facets::ExpressionValue;
    let end = start.checked_add(raw_class.len())?;
    let owners: Vec<_> = authority
        .declaration_facets
        .iter()
        .filter(|facet| {
            facet.declaration.source == authority.source
                && facet.declaration.start >= start
                && facet.declaration.end == end
        })
        .collect();
    let [owner] = owners.as_slice() else {
        return None;
    };
    let kind = declaration_role(authority, &owner.declaration)?;
    let decorators: Vec<_> = owner
        .decorators
        .iter()
        .filter(|decorator| {
            matches!(&decorator.callee.value, ExpressionValue::Reference(resolution)
            if decorator_role(authority, resolution) == Some(kind))
        })
        .collect();
    let [decorator] = decorators.as_slice() else {
        return None;
    };
    let argument = decorator
        .arguments
        .first()
        .and_then(|argument| source.get(argument.start..argument.end))
        .unwrap_or("");
    let selector = matches!(kind, ClassKind::Component | ClassKind::Directive)
        .then(|| super::decorator_args::parse_object_literal(argument).selector)
        .flatten();
    let pipe_name = (kind == ClassKind::Pipe)
        .then(|| super::decorator_args::parse_pipe_fields(argument).0)
        .flatten();
    Some(ClassRole {
        kind,
        selector,
        pipe_name,
    })
}
