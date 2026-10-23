//! Source-owned constructor occurrences with independent type/value meanings.
use super::{DeclarationId, Namespace, Resolution, Resolver};
use crate::ir::invocation_evidence::SourceSpan;
use std::{collections::BTreeSet, path::Path};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct ConstructorParameter {
    pub owner: DeclarationId,
    pub constructor: (usize, usize),
    pub parameter: (usize, usize),
    pub annotation: Option<(usize, usize)>,
    pub shape: String,
    pub path: Vec<String>,
    pub type_resolution: Option<Resolution>,
    pub value_resolution: Option<Resolution>,
    /// Closed bare absence, not a failed lookup or a runtime declaration.
    pub unbound: bool,
    pub decorators: Vec<(usize, usize)>,
    #[serde(default)]
    pub decorator_facets: Vec<super::declaration_facets::DecoratorFacet>,
}

pub(super) fn capture(
    resolver: &mut Resolver,
    file: &Path,
    source: &str,
    declarations: &[DeclarationId],
) -> Vec<ConstructorParameter> {
    resolver.in_source_project(file, |resolver| {
        let Some(tree) = resolver.source_tree(file) else {
            return Vec::new();
        };
        let mut result = Vec::new();
        for owner in declarations.iter().filter(|id| {
            matches!(
                id.kind.as_str(),
                "class_declaration" | "abstract_class_declaration"
            )
        }) {
            let Some(class) = tree
                .root_node()
                .named_descendant_for_byte_range(owner.start, owner.end)
                .filter(|node| node.start_byte() == owner.start && node.end_byte() == owner.end)
            else {
                continue;
            };
            let Some(body) = class.child_by_field_name("body") else {
                continue;
            };
            let mut cursor = body.walk();
            for constructor in body.named_children(&mut cursor).filter(|node| {
                node.kind() == "method_definition"
                    && node.child_by_field_name("name").is_some_and(|name| {
                        name.utf8_text(source.as_bytes()).ok() == Some("constructor")
                    })
            }) {
                let Some(parameters) = constructor.child_by_field_name("parameters") else {
                    continue;
                };
                let mut cursor = parameters.walk();
                for parameter in parameters.named_children(&mut cursor).filter(|node| {
                    matches!(node.kind(), "required_parameter" | "optional_parameter")
                }) {
                    let mut cursor = parameter.walk();
                    let decorators = parameter
                        .named_children(&mut cursor)
                        .filter(|node| node.kind() == "decorator")
                        .map(|node| (node.start_byte(), node.end_byte()))
                        .collect();
                    let annotation = parameter
                        .child_by_field_name("type")
                        .and_then(|annotation| {
                            let mut cursor = annotation.walk();
                            annotation
                                .named_children(&mut cursor)
                                .find(|node| node.kind() != "comment")
                        });
                    let mut facet = ConstructorParameter {
                        owner: owner.clone(),
                        constructor: (constructor.start_byte(), constructor.end_byte()),
                        parameter: (parameter.start_byte(), parameter.end_byte()),
                        annotation: annotation.map(|node| (node.start_byte(), node.end_byte())),
                        shape: annotation.map_or("untyped", |node| node.kind()).into(),
                        path: Vec::new(),
                        type_resolution: None,
                        value_resolution: None,
                        unbound: false,
                        decorators,
                        decorator_facets: super::parameter_decorators::capture(resolver, file, source, parameter),
                    };
                    if let Some(annotation) = annotation {
                        let target = if annotation.kind() == "generic_type" {
                            annotation.child_by_field_name("name").unwrap_or(annotation)
                        } else {
                            annotation
                        };
                        let mut parts = Vec::new();
                        if super::type_references::type_path(target, source, &mut parts)
                            && let Some((name, members)) = parts.split_first()
                        {
                            facet.path = parts.iter().map(|part| (*part).to_string()).collect();
                            if let Ok(span) =
                                SourceSpan::new(target.start_byte(), target.end_byte())
                            {
                                facet.type_resolution = resolver.type_reference(file, source, span);
                            }
                            let mut root_name = target;
                            while matches!(root_name.kind(), "nested_type_identifier" | "nested_identifier") {
                                let mut cursor = root_name.walk();
                                let Some(first) = root_name.named_children(&mut cursor)
                                    .find(|node| node.kind() != "comment") else { break; };
                                root_name = first;
                            }
                            let lexical = file.to_str().zip(SourceSpan::new(root_name.start_byte(), root_name.end_byte()).ok())
                                .map(|(file, span)| crate::ir::typescript_identifier_binding::lookup_value_at_type_site_in_tree(
                                    file, source, span, tree.root_node()));
                            facet.value_resolution = Some(match &lexical {
                                Some(Ok(None)) if members.is_empty() => resolver.local(file, name, Namespace::Value),
                                Some(Ok(None)) => resolver.namespace_chain(file, name, members, Namespace::Value),
                                Some(Ok(Some(binding))) => {
                                    let span = binding.declaration_span();
                                    let node = tree.root_node().named_descendant_for_byte_range(span.start_byte(), span.end_byte());
                                    match (resolver.host.normalized(file), node) {
                                        (Some(source_file), Some(node)) => {
                                            let declaration = node.parent().filter(|parent| matches!(parent.kind(),
                                                "required_parameter" | "optional_parameter" | "variable_declarator")).unwrap_or(node);
                                            Resolution::Resolved(DeclarationId { source: source_file,
                                                source_hash: crate::cache::content_hash(source.as_bytes()),
                                                start: declaration.start_byte(), end: declaration.end_byte(),
                                                name: (*name).into(), kind: declaration.kind().into() })
                                        }
                                        _ => Resolution::Unresolved("Constructor value binding unavailable".into()),
                                    }
                                }
                                _ => Resolution::Unresolved("Constructor value binding is ambiguous or unsupported".into()),
                            });
                            // A resolved generic/lexical type cannot gain value authority
                            // from a same-spelled outer value: the consumer requires ID agreement.
                            if matches!(lexical, Some(Ok(None))) && members.is_empty()
                                && matches!(facet.type_resolution, Some(Resolution::Unresolved(_)))
                                && resolver
                                    .unit(file)
                                    .is_some_and(|unit| !unit.locals.contains_key(*name))
                            {
                                let (values, value_closed) = resolver.global_bindings(
                                    file,
                                    name,
                                    Namespace::Value,
                                    &mut BTreeSet::new(),
                                );
                                let (types, type_closed) = resolver.global_bindings(
                                    file,
                                    name,
                                    Namespace::Type,
                                    &mut BTreeSet::new(),
                                );
                                facet.unbound = value_closed
                                    && type_closed
                                    && values.is_empty()
                                    && types.is_empty();
                            }
                        }
                    }
                    result.push(facet);
                }
            }
        }
        result
    })
}

