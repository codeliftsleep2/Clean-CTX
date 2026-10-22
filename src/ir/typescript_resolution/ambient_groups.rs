//! Merge proof remains confined to the selected admitted ambient module scope.
use super::declared_scopes::NamespaceScope;
use super::{
    DeclarationId, Namespace, Resolver,
    syntax::{Binding, admits},
};
use std::collections::BTreeSet;

impl Resolver {
    pub(super) fn prove_ambient_group(
        &mut self,
        scope: &NamespaceScope,
        ids: &BTreeSet<DeclarationId>,
        namespace: Namespace,
    ) -> bool {
        if scope.blocks.is_empty() || ids.len() < 2 {
            return false;
        }
        let Ok(project) = self.request_project(&scope.file) else {
            return false;
        };
        let Ok(program) = self.program_units(&scope.file) else {
            return false;
        };
        let mut owners = BTreeSet::new();
        for block in &scope.blocks {
            let mut found = false;
            for unit in &program {
                let Ok(external) = self.program_source_external(unit, &project) else {
                    return false;
                };
                if external {
                    continue;
                }
                for (owner, body) in &unit.namespace_bodies {
                    if owner.kind == "ambient_module"
                        && (owner == block || body.body(block).is_some())
                    {
                        owners.insert(owner.name.clone());
                        found = true;
                    }
                }
            }
            if !found {
                return false;
            }
        }
        if owners.len() != 1 {
            return false;
        }
        let Some(units) = self.scope_units(scope) else {
            return false;
        };
        let Some(first) = ids.first() else {
            return false;
        };
        let available: BTreeSet<_> = units
            .iter()
            .flat_map(|unit| unit.exports.get(&first.name).into_iter().flatten())
            .filter_map(|binding| match binding {
                Binding::Declaration(id, _)
                    if admits(binding, namespace) || id.kind == "internal_module" =>
                {
                    Some(id.clone())
                }
                _ => None,
            })
            .collect();
        if &available != ids || ids.iter().any(|id| id.name != first.name) {
            return false;
        }
        let namespaces = ids.iter().all(|id| id.kind == "internal_module");
        let types = namespace == Namespace::Type
            && ids.iter().all(|id| {
                matches!(
                    id.kind.as_str(),
                    "interface_declaration" | "class_declaration" | "abstract_class_declaration"
                )
            })
            && ids
                .iter()
                .filter(|id| id.kind != "interface_declaration")
                .count()
                <= 1;
        if !namespaces && !types {
            return false;
        }
        self.record_program_group(&scope.file, ids);
        true
    }
}
