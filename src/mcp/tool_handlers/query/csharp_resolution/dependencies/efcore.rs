//! Exact request-local EF Core entity projection.

use super::super::{
    BaseReference, Catalog, Declaration, TypeKind, TypeShape, find_first_base_shape,
};
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};
use crate::workspace::index::WorkspaceIndex;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use tree_sitter::Node;

#[cfg(test)]
use crate::mcp::tool_handlers::query::work_metrics;

const DOTNET: &str = "dotnet";
const DB_CONTEXT: &str = "DbContext";
const ENTITY: &str = "Entity";

pub(in crate::mcp::tool_handlers::query) struct EfCoreCatalog {
    edges: Vec<SemanticEdge>,
}

impl EfCoreCatalog {
    pub(in crate::mcp::tool_handlers::query) fn build(
        catalog: &Catalog,
        index: &WorkspaceIndex,
    ) -> Self {
        #[cfg(test)]
        work_metrics::record_ef_catalog_build();
        let source = neutral_source_index(catalog);
        let configurations = configuration_entities(catalog, &source);
        let mut edges = Vec::new();
        let mut project_lookup = ProjectLookup::default();
        for context in catalog.declarations.iter().filter(|declaration| {
            #[cfg(test)]
            work_metrics::record_ef_context_declaration();
            declaration.kind == TypeKind::Class && is_ef_db_context(catalog, index, declaration)
        }) {
            let Some(model) = OnModelCreating::parse(&context.source) else {
                continue;
            };
            let mut entities = BTreeMap::new();
            for written in model.direct_entities {
                #[cfg(test)]
                work_metrics::record_ef_model_occurrence();
                if let Some(entity) = resolve_written_class(catalog, context, &written) {
                    entities.insert(entity_key(entity), entity);
                }
            }
            for written in model.explicit_configurations {
                #[cfg(test)]
                work_metrics::record_ef_model_occurrence();
                let Some(configuration) = resolve_written_class(catalog, context, &written) else {
                    continue;
                };
                if let Some(entity) = configurations.get(&declaration_key(configuration)) {
                    entities.insert(entity_key(entity), *entity);
                }
            }
            for marker in model.assembly_markers {
                #[cfg(test)]
                work_metrics::record_ef_model_occurrence();
                let Some(marker) = resolve_written_class(catalog, context, &marker) else {
                    continue;
                };
                let Some(project) = project_lookup.common_project(&context.file, &marker.file)
                else {
                    continue;
                };
                for (configuration, entity) in &configurations {
                    #[cfg(test)]
                    work_metrics::record_ef_assembly_configuration_comparison();
                    if project_lookup.nearest_project(&configuration.file).as_ref()
                        == Some(&project)
                    {
                        entities.insert(entity_key(entity), *entity);
                    }
                }
            }
            for entity in entities.into_values() {
                edges.push(has_entity_edge(context, entity));
            }
        }
        edges.sort_by(|left, right| {
            (&left.subject.name, &left.object.name, &left.subject.file).cmp(&(
                &right.subject.name,
                &right.object.name,
                &right.subject.file,
            ))
        });
        edges.dedup_by(|left, right| {
            left.relation == right.relation
                && left.subject == right.subject
                && left.object == right.object
                && left.call_evidence == right.call_evidence
        });
        Self { edges }
    }

    pub(in crate::mcp::tool_handlers::query) fn forward(
        &self,
        domain: &str,
        entity_type: &str,
        name: &str,
    ) -> Vec<SemanticEdge> {
        self.edges
            .iter()
            .filter(|edge| {
                edge.subject.domain == domain
                    && edge.subject.entity_type == entity_type
                    && edge.subject.name == name
            })
            .cloned()
            .collect()
    }

    pub(in crate::mcp::tool_handlers::query) fn reverse(
        &self,
        domain: &str,
        entity_type: &str,
        name: &str,
    ) -> Vec<SemanticEdge> {
        self.edges
            .iter()
            .filter(|edge| {
                edge.object.domain == domain
                    && edge.object.entity_type == entity_type
                    && edge.object.name == name
            })
            .cloned()
            .collect()
    }
}

