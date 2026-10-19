//! Structured, query-time C# type resolution for neutral base-list facts.
//!
//! Durable publication intentionally keeps the written TypeRef unchanged. This
//! catalog parses the exact source snapshots already owned by the session and
//! refines only references whose namespace, nesting, terminal name, and generic
//! arity select one indexed Class or Interface declaration.

use crate::layers::meta::semantic::SemanticEdge;
use crate::mcp::McpState;
use crate::workspace::{index::WorkspaceIndex, scope::WorkspaceScope};
use std::collections::HashSet;
use tree_sitter::Node;
mod declaration_candidates;
mod dependencies;
pub(super) mod direct_aspnet;
mod reference_candidates;
mod using_aliases;
pub(super) use dependencies::{DependencyGraph, EfCoreCatalog};
pub(super) use reference_candidates::terminal_lookup_name;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(super) enum TypeKind {
    Class,
    Interface,
}
impl TypeKind {
    pub(super) fn entity_type(self) -> &'static str {
        match self {
            Self::Class => "Class",
            Self::Interface => "Interface",
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct Declaration {
    kind: TypeKind,
    file: String,
    path: Vec<String>,
    name: String,
    arity: usize,
    source: String,
}
#[derive(Clone, Debug)]
struct BaseReference {
    file: String,
    owner: String,
    namespace: Vec<String>,
    nesting: Vec<String>,
    written: String,
    shape: TypeShape,
    imports: Vec<Vec<String>>,
    aliases: Vec<crate::ir::csharp_resolution::CSharpAliasDirective>,
}
#[derive(Clone, Debug)]
struct TypeShape {
    qualifier: Vec<String>,
    name: String,
    arity: usize,
}
#[derive(Default)]
pub(super) struct Catalog {
    declarations: Vec<Declaration>,
    references: Vec<BaseReference>,
    declaration_candidates: declaration_candidates::DeclarationCandidateIndex,
    reference_candidates: reference_candidates::ReferenceCandidateIndex,
}

#[derive(Clone, Debug)]
pub(super) struct ResolvedTarget {
    pub(super) kind: TypeKind,
    pub(super) name: String,
}

pub(super) struct AspNetProjection {
    pub(super) file: String,
    pub(super) edges: Vec<SemanticEdge>,
}

impl Catalog {
    pub(super) fn build(
        state: &McpState,
        index: &WorkspaceIndex,
        scope: Option<&WorkspaceScope>,
    ) -> Self {
        #[cfg(test)]
        super::work_metrics::record_csharp_catalog_build();
        let mut catalog = Self::default();
        for file in index.file_map().keys().filter(|file| {
            file.to_ascii_lowercase().ends_with(".cs")
                && scope.is_none_or(|scope| scope.admits(file))
        }) {
            #[cfg(test)]
            super::work_metrics::record_csharp_file_considered();
            let Some(source) = state.cached_source_snapshot(file) else {
                continue;
            };
            catalog.add_file(file, &source);
        }
        catalog.declarations.sort_by(|left, right| {
            (&left.path, left.arity, left.kind.entity_type(), &left.file).cmp(&(
                &right.path,
                right.arity,
                right.kind.entity_type(),
                &right.file,
            ))
        });
        catalog.declarations.dedup();
        catalog.declaration_candidates =
            declaration_candidates::DeclarationCandidateIndex::build(&catalog.declarations);
        catalog.reference_candidates =
            reference_candidates::ReferenceCandidateIndex::build(&catalog.references);
        catalog
    }

    pub(super) fn recognizes_edge(&self, edge: &SemanticEdge) -> bool {
        let file = edge.subject.file.as_deref().or(edge.object.file.as_deref());
        file.is_some_and(|file| {
            self.references.iter().any(|reference| {
                reference.file == file
                    && reference.owner == edge.subject.name
                    && reference.written == edge.object.name
            })
        })
    }
    pub(super) fn resolve_edge(&self, edge: &SemanticEdge) -> Option<ResolvedTarget> {
        let file = edge
            .subject
            .file
            .as_deref()
            .or(edge.object.file.as_deref())?;
        let reference = self.references.iter().find(|reference| {
            reference.file == file
                && reference.owner == edge.subject.name
                && reference.written == edge.object.name
        })?;
        self.resolve(reference)
    }
    pub(super) fn written_references_to(&self, kind: TypeKind, name: &str) -> Vec<String> {
        let mut written: Vec<_> = self
            .references
            .iter()
            .filter_map(|reference| {
                self.resolve(reference)
                    .filter(|target| target.kind == kind && target.name == name)
                    .map(|_| reference.written.clone())
            })
            .collect();
        written.sort();
        written.dedup();
        written
    }

    pub(super) fn unique_class_declaration_file(&self, written: &str) -> Option<&str> {
        let terminal = terminal_lookup_name(written)?;
        let mut matches = self.declarations.iter().filter(|declaration| {
            declaration.kind == TypeKind::Class && declaration.name == terminal
        });
        let declaration = matches.next()?;
        matches
            .next()
            .is_none()
            .then_some(declaration.file.as_str())
    }

    pub(super) fn aspnet_base_candidate(&self, name: &str) -> Option<String> {
        let declaration = self.unique_named_class(name)?;
        self.base_reference(declaration)
            .map(|reference| reference.shape.name.clone())
    }

    pub(super) fn aspnet_projection(
        &self,
        name: &str,
        fidelity: crate::compression::Fidelity,
    ) -> Option<AspNetProjection> {
        let original = self.unique_named_class(name)?;
        if original.source.contains("[ApiController]") || original.source.contains("[Controller]") {
            return direct_aspnet::projection(original, fidelity);
        }
        let mut current = original;
        let mut visited = HashSet::new();
        loop {
            if !visited.insert((current.file.clone(), current.path.clone(), current.arity)) {
                return None;
            }
            let reference = self.base_reference(current)?;
            if is_qualified_aspnet_framework_base(reference) {
                break;
            }
            if let Some(declaration) = self.resolve_declaration(reference) {
                if declaration.kind != TypeKind::Class {
                    return None;
                }
                current = declaration;
                continue;
            }
            if is_imported_aspnet_framework_base(reference) {
                break;
            }
            return None;
        }
        let mut analysis =
            crate::dotnet_meta::aspnet::analyze_authorized_controller(&original.source, fidelity)?;
        for edge in &mut analysis.semantic_edges {
            edge.subject.file = Some(original.file.clone());
            edge.object.file = Some(original.file.clone());
        }
        Some(AspNetProjection {
            file: original.file.clone(),
            edges: analysis.semantic_edges,
        })
    }

    pub(super) fn has_aspnet_controller_authority(&self, name: &str) -> bool {
        let Some(declaration) = self.unique_named_class(name) else {
            return false;
        };
        declaration.source.contains("[ApiController]")
            || declaration.source.contains("[Controller]")
            || self
                .aspnet_projection(name, crate::compression::Fidelity::High)
                .is_some()
    }

    fn unique_named_class(&self, name: &str) -> Option<&Declaration> {
        let mut matches = self
            .declarations
            .iter()
            .filter(|declaration| declaration.kind == TypeKind::Class && declaration.name == name);
        let declaration = matches.next()?;
        matches.next().is_none().then_some(declaration)
    }

    fn base_reference(&self, declaration: &Declaration) -> Option<&BaseReference> {
        let owner_parent = &declaration.path[..declaration.path.len().saturating_sub(1)];
        self.references.iter().find(|reference| {
            let mut reference_parent = reference.namespace.clone();
            reference_parent.extend(reference.nesting.iter().cloned());
            reference.file == declaration.file
                && reference.owner == declaration.name
                && owner_parent == reference_parent.as_slice()
        })
    }

    fn add_file(&mut self, file: &str, source: &str) {
        let Some(language) = crate::compression::language::safe_csharp_language() else {
            return;
        };
        let mut parser = tree_sitter::Parser::new();
        if parser.set_language(&language).is_err() {
            return;
        }
        let Some(tree) = parser.parse(source, None) else {
            return;
        };
        #[cfg(test)]
        super::work_metrics::record_csharp_file_parsed();
        visit_container(
            tree.root_node(),
            source.as_bytes(),
            &TraversalScope::new(file),
            self,
        );
    }

    fn resolve(&self, reference: &BaseReference) -> Option<ResolvedTarget> {
        let mut paths = candidate_paths(reference);
        paths.dedup();

        for path in paths {
            let matches: HashSet<_> = self
                .declarations
                .iter()
                .filter(|declaration| {
                    declaration.path == path && declaration.arity == reference.shape.arity
                })
                .map(|declaration| (declaration.kind, declaration.name.clone()))
                .collect();
            if matches.len() == 1 {
                let (kind, name) = matches.into_iter().next()?;
                return Some(ResolvedTarget { kind, name });
            }
            if !matches.is_empty() {
                return None;
            }
        }
        None
    }
}

fn candidate_paths(reference: &BaseReference) -> Vec<Vec<String>> {
    let mut paths = Vec::new();
    if reference.shape.qualifier.is_empty() {
        for depth in (0..=reference.nesting.len()).rev() {
            let mut path = reference.namespace.clone();
            path.extend(reference.nesting[..depth].iter().cloned());
            path.push(reference.shape.name.clone());
            paths.push(path);
        }
        paths.push(vec![reference.shape.name.clone()]);
    } else {
        let mut relative = reference.namespace.clone();
        relative.extend(reference.shape.qualifier.iter().cloned());
        relative.push(reference.shape.name.clone());
        paths.push(relative);

        let mut absolute = reference.shape.qualifier.clone();
        absolute.push(reference.shape.name.clone());
        paths.push(absolute);
    }
    paths
}

fn is_qualified_aspnet_framework_base(reference: &BaseReference) -> bool {
    if !matches!(
        reference.shape.name.as_str(),
        "ControllerBase" | "Controller"
    ) {
        return false;
    }
    let written = reference.written.trim_start_matches("global::");
    written == format!("Microsoft.AspNetCore.Mvc.{}", reference.shape.name)
}

fn is_imported_aspnet_framework_base(reference: &BaseReference) -> bool {
    matches!(
        reference.shape.name.as_str(),
        "ControllerBase" | "Controller"
    ) && reference.shape.qualifier.is_empty()
        && reference
            .imports
            .iter()
            .any(|path| path.join(".") == "Microsoft.AspNetCore.Mvc")
}

fn direct_using_namespaces(node: Node<'_>, source: &[u8]) -> Vec<Vec<String>> {
    let mut imports = Vec::new();
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_using_namespace(child, source, &mut imports);
    }
    imports.sort();
    imports.dedup();
    imports
}

fn collect_using_namespace(node: Node<'_>, source: &[u8], imports: &mut Vec<Vec<String>>) {
    if node.kind() == "using_directive" {
        let text = node.utf8_text(source).unwrap_or_default();
        if node.child_by_field_name("name").is_none()
            && !text.contains("using static ")
            && let Some(name) = node.named_child(0)
        {
            let segments = name_segments(name, source);
            if !segments.is_empty() {
                imports.push(segments);
            }
        }
    }
}

#[derive(Clone)]
struct TraversalScope {
    file: String,
    namespace: Vec<String>,
    nesting: Vec<String>,
    imports: Vec<Vec<String>>,
    aliases: Vec<crate::ir::csharp_resolution::CSharpAliasDirective>,
}

impl TraversalScope {
    fn new(file: &str) -> Self {
        Self {
            file: file.to_string(),
            namespace: Vec::new(),
            nesting: Vec::new(),
            imports: Vec::new(),
            aliases: Vec::new(),
        }
    }
}

fn visit_container(
    node: Node<'_>,
    source: &[u8],
    scope: &TraversalScope,
    catalog: &mut Catalog,
) {
    let mut scoped = scope.clone();
    scoped.imports.extend(direct_using_namespaces(node, source));
    scoped.imports.sort();
    scoped.imports.dedup();
    scoped.aliases.extend(using_aliases::collect_direct(
        node,
        source,
        &scope.file,
        scope.namespace.len(),
    ));
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        match child.kind() {
            "file_scoped_namespace_declaration" => {
                if let Some(name) = child.child_by_field_name("name") {
                    scoped.namespace.extend(name_segments(name, source));
                }
            }
            "namespace_declaration" => {
                let mut nested_scope = scoped.clone();
                if let Some(name) = child.child_by_field_name("name") {
                    nested_scope.namespace.extend(name_segments(name, source));
                }
                if let Some(body) = child.child_by_field_name("body") {
                    visit_container(body, source, &nested_scope, catalog);
                }
            }
            "class_declaration" | "interface_declaration" => {
                visit_type(child, source, &scoped, catalog);
            }
            _ => visit_container(child, source, &scoped, catalog),
        }
    }
}

