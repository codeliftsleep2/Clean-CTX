//! Workspace-owned refinement of bounded direct-relative NgRx imports.

use super::edges::{EdgeKey, StoredEdge};
use super::{WorkspaceIndex, entity_key};
use crate::layers::meta::semantic::SemanticEdge;
use std::path::{Path, PathBuf};

impl WorkspaceIndex {
    pub(super) fn rebuild_ngrx_import_refinements(&mut self) {
        for key in std::mem::take(&mut self.ngrx_refined_edges) {
            self.edge_set.remove(&key);
            self.edge_count = self.edge_count.saturating_sub(1);
            if let Some(keys) = self.file_edges.get_mut(&key.asserting_file) {
                keys.retain(|candidate| candidate != &key);
            }
            self.drop_occurrence(&key);
        }

        let candidates: Vec<_> = self
            .ngrx_refinement_candidates
            .iter()
            .flat_map(|(file, edges)| edges.iter().cloned().map(|edge| (file.clone(), edge)))
            .collect();
        for (asserting_file, mut edge) in candidates {
            let imported_entity_is_subject =
                edge.relation == crate::layers::meta::semantic::SemanticRelation::TriggersReducer;
            let imported_entity = if imported_entity_is_subject {
                &edge.subject
            } else {
                &edge.object
            };
            let Some(module) = imported_entity.file.as_deref() else {
                continue;
            };
            let Some(target) = resolve_typescript_module(&asserting_file, module) else {
                continue;
            };
            let declarations = self.declarations_by_identity(
                imported_entity.domain,
                imported_entity.entity_type,
                &imported_entity.name,
            );
            if declarations
                .iter()
                .filter(|declaration| declaration.file.as_deref() == Some(target.as_str()))
                .count()
                != 1
                || !exports_const(&target, &imported_entity.name)
            {
                continue;
            }
            edge.subject.file = Some(asserting_file.clone());
            if imported_entity_is_subject {
                edge.subject.file = Some(target);
                edge.object
                    .file
                    .get_or_insert_with(|| asserting_file.clone());
            } else {
                edge.object.file = Some(target);
            }
            edge.layer = "ngrx";
            self.insert_refined_edge(&asserting_file, edge);
        }
    }

    fn insert_refined_edge(&mut self, asserting_file: &str, edge: SemanticEdge) {
        let subject = entity_key(&edge.subject);
        let object = entity_key(&edge.object);
        let key = EdgeKey::new(asserting_file, &edge);
        if !self.edge_set.insert(key.clone()) {
            return;
        }
        self.edge_count += 1;
        self.file_edges
            .entry(asserting_file.to_string())
            .or_default()
            .push(key.clone());
        self.forward
            .entry(subject)
            .or_default()
            .push(StoredEdge::new(asserting_file, edge.clone()));
        self.reverse
            .entry(object)
            .or_default()
            .push(StoredEdge::new(asserting_file, edge));
        self.ngrx_refined_edges.push(key);
    }
}

fn resolve_typescript_module(asserting_file: &str, module: &str) -> Option<String> {
    if !(module.starts_with("./") || module.starts_with("../")) {
        return None;
    }
    let parent = Path::new(asserting_file).parent()?;
    let mut written = parent.to_path_buf();
    for component in module.split('/') {
        match component {
            "." | "" => {}
            ".." => {
                if !written.pop() {
                    return None;
                }
            }
            part => written.push(part),
        }
    }
    let candidate = if written.extension().and_then(|suffix| suffix.to_str()) == Some("ts") {
        written
    } else {
        PathBuf::from(format!("{}.ts", written.to_string_lossy()))
    };
    candidate
        .canonicalize()
        .ok()
        .map(|path| crate::dictionary::path::canonical_identity_key(&path.to_string_lossy()))
}

fn exports_const(path: &str, name: &str) -> bool {
    let Ok(source) = std::fs::read_to_string(path) else {
        return false;
    };
    let Some(language) = crate::compression::language::safe_typescript_language() else {
        return false;
    };
    let mut parser = tree_sitter::Parser::new();
    if parser.set_language(&language).is_err() {
        return false;
    }
    let Some(tree) = parser.parse(&source, None) else {
        return false;
    };
    let root = tree.root_node();
    let mut cursor = root.walk();
    root.named_children(&mut cursor)
        .filter(|node| node.kind() == "export_statement")
        .any(|export| exported_const_descendant(export, source.as_bytes(), name))
}

fn exported_const_descendant(node: tree_sitter::Node<'_>, source: &[u8], name: &str) -> bool {
    if node.kind() == "variable_declarator"
        && node
            .child_by_field_name("name")
            .and_then(|name| name.utf8_text(source).ok())
            == Some(name)
        && node
            .parent()
            .is_some_and(|parent| parent.kind() == "lexical_declaration")
    {
        return true;
    }
    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .any(|child| exported_const_descendant(child, source, name))
}