fn is_ef_db_context(catalog: &Catalog, index: &WorkspaceIndex, declaration: &Declaration) -> bool {
    if catalog
        .references_for_owner(&declaration.file, &declaration.name)
        .next()
        .is_none()
    {
        return false;
    }
    #[cfg(test)]
    work_metrics::record_ef_context_reference_comparison();
    index
        .find_entities_by_name(&declaration.name)
        .into_iter()
        .any(|entity| {
            entity.domain == DOTNET
                && entity.entity_type == DB_CONTEXT
                && entity.file.as_deref() == Some(declaration.file.as_str())
        })
}

#[derive(Default)]
struct OnModelCreating {
    direct_entities: BTreeSet<String>,
    explicit_configurations: BTreeSet<String>,
    assembly_markers: BTreeSet<String>,
}

impl OnModelCreating {
    fn parse(source: &str) -> Option<Self> {
        let language = crate::compression::language::safe_csharp_language()?;
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&language).ok()?;
        let tree = parser.parse(source, None)?;
        let mut model = Self::default();
        collect_on_model_creating(tree.root_node(), source.as_bytes(), &mut model);
        Some(model)
    }
}

fn collect_on_model_creating(node: Node<'_>, source: &[u8], model: &mut OnModelCreating) {
    if node.kind() == "method_declaration"
        && node
            .child_by_field_name("name")
            .and_then(|name| name.utf8_text(source).ok())
            == Some("OnModelCreating")
    {
        let Some(builder) = model_builder_parameter(node, source) else {
            return;
        };
        collect_model_invocations(node, source, &builder, model);
        return;
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_on_model_creating(child, source, model);
    }
}

fn model_builder_parameter(node: Node<'_>, source: &[u8]) -> Option<String> {
    let parameters = node.child_by_field_name("parameters")?;
    let mut cursor = parameters.walk();
    parameters
        .named_children(&mut cursor)
        .find_map(|parameter| {
            let type_name = parameter
                .child_by_field_name("type")?
                .utf8_text(source)
                .ok()?;
            if !type_name.trim_end().ends_with("ModelBuilder") {
                return None;
            }
            parameter
                .child_by_field_name("name")?
                .utf8_text(source)
                .ok()
                .map(str::to_string)
        })
}

fn collect_model_invocations(
    node: Node<'_>,
    source: &[u8],
    builder: &str,
    model: &mut OnModelCreating,
) {
    if node.kind() == "invocation_expression"
        && let Ok(text) = node.utf8_text(source)
    {
        let direct = format!("{builder}.Entity<");
        let explicit = format!("{builder}.ApplyConfiguration(");
        let assembly = format!("{builder}.ApplyConfigurationsFromAssembly(");
        if let Some(written) = generic_argument_after(text, &direct) {
            model.direct_entities.insert(written);
        } else if let Some(rest) = text.strip_prefix(&explicit)
            && let Some(written) = constructed_type(rest)
        {
            model.explicit_configurations.insert(written);
        } else if let Some(rest) = text.strip_prefix(&assembly)
            && let Some(written) = local_assembly_marker(rest)
        {
            model.assembly_markers.insert(written);
        }
        return;
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_model_invocations(child, source, builder, model);
    }
}

fn generic_argument_after(text: &str, prefix: &str) -> Option<String> {
    let rest = text.strip_prefix(prefix)?;
    let end = rest.find('>')?;
    let written = rest[..end].trim();
    (!written.is_empty()).then(|| written.to_string())
}

fn constructed_type(rest: &str) -> Option<String> {
    let rest = rest.trim_start();
    let rest = rest.strip_prefix("new ")?;
    let end = rest.find('(')?;
    let written = rest[..end].trim();
    (!written.is_empty()).then(|| written.to_string())
}

fn local_assembly_marker(rest: &str) -> Option<String> {
    let rest = rest.trim_start().strip_prefix("typeof(")?;
    let end = rest.find(')')?;
    let written = rest[..end].trim();
    rest[end + 1..]
        .trim_start()
        .starts_with(".Assembly")
        .then(|| written.to_string())
}

