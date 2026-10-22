//! Admitted script-owned ambient module scopes; no synthetic filesystem targets.
use super::declared_scopes::NamespaceScope;
use super::{DeclarationId, Resolver, module_resolution::ModuleRequestKind, syntax::Unit};
use std::path::Path;

impl Resolver {
    pub(super) fn ambient_scope_in_units(
        &mut self,
        file: &Path,
        specifier: &str,
        units: &[Unit],
    ) -> Result<Option<NamespaceScope>, String> {
        let project = self.request_project(file)?;
        let mut blocks = Vec::new();
        for unit in units {
            if self.program_source_external(unit, &project)? {
                continue;
            }
            blocks.extend(
                unit.namespace_bodies
                    .keys()
                    .filter(|id| id.kind == "ambient_module")
                    .cloned(),
            );
        }
        let exact = !specifier.starts_with('.')
            && !specifier.starts_with('/')
            && blocks.iter().any(|id| id.name == specifier);
        if exact {
            blocks.retain(|id| id.name == specifier);
        } else {
            let mut patterns = std::collections::BTreeMap::new();
            for id in &blocks {
                if let Some((prefix, suffix)) = id.name.split_once('*') {
                    if suffix.contains('*') {
                        return Err("Invalid ambient module pattern".into());
                    }
                    if specifier.len() >= prefix.len() + suffix.len()
                        && specifier.starts_with(prefix)
                        && specifier.ends_with(suffix)
                    {
                        patterns.insert(id.name.clone(), prefix.len());
                    }
                }
            }
            let longest = patterns.values().copied().max();
            let best: Vec<_> = patterns
                .iter()
                .filter(|(_, size)| Some(**size) == longest)
                .map(|(name, _)| name)
                .collect();
            if best.len() > 1 {
                return Err("Competing ambient module patterns".into());
            }
            blocks.retain(|id| {
                best.first()
                    .is_some_and(|name| id.name.as_str() == name.as_str())
            });
        }
        blocks.sort();
        blocks.dedup();
        Ok((!blocks.is_empty()).then(|| NamespaceScope {
            file: file.to_path_buf(),
            blocks,
        }))
    }

    pub(super) fn module_scope(
        &mut self,
        file: &Path,
        specifier: &str,
        kind: ModuleRequestKind,
    ) -> Result<NamespaceScope, String> {
        if self.program_acquiring(file) {
            return self
                .module_request(file, specifier, kind)
                .map(|target| NamespaceScope::module(&target));
        }
        let units = self.program_units(file)?;
        let ambient = self.ambient_scope_in_units(file, specifier, &units)?;
        if let Some(scope) = &ambient
            && scope.blocks.iter().all(|id| id.name == specifier)
        {
            return Ok(scope.clone());
        }
        match self.module_request(file, specifier, kind) {
            Ok(target) => Ok(NamespaceScope::module(&target)),
            Err(error) => ambient.ok_or(error),
        }
    }

    pub(super) fn program_acquiring(&mut self, file: &Path) -> bool {
        self.request_project(file)
            .ok()
            .and_then(|project| project.config)
            .is_some_and(|config| self.program_active.contains(&config))
    }

    pub(super) fn ambient_declarations(scope: &NamespaceScope) -> Vec<DeclarationId> {
        scope
            .blocks
            .iter()
            .filter(|id| id.kind == "ambient_module")
            .cloned()
            .collect()
    }
}
