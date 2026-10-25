//! Angular-owned provider registration interpretation.
//!
//! Provider registration is deliberately separate from dependency consumption.
//! This module consumes structured TypeScript syntax and publishes only bindings
//! whose owner, token and strategy endpoint have sufficient source authority.

use crate::angular_meta::decorators::ClassKind;
use crate::ir::invocation_evidence::SourceSpan;
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProviderStrategy {
    Class,
    Factory,
    Value,
}

#[derive(Debug, Clone)]
struct ProviderBinding {
    token: String,
    strategy: ProviderStrategy,
    endpoint: String,
}

#[cfg(test)]
pub(crate) fn project(
    source_id: &str,
    source: &str,
    owner_name: &str,
    owner_kind: ClassKind,
) -> Vec<SemanticEdge> {
    project_with_authority(source_id, source, owner_name, owner_kind, None)
}

pub(crate) fn project_with_authority(
    source_id: &str,
    source: &str,
    owner_name: &str,
    owner_kind: ClassKind,
    authority: Option<&crate::ir::typescript_resolution::TypeScriptAuthority>,
) -> Vec<SemanticEdge> {
    if !matches!(owner_kind, ClassKind::Component | ClassKind::Module) {
        return Vec::new();
    }
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
    let Some(class) = find_named_class(tree.root_node(), owner_name, source.as_bytes()) else {
        return Vec::new();
    };
    let retained = authority
        .and_then(|authority| super::string_token_authority::provider_array(authority, owner_name));
    let array = retained.and_then(|(_, expression)| {
        tree.root_node()
            .descendant_for_byte_range(expression.start, expression.end)
            .filter(|node| {
                node.kind() == "array"
                    && node.start_byte() == expression.start
                    && node.end_byte() == expression.end
            })
    });
    let Some(provider_array) = array.or_else(|| provider_array(source_id, source, class)) else {
        return Vec::new();
    };
    let owner_type = if owner_kind == ClassKind::Module {
        "Module"
    } else {
        "Component"
    };
    let owner = EntityRef::new("angular", owner_type, owner_name);
    let mut edges = Vec::new();
    let mut cursor = provider_array.walk();
    for entry in provider_array.named_children(&mut cursor) {
        if pair_value(entry, "useExisting", source.as_bytes()).is_some() {
            if let (Some(authority), Some((id, _))) = (authority, retained)
                && let Some((token, target)) = super::provider_pair_authority::pair(
                    authority,
                    owner_name,
                    entry.start_byte(),
                    entry.end_byte(),
                    super::provider_pair_authority::Strategy::Existing,
                )
            {
                edges.push(SemanticEdge {
                    relation: SemanticRelation::RegistersProvider,
                    subject: super::string_token_authority::owner(id, owner_type),
                    object: token.clone(),
                    layer: "angular",
                    call_evidence: None,
                });
                edges.push(SemanticEdge {
                    relation: SemanticRelation::ProviderUsesExisting,
                    subject: token,
                    object: target,
                    layer: "angular",
                    call_evidence: None,
                });
            }
            continue;
        }
        let mut canonical = None;
        if let Some(authority) = authority
            && let Some(token_node) = pair_value(entry, "provide", source.as_bytes())
            && matches!(
                token_node.kind(),
                "string" | "identifier" | "member_expression"
            )
        {
            canonical = retained.and_then(|(id, array)| {
                super::string_token_authority::provider_literal(
                    authority,
                    array,
                    token_node.start_byte(),
                    token_node.end_byte(),
                )
                .map(|(expression, units)| {
                    (
                        super::string_token_authority::owner(id, owner_type),
                        super::string_token_authority::token(
                            id,
                            expression,
                            &units,
                            "ProviderToken",
                        ),
                    )
                })
            });
            if canonical.is_none() && token_node.kind() == "string" {
                continue;
            }
        }
        if canonical.is_none()
            && let (Some(_), Some((id, array))) = (authority, retained)
            && let Some(token_node) =
                pair_value(entry, "provide", source.as_bytes()).or_else(|| {
                    matches!(entry.kind(), "identifier" | "member_expression").then_some(entry)
                })
            && let Some(expression) = super::string_token_authority::provider_value(
                array,
                token_node.start_byte(),
                token_node.end_byte(),
            )
            && let crate::ir::typescript_resolution::declaration_facets::ExpressionValue::Reference(
                crate::ir::typescript_resolution::Resolution::Resolved(token_id),
            ) = &expression.value
            && (entry.kind() == "object" || token_id.kind == "class_declaration")
        {
            let mut token = EntityRef::new("angular", "ProviderToken", &token_id.name)
                .with_file(token_id.source.to_string_lossy().into_owned());
            token.canonical = Some(
                crate::layers::meta::canonical_endpoint::CanonicalEndpoint::Declaration {
                    declaration: token_id.into(),
                    site: None,
                },
            );
            canonical = Some((super::string_token_authority::owner(id, owner_type), token));
        }
        let token_label = canonical.as_ref().map(|(_, token)| token.name.clone());
        let Some(binding) = parse_provider(
            source_id,
            source,
            tree.root_node(),
            entry,
            token_label,
            retained.map(|(_, array)| array),
        ) else {
            continue;
        };
        let (registration_owner, token) = canonical.unwrap_or_else(|| {
            (
                owner.clone(),
                EntityRef::new("angular", "ProviderToken", &binding.token),
            )
        });
        edges.push(SemanticEdge {
            relation: SemanticRelation::RegistersProvider,
            subject: registration_owner,
            object: token.clone(),
            layer: "angular",
            call_evidence: None,
        });
        let (relation, endpoint_type) = match binding.strategy {
            ProviderStrategy::Class => (SemanticRelation::ProviderUsesClass, "ProviderClass"),
            ProviderStrategy::Factory => (SemanticRelation::ProviderUsesFactory, "ProviderFactory"),
            ProviderStrategy::Value => (SemanticRelation::ProviderUsesValue, "ProviderValue"),
        };
        let factory_declaration = retained.and_then(|(_, array)| {
            let node = pair_value(entry, "useFactory", source.as_bytes())?;
            super::string_token_authority::provider_field(array, "useFactory", node.start_byte(), node.end_byte())
        }).is_some_and(|expression| matches!(&expression.value,
            crate::ir::typescript_resolution::declaration_facets::ExpressionValue::Reference(
                crate::ir::typescript_resolution::Resolution::Resolved(id)) if id.kind == "function_declaration"));
        if (matches!(binding.strategy, ProviderStrategy::Class)
            || (matches!(binding.strategy, ProviderStrategy::Factory) && factory_declaration))
            && let Some(authority) = authority
            && retained.is_some()
        {
            let strategy = if matches!(binding.strategy, ProviderStrategy::Class) {
                super::provider_pair_authority::Strategy::Class
            } else {
                super::provider_pair_authority::Strategy::Factory
            };
            if let Some((subject, object)) = super::provider_pair_authority::pair(
                authority,
                owner_name,
                entry.start_byte(),
                entry.end_byte(),
                strategy,
            ) {
                edges.push(SemanticEdge {
                    relation,
                    subject,
                    object,
                    layer: "angular",
                    call_evidence: None,
                });
            }
            continue;
        }
        // Unmigrated strategy forms retain their noncanonical representation;
        // token-value equality does not certify a strategy endpoint or scope.
        let mut strategy_token = token;
        strategy_token.canonical = None;
        edges.push(SemanticEdge {
            relation,
            subject: strategy_token,
            object: EntityRef::new("angular", endpoint_type, binding.endpoint),
            layer: "angular",
            call_evidence: None,
        });
    }
    edges
}

