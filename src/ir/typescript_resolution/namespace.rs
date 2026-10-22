//! Namespace objects retain module identity until a concrete export is selected.
use super::declared_scopes::{NamespaceScope, namespace_admits};
use super::syntax::Binding;
use super::{Namespace, Resolution, Resolver};
use std::collections::BTreeSet;
use std::path::Path;

impl Resolver {
    pub(super) fn namespace_member(
        &mut self,
        file: &Path,
        object: &str,
        member: &str,
        namespace: Namespace,
    ) -> Resolution {
        self.namespace_chain(file, object, &[member], namespace)
    }

    pub(super) fn namespace_chain(
        &mut self,
        file: &Path,
        object: &str,
        members: &[&str],
        namespace: Namespace,
    ) -> Resolution {
        let Some(file) = self.resolution_file(file) else {
            return Resolution::Unresolved("Namespace source outside scope".into());
        };
        self.in_source_project(&file, |resolver| {
            resolver.namespace_chain_from(
                &NamespaceScope::module(&file),
                object,
                members,
                namespace,
                false,
            )
        })
    }

    pub(super) fn namespace_chain_from(
        &mut self,
        scope: &NamespaceScope,
        object: &str,
        members: &[&str],
        namespace: Namespace,
        exported: bool,
    ) -> Resolution {
        let Some((leaf, intermediates)) = members.split_last() else {
            return Resolution::Unresolved("Namespace member path is empty".into());
        };
        let (targets, complete) =
            self.namespace_targets(scope, object, namespace, exported, &mut BTreeSet::new());
        if !complete || targets.len() != 1 {
            return Resolution::Unresolved("Namespace identity is not unique and closed".into());
        }
        let mut target = targets.into_iter().next().expect("one namespace");
        for member in intermediates {
            let (targets, complete) =
                self.namespace_targets(&target, member, namespace, true, &mut BTreeSet::new());
            if !complete || targets.len() != 1 {
                return Resolution::Unresolved(
                    "Intermediate namespace is not unique and closed".into(),
                );
            }
            target = targets.into_iter().next().expect("one namespace");
        }
        self.scope_export(&target, leaf, namespace)
    }

    fn namespace_targets(
        &mut self,
        scope: &NamespaceScope,
        name: &str,
        namespace: Namespace,
        exported: bool,
        active: &mut BTreeSet<(NamespaceScope, String, Namespace, bool)>,
    ) -> (BTreeSet<NamespaceScope>, bool) {
        let key = (scope.clone(), name.to_owned(), namespace, exported);
        let file = scope.file.as_path();
        if !active.insert(key.clone()) {
            return (BTreeSet::new(), true);
        }
        let result = (|| {
            let Some(mut units) = self.scope_units(scope) else {
                return (BTreeSet::new(), false);
            };
            if scope.blocks.is_empty() && !exported {
                let local_present = units
                    .iter()
                    .any(|unit| unit.external && unit.locals.contains_key(name));
                if !local_present {
                    match self.global_units(file) {
                        Ok(globals) => {
                            // A script already belongs to the program; do not count it twice.
                            if !globals.is_empty() {
                                units.retain(|unit| unit.external);
                                units.extend(globals);
                            }
                        }
                        Err(_) => return (BTreeSet::new(), false),
                    }
                }
            }
            let mut bindings = Vec::new();
            let mut present = false;
            for unit in &units {
                let found = if exported {
                    unit.exports.get(name)
                } else {
                    unit.locals.get(name)
                };
                if let Some(found) = found {
                    present = true;
                    bindings.extend(
                        found
                            .iter()
                            .filter(|binding| namespace_admits(binding, namespace))
                            .cloned(),
                    );
                }
            }
            let mut targets = BTreeSet::new();
            let mut complete = true;
            if present {
                let declarations: BTreeSet<_> = bindings
                    .iter()
                    .filter_map(|binding| {
                        if let Binding::Declaration(id, _) = binding {
                            Some(id.clone())
                        } else {
                            None
                        }
                    })
                    .collect();
                if bindings.len() > 1
                    && (declarations.len() != bindings.len()
                        || !(super::declaration_groups::mergeable(&declarations)
                            || self.prove_ambient_group(scope, &declarations, namespace)
                            || self.prove_program_namespaces(file, &declarations)))
                {
                    return (BTreeSet::new(), false);
                }
                let blocks: Vec<_> = declarations
                    .iter()
                    .filter(|id| id.kind == "internal_module")
                    .cloned()
                    .collect();
                if !blocks.is_empty() {
                    targets.insert(NamespaceScope {
                        file: file.to_path_buf(),
                        blocks,
                    });
                }
                for binding in &bindings {
                    match binding {
                        Binding::Alias(path) => {
                            let mut parts = path.iter();
                            let Some(first) = parts.next() else {
                                complete = false;
                                continue;
                            };
                            let (mut found, mut closed) =
                                self.namespace_targets(scope, first, namespace, false, active);
                            for member in parts {
                                if !closed || found.len() != 1 {
                                    closed = false;
                                    break;
                                }
                                let target = found.iter().next().expect("one namespace").clone();
                                (found, closed) = self
                                    .namespace_targets(&target, member, namespace, true, active);
                            }
                            targets.extend(found);
                            complete &= closed;
                        }
                        Binding::Require { module, .. } => match self.module_request(
                            file,
                            module,
                            super::module_resolution::ModuleRequestKind::Require,
                        ) {
                            Ok(target) => {
                                targets.insert(NamespaceScope::module(&target));
                            }
                            Err(_) => complete = false,
                        },
                        Binding::Namespace { module, mode, .. } => {
                            match self.module_scope(file, module, *mode) {
                                Ok(target) => {
                                    targets.insert(target);
                                }
                                Err(_) => complete = false,
                            }
                        }
                        Binding::Import {
                            module,
                            imported,
                            mode,
                            ..
                        } => {
                            let (found, closed) = if module.is_empty() {
                                self.namespace_targets(scope, imported, namespace, false, active)
                            } else {
                                match self.module_scope(file, module, *mode) {
                                    Ok(target) => self.namespace_targets(
                                        &target, imported, namespace, true, active,
                                    ),
                                    Err(_) => (BTreeSet::new(), false),
                                }
                            };
                            targets.extend(found);
                            complete &= closed;
                        }
                        Binding::Declaration(..) => {
                            if !declarations.iter().any(|id| id.kind == "internal_module") {
                                complete = false;
                            }
                        }
                    }
                }
            } else if !exported && !scope.blocks.is_empty() {
                if let Some(parent) = self.lexical_parent_scope(scope) {
                    return self.namespace_targets(&parent, name, namespace, false, active);
                }
                return (BTreeSet::new(), false);
            } else if exported && name != "default" {
                for (module, type_only) in units.iter().flat_map(|unit| &unit.stars) {
                    if *type_only && namespace == Namespace::Value {
                        continue;
                    }
                    let (found, closed) = match self.module(file, module) {
                        Ok(target) => self.namespace_targets(
                            &NamespaceScope::module(&target),
                            name,
                            namespace,
                            true,
                            active,
                        ),
                        Err(_) => (BTreeSet::new(), false),
                    };
                    targets.extend(found);
                    complete &= closed;
                }
            }
            (targets, complete)
        })();
        active.remove(&key);
        result
    }
}
