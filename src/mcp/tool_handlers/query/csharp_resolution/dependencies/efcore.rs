//! Exact request-local EF Core entity projection.

use super::super::{
    BaseReference, Catalog, Declaration, TypeKind, TypeShape, find_first_base_shape,
};
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};
use crate::workspace::index::WorkspaceIndex;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
mod model;
use model::OnModelCreating;
#[cfg(feature = "csharp")]
mod obligations;
#[cfg(feature = "csharp")]
pub(in crate::mcp::tool_handlers::query) use obligations::{
    Evidence as EfDependencyEvidence, prepare,
};

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
        allowed: Option<&BTreeSet<String>>,
    ) -> Self {
        #[cfg(test)]
        work_metrics::record_ef_catalog_build();
        let sources = source_indexes(catalog, allowed);
        let configurations = configuration_entities(catalog, &sources, allowed);
        let mut edges = Vec::new();
        let mut project_lookup = ProjectLookup::default();
        for context in catalog.declarations.iter().filter(|declaration| {
            #[cfg(test)]
            work_metrics::record_ef_context_declaration();
            declaration.kind == TypeKind::Class
                && allowed.is_none_or(|allowed| allowed.contains(&declaration.file))
                && is_ef_db_context(catalog, index, declaration)
        }) {
            let source = &sources;
            let Some(model) = OnModelCreating::parse(&context.source) else {
                continue;
            };
            let mut entities = BTreeMap::new();
            for written in model.direct_entities {
                #[cfg(test)]
                work_metrics::record_ef_model_occurrence();
                if let Some(entity) = resolve_written_class(catalog, source, context, &written) {
                    entities.insert(entity_key(entity), entity);
                }
            }
            for written in model.explicit_configurations {
                #[cfg(test)]
                work_metrics::record_ef_model_occurrence();
                let Some(configuration) = resolve_written_class(catalog, source, context, &written)
                else {
                    continue;
                };
                if let Some(entity) = configurations.iter().find_map(|(key, entity)| {
                    (key.path == configuration.path && key.arity == configuration.arity)
                        .then_some(entity)
                }) {
                    entities.insert(entity_key(entity), *entity);
                }
            }
            for marker in model.assembly_markers {
                #[cfg(test)]
                work_metrics::record_ef_model_occurrence();
                let Some(marker) = resolve_written_class(catalog, source, context, &marker) else {
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
    #[cfg(test)]
    if catalog
        .references_for_owner(&declaration.file, &declaration.name)
        .next()
        .is_some()
    {
        work_metrics::record_ef_context_reference_comparison();
    }
    let _ = catalog;
    has_ef_role(index, declaration)
}

fn has_ef_role(index: &WorkspaceIndex, declaration: &Declaration) -> bool {
    index
        .find_entities_by_name(&declaration.name)
        .into_iter()
        .any(|entity| {
            entity.domain == DOTNET
                && entity.entity_type == DB_CONTEXT
                && entity.file.as_deref() == Some(declaration.file.as_str())
        })
}

fn configuration_entities<'a>(
    catalog: &'a Catalog,
    sources: &SourceIndexes,
    allowed: Option<&BTreeSet<String>>,
) -> BTreeMap<DeclarationKey, &'a Declaration> {
    let mut mappings = BTreeMap::new();
    for configuration in catalog.declarations.iter().filter(|declaration| {
        declaration.kind == TypeKind::Class
            && allowed.is_none_or(|allowed| allowed.contains(&declaration.file))
    }) {
        let source = sources;
        let Some(key) = symbol_key(catalog, configuration) else {
            continue;
        };
        if !matches!(
            source.get(&key),
            Some(crate::ir::csharp_symbols::CSharpSymbolComposition::Resolved(_))
        ) {
            continue;
        }
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
        let Some(entity) = resolve_with_reference(catalog, source, reference, &entity_written)
        else {
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
    let alias = reference
        .aliases
        .iter()
        .filter(|alias| alias.alias == reference.shape.name)
        .max_by_key(|alias| alias.scope_depth);
    let resolution = resolve_reference(reference, source, &external);
    let crate::ir::csharp_resolution::CSharpResolution::Resolved { identity, .. } = resolution
    else {
        return None;
    };
    if identity != external[0] {
        return None;
    }
    first_generic_argument(alias.map_or(reference.written.as_str(), |alias| &alias.target))
}

type SourceIndexes = crate::ir::csharp_symbols::CSharpSourceSymbolIndex;

fn source_indexes(catalog: &Catalog, allowed: Option<&BTreeSet<String>>) -> SourceIndexes {
    let mut parts = catalog.source_parts.clone();
    // Canonical compilation units own class parts. The existing neutral syntax
    // catalog also captures interfaces, which must remain source-shadow evidence.
    for declaration in catalog
        .declarations
        .iter()
        .filter(|declaration| declaration.kind == TypeKind::Interface)
    {
        if let Some(key) = symbol_key(catalog, declaration) {
            parts.push(crate::ir::csharp_symbols::CSharpDeclarationPart {
                key,
                kind: crate::ir::csharp_resolution::CSharpDeclarationKind::Interface,
                is_partial: declaration.source.contains("partial interface"),
                file: declaration.file.clone(),
                span: crate::ir::csharp_resolution::CSharpSourceSpan {
                    start_byte: 0,
                    end_byte: declaration.source.len(),
                },
                base_references: Vec::new(),
            });
        }
    }
    crate::ir::csharp_symbols::CSharpSourceSymbolIndex::build(
        parts
            .into_iter()
            .filter(|part| allowed.is_none_or(|allowed| allowed.contains(&part.file))),
    )
}

fn symbol_key(
    catalog: &Catalog,
    declaration: &Declaration,
) -> Option<crate::ir::csharp_symbols::CSharpSymbolKey> {
    let scope = catalog.declaration_scopes.get(&(
        declaration.file.clone(),
        declaration.path.clone(),
        declaration.arity,
    ))?;
    Some(crate::ir::csharp_symbols::CSharpSymbolKey {
        namespace_path: scope.namespace.clone(),
        containing_type_path: scope.nesting.clone(),
        name: declaration.name.clone(),
        generic_arity: declaration.arity,
    })
}

fn first_generic_argument(written: &str) -> Option<String> {
    let start = written.find('<')? + 1;
    let end = written.rfind('>')?;
    let argument = written[start..end].trim();
    (!argument.is_empty() && !argument.contains(',')).then(|| argument.to_string())
}

fn resolve_written_class<'a>(
    catalog: &'a Catalog,
    source: &crate::ir::csharp_symbols::CSharpSourceSymbolIndex,
    owner: &Declaration,
    written: &str,
) -> Option<&'a Declaration> {
    #[cfg(test)]
    work_metrics::record_ef_owner_reference_comparison();
    let reference =
        catalog
            .declaration_scopes
            .get(&(owner.file.clone(), owner.path.clone(), owner.arity))?;
    resolve_with_reference(catalog, source, reference, written)
}

fn resolve_with_reference<'a>(
    catalog: &'a Catalog,
    source: &crate::ir::csharp_symbols::CSharpSourceSymbolIndex,
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
    declaration_for_resolution(catalog, source, resolve_reference(&reference, source, &[]))
}

