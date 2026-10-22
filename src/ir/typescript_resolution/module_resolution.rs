use super::package::Ordered;
use super::resolver::Resolver;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) enum ModuleRequestKind {
    StaticImport,
    Import,
    Require,
}

impl Resolver {
    pub(crate) fn module(&mut self, file: &Path, specifier: &str) -> Result<PathBuf, String> {
        self.module_request(file, specifier, ModuleRequestKind::StaticImport)
    }

    pub(super) fn module_request(
        &mut self,
        file: &Path,
        specifier: &str,
        kind: ModuleRequestKind,
    ) -> Result<PathBuf, String> {
        let file = self.resolution_file(file).ok_or("Source outside scope")?;
        let mut project = self.request_project(&file)?;
        if !matches!(
            project.mode.as_str(),
            "bundler" | "node10" | "node" | "node16" | "nodenext" | "classic"
        ) {
            return Err(format!(
                "{} resolution semantics pending conformance",
                project.mode
            ));
        }
        let directory = file.parent().ok_or("Source has no parent")?;
        if project.mode == "classic" {
            return self.classic_module(directory, specifier, &project);
        }
        let node = matches!(project.mode.as_str(), "node16" | "nodenext");
        let esm = self.apply_request_conditions(&file, kind, &mut project)?;
        let modern = matches!(project.mode.as_str(), "bundler" | "node16" | "nodenext");
        let exports_enabled = project
            .options
            .get("resolvePackageJsonExports")
            .and_then(|option| option.value.as_bool())
            .unwrap_or(modern);
        let imports_enabled = project
            .options
            .get("resolvePackageJsonImports")
            .and_then(|option| option.value.as_bool())
            .unwrap_or(modern);
        if specifier.starts_with('.') {
            if node && esm && Path::new(specifier).extension().is_none() {
                return Err("Node ESM relative references require an explicit extension".into());
            }
            return self.relative_module(directory, specifier, &project, !(node && esm));
        }
        if let Some(target) = self.optional_module(specifier, &project, true) {
            return Ok(target);
        }
        if specifier.starts_with('#') && imports_enabled {
            return self.package_import(
                directory,
                specifier,
                &project,
                &mut std::collections::BTreeSet::new(),
            );
        }
        self.package_reference(directory, specifier, &project, exports_enabled)
    }

    pub(super) fn apply_request_conditions(
        &mut self,
        file: &Path,
        kind: ModuleRequestKind,
        project: &mut super::project::Project,
    ) -> Result<bool, String> {
        let node = matches!(project.mode.as_str(), "node16" | "nodenext");
        let esm = if kind == ModuleRequestKind::Import {
            true
        } else if kind == ModuleRequestKind::Require {
            false
        } else if node {
            self.source_is_esm(file)?
        } else {
            false
        };
        if node || project.mode == "bundler" {
            let require = kind == ModuleRequestKind::Require
                || (kind == ModuleRequestKind::StaticImport
                    && if node {
                        !esm
                    } else {
                        project
                            .options
                            .get("module")
                            .and_then(|option| option.value.as_str())
                            .is_some_and(|module| module.eq_ignore_ascii_case("commonjs"))
                    });
            project.conditions = vec![
                "types".into(),
                if require { "require" } else { "import" }.into(),
            ];
            if node {
                project.conditions.push("node".into());
            }
            if let Some(option) = project.options.get("customConditions") {
                for condition in option.value.as_array().ok_or("Invalid customConditions")? {
                    project
                        .conditions
                        .push(condition.as_str().ok_or("Invalid custom condition")?.into());
                }
            }
        }
        Ok(esm)
    }

    pub(super) fn optional_module(
        &mut self,
        specifier: &str,
        project: &super::project::Project,
        directory_lookup: bool,
    ) -> Option<PathBuf> {
        let mut mappings: Vec<_> = project
            .paths
            .iter()
            .filter_map(|(pattern, targets)| {
                if pattern == specifier {
                    return Some((usize::MAX, String::new(), targets));
                }
                let (prefix, suffix) = pattern.split_once('*')?;
                let middle = specifier.strip_prefix(prefix)?.strip_suffix(suffix)?;
                Some((prefix.len(), middle.to_string(), targets))
            })
            .collect();
        mappings.sort_by(|left, right| right.0.cmp(&left.0));
        if let Some((_, middle, targets)) = mappings.first() {
            for target in *targets {
                let target = PathBuf::from(target.to_string_lossy().replace('*', middle));
                if let Some(file) = self.optional_candidate(&target, project, directory_lookup) {
                    return Some(file);
                }
            }
        }
        if project.options.contains_key("baseUrl")
            && let Some(target) =
                self.optional_candidate(&project.base.join(specifier), project, directory_lookup)
        {
            return Some(target);
        }
        None
    }

    fn optional_candidate(
        &mut self,
        path: &Path,
        project: &super::project::Project,
        directory_lookup: bool,
    ) -> Option<PathBuf> {
        if directory_lookup {
            self.probe(path, project)
        } else {
            self.probe_file(path, project)
        }
    }

    pub(super) fn nearest_manifest(
        &mut self,
        directory: &Path,
    ) -> Result<Option<(PathBuf, (PathBuf, Ordered))>, String> {
        let boundary = self
            .host
            .boundary(directory)
            .ok_or("Package scope outside authorized roots")?;
        let mut directory = directory;
        loop {
            let path = directory.join("package.json");
            if let Some(json) = self.package_document(&path)? {
                return Ok(Some((path, (directory.to_path_buf(), json))));
            }
            if directory == boundary {
                return Ok(None);
            }
            directory = directory.parent().ok_or("Missing package boundary")?;
        }
    }
}