fn find_named_class<'tree>(
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

fn provider_array<'tree>(
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

fn parse_provider(
    source_id: &str,
    source: &str,
    root: tree_sitter::Node<'_>,
    entry: tree_sitter::Node<'_>,
    proved_token_label: Option<String>,
    retained_array: Option<&crate::ir::typescript_resolution::declaration_facets::ExpressionFacet>,
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
            if let Some(array) = retained_array {
                let field = if matches!(strategy, ProviderStrategy::Class) {
                    "useClass"
                } else {
                    "useFactory"
                };
                let expression = super::string_token_authority::provider_field(
                    array,
                    field,
                    endpoint_node.start_byte(),
                    endpoint_node.end_byte(),
                )?;
                let crate::ir::typescript_resolution::declaration_facets::ExpressionValue::Reference(
                    crate::ir::typescript_resolution::Resolution::Resolved(id),
                ) = &expression.value else {
                    return None;
                };
                id.name.clone()
            } else {
                reference_identity(source_id, source, root, endpoint_node)?
            }
        }
        ProviderStrategy::Value => value_identity(source_id, source, root, endpoint_node)?,
    };
    Some(ProviderBinding {
        token,
        strategy,
        endpoint,
    })
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

fn pair_value<'tree>(
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

#[cfg(test)]
#[path = "../tests/angular_meta/provider_semantics.rs"]
mod tests;
