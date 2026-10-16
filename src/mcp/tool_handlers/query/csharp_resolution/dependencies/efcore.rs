//! Exact request-local EF Core entity projection.

use super::super::{
    BaseReference, Catalog, Declaration, TypeKind, TypeShape, find_first_base_shape,
};
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};
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
    pub(in crate::mcp::tool_handlers::query) fn build(catalog: &Catalog) -> Self {
        #[cfg(test)]
        work_metrics::record_ef_catalog_build();
        let configurations = configuration_entities(catalog);
        let mut edges = Vec::new();
        for context in catalog.declarations.iter().filter(|declaration| {
            #[cfg(test)]
            work_metrics::record_ef_context_declaration();
            declaration.kind == TypeKind::Class && is_ef_db_context(catalog, declaration)
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
                #[cfg(test)]
                work_metrics::record_ef_project_root_lookup();
                let Some(project) = common_project(&context.file, &marker.file) else {
                    continue;
                };
                for (configuration, entity) in &configurations {
                    #[cfg(test)]
                    work_metrics::record_ef_assembly_configuration_comparison();
                    #[cfg(test)]
                    work_metrics::record_ef_project_root_lookup();
                    if nearest_project(&configuration.file).as_ref() == Some(&project) {
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

fn is_ef_db_context(catalog: &Catalog, declaration: &Declaration) -> bool {
    catalog
        .references_for_owner(&declaration.file, &declaration.name)
        .any(|reference| {
            #[cfg(test)]
            work_metrics::record_ef_context_reference_comparison();
            reference.shape.name == DB_CONTEXT
                && reference.shape.arity == 0
                && is_ef_framework_type(reference, DB_CONTEXT)
        })
}

fn is_ef_framework_type(reference: &BaseReference, name: &str) -> bool {
    let written = reference.written.trim_start_matches("global::");
    let qualified = format!("Microsoft.EntityFrameworkCore.{name}");
    written == qualified
        || written
            .strip_prefix(&qualified)
            .is_some_and(|suffix| suffix.starts_with('<'))
        || (reference.shape.qualifier.is_empty()
            && reference
                .imports
                .iter()
                .any(|import| import.join(".") == "Microsoft.EntityFrameworkCore"))
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

fn configuration_entities(catalog: &Catalog) -> BTreeMap<DeclarationKey, &Declaration> {
    let mut mappings = BTreeMap::new();
    for configuration in catalog
        .declarations
        .iter()
        .filter(|declaration| declaration.kind == TypeKind::Class)
    {
        #[cfg(test)]
        work_metrics::record_ef_configuration_declaration();
        let Some(reference) = catalog
            .references_for_owner(&configuration.file, &configuration.name)
            .find(|reference| {
                #[cfg(test)]
                work_metrics::record_ef_configuration_reference_comparison();
                is_ef_configuration_interface(reference)
            })
        else {
            continue;
        };
        let Some(written) = first_generic_argument(&reference.written) else {
            continue;
        };
        let Some(entity) = resolve_with_reference(catalog, reference, &written) else {
            continue;
        };
        if entity.kind == TypeKind::Class {
            mappings.insert(declaration_key(configuration), entity);
        }
    }
    mappings
}

fn is_ef_configuration_interface(reference: &BaseReference) -> bool {
    if reference.shape.name != "IEntityTypeConfiguration" || reference.shape.arity != 1 {
        return false;
    }
    is_ef_framework_type(reference, "IEntityTypeConfiguration")
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

fn common_project(left: &str, right: &str) -> Option<PathBuf> {
    let left = nearest_project(left)?;
    (nearest_project(right).as_ref() == Some(&left)).then_some(left)
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
