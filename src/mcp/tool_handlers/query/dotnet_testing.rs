//! Ephemeral declaration-proven .NET TestClass relationships.

use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};
use crate::mcp::McpState;
use crate::workspace::{index::WorkspaceIndex, scope::WorkspaceScope};

pub(super) struct Catalog {
    edges: Vec<SemanticEdge>,
}

impl Catalog {
    pub(super) fn build(
        state: &McpState,
        index: &WorkspaceIndex,
        scope: Option<&WorkspaceScope>,
        declarations: &super::csharp_resolution::Catalog,
    ) -> Self {
        let mut edges = Vec::new();
        for file in index.file_map().keys().filter(|file| {
            file.to_ascii_lowercase().ends_with(".cs")
                && scope.is_none_or(|scope| scope.admits(file))
        }) {
            let Some(source) = state.cached_source_snapshot(file) else {
                continue;
            };
            for candidate in crate::dotnet_meta::testing::test_subject_candidates_in_source(&source)
            {
                let test_declared_here = index
                    .declarations_by_identity("dotnet", "TestClass", &candidate.test_class)
                    .into_iter()
                    .any(|declaration| declaration.file.as_deref() == Some(file.as_str()));
                if !test_declared_here {
                    continue;
                }
                let Some(target_file) =
                    declarations.unique_class_declaration_file(&candidate.subject)
                else {
                    continue;
                };
                let Some(target_name) =
                    super::csharp_resolution::terminal_lookup_name(&candidate.subject)
                else {
                    continue;
                };
                let target_declared = index
                    .declarations_by_identity("builtin", "Class", &target_name)
                    .into_iter()
                    .any(|declaration| declaration.file.as_deref() == Some(target_file));
                if !target_declared {
                    continue;
                }
                let mut subject = EntityRef::new("dotnet", "TestClass", candidate.test_class);
                subject.file = Some(file.clone());
                let mut object = EntityRef::new("builtin", "Class", target_name);
                object.file = Some(target_file.to_string());
                let edge = SemanticEdge {
                    relation: SemanticRelation::Tests,
                    subject,
                    object,
                    layer: "dotnet",
                    call_evidence: None,
                };
                if !edges.iter().any(|existing: &SemanticEdge| {
                    existing.relation == edge.relation
                        && existing.subject == edge.subject
                        && existing.subject.file == edge.subject.file
                        && existing.object == edge.object
                        && existing.object.file == edge.object.file
                }) {
                    edges.push(edge);
                }
            }
        }
        Self { edges }
    }

    pub(super) fn forward(&self, domain: &str, entity_type: &str, name: &str) -> Vec<SemanticEdge> {
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

    pub(super) fn reverse(&self, domain: &str, entity_type: &str, name: &str) -> Vec<SemanticEdge> {
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

pub(super) fn candidate_targets_for_test(
    state: &McpState,
    index: &WorkspaceIndex,
    scope: Option<&WorkspaceScope>,
    test_name: &str,
) -> Vec<String> {
    let mut targets = Vec::new();
    for declaration in index.declarations_by_identity("dotnet", "TestClass", test_name) {
        let Some(file) = declaration.file.as_deref() else {
            continue;
        };
        if scope.is_some_and(|scope| !scope.admits(file)) {
            continue;
        }
        let Some(source) = state.cached_source_snapshot(file) else {
            continue;
        };
        targets.extend(
            crate::dotnet_meta::testing::test_subject_candidates_in_source(&source)
                .into_iter()
                .filter(|candidate| candidate.test_class == test_name)
                .map(|candidate| candidate.subject),
        );
    }
    targets.sort();
    targets.dedup();
    targets
}