fn configuration_entities<'a>(
    catalog: &'a Catalog,
    source: &crate::ir::csharp_symbols::CSharpSourceSymbolIndex,
) -> BTreeMap<DeclarationKey, &'a Declaration> {
    let mut mappings = BTreeMap::new();
    for configuration in catalog
        .declarations
        .iter()
        .filter(|declaration| declaration.kind == TypeKind::Class)
    {
        #[cfg(test)]
        work_metrics::record_ef_configuration_declaration();
        let Some((reference, entity_written)) = catalog
            .references_for_owner(&configuration.file, &configuration.name)
            .find_map(|reference| {
                #[cfg(test)]
                work_metrics::record_ef_configuration_reference_comparison();
                resolved_ef_configuration_entity(reference, source)
                    .map(|entity| (reference, entity))
            })
        else {
            continue;
        };
        let Some(entity) = resolve_with_reference(catalog, reference, &entity_written) else {
            continue;
        };
        if entity.kind == TypeKind::Class {
            mappings.insert(declaration_key(configuration), entity);
        }
    }
    mappings
}

fn resolved_ef_configuration_entity(
    reference: &BaseReference,
    source: &crate::ir::csharp_symbols::CSharpSourceSymbolIndex,
) -> Option<String> {
    let external = [
        crate::ir::csharp_resolution::CSharpTypeIdentity::known_external_with_arity(
            "Microsoft.EntityFrameworkCore.IEntityTypeConfiguration",
            1,
        ),
    ];
    let aliases = crate::ir::csharp_resolution::CSharpAliasContext {
        directives: reference.aliases.clone(),
    };
    let alias = reference
        .aliases
        .iter()
        .filter(|alias| alias.alias == reference.shape.name)
        .max_by_key(|alias| alias.scope_depth);
    let resolution = if alias.is_some() {
        crate::ir::csharp_resolution::resolve_alias_type_reference(
            &reference.written,
            &aliases,
            &external,
            source,
        )
    } else if reference.shape.qualifier.is_empty() {
        crate::ir::csharp_resolution::resolve_ordinary_import_type_reference(
            &reference.written,
            &crate::ir::csharp_resolution::CSharpOrdinaryImportContext {
                namespace_path: reference.namespace.clone(),
                containing_type_path: reference.nesting.clone(),
                ordinary_imports: reference.imports.clone(),
            },
            &external,
            source,
        )
    } else {
        crate::ir::csharp_resolution::resolve_explicit_type_reference(
            &reference.written,
            &external,
            source,
        )
    };
    let crate::ir::csharp_resolution::CSharpResolution::Resolved { identity, .. } = resolution
    else {
        return None;
    };
    if identity != external[0] {
        return None;
    }
    first_generic_argument(alias.map_or(reference.written.as_str(), |alias| &alias.target))
}

fn neutral_source_index(catalog: &Catalog) -> crate::ir::csharp_symbols::CSharpSourceSymbolIndex {
    crate::ir::csharp_symbols::CSharpSourceSymbolIndex::build(catalog.declarations.iter().map(
        |declaration| crate::ir::csharp_symbols::CSharpDeclarationPart {
            key: crate::ir::csharp_symbols::CSharpSymbolKey {
                namespace_path:
                    declaration.path[..declaration.path.len().saturating_sub(1)].to_vec(),
                containing_type_path: Vec::new(),
                name: declaration.name.clone(),
                generic_arity: declaration.arity,
            },
            kind: match declaration.kind {
                TypeKind::Class => crate::ir::csharp_resolution::CSharpDeclarationKind::Class,
                TypeKind::Interface => {
                    crate::ir::csharp_resolution::CSharpDeclarationKind::Interface
                }
            },
            is_partial: false,
            file: declaration.file.clone(),
            span: crate::ir::csharp_resolution::CSharpSourceSpan {
                start_byte: 0,
                end_byte: declaration.source.len(),
            },
            base_references: Vec::new(),
        },
    ))
}

fn first_generic_argument(written: &str) -> Option<String> {
    let start = written.find('<')? + 1;
    let end = written.rfind('>')?;
    let argument = written[start..end].trim();
    (!argument.is_empty() && !argument.contains(',')).then(|| argument.to_string())
}

