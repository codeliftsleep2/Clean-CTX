//! Project-owned contributors and merge evidence, never a workspace name scan.
use super::{
    DeclarationId, Namespace, Resolver,
    syntax::{Binding, Unit},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct ProgramEvidence {
    pub config: PathBuf,
    /// Requests whose relative candidate inventory was proved absent.
    #[serde(default)]
    pub missing_modules: Vec<(PathBuf, super::module_references::ModuleReference)>,
    pub sources: Vec<PathBuf>,
    pub groups: Vec<Vec<DeclarationId>>,
    pub augmentations: Vec<AugmentationEvidence>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct AugmentationEvidence {
    pub source: PathBuf,
    pub specifier: String,
    pub target: PathBuf,
    pub declarations: Vec<DeclarationId>,
}

impl Resolver {
    fn program_roots(
        &mut self,
        config: &Path,
        inputs: &super::project_config::Config,
    ) -> Result<Vec<PathBuf>, String> {
        let directory = config.parent().ok_or("Config parent absent")?;
        let mut roots = BTreeSet::new();
        if let Some(files) = inputs.roots.get("files") {
            for entry in files.value.as_array().ok_or("Invalid program files")? {
                roots.insert(
                    files
                        .directory
                        .join(entry.as_str().ok_or("Invalid program root")?),
                );
            }
        }
        if !inputs.roots.contains_key("files") || inputs.roots.contains_key("include") {
            let mut pending = vec![directory.to_path_buf()];
            if let Some(includes) = inputs.roots.get("include") {
                pending.clear();
                for pattern in includes.value.as_array().ok_or("Invalid program include")? {
                    let pattern = pattern.as_str().ok_or("Invalid program pattern")?;
                    let mut base = includes.directory.clone();
                    for part in pattern.replace('\\', "/").split('/') {
                        if part.contains(['*', '?']) {
                            break;
                        }
                        base.push(part);
                    }
                    if base.extension().is_some() {
                        if let Some(parent) = base.parent() {
                            base = parent.to_path_buf();
                        }
                    }
                    pending.push(base);
                }
            }
            let mut seen = BTreeSet::new();
            while let Some(directory_path) = pending.pop() {
                let physical = self
                    .host
                    .normalized(&directory_path)
                    .ok_or("Program inventory outside scope")?;
                if !seen.insert(physical.clone()) {
                    continue;
                }
                for entry in self.host.entries(&physical)? {
                    let physical = self
                        .host
                        .normalized(&entry)
                        .ok_or("Program entry outside scope")?;
                    let is_file = self.host.file(&physical);
                    if !is_file && physical.is_dir() {
                        let name = entry
                            .file_name()
                            .and_then(|name| name.to_str())
                            .ok_or("Nonlossless program path")?;
                        if name.starts_with('.')
                            || matches!(name, "node_modules" | "bower_components" | "jspm_packages")
                        {
                            continue;
                        }
                        pending.push(entry);
                    } else if is_file
                        && super::project_membership::member(
                            &mut self.host,
                            inputs,
                            directory,
                            &physical,
                        )?
                    {
                        roots.insert(physical);
                    }
                }
            }
        }
        Ok(roots.into_iter().collect())
    }

    pub(super) fn program_units(&mut self, file: &Path) -> Result<Vec<Unit>, String> {
        let project = self.request_project(file)?;
        let Some(config) = project.config else {
            return Ok(Vec::new());
        };
        if let Some(units) = self.programs.get(&config) {
            return units.clone();
        }
        if !self.program_active.insert(config.clone()) {
            return Err("Recursive program acquisition".into());
        }
        let result = (|| {
            let config_inputs =
                super::project_config::document(&mut self.host, &config, &mut BTreeSet::new())?;
            let no_resolve = project
                .options
                .get("noResolve")
                .map(|option| option.value.as_bool().ok_or("Invalid noResolve"))
                .transpose()?
                .unwrap_or(false);
            let mut pending = self.program_roots(&config, &config_inputs)?;
            pending.extend(self.automatic_type_sources(file)?);
            let mut units = BTreeMap::new();
            let mut module_requests: Vec<(PathBuf, super::module_references::ModuleReference)> =
                Vec::new();
            let mut missing_modules = Vec::new();
            let mut defaults_considered = false;
            let mut suppress_default = false;
            loop {
                if pending.is_empty() && module_requests.is_empty() && !defaults_considered {
                    defaults_considered = true;
                    if !suppress_default {
                        pending.extend(self.initial_libraries(file)?);
                    }
                }
                if pending.is_empty() {
                    if module_requests.is_empty() {
                        break;
                    }
                    let acquired: Vec<_> = units.values().cloned().collect();
                    for (owner, request) in std::mem::take(&mut module_requests) {
                        if !request.mode_valid {
                            return Err(request
                                .resolved
                                .err()
                                .unwrap_or_else(|| "Invalid request mode".into()));
                        }
                        let ambient =
                            self.ambient_scope_in_units(&owner, &request.specifier, &acquired)?;
                        if ambient.as_ref().is_some_and(|scope| {
                            scope.blocks.iter().all(|id| id.name == request.specifier)
                        }) {
                            continue;
                        }
                        match self.module_request_target(&owner, &request.specifier, request.kind) {
                            Ok(super::module_resolution::ModuleTarget::Found(target)) => {
                                pending.push(target)
                            }
                            Ok(super::module_resolution::ModuleTarget::Absent(_))
                                if ambient.is_some() => {}
                            Ok(super::module_resolution::ModuleTarget::Absent(reason)) => {
                                let mut request = request;
                                request.resolved = Err(reason.into());
                                missing_modules.push((owner, request));
                            }
                            Err(_) if ambient.is_some() => {}
                            Err(error) => return Err(error),
                        }
                    }
                    if pending.is_empty() {
                        continue;
                    }
                }
                let path = pending.pop().expect("pending program source");
                let path = self
                    .host
                    .normalized(&path)
                    .ok_or("Program source outside scope")?;
                if units.contains_key(&path) {
                    continue;
                }
                let unit = self
                    .unit(&path)
                    .ok_or("Program source unavailable or invalid")?;
                let source = self
                    .host
                    .read(&path)
                    .ok_or("Program snapshot unavailable")?;
                let directives = self.directive_requests(&path, &source, no_resolve)?;
                self.directive_evidence.extend(directives.clone());
                units.insert(path.clone(), unit);
                for directive in directives {
                    if matches!(
                        directive.kind,
                        super::program_directives::DirectiveKind::NoDefaultLibrary
                    ) {
                        suppress_default = true;
                        continue;
                    }
                    if !directive.ignored {
                        pending.push(directive.resolved?);
                    }
                }
                if !no_resolve {
                    // Imports, including side effects, extend the admitted program.
                    let requests =
                        super::module_references::capture_deferred(self, &path, &source)?;
                    module_requests
                        .extend(requests.into_iter().map(|request| (path.clone(), request)));
                }
            }
            self.program_evidence
                .entry(config.clone())
                .or_insert(ProgramEvidence {
                    config: config.clone(),
                    missing_modules,
                    sources: units.keys().cloned().collect(),
                    groups: Vec::new(),
                    augmentations: Vec::new(),
                });
            Ok(units.into_values().collect())
        })();
        self.program_active.remove(&config);
        self.programs.insert(config, result.clone());
        result
    }

    pub(super) fn global_units(&mut self, file: &Path) -> Result<Vec<Unit>, String> {
        let project = self.request_project(file)?;
        let mut globals = Vec::new();
        for unit in self.program_units(file)? {
            if self.program_source_external(&unit, &project)? {
                globals.extend(unit.ambient_globals);
            } else {
                globals.push(unit);
            }
        }
        Ok(globals)
    }

    pub(super) fn prove_program_namespaces(
        &mut self,
        file: &Path,
        ids: &BTreeSet<DeclarationId>,
    ) -> bool {
        let proven = self.prove_program_binding_group(file, ids, Namespace::Value);
        if proven {
            self.record_program_group(file, ids);
        }
        proven
    }

    pub(super) fn record_program_group(&mut self, file: &Path, ids: &BTreeSet<DeclarationId>) {
        if let Ok(project) = self.request_project(file)
            && let Some(config) = project.config
            && let Some(evidence) = self.program_evidence.get_mut(&config)
        {
            let group: Vec<_> = ids.iter().cloned().collect();
            if !evidence.groups.contains(&group) {
                evidence.groups.push(group);
            }
        }
    }

    pub(super) fn augment_type(
        &mut self,
        file: &Path,
        name: &str,
        ids: &mut BTreeSet<DeclarationId>,
    ) -> bool {
        if ids.is_empty() {
            return true;
        }
        let Ok(units) = self.program_units(file) else {
            return false;
        };
        let Some(physical) = self.host.normalized(file) else {
            return false;
        };
        let original = ids.clone();
        for unit in units {
            let Ok(project) = self.request_project(file) else {
                return false;
            };
            match self.program_source_external(&unit, &project) {
                Ok(true) => {}
                Ok(false) => continue,
                Err(_) => return false,
            }
            let Some(owner) = unit.source else {
                return false;
            };
            for (specifier, body) in unit.augmentations {
                let Ok(target) = self.module(&owner, &specifier) else {
                    return false;
                };
                if self.host.normalized(&target).as_ref() != Some(&physical) {
                    continue;
                }
                if let Some(bindings) = body.locals.get(name) {
                    let mut contributors = Vec::new();
                    for binding in bindings {
                        match binding {
                            Binding::Declaration(id, _) if id.kind == "interface_declaration" => {
                                ids.insert(id.clone());
                                contributors.push(id.clone());
                            }
                            _ => return false,
                        }
                    }
                    if let Ok(project) = self.request_project(file)
                        && let Some(config) = project.config
                        && let Some(evidence) = self.program_evidence.get_mut(&config)
                    {
                        evidence.augmentations.push(AugmentationEvidence {
                            source: owner.clone(),
                            specifier,
                            target: physical.clone(),
                            declarations: contributors,
                        });
                    }
                }
            }
        }
        if ids != &original {
            if ids.iter().any(|id| {
                !matches!(
                    id.kind.as_str(),
                    "class_declaration" | "abstract_class_declaration" | "interface_declaration"
                )
            }) {
                return false;
            }
            if ids
                .iter()
                .filter(|id| id.kind != "interface_declaration")
                .count()
                > 1
            {
                return false;
            }
            self.record_program_group(file, ids);
        }
        true
    }

    pub(super) fn program_group_proven(&self, ids: &BTreeSet<DeclarationId>) -> bool {
        let group: Vec<_> = ids.iter().cloned().collect();
        self.program_evidence
            .values()
            .any(|evidence| evidence.groups.contains(&group))
    }
}