fn visit_type(
    node: Node<'_>,
    source: &[u8],
    scope: &TraversalScope,
    catalog: &mut Catalog,
) {
    let Some(name_node) = node.child_by_field_name("name") else {
        return;
    };
    let Ok(name) = name_node.utf8_text(source) else {
        return;
    };
    let name = name.to_string();
    let kind = if node.kind() == "class_declaration" {
        TypeKind::Class
    } else {
        TypeKind::Interface
    };
    let arity = named_child(node, "type_parameter_list")
        .map(|parameters| named_child_count(parameters, "type_parameter"))
        .unwrap_or(0);
    let mut path = scope.namespace.clone();
    path.extend(scope.nesting.iter().cloned());
    path.push(name.clone());
    catalog.declarations.push(Declaration {
        kind,
        file: scope.file.clone(),
        path,
        name: name.clone(),
        arity,
        source: node.utf8_text(source).unwrap_or_default().to_string(),
    });

    if kind == TypeKind::Class
        && let Some(base_list) = named_child(node, "base_list")
    {
        let mut cursor = base_list.walk();
        for child in base_list.named_children(&mut cursor) {
            let type_node = if child.kind() == "primary_constructor_base_type" {
                child.child_by_field_name("type")
            } else if child.kind() == "argument_list" {
                None
            } else {
                Some(child)
            };
            let Some(type_node) = type_node else { continue };
            let Some(shape) = type_shape(type_node, source) else {
                continue;
            };
            let Ok(written) = type_node.utf8_text(source) else {
                continue;
            };
            catalog.references.push(BaseReference {
                file: scope.file.clone(),
                owner: name.clone(),
                namespace: scope.namespace.clone(),
                nesting: scope.nesting.clone(),
                written: written.trim().to_string(),
                shape,
                imports: scope.imports.clone(),
                aliases: scope.aliases.clone(),
            });
        }
    }

    let context = dependencies::ConstructorContext {
        file: &scope.file,
        namespace: &scope.namespace,
        nesting: &scope.nesting,
        imports: &scope.imports,
        aliases: &scope.aliases,
        owner: &name,
    };
    dependencies::collect_constructor_references(node, source, &context, catalog);

    if let Some(body) = node.child_by_field_name("body") {
        let mut nested_scope = scope.clone();
        nested_scope.nesting.push(name);
        visit_container(body, source, &nested_scope, catalog);
    }
}