fn resolve_written_class<'a>(
    catalog: &'a Catalog,
    owner: &Declaration,
    written: &str,
) -> Option<&'a Declaration> {
    let reference = catalog
        .references_for_owner(&owner.file, &owner.name)
        .find(|_| {
            #[cfg(test)]
            work_metrics::record_ef_owner_reference_comparison();
            true
        })?;
    resolve_with_reference(catalog, reference, written)
}

fn resolve_with_reference<'a>(
    catalog: &'a Catalog,
    context: &BaseReference,
    written: &str,
) -> Option<&'a Declaration> {
    let shape = parse_type_shape(written)?;
    let reference = BaseReference {
        file: context.file.clone(),
        owner: context.owner.clone(),
        namespace: context.namespace.clone(),
        nesting: context.nesting.clone(),
        written: written.to_string(),
        shape,
        imports: context.imports.clone(),
        aliases: context.aliases.clone(),
    };
    catalog.resolve_declaration(&reference)
}

fn parse_type_shape(written: &str) -> Option<TypeShape> {
    #[cfg(test)]
    work_metrics::record_ef_synthetic_type_parse();
    let source = format!("class __CleanCtxProbe : {written} {{}}");
    let language = crate::compression::language::safe_csharp_language()?;
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&language).ok()?;
    let tree = parser.parse(&source, None)?;
    find_first_base_shape(tree.root_node(), source.as_bytes())
}

fn has_entity_edge(context: &Declaration, entity: &Declaration) -> SemanticEdge {
    let mut subject = EntityRef::new(DOTNET, DB_CONTEXT, &context.name);
    subject.file = Some(context.file.clone());
    let mut object = EntityRef::new(DOTNET, ENTITY, &entity.name);
    object.file = Some(entity.file.clone());
    SemanticEdge {
        relation: SemanticRelation::HasEntity,
        subject,
        object,
        layer: DOTNET,
        call_evidence: None,
    }
}

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
struct DeclarationKey {
    file: String,
    path: Vec<String>,
    arity: usize,
}

fn declaration_key(declaration: &Declaration) -> DeclarationKey {
    DeclarationKey {
        file: declaration.file.clone(),
        path: declaration.path.clone(),
        arity: declaration.arity,
    }
}

fn entity_key(declaration: &Declaration) -> (Vec<String>, usize, String) {
    (
        declaration.path.clone(),
        declaration.arity,
        declaration.file.clone(),
    )
}

#[derive(Default)]
struct ProjectLookup {
    by_file: BTreeMap<String, Option<PathBuf>>,
}

impl ProjectLookup {
    fn common_project(&mut self, left: &str, right: &str) -> Option<PathBuf> {
        #[cfg(test)]
        work_metrics::record_ef_project_root_lookup();
        let left_project = self.resolve(left, false)?;
        (self.resolve(right, false).as_ref() == Some(&left_project)).then_some(left_project)
    }

    fn nearest_project(&mut self, file: &str) -> Option<PathBuf> {
        #[cfg(test)]
        work_metrics::record_ef_project_root_lookup();
        self.resolve(file, true)
    }

    fn resolve(&mut self, file: &str, observe_hit: bool) -> Option<PathBuf> {
        if let Some(project) = self.by_file.get(file) {
            #[cfg(test)]
            if observe_hit {
                work_metrics::record_ef_project_root_cache_hit();
            }
            #[cfg(not(test))]
            let _ = observe_hit;
            return project.clone();
        }
        #[cfg(test)]
        work_metrics::record_ef_project_root_filesystem_resolution();
        let project = nearest_project(file);
        self.by_file.insert(file.to_owned(), project.clone());
        project
    }
}

fn nearest_project(file: &str) -> Option<PathBuf> {
    let mut directory = Path::new(file).parent()?;
    loop {
        let mut projects: Vec<_> = std::fs::read_dir(directory)
            .ok()?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| {
                path.extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("csproj"))
            })
            .collect();
        projects.sort();
        if projects.len() == 1 {
            return projects.pop();
        }
        if projects.len() > 1 {
            return None;
        }
        directory = directory.parent()?;
    }
}

#[cfg(test)]
#[path = "../../../../../tests/mcp/ef_project_lookup.rs"]
mod tests_project_lookup;
