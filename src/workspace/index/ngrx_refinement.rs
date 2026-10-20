//! Workspace-owned refinement of bounded direct-relative NgRx imports.

use super::edges::{EdgeKey, StoredEdge};
use super::{WorkspaceIndex, entity_key};
use crate::layers::meta::semantic::SemanticEdge;
use crate::layers::meta::semantic::{EntityRef, SemanticRelation};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

impl WorkspaceIndex {
    pub(crate) fn has_authoritative_object_actions_in_file(&self, file: &str) -> bool {
        self.entities.iter().any(|(key, occurrences)| {
            key.0 == "ngrx"
                && key.1 == "Action"
                && key.2.contains('.')
                && occurrences
                    .iter()
                    .any(|entity| entity.file.as_deref() == Some(file))
        })
    }

    pub(crate) fn has_authoritative_object_switch_in_file(&self, file: &str) -> bool {
        self.forward.values().flatten().any(|stored| {
            stored.asserting_file == file
                && stored.edge.layer == "ngrx"
                && stored.edge.relation == SemanticRelation::TriggersReducer
                && stored.edge.subject.entity_type == "Action"
                && stored.edge.subject.name.contains('.')
        })
    }

    pub(super) fn rebuild_ngrx_import_refinements(&mut self) {
        for key in std::mem::take(&mut self.ngrx_refined_edges) {
            self.edge_set.remove(&key);
            self.edge_count = self.edge_count.saturating_sub(1);
            if let Some(keys) = self.file_edges.get_mut(&key.asserting_file) {
                keys.retain(|candidate| candidate != &key);
            }
            self.drop_occurrence(&key);
        }
        self.drop_ngrx_refined_entities();

        let candidates: Vec<_> = self
            .ngrx_refinement_candidates
            .iter()
            .flat_map(|(file, edges)| edges.iter().cloned().map(|edge| (file.clone(), edge)))
            .collect();
        let mut action_authorities = BTreeSet::new();
        let mut reducer_authorities = BTreeSet::new();

        for (asserting_file, edge) in &candidates {
            if edge.layer == crate::angular_meta::ngrx::object_actions::DISPATCH_LAYER
                && let Some(target) =
                    resolve_candidate_target(asserting_file, edge.object.file.as_deref())
                && self.has_object_member(&edge.object.name, &target)
            {
                action_authorities.insert((edge.object.name.clone(), target));
            } else if edge.layer == crate::angular_meta::ngrx::object_actions::REGISTRATION_LAYER
                && let Some(target) =
                    resolve_candidate_target(asserting_file, edge.object.file.as_deref())
                && self.has_reducer_declaration(&edge.object.name, &target)
                && exports_function(&target, &edge.object.name)
            {
                reducer_authorities.insert((edge.object.name.clone(), target));
            }
        }

        for (asserting_file, mut edge) in candidates {
            if edge.layer == crate::angular_meta::ngrx::object_actions::DISPATCH_LAYER {
                let Some(target) =
                    resolve_candidate_target(&asserting_file, edge.object.file.as_deref())
                else {
                    continue;
                };
                if !action_authorities.contains(&(edge.object.name.clone(), target.clone())) {
                    continue;
                }
                edge.subject.file = Some(asserting_file.clone());
                edge.object.file = Some(target);
                edge.layer = "ngrx";
                self.insert_refined_edge(&asserting_file, edge);
                continue;
            }
            if edge.layer == crate::angular_meta::ngrx::object_actions::REGISTRATION_LAYER {
                let Some(target) =
                    resolve_candidate_target(&asserting_file, edge.object.file.as_deref())
                else {
                    continue;
                };
                if !reducer_authorities.contains(&(edge.object.name.clone(), target.clone())) {
                    continue;
                }
                edge.subject.file = Some(asserting_file.clone());
                edge.object.file = Some(target);
                edge.layer = "ngrx";
                self.insert_refined_edge(&asserting_file, edge);
                continue;
            }
            if edge.layer == crate::angular_meta::ngrx::object_actions::SWITCH_LAYER {
                let Some(action_target) =
                    resolve_candidate_target(&asserting_file, edge.subject.file.as_deref())
                else {
                    continue;
                };
                let reducer_target = edge
                    .object
                    .file
                    .clone()
                    .unwrap_or_else(|| asserting_file.clone());
                if !action_authorities.contains(&(edge.subject.name.clone(), action_target.clone()))
                    || !reducer_authorities
                        .contains(&(edge.object.name.clone(), reducer_target.clone()))
                {
                    continue;
                }
                edge.subject.file = Some(action_target);
                edge.object.file = Some(reducer_target);
                edge.layer = "ngrx";
                self.insert_refined_edge(&asserting_file, edge);
                continue;
            }
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
            let Some(import_target) = resolve_typescript_module(&asserting_file, module) else {
                continue;
            };
            let target = if self.has_exact_exported_declaration(imported_entity, &import_target) {
                import_target
            } else {
                let Some(next_module) =
                    one_hop_named_reexport(&import_target, &imported_entity.name)
                else {
                    continue;
                };
                let Some(target) = resolve_typescript_module(&import_target, &next_module) else {
                    continue;
                };
                if !self.has_exact_exported_declaration(imported_entity, &target) {
                    continue;
                }
                target
            };
            if target == asserting_file {
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

    fn has_object_member(&self, name: &str, target: &str) -> bool {
        self.declarations_by_identity(
            crate::ir::typescript_object_members::DOMAIN,
            crate::ir::typescript_object_members::ENTITY_TYPE,
            name,
        )
        .iter()
        .filter(|declaration| declaration.file.as_deref() == Some(target))
        .count()
            == 1
    }

    fn has_reducer_declaration(&self, name: &str, target: &str) -> bool {
        self.declarations_by_identity(
            crate::ir::semantic_projection::BUILTIN_DOMAIN,
            crate::ir::semantic_projection::METHOD_ENTITY_TYPE,
            name,
        )
        .iter()
        .filter(|declaration| declaration.file.as_deref() == Some(target))
        .count()
            == 1
    }

    fn has_exact_exported_declaration(
        &self,
        entity: &crate::layers::meta::semantic::EntityRef,
        target: &str,
    ) -> bool {
        self.declarations_by_identity(entity.domain, entity.entity_type, &entity.name)
            .iter()
            .filter(|declaration| declaration.file.as_deref() == Some(target))
            .count()
            == 1
            && exports_const(target, &entity.name)
    }

    fn insert_refined_edge(&mut self, asserting_file: &str, edge: SemanticEdge) {
        let subject = entity_key(&edge.subject);
        let object = entity_key(&edge.object);
        let key = EdgeKey::new(asserting_file, &edge);
        if !self.edge_set.insert(key.clone()) {
            return;
        }
        self.register_refined_entity(&edge.subject);
        self.register_refined_entity(&edge.object);
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

    fn register_refined_entity(&mut self, entity: &EntityRef) {
        let Some(file) = entity.file.as_ref() else {
            return;
        };
        let key = entity_key(entity);
        let exists = self
            .entities
            .get(&key)
            .is_some_and(|items| items.iter().any(|item| item.file.as_ref() == Some(file)));
        if exists {
            return;
        }
        let mut keys = Vec::new();
        self.register_entity(&key, entity.clone(), &mut keys);
        let file_keys = self.file_map.entry(file.clone()).or_default();
        for key in keys {
            if !file_keys.contains(&key) {
                file_keys.push(key.clone());
            }
        }
        self.ngrx_refined_entities.push((key, file.clone()));
    }

    fn drop_ngrx_refined_entities(&mut self) {
        for (key, file) in std::mem::take(&mut self.ngrx_refined_entities) {
            if let Some(items) = self.entities.get_mut(&key) {
                items.retain(|item| item.file.as_deref() != Some(&file));
                if items.is_empty() {
                    self.entities.remove(&key);
                }
            }
            if let Some(keys) = self.file_map.get_mut(&file) {
                keys.retain(|item| item != &key);
            }
            if !self.entities.contains_key(&key)
                && let Some(keys) = self.name_index.get_mut(&key.2)
            {
                keys.retain(|item| item != &key);
                if keys.is_empty() {
                    self.name_index.remove(&key.2);
                }
            }
        }
    }
}

fn resolve_candidate_target(asserting_file: &str, module: Option<&str>) -> Option<String> {
    let module = module?;
    if module == asserting_file {
        Some(asserting_file.to_owned())
    } else {
        resolve_typescript_module(asserting_file, module)
    }
}

fn one_hop_named_reexport(path: &str, name: &str) -> Option<String> {
    let source = std::fs::read_to_string(path).ok()?;
    let language = crate::compression::language::safe_typescript_language()?;
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&language).ok()?;
    let tree = parser.parse(&source, None)?;
    if tree.root_node().has_error() {
        return None;
    }
    let root = tree.root_node();
    let source_bytes = source.as_bytes();
    if exported_local_occurrences(root, name, source_bytes) != 1 {
        return None;
    }
    let mut modules = Vec::new();
    collect_named_import_modules(root, name, source_bytes, &mut modules);
    let [module] = modules.as_slice() else {
        return None;
    };
    Some(module.clone())
}

fn exported_local_occurrences(node: tree_sitter::Node<'_>, name: &str, source: &[u8]) -> usize {
    if node.kind() == "export_statement" {
        if node.child_by_field_name("source").is_some() {
            return 0;
        }
        return count_unaliased_specifiers(node, name, source);
    }
    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .map(|child| exported_local_occurrences(child, name, source))
        .sum()
}

fn count_unaliased_specifiers(node: tree_sitter::Node<'_>, name: &str, source: &[u8]) -> usize {
    if node.kind() == "export_specifier" {
        return usize::from(
            node.child_by_field_name("alias").is_none()
                && node
                    .child_by_field_name("name")
                    .and_then(|identifier| identifier.utf8_text(source).ok())
                    == Some(name),
        );
    }
    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .map(|child| count_unaliased_specifiers(child, name, source))
        .sum()
}

fn collect_named_import_modules(
    node: tree_sitter::Node<'_>,
    name: &str,
    source: &[u8],
    modules: &mut Vec<String>,
) {
    if node.kind() == "import_statement" {
        if node
            .children(&mut node.walk())
            .any(|child| child.kind() == "type")
        {
            return;
        }
        let Some(module) = node
            .child_by_field_name("source")
            .and_then(|string| string.named_child(0))
            .and_then(|fragment| fragment.utf8_text(source).ok())
            .filter(|module| module.starts_with("./") || module.starts_with("../"))
        else {
            return;
        };
        if count_unaliased_import_specifiers(node, name, source) == 1 {
            modules.push(module.to_owned());
        }
        return;
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_named_import_modules(child, name, source, modules);
    }
}

fn count_unaliased_import_specifiers(
    node: tree_sitter::Node<'_>,
    name: &str,
    source: &[u8],
) -> usize {
    if node.kind() == "import_specifier" {
        return usize::from(
            node.child_by_field_name("alias").is_none()
                && node
                    .child_by_field_name("name")
                    .and_then(|identifier| identifier.utf8_text(source).ok())
                    == Some(name),
        );
    }
    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .map(|child| count_unaliased_import_specifiers(child, name, source))
        .sum()
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

fn exports_function(path: &str, name: &str) -> bool {
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
    let Some(tree) = parser
        .parse(&source, None)
        .filter(|tree| !tree.root_node().has_error())
    else {
        return false;
    };
    let root = tree.root_node();
    root.named_children(&mut root.walk())
        .filter(|node| node.kind() == "export_statement")
        .filter_map(|export| export.named_child(0))
        .any(|declaration| {
            declaration.kind() == "function_declaration"
                && declaration
                    .child_by_field_name("name")
                    .and_then(|identifier| identifier.utf8_text(source.as_bytes()).ok())
                    == Some(name)
        })
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
