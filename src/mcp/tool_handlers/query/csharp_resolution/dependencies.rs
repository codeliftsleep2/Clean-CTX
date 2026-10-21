//! Request-local exact C# dependency adjacency.

use super::{BaseReference, Catalog, Declaration, TypeKind, type_shape};
use crate::layers::meta::semantic::SemanticRelation;
use crate::workspace::{index::WorkspaceIndex, scope::WorkspaceScope};
use serde::Serialize;
use std::collections::{BTreeSet, HashMap};
use tree_sitter::Node;

mod efcore;
pub(in crate::mcp::tool_handlers::query) use efcore::EfCoreCatalog;
#[cfg(feature = "csharp")]
pub(in crate::mcp::tool_handlers::query) use efcore::EfDependencyEvidence;
#[cfg(feature = "csharp")]
pub(in crate::mcp::tool_handlers::query) use efcore::prepare as prepare_ef_dependencies;

const CLASS: &str = "Class";
const DOMAIN: &str = "builtin";

#[derive(Clone)]
struct DependencyNode {
    name: String,
    file: String,
}

#[derive(Clone)]
struct DependencyArc {
    target: usize,
    relation: SemanticRelation,
    asserting_file: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(in crate::mcp::tool_handlers::query) struct DependencyEntity {
    domain: &'static str,
    entity_type: &'static str,
    name: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(in crate::mcp::tool_handlers::query) struct DependencyStep {
    subject: DependencyEntity,
    relation: SemanticRelation,
    object: DependencyEntity,
    asserting_file: String,
}

pub(in crate::mcp::tool_handlers::query) struct DependencyGraph {
    nodes: Vec<DependencyNode>,
    adjacency: Vec<Vec<DependencyArc>>,
}

pub(super) struct ConstructorContext<'a> {
    pub(super) file: &'a str,
    pub(super) namespace: &'a [String],
    pub(super) nesting: &'a [String],
    pub(super) imports: &'a [Vec<String>],
    pub(super) aliases: &'a [crate::ir::csharp_resolution::CSharpAliasDirective],
    pub(super) owner: &'a str,
}

impl DependencyGraph {
    pub(in crate::mcp::tool_handlers::query) fn build(
        catalog: &Catalog,
        index: &WorkspaceIndex,
        scope: Option<&WorkspaceScope>,
    ) -> Self {
        let declarations: Vec<&Declaration> = catalog
            .declarations
            .iter()
            .filter(|declaration| declaration.kind == TypeKind::Class)
            .collect();
        let nodes: Vec<DependencyNode> = declarations
            .iter()
            .map(|declaration| DependencyNode {
                name: declaration.name.clone(),
                file: declaration.file.clone(),
            })
            .collect();
        let target_positions: HashMap<*const Declaration, usize> = declarations
            .iter()
            .enumerate()
            .map(|(position, declaration)| {
                #[cfg(test)]
                super::super::work_metrics::record_dependency_graph_target_index_entry();
                (*declaration as *const Declaration, position)
            })
            .collect();
        let mut adjacency = vec![Vec::new(); declarations.len()];
        for (source, declaration) in declarations.iter().enumerate() {
            let edges = match scope {
                Some(scope) => index.forward_edges_by_identity_in_scope(
                    DOMAIN,
                    CLASS,
                    &declaration.name,
                    scope,
                ),
                None => index.forward_edges_by_identity(DOMAIN, CLASS, &declaration.name),
            };
            for edge in edges.into_iter().filter(|edge| {
                edge.subject.file.as_deref() == Some(declaration.file.as_str())
                    && matches!(
                        edge.relation,
                        SemanticRelation::Extends
                            | SemanticRelation::HasBaseType
                            | SemanticRelation::HasConstructorParameterType
                    )
            }) {
                #[cfg(test)]
                super::super::work_metrics::record_dependency_graph_edge();
                let Some(reference) = catalog
                    .references_for_owner(&declaration.file, &declaration.name)
                    .find(|reference| {
                        #[cfg(test)]
                        super::super::work_metrics::record_dependency_graph_reference_comparison();
                        reference.written == edge.object.name
                    })
                else {
                    continue;
                };
                let Some(target) = catalog.resolve_declaration(reference) else {
                    continue;
                };
                if target.kind != TypeKind::Class {
                    continue;
                }
                #[cfg(test)]
                super::super::work_metrics::record_dependency_graph_target_index_lookup();
                let Some(&target) = target_positions.get(&(target as *const Declaration)) else {
                    continue;
                };
                adjacency[source].push(DependencyArc {
                    target,
                    relation: edge.relation,
                    asserting_file: declaration.file.clone(),
                });
            }
            adjacency[source].sort_by(|left, right| {
                (
                    &nodes[left.target].name,
                    &nodes[left.target].file,
                    format!("{:?}", left.relation),
                )
                    .cmp(&(
                        &nodes[right.target].name,
                        &nodes[right.target].file,
                        format!("{:?}", right.relation),
                    ))
            });
            adjacency[source].dedup_by(|left, right| {
                left.target == right.target && left.relation == right.relation
            });
        }
        Self { nodes, adjacency }
    }

