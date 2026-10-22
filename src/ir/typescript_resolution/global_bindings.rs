//! Bare global symbols use the same admitted program as namespace composition.
use super::{
    DeclarationId, Namespace, Resolver,
    resolver::ExportVisit,
    syntax::{Binding, admits},
};
use std::{collections::BTreeSet, path::Path};

impl Resolver {
    pub(super) fn global_bindings(
        &mut self,
        file: &Path,
        name: &str,
        namespace: Namespace,
        active: &mut BTreeSet<ExportVisit>,
    ) -> (BTreeSet<DeclarationId>, bool) {
        let Ok(units) = self.global_units(file) else {
            return (BTreeSet::new(), false);
        };
        let mut ids = BTreeSet::new();
        let mut complete = true;
        for unit in units {
            if let Some(bindings) = unit.locals.get(name) {
                let eligible: Vec<_> = bindings
                    .iter()
                    .filter(|binding| admits(binding, namespace))
                    .collect();
                if eligible.len() > 1
                    && eligible
                        .iter()
                        .any(|binding| !matches!(binding, Binding::Declaration(..)))
                {
                    complete = false;
                    continue;
                }
                let owner = unit.source.as_deref().unwrap_or(file);
                for binding in eligible {
                    let (found, closed) = self.binding(owner, binding, namespace, active);
                    ids.extend(found);
                    complete &= closed;
                }
            }
        }
        // Only declarations gathered from this admitted global scope can gain
        // cross-source merge proof. Ordinary same-name file groups stay separate.
        if complete && ids.len() > 1 {
            if self.prove_program_binding_group(file, &ids, namespace) {
                self.record_program_group(file, &ids);
            }
        }
        (ids, complete)
    }

    pub(super) fn prove_program_binding_group(
        &mut self,
        file: &Path,
        ids: &BTreeSet<DeclarationId>,
        namespace: Namespace,
    ) -> bool {
        let Ok(units) = self.global_units(file) else {
            return false;
        };
        let available: BTreeSet<_> = units
            .iter()
            .flat_map(|unit| {
                unit.locals
                    .values()
                    .flatten()
                    .filter_map(|binding| match binding {
                        Binding::Declaration(id, _) => Some(id.clone()),
                        _ => None,
                    })
            })
            .collect();
        let Some(first) = ids.first() else {
            return false;
        };
        if !ids
            .iter()
            .all(|id| id.name == first.name && available.contains(id))
        {
            return false;
        }
        let runtime: Vec<_> = ids
            .iter()
            .filter(|id| id.kind != "internal_module")
            .collect();
        // Cross-file namespace parts share this program's symbol. Within one
        // source, retain the established runtime-before-augmentation constraint.
        if ids
            .iter()
            .filter(|id| id.kind == "internal_module")
            .any(|part| {
                runtime
                    .iter()
                    .any(|id| id.source == part.source && part.start < id.end)
            })
        {
            return false;
        }
        if runtime.is_empty() {
            return true;
        }
        let functions = runtime.iter().all(|id| {
            matches!(
                id.kind.as_str(),
                "function_signature" | "function_declaration"
            )
        }) && runtime
            .iter()
            .filter(|id| id.kind == "function_declaration")
            .count()
            <= 1;
        let enums = runtime.iter().all(|id| id.kind == "enum_declaration");
        let types = namespace == Namespace::Type
            && runtime.iter().all(|id| {
                matches!(
                    id.kind.as_str(),
                    "interface_declaration" | "class_declaration" | "abstract_class_declaration"
                )
            })
            && runtime
                .iter()
                .filter(|id| id.kind != "interface_declaration")
                .count()
                <= 1;
        let class_augmentation = runtime.len() == 1
            && ids.len() > 1
            && matches!(
                runtime[0].kind.as_str(),
                "class_declaration" | "abstract_class_declaration"
            );
        functions || enums || types || class_augmentation
    }
}
