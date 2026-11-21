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
    /// Requests proved absent by candidate inventory or an applicable export map.
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

    pub(super) fn program_units(
        &mut self,
        file: &Path,
    ) -> Result<super::shared_acquisition::ProgramUnits, String> {
        let scope = self.host.witnesses.begin();
        self.host.witnesses.roots();
        let result = self.program_units_scoped(file);
        self.host.witnesses.finish(scope);
        result
    }

    fn program_units_scoped(
        &mut self,
        file: &Path,
    ) -> Result<super::shared_acquisition::ProgramUnits, String> {
        use super::shared_acquisition::{ProjectProof, basis, reusable_sources};
        use std::sync::Arc;
        let project_scope = self.host.witnesses.begin();
        let project = self.request_project(file);
        let project_inputs = self.host.witnesses.finish(project_scope);
        let project = project?;
        let Some(config) = project.config.clone() else {
            return Ok(Arc::new(BTreeMap::new()));
        };
        if let Some((value, inputs)) = self.programs.get(&config) {
            self.host.witnesses.attribute(inputs);
            return value.clone();
        }
        if !self.program_active.insert(config.clone()) {
            return Err("Recursive program acquisition".into());
        }
        let operation = self.operation.clone();
        let slot = operation
            .as_ref()
            .filter(|_| self.program_active.len() == 1)
            .and_then(|operation| basis(self, file, &project).map(|basis| operation.slot(basis)));
        // No mutable Resolver is stored in the group. The active recursion guard precedes this lock.
        let mut slot = slot
            .as_ref()
            .map(|slot| slot.lock().unwrap_or_else(|poison| poison.into_inner()));
        let candidate = slot.as_ref().and_then(|slot| slot.as_ref().cloned());
        // Once sealed, qualification is parallel and never holds the group lock.
        if candidate.is_some() {
            drop(slot.take());
        }
        if let (Some(proof), Some(operation)) = (candidate, operation.as_ref())
            && let Some(qualified) = proof.qualify(self, operation)
        {
            self.host
                .witnesses
                .import(qualified.witnesses, &qualified.bound.inputs);
            self.program_evidence
                .insert(config.clone(), proof.program.clone());
            self.library_evidence
                .extend(proof.libraries.iter().cloned());
            self.automatic_type_evidence
                .extend(proof.types.iter().cloned());
            self.directive_evidence
                .extend(proof.directives.iter().cloned());
            self.consumed_entries.extend(proof.entries.iter().cloned());
            self.host.shared_inputs.push(qualified.bound.clone());
            self.attached_programs.push(qualified.bound.clone());
            self.program_active.remove(&config);
            let units = Arc::clone(&proof.units);
            self.programs
                .insert(config, (Ok(Arc::clone(&units)), qualified.bound.inputs));
            return Ok(units);
        }
        let scope = self.host.witnesses.begin();
        // Explicit project receipt can predate the program scope; a cache hit attributes it.
        self.host.witnesses.attribute(&project_inputs);
        self.host.witnesses.roots();
        let libraries = self.library_evidence.len();
        let types = self.automatic_type_evidence.len();
        let directives = self.directive_evidence.len();
        let entries = self.consumed_entries.len();
        let result = self.acquire_program_units(file, &project, &config);
        let inputs = self.host.witnesses.finish(scope);
        self.program_active.remove(&config);
        if let (Ok(units), Some(operation)) = (&result, operation.as_ref()) {
            operation.completed();
            if let Some(slot) = slot.as_mut()
                && slot.is_none()
                && let Some(sources) = reusable_sources(&inputs, operation)
            {
                // Failure/partial evidence is never installed in the cross-owner slot.
                **slot = Some(Arc::new(ProjectProof {
                    units: Arc::clone(units),
                    inputs: Arc::clone(&inputs),
                    sources: sources.all,
                    supplied_sources: sources.supplied,
                    program: self.program_evidence[&config].clone(),
                    libraries: self.library_evidence[libraries..].to_vec(),
                    types: self.automatic_type_evidence[types..].to_vec(),
                    directives: self.directive_evidence[directives..].to_vec(),
                    entries: self.consumed_entries[entries..].to_vec(),
                }));
            }
        }
        self.programs.insert(config, (result.clone(), inputs));
        result
    }

    fn acquire_program_units(
        &mut self,
        file: &Path,
        project: &super::project::Project,
        config: &Path,
    ) -> Result<super::shared_acquisition::ProgramUnits, String> {
        #[cfg(all(test, feature = "angular"))]
        crate::ops_r1_observation::record(6, 1);
        if let Some(operation) = &self.operation {
            operation.attempt();
        }
        (|| {
            let config_inputs =
                super::project_config::document(&mut self.host, config, &mut BTreeSet::new())?;
            let no_resolve = project
                .options
                .get("noResolve")
                .map(|option| option.value.as_bool().ok_or("Invalid noResolve"))
                .transpose()?
                .unwrap_or(false);
            let mut pending = self.program_roots(config, &config_inputs)?;
            pending.extend(self.automatic_type_sources(file)?);
            let mut units = BTreeMap::new();
            let mut module_requests: Vec<(PathBuf, super::module_references::ModuleReference)> =
                Vec::new();
            let mut missing_modules = Vec::new();
            let mut defaults_considered = false;
            let mut suppress_default = false;
            loop {
                #[cfg(all(test, feature = "angular"))]
                crate::ops_r1_observation::record(13, 1);
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
                #[cfg(all(test, feature = "angular"))]
                crate::ops_r1_observation::record(7, 1);
                if let Some(operation) = &self.operation {
                    operation.visit();
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
                .entry(config.to_path_buf())
                .or_insert(ProgramEvidence {
                    config: config.to_path_buf(),
                    missing_modules,
                    sources: units.keys().cloned().collect(),
                    groups: Vec::new(),
                    augmentations: Vec::new(),
                });
            Ok(std::sync::Arc::new(units))
        })()
    }

    pub(super) fn global_units(&mut self, file: &Path) -> Result<Vec<Unit>, String> {
        let project = self.request_project(file)?;
        let mut globals = Vec::new();
        for unit in self.program_units(file)?.values().cloned() {
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
        for unit in units.values().cloned() {
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
