//! Existing syntax and compatibility provider interpretation.
use super::{ProviderBinding, ProviderStrategy};
use crate::ir::invocation_evidence::SourceSpan;
pub(super) fn find_named_class<'tree>(
    node: tree_sitter::Node<'tree>,
    name: &str,
    source: &[u8],
) -> Option<tree_sitter::Node<'tree>> {
    if node.kind() == "class_declaration"
        && node
            .child_by_field_name("name")
            .and_then(|value| value.utf8_text(source).ok())
            == Some(name)
    {
        return Some(node);
    }
    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .find_map(|child| find_named_class(child, name, source))
}

pub(crate) fn decorator_owner(class: tree_sitter::Node<'_>) -> tree_sitter::Node<'_> {
    class
        .parent()
        .filter(|parent| parent.kind() == "export_statement")
        .unwrap_or(class)
}

pub(super) fn provider_array<'tree>(
    source_id: &str,
    source: &str,
    class: tree_sitter::Node<'tree>,
) -> Option<tree_sitter::Node<'tree>> {
    let decorator_owner = decorator_owner(class);
    let mut cursor = decorator_owner.walk();
    for decorator in decorator_owner
        .named_children(&mut cursor)
        .filter(|node| node.kind() == "decorator")
    {
        let call = decorator.named_child(0)?;
        if call.kind() != "call_expression" {
            continue;
        }
        let function = call.child_by_field_name("function")?;
        let decorator_name = function.utf8_text(source.as_bytes()).ok()?;
        if !matches!(decorator_name, "Component" | "NgModule")
            || !angular_core_import(source_id, source, function, decorator_name)
        {
            continue;
        }
        let arguments = call.child_by_field_name("arguments")?;
        let object = first_descendant_kind(arguments, "object")?;
        if let Some(value) = pair_value(object, "providers", source.as_bytes())
            && value.kind() == "array"
        {
            return Some(value);
        }
    }
    None
}

fn angular_core_import(
    source_id: &str,
    source: &str,
    reference: tree_sitter::Node<'_>,
    expected_name: &str,
) -> bool {
    let Ok(span) = SourceSpan::new(reference.start_byte(), reference.end_byte()) else {
        return false;
    };
    crate::ir::typescript_value_import_binding::bind_named_value_import(source_id, source, span)
        .is_some_and(|binding| {
            binding.module_specifier == "@angular/core" && binding.imported_name == expected_name
        })
}

pub(super) fn parse_provider(
    source_id: &str,
    source: &str,
    root: tree_sitter::Node<'_>,
    entry: tree_sitter::Node<'_>,
    proved_token_label: Option<String>,
    retained_array: Option<(
        &crate::ir::typescript_resolution::TypeScriptAuthority,
        &crate::ir::typescript_resolution::declaration_facets::ExpressionFacet,
    )>,
) -> Option<ProviderBinding> {
    if entry.kind() == "identifier"
        || (entry.kind() == "member_expression" && proved_token_label.is_some())
    {
        let identity =
            proved_token_label.or_else(|| reference_identity(source_id, source, root, entry))?;
        return Some(ProviderBinding {
            token: identity.clone(),
            strategy: ProviderStrategy::Class,
            endpoint: identity,
        });
    }
    if entry.kind() != "object" {
        return None;
    }
    let token_node = pair_value(entry, "provide", source.as_bytes())?;
    let token =
        proved_token_label.or_else(|| token_identity(source_id, source, root, token_node))?;
    let strategies = [
        ("useClass", ProviderStrategy::Class),
        ("useFactory", ProviderStrategy::Factory),
        ("useValue", ProviderStrategy::Value),
    ];
    let mut selected = strategies.into_iter().filter_map(|(key, strategy)| {
        pair_value(entry, key, source.as_bytes()).map(|node| (strategy, node))
    });
    let (strategy, endpoint_node) = selected.next()?;
    if selected.next().is_some() {
        return None;
    }
    let endpoint = match strategy {
        ProviderStrategy::Class | ProviderStrategy::Factory => {
            if let Some((authority, array)) = retained_array {
                let field = if matches!(strategy, ProviderStrategy::Class) {
                    "useClass"
                } else {
                    "useFactory"
                };
                proved_reference(
                    authority,
                    array,
                    field,
                    SourceSpan::new(endpoint_node.start_byte(), endpoint_node.end_byte()).ok()?,
                )?
                .name
                .clone()
            } else {
                reference_identity(source_id, source, root, endpoint_node)?
            }
        }
        ProviderStrategy::Value => {
            if let Some((authority, array)) = retained_array
                && matches!(endpoint_node.kind(), "identifier" | "member_expression")
            {
                proved_reference(
                    authority,
                    array,
                    "useValue",
                    SourceSpan::new(endpoint_node.start_byte(), endpoint_node.end_byte()).ok()?,
                )?
                .name
                .clone()
            } else {
                value_identity(source_id, source, root, endpoint_node)?
            }
        }
    };
    Some(ProviderBinding {
        token,
        strategy,
        endpoint,
    })
}

