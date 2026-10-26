//! Angular-owned provider registration interpretation.
//!
//! Provider registration is deliberately separate from dependency consumption.
//! This module consumes structured TypeScript syntax and publishes only bindings
//! whose owner, token and strategy endpoint have sufficient source authority.

use crate::angular_meta::decorators::ClassKind;
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
    use super::provider_position::Position;
    [Position::Ordinary, Position::View]
        .into_iter()
        .flat_map(|position| {
            project_at(
                source_id, source, owner_name, owner_kind, authority, position,
            )
        })
        .collect()
}

fn project_at(
    source_id: &str,
    source: &str,
    owner_name: &str,
    owner_kind: ClassKind,
    authority: Option<&crate::ir::typescript_resolution::TypeScriptAuthority>,
    position: super::provider_position::Position,
) -> Vec<SemanticEdge> {
    use super::provider_position::Position;
    if !position.admits(owner_kind) {
        return Vec::new();
    }
    if !matches!(
        owner_kind,
        ClassKind::Component | ClassKind::Module | ClassKind::Directive
    ) {
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
    let retained = authority.and_then(|authority| {
        super::string_token_authority::provider_array_at(authority, owner_name, position)
    });
    // Directive ownership is admitted only through exact retained native role proof.
    if (owner_kind == ClassKind::Directive || position == Position::View)
        && !retained.is_some_and(|(id, _)| {
            authority.is_some_and(|authority| {
                id.source_hash == authority.source_hash
                    && super::class_role_authority::declaration_role(authority, id)
                        == Some(owner_kind)
            })
        })
    {
        return Vec::new();
    }
    let array = retained.and_then(|(_, expression)| {
        tree.root_node()
            .descendant_for_byte_range(expression.start, expression.end)
            .filter(|node| {
                node.kind() == "array"
                    && node.start_byte() == expression.start
                    && node.end_byte() == expression.end
            })
    });
    let native_owner = authority
        .zip(retained)
        .is_some_and(|(authority, (owner, _))| position.owner_type(authority, owner).is_some());
    let entries: Vec<_> = if native_owner {
        let (authority, (_, expression)) = authority
            .zip(retained)
            .expect("proved native provider owner");
        super::provider_composition::entries(authority, expression, true)
            .into_iter()
            .filter_map(|entry| {
                tree.root_node()
                    .descendant_for_byte_range(entry.start, entry.end)
                    .filter(|node| node.start_byte() == entry.start && node.end_byte() == entry.end)
            })
            .collect()
    } else {
        let Some(provider_array) = array.or_else(|| {
            (owner_kind != ClassKind::Directive && position == Position::Ordinary)
                .then(|| provider_array(source_id, source, class))
                .flatten()
        }) else {
            return Vec::new();
        };
        let mut cursor = provider_array.walk();
        provider_array.named_children(&mut cursor).collect()
    };
    let owner_type = match owner_kind {
        ClassKind::Module => "Module",
        ClassKind::Directive => "Directive",
        _ => "Component",
    };
    let owner = EntityRef::new("angular", owner_type, owner_name);
    let composed = native_owner
        && authority.is_some_and(|authority| {
            super::provider_composition::has_composition(authority, owner_name)
        });
    let mut edges = Vec::new();
    for entry in entries {
        if (position == Position::View || composed)
            && !authority.is_some_and(|authority| {
                super::provider_position::registration_token(
                    authority,
                    owner_name,
                    entry.start_byte(),
                    entry.end_byte(),
                    position,
                )
                .is_some()
            })
        {
            continue;
        }
        if pair_value(entry, "useExisting", source.as_bytes()).is_some() {
            if let (Some(authority), Some((id, _))) = (authority, retained)
                && let Some((token, target)) = super::provider_pair_authority::pair_at(
                    authority,
                    owner_name,
                    entry.start_byte(),
                    entry.end_byte(),
                    super::provider_pair_authority::Strategy::Existing,
                    position,
                )
            {
                edges.push(SemanticEdge {
                    relation: position.relation(),
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
            && let (Some(authority), Some((id, array))) = (authority, retained)
            && let Some(token_node) =
                pair_value(entry, "provide", source.as_bytes()).or_else(|| {
                    matches!(entry.kind(), "identifier" | "member_expression").then_some(entry)
                })
            && let Some(expression) = super::string_token_authority::provider_value(
                authority,
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
        // A Directive registration must never recover rejected token proof from
        // the legacy name binder (for example, after duplicate provide fields).
        if (owner_kind == ClassKind::Directive || position == Position::View) && canonical.is_none()
        {
            continue;
        }
        let token_label = canonical.as_ref().map(|(_, token)| token.name.clone());
        if authority.is_some()
            && retained.is_some()
            && canonical.is_none()
            && pair_value(entry, "useValue", source.as_bytes())
                .is_some_and(|node| matches!(node.kind(), "identifier" | "member_expression"))
        {
            continue;
        }
        let Some(binding) = parse_provider(
            source_id,
            source,
            tree.root_node(),
            entry,
            token_label,
            retained.and_then(|(_, array)| authority.map(|authority| (authority, array))),
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
            relation: position.relation(),
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
            super::string_token_authority::provider_field(authority?, array, "useFactory", node.start_byte(), node.end_byte())
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
            if let Some((subject, object)) = super::provider_pair_authority::pair_at(
                authority,
                owner_name,
                entry.start_byte(),
                entry.end_byte(),
                strategy,
                position,
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
    if native_owner && let Some(authority) = authority {
        edges.extend(super::provider_imported::project(
            authority, owner_name, position,
        ));
    }
    edges
}

#[path = "provider_syntax.rs"]
mod syntax;
#[cfg(test)]
pub(crate) use syntax::proved_reference;
pub(crate) use syntax::{decorator_owner, reference_identity};
use syntax::{find_named_class, pair_value, parse_provider, provider_array};

#[cfg(test)]
#[path = "../tests/angular_meta/provider_semantics.rs"]
mod tests;
