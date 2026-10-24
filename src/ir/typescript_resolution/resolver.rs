use super::project::Project;
use super::syntax::{Binding, Unit, admits};
use super::{DeclarationId, Host, Namespace};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) enum Resolution {
    Resolved(DeclarationId),
    /// One source-owned symbol with every contributing declaration retained.
    Merged(Vec<DeclarationId>),
    Ambiguous(Vec<DeclarationId>),
    Unresolved(String),
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(super) enum ExportSlot {
    Named(String),
    Assignment,
}
pub(super) type ExportVisit = (PathBuf, ExportSlot, Namespace);

pub(crate) struct Resolver {
    pub(super) consumed_entries: Vec<super::package_evidence::ConsumedModuleEntry>,
    pub host: Host,
    pub(super) unbound_value_sites: BTreeSet<(PathBuf, usize, usize, String)>,
    pub(super) type_version_active: BTreeSet<(PathBuf, String)>,
    pub(super) alias_active: BTreeSet<(
        super::declared_scopes::NamespaceScope,
        Vec<String>,
        Namespace,
    )>,
    units: BTreeMap<PathBuf, Option<Unit>>,
    pub(super) programs: BTreeMap<PathBuf, Result<Vec<Unit>, String>>,
    pub(super) library_evidence: Vec<super::compiler_libraries::LibraryRequest>,
    pub(super) automatic_type_evidence: Vec<super::automatic_types::AutomaticTypeRequest>,
    pub(super) directive_evidence: Vec<super::program_directives::DirectiveRequest>,
    pub(super) program_active: BTreeSet<PathBuf>,
    pub(super) scope_export_active:
        BTreeSet<(super::declared_scopes::NamespaceScope, String, Namespace)>,
    pub(super) program_evidence: BTreeMap<PathBuf, super::program::ProgramEvidence>,
    projects: BTreeMap<PathBuf, Result<Project, String>>,
    requested_project: Option<PathBuf>,
    project_context: Option<Result<Project, String>>,
}

impl Resolver {
    pub(super) fn in_source_project<T>(
        &mut self,
        file: &Path,
        action: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let inherited = self.project_context.take();
        self.project_context = Some(
            inherited
                .clone()
                .unwrap_or_else(|| self.project_identity(file)),
        );
        let result = action(self);
        self.project_context = inherited;
        result
    }

    pub(super) fn request_project(&mut self, file: &Path) -> Result<Project, String> {
        self.project_context
            .clone()
            .unwrap_or_else(|| self.project_identity(file))
    }
    pub(super) fn resolution_file(&mut self, file: &Path) -> Option<PathBuf> {
        let project = self.request_project(file).ok();
        if project.as_ref().is_some_and(|project| {
            project
                .options
                .get("preserveSymlinks")
                .and_then(|option| option.value.as_bool())
                == Some(true)
        }) {
            self.host.logical(file)
        } else {
            self.host.normalized(file)
        }
    }

    pub(super) fn project_path(&mut self, path: &Path, project: &Project) -> Option<PathBuf> {
        if project
            .options
            .get("preserveSymlinks")
            .and_then(|option| option.value.as_bool())
            == Some(true)
        {
            self.host.logical(path)
        } else {
            self.host.normalized(path)
        }
    }

    pub(super) fn source_tree(&mut self, file: &Path) -> Option<tree_sitter::Tree> {
        self.unit(file)?.tree
    }
    pub(super) fn source_declarations(
        &mut self,
        file: &Path,
    ) -> Result<Vec<DeclarationId>, String> {
        let unit = self
            .unit(file)
            .ok_or("TypeScript source syntax has no supported binding unit")?;
        Ok(unit.inventory().into_iter().collect())
    }

    pub(super) fn source_names(&mut self, file: &Path) -> Vec<String> {
        self.unit(file)
            .map(|unit| unit.locals.into_keys().collect())
            .unwrap_or_default()
    }

    pub(super) fn project_identity(&mut self, file: &Path) -> Result<Project, String> {
        let file = self.host.normalized(file).ok_or("Source outside scope")?;
        if let Some(project) = self.projects.get(&file) {
            return project.clone();
        }
        let project = match self.requested_project.as_deref() {
            Some(config) => Project::explicit(&mut self.host, config, &file),
            None => Project::discover(&mut self.host, &file),
        };
        self.projects.insert(file, project.clone());
        project
    }
    pub(crate) fn new(roots: &[PathBuf]) -> Result<Self, String> {
        Self::with_project(roots, None)
    }
    pub(crate) fn with_project(roots: &[PathBuf], project: Option<&Path>) -> Result<Self, String> {
        Ok(Self {
            host: Host::new(roots)?,
            consumed_entries: Vec::new(),
            unbound_value_sites: BTreeSet::new(),
            type_version_active: BTreeSet::new(),
            alias_active: BTreeSet::new(),
            scope_export_active: BTreeSet::new(),
            units: BTreeMap::new(),
            programs: BTreeMap::new(),
            program_active: BTreeSet::new(),
            directive_evidence: Vec::new(),
            automatic_type_evidence: Vec::new(),
            library_evidence: Vec::new(),
            program_evidence: BTreeMap::new(),
            projects: BTreeMap::new(),
            requested_project: project.map(Path::to_path_buf),
            project_context: None,
        })
    }

    pub(crate) fn local(&mut self, file: &Path, name: &str, namespace: Namespace) -> Resolution {
        let Some(file) = self.resolution_file(file) else {
            return Resolution::Unresolved("Source outside scope".into());
        };
        self.in_source_project(&file, |resolver| {
            let mut active = BTreeSet::new();
            let (declarations, complete) = resolver.local_set(&file, name, namespace, &mut active);
            resolver.outcome(declarations, complete)
        })
    }

    pub(super) fn outcome(
        &self,
        declarations: BTreeSet<DeclarationId>,
        complete: bool,
    ) -> Resolution {
        if !complete {
            return Resolution::Unresolved("Resolution has unproven contributors".into());
        }
        match declarations.len() {
            0 => Resolution::Unresolved("No declaration".into()),
            1 => Resolution::Resolved(declarations.into_iter().next().expect("one declaration")),
            _ if super::declaration_groups::mergeable(&declarations)
                || self.program_group_proven(&declarations) =>
            {
                Resolution::Merged(declarations.into_iter().collect())
            }
            _ => Resolution::Ambiguous(declarations.into_iter().collect()),
        }
    }

    pub(super) fn unit(&mut self, file: &Path) -> Option<Unit> {
        let file = self.host.normalized(file)?;
        if !self.units.contains_key(&file) {
            let unit = self
                .host
                .read(&file)
                .and_then(|source| Unit::parse(&file, &source));
            self.units.insert(file.to_path_buf(), unit);
        }
        self.units.get(&file).cloned().flatten()
    }

    fn local_set(
        &mut self,
        file: &Path,
        name: &str,
        namespace: Namespace,
        active: &mut BTreeSet<ExportVisit>,
    ) -> (BTreeSet<DeclarationId>, bool) {
        let Some(unit) = self.unit(file) else {
            return (BTreeSet::new(), false);
        };
        let Some(bindings) = unit.locals.get(name) else {
            return self.global_bindings(file, name, namespace, active);
        };
        let Ok(project) = self.request_project(file) else {
            return (BTreeSet::new(), false);
        };
        if project.config.is_some() {
            match self.program_source_external(&unit, &project) {
                Ok(false) => return self.global_bindings(file, name, namespace, active),
                Err(_) => return (BTreeSet::new(), false),
                _ => {}
            }
        }
        // Type and value spaces may legally contain different declarations.
        // Multiple contributors within one space still require merge authority;
        // never choose the first declaration as a substitute for that proof.
        let eligible: Vec<_> = bindings
            .iter()
            .filter(|binding| admits(binding, namespace))
            .collect();
        if eligible.is_empty() {
            if bindings
                .iter()
                .all(|binding| matches!(binding, Binding::Declaration(..)))
            {
                return self.global_bindings(file, name, namespace, active);
            }
            return (BTreeSet::new(), true);
        }
        if eligible.len() != 1 {
            let declarations: BTreeSet<_> = eligible
                .iter()
                .filter_map(|binding| match binding {
                    Binding::Declaration(id, _) => Some(id.clone()),
                    _ => None,
                })
                .collect();
            let complete = declarations.len() == eligible.len()
                && super::declaration_groups::mergeable(&declarations);
            return (declarations, complete);
        }
        self.binding(file, eligible[0], namespace, active)
    }

    pub(super) fn binding(
        &mut self,
        file: &Path,
        binding: &Binding,
        namespace: Namespace,
        active: &mut BTreeSet<ExportVisit>,
    ) -> (BTreeSet<DeclarationId>, bool) {
        if !admits(binding, namespace) {
            return (BTreeSet::new(), true);
        }
        match binding {
            Binding::Alias(path) => self.alias_declaration(
                &super::declared_scopes::NamespaceScope::module(file),
                path,
                namespace,
            ),
            Binding::Declaration(declaration, _) => (BTreeSet::from([declaration.clone()]), true),
            Binding::Import {
                module, imported, ..
            } if module.is_empty() => self.local_set(file, imported, namespace, active),
            Binding::Import {
                module,
                imported,
                mode,
                ..
            } => {
                let Ok(target) = self.module_scope(file, module, *mode) else {
                    return (BTreeSet::new(), false);
                };
                if target.blocks.is_empty() {
                    self.export_set(&target.file, imported, namespace, active)
                } else {
                    match self.scope_export(&target, imported, namespace) {
                        Resolution::Resolved(id) => (BTreeSet::from([id]), true),
                        Resolution::Merged(ids) | Resolution::Ambiguous(ids) => {
                            (ids.into_iter().collect(), true)
                        }
                        Resolution::Unresolved(_) => (BTreeSet::new(), false),
                    }
                }
            }
            Binding::Require { module, .. } => {
                let Ok(target) = self.module_request(
                    file,
                    module,
                    super::module_resolution::ModuleRequestKind::Require,
                ) else {
                    return (BTreeSet::new(), false);
                };
                self.assignment_set(&target, namespace, active)
            }
            Binding::Namespace { module, .. } => {
                // A module namespace is not an exported declaration. Resolve
                // the module witness but require member selection separately.
                let _ = self.module(file, module);
                (BTreeSet::new(), false)
            }
        }
    }

    fn assignment_set(
        &mut self,
        file: &Path,
        namespace: Namespace,
        active: &mut BTreeSet<ExportVisit>,
    ) -> (BTreeSet<DeclarationId>, bool) {
        let key = (file.to_path_buf(), ExportSlot::Assignment, namespace);
        if !active.insert(key.clone()) {
            return (BTreeSet::new(), true);
        }
        let result = if let Some(unit) = self.unit(file) {
            if unit.assignment.len() == 1 {
                self.binding(file, &unit.assignment[0], namespace, active)
            } else {
                (BTreeSet::new(), false)
            }
        } else {
            (BTreeSet::new(), false)
        };
        active.remove(&key);
        result
    }

    pub(super) fn export_set(
        &mut self,
        file: &Path,
        name: &str,
        namespace: Namespace,
        active: &mut BTreeSet<ExportVisit>,
    ) -> (BTreeSet<DeclarationId>, bool) {
        let key = (
            file.to_path_buf(),
            ExportSlot::Named(name.to_string()),
            namespace,
        );
        // No early-empty result is memoized: each rooted DFS explores all
        // reachable exports. Cyclic routes supply no new rooted declaration.
        if !active.insert(key.clone()) {
            return (BTreeSet::new(), true);
        }
        let result = (|| {
            let Some(unit) = self.unit(file) else {
                return (BTreeSet::new(), false);
            };
            let mut declarations = BTreeSet::new();
            let mut complete = true;
            if let Some(bindings) = unit.exports.get(name) {
                let eligible: Vec<_> = bindings
                    .iter()
                    .filter(|binding| admits(binding, namespace))
                    .collect();
                // Repeated explicit aliases are conflicting export bindings,
                // even when traversal would deduplicate their terminal target.
                // Written declaration groups are classified by outcome instead.
                if eligible.len() > 1
                    && eligible
                        .iter()
                        .any(|binding| !matches!(binding, Binding::Declaration(..)))
                {
                    return (BTreeSet::new(), false);
                }
                for binding in bindings {
                    let (found, closed) = self.binding(file, binding, namespace, active);
                    declarations.extend(found);
                    complete &= closed;
                }
            } else if name != "default" {
                for (module, type_only) in &unit.stars {
                    if *type_only && namespace == Namespace::Value {
                        continue;
                    }
                    match self.module(file, module) {
                        Ok(target) => {
                            let (found, closed) = self.export_set(&target, name, namespace, active);
                            declarations.extend(found);
                            complete &= closed;
                        }
                        Err(_) => complete = false,
                    }
                }
            }
            if namespace == Namespace::Type {
                complete &= self.augment_type(file, name, &mut declarations);
            }
            (declarations, complete)
        })();
        active.remove(&key);
        result
    }
}