fn declaration_for_resolution<'a>(
    catalog: &'a Catalog,
    source: &crate::ir::csharp_symbols::CSharpSourceSymbolIndex,
    resolution: crate::ir::csharp_resolution::CSharpResolution<
        crate::ir::csharp_resolution::CSharpTypeIdentity,
    >,
) -> Option<&'a Declaration> {
    use crate::ir::csharp_resolution::CSharpResolution;
    use crate::ir::csharp_symbols::{CSharpSymbolComposition, CSharpSymbolKey};
    let CSharpResolution::Resolved { identity, .. } = resolution else {
        return None;
    };
    let key = CSharpSymbolKey {
        namespace_path: identity.namespace_path,
        containing_type_path: identity.containing_type_path,
        name: identity.terminal_name,
        generic_arity: identity.generic_arity,
    };
    let Some(CSharpSymbolComposition::Resolved(symbol)) = source.get(&key) else {
        return None;
    };
    let mut path = key.namespace_path.clone();
    path.extend(key.containing_type_path);
    path.push(key.name);
    // Canonical partial composition chooses one stable physical representative.
    let part = symbol.parts.first()?;
    catalog.declarations.iter().find(|declaration| {
        declaration.file == part.file
            && declaration.path == path
            && declaration.arity == key.generic_arity
    })
}

fn resolve_reference(
    reference: &BaseReference,
    source: &crate::ir::csharp_symbols::CSharpSourceSymbolIndex,
    external: &[crate::ir::csharp_resolution::CSharpTypeIdentity],
) -> crate::ir::csharp_resolution::CSharpResolution<crate::ir::csharp_resolution::CSharpTypeIdentity>
{
    use crate::ir::csharp_resolution::*;
    let head = reference
        .written
        .split(['.', ':', '<'])
        .next()
        .unwrap_or_default()
        .trim();
    if reference.aliases.iter().any(|alias| alias.alias == head) {
        resolve_alias_type_reference(
            &reference.written,
            &CSharpAliasContext {
                directives: reference.aliases.clone(),
            },
            external,
            source,
        )
    } else if reference.written.contains('.') || reference.written.starts_with("global::") {
        resolve_explicit_type_reference(&reference.written, external, source)
    } else {
        resolve_ordinary_import_type_reference(
            &reference.written,
            &CSharpOrdinaryImportContext {
                namespace_path: reference.namespace.clone(),
                containing_type_path: reference.nesting.clone(),
                ordinary_imports: reference.imports.clone(),
            },
            external,
            source,
        )
    }
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