impl ConstructorParameter {
    /// Independent meanings must identify the same runtime class. A proved
    /// same-source interface augmentation may add type-only contributors.
    pub(crate) fn runtime_class(&self) -> Option<&DeclarationId> {
        let types = contributors(self.type_resolution.as_ref()?)?;
        let values = contributors(self.value_resolution.as_ref()?)?;
        let class = unique_class(values)?;
        if unique_class(types)? != class {
            return None;
        }
        if types == values {
            return Some(class);
        }
        let type_group = types.iter().cloned().collect();
        if !super::declaration_groups::mergeable(&type_group)
            || !values.iter().all(|id| types.contains(id))
            || !types
                .iter()
                .filter(|id| !values.contains(id))
                .all(|id| id.kind == "interface_declaration")
        {
            return None;
        }
        Some(class)
    }
}

fn contributors(resolution: &Resolution) -> Option<&[DeclarationId]> {
    match resolution {
        Resolution::Resolved(id) => Some(std::slice::from_ref(id)),
        Resolution::Merged(ids) => Some(ids.as_slice()),
        _ => None,
    }
}

fn unique_class(ids: &[DeclarationId]) -> Option<&DeclarationId> {
    let mut classes = ids.iter().filter(|id| {
        matches!(
            id.kind.as_str(),
            "class_declaration" | "abstract_class_declaration"
        )
    });
    let class = classes.next()?;
    classes.next().is_none().then_some(class)
}
