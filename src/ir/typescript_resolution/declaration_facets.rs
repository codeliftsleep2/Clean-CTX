//! Demanded syntax/reference facets, acquired by the existing snapshot resolver.
use super::{DeclarationId, Resolution, Resolver};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct ExpressionFacet {
    pub start: usize,
    pub end: usize,
    pub value: ExpressionValue,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) enum ExpressionValue {
    Reference(Resolution),
    Array(Vec<ExpressionFacet>),
    Object(Vec<(String, ExpressionFacet)>),
    Spread(Box<ExpressionFacet>),
    Call(Box<ExpressionFacet>, Vec<ExpressionFacet>),
    Member(Box<ExpressionFacet>, String),
    Literal,
    /// Exact UTF-16 code units; containing facet and declaration retain the
    /// immutable source/hash/occurrence. No framework meaning is implied.
    StringValue(Vec<u16>),
    Boolean(bool),
    Unsupported(String),
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct DecoratorFacet {
    pub start: usize,
    pub end: usize,
    pub callee: ExpressionFacet,
    pub arguments: Vec<ExpressionFacet>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct DeclarationFacet {
    pub declaration: DeclarationId,
    pub decorators: Vec<DecoratorFacet>,
    pub initializer: Option<ExpressionFacet>,
}

struct Demand<'a> {
    resolver: &'a mut Resolver,
    visited: BTreeSet<DeclarationId>,
    facets: BTreeMap<DeclarationId, DeclarationFacet>,
    packages: Vec<super::package_evidence::PackageExportEvidence>,
    fields: BTreeSet<String>,
}

pub(super) fn capture(
    resolver: &mut Resolver,
    request: &Path,
    declarations: &[DeclarationId],
    fields: &[&str],
    parameters: &[super::constructor_parameters::ConstructorParameter],
) -> (
    Vec<DeclarationFacet>,
    Vec<super::package_evidence::PackageExportEvidence>,
) {
    resolver.in_source_project(request, |resolver| {
        let mut demand = Demand {
            resolver,
            visited: BTreeSet::new(),
            facets: BTreeMap::new(),
            packages: Vec::new(),
            fields: fields.iter().map(|field| (*field).to_string()).collect(),
        };
        // Source-owned decorated classes are the demand roots. Dependencies are
        // visited only through their decorator arguments and array references.
        for id in declarations.iter().filter(|id| {
            matches!(
                id.kind.as_str(),
                "class_declaration" | "abstract_class_declaration"
            )
        }) {
            demand.declaration(id);
        }
        // Directly annotated source-owned callable declarations are demanded
        // owners for the established functional consumers. Retain declaration
        // proof without evaluating function bodies or visiting dependency owners.
        if let Some(tree) = demand.resolver.source_tree(request) {
            for id in declarations
                .iter()
                .filter(|id| id.kind == "variable_declarator")
            {
                if exact(tree.root_node(), id).is_some_and(|node| {
                    node.child_by_field_name("type").is_some()
                        && node.child_by_field_name("value").is_some_and(|value| {
                            matches!(value.kind(), "arrow_function" | "function_expression")
                        })
                }) {
                    demand.declaration(id);
                }
            }
        }
        // Only references demanded by retained parameter decorators are roots;
        // this does not enumerate unrelated program initializers.
        for argument in parameters
            .iter()
            .flat_map(|parameter| &parameter.decorator_facets)
            .flat_map(|decorator| &decorator.arguments)
        {
            if let ExpressionValue::Reference(Resolution::Resolved(id)) = &argument.value {
                demand.declaration(id);
            }
        }
        (demand.facets.into_values().collect(), demand.packages)
    })
}

impl Demand<'_> {
    fn declaration(&mut self, id: &DeclarationId) {
        if !matches!(
            id.kind.as_str(),
            "class_declaration" | "abstract_class_declaration" | "variable_declarator"
        ) || !self.visited.insert(id.clone())
        {
            return;
        }
        let Some(source) = self.resolver.host.read(&id.source) else {
            return;
        };
        if crate::cache::content_hash(source.as_bytes()) != id.source_hash {
            return;
        }
        let Some(tree) = self.resolver.source_tree(&id.source) else {
            return;
        };
        let Some(node) = exact(tree.root_node(), id) else {
            return;
        };
        let mut decorators = Vec::new();
        let mut cursor = node.walk();
        let mut nodes: Vec<_> = node
            .named_children(&mut cursor)
            .filter(|child| child.kind() == "decorator")
            .collect();
        if let Some(parent) = node
            .parent()
            .filter(|parent| parent.kind() == "export_statement")
        {
            let mut cursor = parent.walk();
            nodes.extend(
                parent
                    .named_children(&mut cursor)
                    .filter(|child| child.kind() == "decorator"),
            );
        }
        nodes.sort_by_key(|node| node.start_byte());
        for decorator in nodes {
            let mut cursor = decorator.walk();
            let Some(call) = decorator
                .named_children(&mut cursor)
                .find(|node| node.kind() == "call_expression")
            else {
                continue;
            };
            let Some(callee) = call.child_by_field_name("function") else {
                continue;
            };
            let callee = self.reference(callee, &id.source, &source, false);
            let arguments = call
                .child_by_field_name("arguments")
                .map(|args| {
                    let mut cursor = args.walk();
                    args.named_children(&mut cursor)
                        .filter(|node| node.kind() != "comment")
                        .map(|node| self.expression(node, &id.source, &source))
                        .collect()
                })
                .unwrap_or_default();
            decorators.push(DecoratorFacet {
                start: decorator.start_byte(),
                end: decorator.end_byte(),
                callee,
                arguments,
            });
        }
        // Only const initializers can supply immutable composition structure.
        let initializer = (node.kind() == "variable_declarator"
            && node.parent().is_some_and(|parent| {
                let mut cursor = parent.walk();
                parent.kind() == "lexical_declaration"
                    && parent
                        .children(&mut cursor)
                        .any(|child| child.kind() == "const")
            }))
        .then(|| {
            node.child_by_field_name("value")
                .map(|value| self.expression(value, &id.source, &source))
        })
        .flatten();
        self.facets.insert(
            id.clone(),
            DeclarationFacet {
                declaration: id.clone(),
                decorators,
                initializer,
            },
        );
    }

    fn reference(
        &mut self,
        node: tree_sitter::Node<'_>,
        file: &Path,
        source: &str,
        expand: bool,
    ) -> ExpressionFacet {
        let resolution =
            crate::ir::invocation_evidence::SourceSpan::new(node.start_byte(), node.end_byte())
                .ok()
                .and_then(|span| self.resolver.value_reference(file, source, span))
                .unwrap_or_else(|| {
                    Resolution::Unresolved("No exact runtime reference authority".into())
                });
        self.packages.extend(super::package_evidence::capture(
            self.resolver,
            file,
            &[(node.start_byte(), node.end_byte(), resolution.clone())],
        ));
        if expand {
            match &resolution {
                Resolution::Resolved(id) => self.declaration(id),
                Resolution::Merged(ids) => {
                    for id in ids {
                        self.declaration(id);
                    }
                }
                _ => {}
            }
        }
        ExpressionFacet {
            start: node.start_byte(),
            end: node.end_byte(),
            value: ExpressionValue::Reference(resolution),
        }
    }

    fn expression(
        &mut self,
        node: tree_sitter::Node<'_>,
        file: &Path,
        source: &str,
    ) -> ExpressionFacet {
        let mut cursor = node.walk();
        let value = match node.kind() {
            "identifier" | "shorthand_property_identifier" => {
                return self.reference(node, file, source, true);
            }
            "member_expression" => {
                // A qualified namespace reference gets its native endpoint first.
                let reference = self.reference(node, file, source, true);
                if matches!(
                    &reference.value,
                    ExpressionValue::Reference(Resolution::Resolved(_) | Resolution::Merged(_))
                ) {
                    return reference;
                }
                match (
                    node.child_by_field_name("object"),
                    node.child_by_field_name("property"),
                ) {
                    (Some(object), Some(property)) => ExpressionValue::Member(
                        Box::new(self.expression(object, file, source)),
                        property
                            .utf8_text(source.as_bytes())
                            .unwrap_or_default()
                            .into(),
                    ),
                    _ => ExpressionValue::Unsupported(node.kind().into()),
                }
            }
            "array" => ExpressionValue::Array(
                node.named_children(&mut cursor)
                    .filter(|node| node.kind() != "comment")
                    .map(|node| self.expression(node, file, source))
                    .collect(),
            ),
            "object" => {
                let mut fields = Vec::new();
                for property in node
                    .named_children(&mut cursor)
                    .filter(|node| node.kind() != "comment")
                {
                    if property.kind() == "pair" {
                        if let (Some(key), Some(value)) = (
                            property.child_by_field_name("key"),
                            property.child_by_field_name("value"),
                        ) {
                            if key.kind() == "computed_property_name" {
                                fields.push((String::new(), self.expression(value, file, source)));
                                continue;
                            }
                            let raw = key.utf8_text(source.as_bytes()).unwrap_or_default();
                            let key = if key.kind() == "string" {
                                super::string_value::decode(raw).unwrap_or_default()
                            } else {
                                raw.into()
                            };
                            if self.fields.contains(&key) {
                                fields.push((key, self.expression(value, file, source)));
                            }
                        }
                    } else if property.kind() == "shorthand_property_identifier" {
                        let key = property.utf8_text(source.as_bytes()).unwrap_or_default();
                        if self.fields.contains(key) {
                            fields.push((key.into(), self.expression(property, file, source)));
                        }
                    } else {
                        fields.push((String::new(), self.expression(property, file, source)));
                    }
                }
                ExpressionValue::Object(fields)
            }
            "spread_element" => node
                .named_child(0)
                .map(|child| {
                    ExpressionValue::Spread(Box::new(self.expression(child, file, source)))
                })
                .unwrap_or_else(|| ExpressionValue::Unsupported(node.kind().into())),
            "call_expression" => {
                match (
                    node.child_by_field_name("function"),
                    node.child_by_field_name("arguments"),
                ) {
                    (Some(function), Some(args)) => {
                        let function = self.expression(function, file, source);
                        let mut cursor = args.walk();
                        let args = args
                            .named_children(&mut cursor)
                            .filter(|node| node.kind() != "comment")
                            .map(|node| self.expression(node, file, source))
                            .collect();
                        ExpressionValue::Call(Box::new(function), args)
                    }
                    _ => ExpressionValue::Unsupported(node.kind().into()),
                }
            }
            "parenthesized_expression" | "as_expression" | "satisfies_expression" => {
                if let Some(child) = node.named_child(0) {
                    return self.expression(child, file, source);
                }
                ExpressionValue::Unsupported(node.kind().into())
            }
            "true" => ExpressionValue::Boolean(true),
            "false" => ExpressionValue::Boolean(false),
            "string" if !node.has_error() => node
                .utf8_text(source.as_bytes())
                .ok()
                .and_then(super::string_literal::decode)
                .map(ExpressionValue::StringValue)
                .unwrap_or_else(|| ExpressionValue::Unsupported("unproved string literal".into())),
            "number" | "null" => ExpressionValue::Literal,
            _ => ExpressionValue::Unsupported(node.kind().into()),
        };
        ExpressionFacet {
            start: node.start_byte(),
            end: node.end_byte(),
            value,
        }
    }
}

fn exact<'a>(node: tree_sitter::Node<'a>, id: &DeclarationId) -> Option<tree_sitter::Node<'a>> {
    if node.start_byte() == id.start && node.end_byte() == id.end && node.kind() == id.kind {
        return Some(node);
    }
    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .filter(|child| child.start_byte() <= id.start && child.end_byte() >= id.end)
        .find_map(|child| exact(child, id))
}