/// Retained provider fields and the exact native occurrence must agree.
pub(crate) fn proved_reference<'a>(
    authority: &'a crate::ir::typescript_resolution::TypeScriptAuthority,
    array: &'a crate::ir::typescript_resolution::declaration_facets::ExpressionFacet,
    field: &str,
    span: SourceSpan,
) -> Option<&'a crate::ir::typescript_resolution::DeclarationId> {
    let expression = crate::angular_meta::string_token_authority::provider_field(
        authority,
        array,
        field,
        span.start_byte(),
        span.end_byte(),
    )?;
    let crate::ir::typescript_resolution::declaration_facets::ExpressionValue::Reference(
        crate::ir::typescript_resolution::Resolution::Resolved(id),
    ) = &expression.value
    else {
        return None;
    };
    authority
        .value_reference_at(span)
        .filter(|native| *native == id)
}

fn token_identity(
    source_id: &str,
    source: &str,
    root: tree_sitter::Node<'_>,
    node: tree_sitter::Node<'_>,
) -> Option<String> {
    match node.kind() {
        "identifier" => reference_identity(source_id, source, root, node),
        "string" => node
            .named_child(0)
            .and_then(|fragment| fragment.utf8_text(source.as_bytes()).ok())
            .map(str::to_owned),
        _ => None,
    }
}

fn value_identity(
    source_id: &str,
    source: &str,
    root: tree_sitter::Node<'_>,
    node: tree_sitter::Node<'_>,
) -> Option<String> {
    match node.kind() {
        "identifier" => reference_identity(source_id, source, root, node),
        "string" | "number" | "true" | "false" | "null" => {
            node.utf8_text(source.as_bytes()).ok().map(str::to_owned)
        }
        _ => None,
    }
}

pub(crate) fn reference_identity(
    source_id: &str,
    source: &str,
    root: tree_sitter::Node<'_>,
    reference: tree_sitter::Node<'_>,
) -> Option<String> {
    if reference.kind() != "identifier" {
        return None;
    }
    let written = reference.utf8_text(source.as_bytes()).ok()?;
    let span = SourceSpan::new(reference.start_byte(), reference.end_byte()).ok()?;
    if let Some(binding) =
        crate::ir::typescript_value_import_binding::bind_named_value_import(source_id, source, span)
    {
        return Some(binding.imported_name);
    }
    let mut matches = Vec::new();
    collect_local_value_declarations(root, written, source.as_bytes(), &mut matches);
    let [identity] = matches.as_slice() else {
        return None;
    };
    Some(identity.clone())
}

fn collect_local_value_declarations(
    node: tree_sitter::Node<'_>,
    name: &str,
    source: &[u8],
    output: &mut Vec<String>,
) {
    let candidate = match node.kind() {
        "class_declaration" | "function_declaration" => node.child_by_field_name("name"),
        "variable_declarator" => node.child_by_field_name("name"),
        _ => None,
    };
    if let Some(candidate) = candidate
        && matches!(candidate.kind(), "identifier" | "type_identifier")
        && candidate.utf8_text(source).ok() == Some(name)
    {
        output.push(name.to_owned());
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_local_value_declarations(child, name, source, output);
    }
}

pub(super) fn pair_value<'tree>(
    object: tree_sitter::Node<'tree>,
    expected_key: &str,
    source: &[u8],
) -> Option<tree_sitter::Node<'tree>> {
    let mut cursor = object.walk();
    object
        .named_children(&mut cursor)
        .filter(|node| node.kind() == "pair")
        .find_map(|pair| {
            let key = pair.child_by_field_name("key")?;
            (key.utf8_text(source).ok()? == expected_key)
                .then(|| pair.child_by_field_name("value"))
                .flatten()
        })
}

fn first_descendant_kind<'tree>(
    node: tree_sitter::Node<'tree>,
    kind: &str,
) -> Option<tree_sitter::Node<'tree>> {
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if child.kind() == kind {
            return Some(child);
        }
        if let Some(found) = first_descendant_kind(child, kind) {
            return Some(found);
        }
    }
    None
}