    pub(in crate::mcp::tool_handlers::query) fn transitive_dependencies(
        &self,
        name: &str,
        depth: i32,
    ) -> Option<Vec<(String, String, String)>> {
        let start = self.unique_node(name)?;
        let reached = crate::compression::graph_utils::transitive_dependencies(
            start,
            depth.max(0),
            self.nodes.len(),
            |node| {
                self.adjacency[node]
                    .iter()
                    .map(|edge| edge.target)
                    .collect()
            },
        );
        let mut seen = BTreeSet::new();
        Some(
            reached
                .into_iter()
                .filter_map(|node| {
                    let name = self.nodes[node].name.clone();
                    seen.insert(name.clone()).then_some((
                        DOMAIN.to_string(),
                        CLASS.to_string(),
                        name,
                    ))
                })
                .collect(),
        )
    }

    pub(in crate::mcp::tool_handlers::query) fn cycle_witness(
        &self,
    ) -> Option<Vec<DependencyStep>> {
        let mut color = vec![0_u8; self.nodes.len()];
        let mut parents: Vec<Option<(usize, DependencyArc)>> = vec![None; self.nodes.len()];
        for start in 0..self.nodes.len() {
            if color[start] != 0 {
                continue;
            }
            color[start] = 1;
            let mut stack = vec![(start, 0_usize)];
            while let Some((node, next_edge)) = stack.last_mut() {
                if *next_edge == self.adjacency[*node].len() {
                    color[*node] = 2;
                    stack.pop();
                    continue;
                }
                let edge = self.adjacency[*node][*next_edge].clone();
                *next_edge += 1;
                match color[edge.target] {
                    0 => {
                        parents[edge.target] = Some((*node, edge.clone()));
                        color[edge.target] = 1;
                        stack.push((edge.target, 0));
                    }
                    1 => return Some(self.closed_path(*node, edge, &parents)),
                    _ => {}
                }
            }
        }
        None
    }

    fn unique_node(&self, name: &str) -> Option<usize> {
        let mut matches = self
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| node.name == name)
            .map(|(index, _)| index);
        let node = matches.next()?;
        matches.next().is_none().then_some(node)
    }

    fn closed_path(
        &self,
        from: usize,
        closing: DependencyArc,
        parents: &[Option<(usize, DependencyArc)>],
    ) -> Vec<DependencyStep> {
        let mut arcs = vec![(from, closing.clone())];
        let mut cursor = from;
        while cursor != closing.target {
            let (parent, edge) = parents[cursor]
                .clone()
                .expect("a gray descendant has a parent edge");
            arcs.push((parent, edge));
            cursor = parent;
        }
        arcs.reverse();
        arcs.into_iter()
            .map(|(subject, edge)| DependencyStep {
                subject: self.entity(subject),
                relation: edge.relation,
                object: self.entity(edge.target),
                asserting_file: edge.asserting_file,
            })
            .collect()
    }

    fn entity(&self, node: usize) -> DependencyEntity {
        DependencyEntity {
            domain: DOMAIN,
            entity_type: CLASS,
            name: self.nodes[node].name.clone(),
        }
    }
}

pub(super) fn collect_constructor_references(
    class: Node<'_>,
    source: &[u8],
    context: &ConstructorContext<'_>,
    catalog: &mut Catalog,
) {
    let Some(body) = class.child_by_field_name("body") else {
        return;
    };
    let mut body_cursor = body.walk();
    for constructor in body
        .named_children(&mut body_cursor)
        .filter(|child| child.kind() == "constructor_declaration")
    {
        let Some(parameters) = constructor.child_by_field_name("parameters") else {
            continue;
        };
        let mut parameter_cursor = parameters.walk();
        for parameter in parameters.named_children(&mut parameter_cursor) {
            let Some(type_node) = parameter.child_by_field_name("type") else {
                continue;
            };
            let (Some(shape), Ok(written)) =
                (type_shape(type_node, source), type_node.utf8_text(source))
            else {
                continue;
            };
            catalog.references.push(BaseReference {
                file: context.file.to_string(),
                owner: context.owner.to_string(),
                namespace: context.namespace.to_vec(),
                nesting: context.nesting.to_vec(),
                written: written.trim().to_string(),
                shape,
                imports: context.imports.to_vec(),
                aliases: context.aliases.to_vec(),
            });
        }
    }
}