fn find_first_base_shape(node: Node<'_>, source: &[u8]) -> Option<TypeShape> {
    if node.kind() == "base_list" {
        let mut cursor = node.walk();
        return node
            .named_children(&mut cursor)
            .find(|child| child.kind() != "argument_list")
            .and_then(|child| type_shape(child, source));
    }
    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .find_map(|child| find_first_base_shape(child, source))
}

fn type_shape(node: Node<'_>, source: &[u8]) -> Option<TypeShape> {
    match node.kind() {
        "identifier" => Some(TypeShape {
            qualifier: Vec::new(),
            name: node.utf8_text(source).ok()?.to_string(),
            arity: 0,
        }),
        "generic_name" => {
            let name = named_child(node, "identifier")?.utf8_text(source).ok()?;
            let arguments = named_child(node, "type_argument_list")?;
            Some(TypeShape {
                qualifier: Vec::new(),
                name: name.to_string(),
                arity: arguments.named_child_count(),
            })
        }
        "qualified_name" | "alias_qualified_name" => {
            let qualifier = node.child_by_field_name("qualifier")?;
            let terminal = node.child_by_field_name("name")?;
            let mut shape = type_shape(terminal, source)?;
            shape.qualifier = name_segments(qualifier, source);
            Some(shape)
        }
        _ => {
            let mut cursor = node.walk();
            let mut children = node.named_children(&mut cursor);
            let only = children.next()?;
            children
                .next()
                .is_none()
                .then(|| type_shape(only, source))?
        }
    }
}

fn name_segments(node: Node<'_>, source: &[u8]) -> Vec<String> {
    match node.kind() {
        "identifier" => node
            .utf8_text(source)
            .ok()
            .map(|name| vec![name.to_string()])
            .unwrap_or_default(),
        "qualified_name" | "alias_qualified_name" => {
            let mut parts = node
                .child_by_field_name("qualifier")
                .map(|qualifier| name_segments(qualifier, source))
                .unwrap_or_default();
            if let Some(name) = node.child_by_field_name("name") {
                parts.extend(name_segments(name, source));
            }
            parts
        }
        "generic_name" => named_child(node, "identifier")
            .map(|name| name_segments(name, source))
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

fn named_child<'tree>(node: Node<'tree>, kind: &str) -> Option<Node<'tree>> {
    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .find(|child| child.kind() == kind)
}

fn named_child_count(node: Node<'_>, kind: &str) -> usize {
    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .filter(|child| child.kind() == kind)
        .count()
}

#[cfg(test)]
#[path = "../../../tests/mcp/workspace_query_csharp_reference_candidates.rs"]
mod tests_reference_candidates;
