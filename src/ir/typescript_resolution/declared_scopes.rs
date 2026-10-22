//! Declared namespace scope is identified before any member-name presentation.
use super::syntax::{Binding, Unit, admits};
use super::{DeclarationId, Namespace, Resolution, Resolver};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(super) struct NamespaceScope {
    pub file: PathBuf,
    pub blocks: Vec<DeclarationId>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct DeclaredScopeEvidence {
    pub declaration: DeclarationId,
    pub parent: Option<DeclarationId>,
    pub exports: Vec<(String, Namespace, Resolution)>,
}
impl NamespaceScope {
    pub fn module(file: &Path) -> Self {
        Self {
            file: file.to_path_buf(),
            blocks: Vec::new(),
        }
    }
}

type ScopeLayout = (DeclarationId, Option<DeclarationId>, Vec<String>);

impl Unit {
    fn layout(&self, parent: Option<&DeclarationId>, entries: &mut Vec<ScopeLayout>) {
        for (id, body) in &self.namespace_bodies {
            entries.push((
                id.clone(),
                parent.cloned(),
                body.exports.keys().cloned().collect(),
            ));
            body.layout(Some(id), entries);
        }
    }
    pub(super) fn body(&self, id: &DeclarationId) -> Option<Unit> {
        if let Some(body) = self.namespace_bodies.get(id) {
            return Some(body.clone());
        }
        self.namespace_bodies
            .values()
            .find_map(|body| body.body(id))
            .or_else(|| self.ambient_globals.iter().find_map(|body| body.body(id)))
    }
    pub(super) fn inventory(&self) -> BTreeSet<DeclarationId> {
        let mut ids: BTreeSet<_> = self
            .locals
            .values()
            .chain(self.exports.values())
            .flatten()
            .filter_map(|binding| {
                if let Binding::Declaration(id, _) = binding {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect();
        for body in self.namespace_bodies.values().chain(&self.ambient_globals) {
            ids.extend(body.inventory());
        }
        ids
    }
}

impl Resolver {
    pub(super) fn alias_declaration(
        &mut self,
        scope: &NamespaceScope,
        path: &[String],
        namespace: Namespace,
    ) -> (BTreeSet<DeclarationId>, bool) {
        let key = (scope.clone(), path.to_vec(), namespace);
        if !self.alias_active.insert(key.clone()) {
            return (BTreeSet::new(), false);
        }
        let result = if let Some((first, rest)) = path.split_first() {
            if rest.is_empty() {
                let units = self.scope_units(scope);
                if let Some(units) = units {
                    let mut ids = BTreeSet::new();
                    let mut complete = true;
                    let mut present = false;
                    for unit in &units {
                        if let Some(bindings) = unit.locals.get(first) {
                            present = true;
                            let (found, closed) = self.scoped_bindings(
                                scope,
                                unit,
                                bindings,
                                namespace,
                                &mut BTreeSet::new(),
                            );
                            ids.extend(found);
                            complete &= closed;
                        }
                    }
                    if !present {
                        if let Some(parent) = self.lexical_parent_scope(scope) {
                            self.alias_declaration(&parent, path, namespace)
                        } else {
                            (ids, complete)
                        }
                    } else {
                        (ids, complete)
                    }
                } else {
                    (BTreeSet::new(), false)
                }
            } else {
                let members: Vec<_> = rest.iter().map(String::as_str).collect();
                match self.namespace_chain_from(scope, first, &members, namespace, false) {
                    Resolution::Resolved(id) => (BTreeSet::from([id]), true),
                    Resolution::Merged(ids) | Resolution::Ambiguous(ids) => {
                        (ids.into_iter().collect(), true)
                    }
                    Resolution::Unresolved(_) => (BTreeSet::new(), false),
                }
            }
        } else {
            (BTreeSet::new(), false)
        };
        self.alias_active.remove(&key);
        result
    }

    pub(super) fn lexical_parent_scope(
        &mut self,
        scope: &NamespaceScope,
    ) -> Option<NamespaceScope> {
        if scope.blocks.is_empty() {
            return None;
        }
        let root = self.unit(&scope.file)?;
        let mut layout = Vec::new();
        root.layout(None, &mut layout);
        let mut parents = BTreeSet::new();
        for id in &scope.blocks {
            let (_, parent, _) = layout.iter().find(|entry| &entry.0 == id)?;
            if let Some(parent) = parent {
                parents.insert(parent.clone());
            }
        }
        Some(NamespaceScope {
            file: scope.file.clone(),
            blocks: parents.into_iter().collect(),
        })
    }

    pub(super) fn namespace_lexical_reference(
        &mut self,
        file: &Path,
        owners: &[(usize, usize)],
        name: &str,
        members: &[&str],
    ) -> Resolution {
        let mut parent = NamespaceScope::module(file);
        let mut frames = Vec::new();
        for &(start, end) in owners.iter().rev() {
            let Some(units) = self.scope_units(&parent) else {
                return Resolution::Unresolved("Namespace lexical parent absent".into());
            };
            let Some(id) = units
                .iter()
                .flat_map(|unit| unit.namespace_bodies.keys())
                .find(|id| id.start == start && id.end == end)
                .cloned()
            else {
                return Resolution::Unresolved("Namespace lexical owner absent".into());
            };
            let blocks: BTreeSet<_> = units
                .iter()
                .filter_map(|unit| unit.locals.get(&id.name))
                .flatten()
                .filter_map(|binding| match binding {
                    Binding::Declaration(other, _) if other.kind == "internal_module" => {
                        Some(other.clone())
                    }
                    _ => None,
                })
                .collect();
            if blocks.len() > 1 && !super::declaration_groups::mergeable(&blocks) {
                return Resolution::Unresolved("Namespace lexical merge unproved".into());
            }
            let exact = NamespaceScope {
                file: file.to_path_buf(),
                blocks: vec![id],
            };
            parent = NamespaceScope {
                file: file.to_path_buf(),
                blocks: blocks.into_iter().collect(),
            };
            frames.push((exact, parent.clone()));
        }
        for (exact, merged) in frames.iter().rev() {
            let Some(units) = self.scope_units(exact) else {
                return Resolution::Unresolved("Namespace lexical body absent".into());
            };
            for unit in &units {
                if let Some(bindings) = unit.locals.get(name) {
                    if !members.is_empty() {
                        return self.namespace_chain_from(
                            exact,
                            name,
                            members,
                            Namespace::Value,
                            false,
                        );
                    }
                    let (ids, complete) = self.scoped_bindings(
                        exact,
                        unit,
                        bindings,
                        Namespace::Value,
                        &mut BTreeSet::new(),
                    );
                    return self.outcome(ids, complete);
                }
            }
            let Some(parts) = self.scope_units(merged) else {
                return Resolution::Unresolved("Namespace merged body absent".into());
            };
            if parts.iter().any(|unit| unit.exports.contains_key(name)) {
                if !members.is_empty() {
                    return self.namespace_chain_from(
                        merged,
                        name,
                        members,
                        Namespace::Value,
                        true,
                    );
                }
                return self.scope_export(merged, name, Namespace::Value);
            }
        }
        if members.is_empty() {
            self.local(file, name, Namespace::Value)
        } else {
            self.namespace_chain(file, name, members, Namespace::Value)
        }
    }

    pub(super) fn declared_scope_evidence(&mut self, file: &Path) -> Vec<DeclaredScopeEvidence> {
        let Some(unit) = self.unit(file) else {
            return Vec::new();
        };
        let mut layout = Vec::new();
        unit.layout(None, &mut layout);
        layout
            .into_iter()
            .map(|(declaration, parent, names)| {
                let scope = NamespaceScope {
                    file: file.to_path_buf(),
                    blocks: vec![declaration.clone()],
                };
                let mut exports = Vec::new();
                for name in names {
                    for namespace in [Namespace::Value, Namespace::Type] {
                        exports.push((
                            name.clone(),
                            namespace,
                            self.scope_export(&scope, &name, namespace),
                        ));
                    }
                }
                DeclaredScopeEvidence {
                    declaration,
                    parent,
                    exports,
                }
            })
            .collect()
    }
    pub(super) fn scope_units(&mut self, scope: &NamespaceScope) -> Option<Vec<Unit>> {
        let root = self.unit(&scope.file)?;
        if scope.blocks.is_empty() {
            return Some(vec![root]);
        }
        scope
            .blocks
            .iter()
            .map(|id| self.unit(&id.source)?.body(id))
            .collect()
    }

    pub(super) fn scope_export(
        &mut self,
        scope: &NamespaceScope,
        name: &str,
        namespace: Namespace,
    ) -> Resolution {
        let key = (scope.clone(), name.to_owned(), namespace);
        if !self.scope_export_active.insert(key.clone()) {
            return Resolution::Unresolved("Cyclic scoped export".into());
        }
        let result = self.scope_export_inner(scope, name, namespace);
        self.scope_export_active.remove(&key);
        result
    }

    fn scope_export_inner(
        &mut self,
        scope: &NamespaceScope,
        name: &str,
        namespace: Namespace,
    ) -> Resolution {
        if scope.blocks.is_empty() {
            let (ids, complete) =
                self.export_set(&scope.file, name, namespace, &mut BTreeSet::new());
            return self.outcome(ids, complete);
        }
        let Some(units) = self.scope_units(scope) else {
            return Resolution::Unresolved("Declared namespace scope absent".into());
        };
        let mut ids = BTreeSet::new();
        let mut complete = true;
        for unit in &units {
            if let Some(bindings) = unit.exports.get(name) {
                let (found, closed) =
                    self.scoped_bindings(scope, unit, bindings, namespace, &mut BTreeSet::new());
                ids.extend(found);
                complete &= closed;
            }
        }
        if complete && ids.len() > 1 {
            self.prove_ambient_group(scope, &ids, namespace);
        }
        self.outcome(ids, complete)
    }

    fn scoped_bindings(
        &mut self,
        scope: &NamespaceScope,
        unit: &Unit,
        bindings: &[Binding],
        namespace: Namespace,
        active: &mut BTreeSet<String>,
    ) -> (BTreeSet<DeclarationId>, bool) {
        let eligible: Vec<_> = bindings
            .iter()
            .filter(|binding| admits(binding, namespace))
            .collect();
        if eligible.len() > 1
            && eligible
                .iter()
                .any(|binding| !matches!(binding, Binding::Declaration(..)))
        {
            return (BTreeSet::new(), false);
        }
        let mut ids = BTreeSet::new();
        let mut complete = true;
        for binding in eligible {
            let (found, closed) = match binding {
                Binding::Alias(path) => self.alias_declaration(scope, path, namespace),
                Binding::Import {
                    module, imported, ..
                } if module.is_empty() => {
                    if !active.insert(imported.clone()) {
                        return (BTreeSet::new(), false);
                    }
                    let result = if let Some(bindings) = unit.locals.get(imported) {
                        self.scoped_bindings(scope, unit, bindings, namespace, active)
                    } else {
                        (BTreeSet::new(), true)
                    };
                    active.remove(imported);
                    result
                }
                _ => self.binding(
                    unit.source.as_deref().unwrap_or(&scope.file),
                    binding,
                    namespace,
                    &mut BTreeSet::new(),
                ),
            };
            ids.extend(found);
            complete &= closed;
        }
        (ids, complete)
    }
}

pub(super) fn namespace_admits(binding: &Binding, namespace: Namespace) -> bool {
    matches!(binding, Binding::Declaration(id, _) if id.kind == "internal_module")
        || admits(binding, namespace)
}
