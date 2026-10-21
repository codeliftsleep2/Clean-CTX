use super::package::Ordered;
use super::project::Project;
use super::syntax::{Binding, Unit, admits};
use super::{DeclarationId, Host, Namespace};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) enum Resolution {
    Resolved(DeclarationId),
    Ambiguous(Vec<DeclarationId>),
    Unresolved(String),
}

pub(crate) struct Resolver {
    pub host: Host,
    units: BTreeMap<PathBuf, Option<Unit>>,
    projects: BTreeMap<PathBuf, Result<Project, String>>,
}

impl Resolver {
    pub(super) fn source_declarations(
        &mut self,
        file: &Path,
    ) -> Result<Vec<DeclarationId>, String> {
        let unit = self
            .unit(file)
            .ok_or("TypeScript source syntax has no supported binding unit")?;
        Ok(unit
            .locals
            .values()
            .flatten()
            .filter_map(|binding| match binding {
                Binding::Declaration(declaration, _) => Some(declaration.clone()),
                _ => None,
            })
            .collect())
    }

    pub(super) fn source_names(&mut self, file: &Path) -> Vec<String> {
        self.unit(file)
            .map(|unit| unit.locals.into_keys().collect())
            .unwrap_or_default()
    }

    pub(super) fn project_identity(
        &mut self,
        file: &Path,
    ) -> Result<(PathBuf, Option<PathBuf>, String), String> {
        let project = Project::discover(&mut self.host, file)?;
        Ok((project.root, project.config, project.mode))
    }
    pub(crate) fn new(roots: &[PathBuf]) -> Result<Self, String> {
        Ok(Self {
            host: Host::new(roots)?,
            units: BTreeMap::new(),
            projects: BTreeMap::new(),
        })
    }

    pub(crate) fn local(&mut self, file: &Path, name: &str, namespace: Namespace) -> Resolution {
        let Some(file) = self.host.normalized(file) else {
            return Resolution::Unresolved("Source outside scope".into());
        };
        let mut active = BTreeSet::new();
        let (declarations, complete) = self.local_set(&file, name, namespace, &mut active);
        Self::outcome(declarations, complete)
    }

    fn outcome(declarations: BTreeSet<DeclarationId>, complete: bool) -> Resolution {
        if !complete {
            return Resolution::Unresolved("Resolution has unproven contributors".into());
        }
        match declarations.len() {
            0 => Resolution::Unresolved("No declaration".into()),
            1 => Resolution::Resolved(declarations.into_iter().next().expect("one declaration")),
            _ => Resolution::Ambiguous(declarations.into_iter().collect()),
        }
    }

    fn unit(&mut self, file: &Path) -> Option<Unit> {
        if !self.units.contains_key(file) {
            let unit = self
                .host
                .read(file)
                .and_then(|source| Unit::parse(file, &source));
            self.units.insert(file.to_path_buf(), unit);
        }
        self.units.get(file).cloned().flatten()
    }

    fn local_set(
        &mut self,
        file: &Path,
        name: &str,
        namespace: Namespace,
        active: &mut BTreeSet<(PathBuf, String, Namespace)>,
    ) -> (BTreeSet<DeclarationId>, bool) {
        let Some(unit) = self.unit(file) else {
            return (BTreeSet::new(), false);
        };
        let Some(bindings) = unit.locals.get(name) else {
            return (BTreeSet::new(), true);
        };
        // Duplicate lexical bindings are not a valid way to choose a declaration.
        if bindings.len() != 1 {
            return (BTreeSet::new(), false);
        }
        self.binding(file, &bindings[0], namespace, active)
    }

    fn binding(
        &mut self,
        file: &Path,
        binding: &Binding,
        namespace: Namespace,
        active: &mut BTreeSet<(PathBuf, String, Namespace)>,
    ) -> (BTreeSet<DeclarationId>, bool) {
        if !admits(binding, namespace) {
            return (BTreeSet::new(), true);
        }
        match binding {
            Binding::Declaration(declaration, _) => (BTreeSet::from([declaration.clone()]), true),
            Binding::Import {
                module, imported, ..
            } if module.is_empty() => self.local_set(file, imported, namespace, active),
            Binding::Import {
                module, imported, ..
            } => {
                let Ok(target) = self.module(file, module) else {
                    return (BTreeSet::new(), false);
                };
                self.export_set(&target, imported, namespace, active)
            }
            Binding::Namespace { module, .. } => {
                // A module namespace is not an exported declaration. Resolve
                // the module witness but require member selection separately.
                let _ = self.module(file, module);
                (BTreeSet::new(), false)
            }
        }
    }

    fn export_set(
        &mut self,
        file: &Path,
        name: &str,
        namespace: Namespace,
        active: &mut BTreeSet<(PathBuf, String, Namespace)>,
    ) -> (BTreeSet<DeclarationId>, bool) {
        let key = (file.to_path_buf(), name.to_string(), namespace);
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
            (declarations, complete)
        })();
        active.remove(&key);
        result
    }

    pub(crate) fn module(&mut self, file: &Path, specifier: &str) -> Result<PathBuf, String> {
        let file = self.host.normalized(file).ok_or("Source outside scope")?;
        if !self.projects.contains_key(&file) {
            let project = Project::discover(&mut self.host, &file);
            self.projects.insert(file.clone(), project);
        }
        let project = self
            .projects
            .get(&file)
            .expect("inserted project")
            .clone()?;
        if !matches!(project.mode.as_str(), "bundler" | "node10" | "node") {
            return Err(format!(
                "{} resolution semantics pending conformance",
                project.mode
            ));
        }
        let directory = file.parent().ok_or("Source has no parent")?;
        if specifier.starts_with('.') {
            return self
                .probe(&directory.join(specifier))
                .ok_or_else(|| "Relative module absent".into());
        }
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
                if let Some(file) = self.probe(&target) {
                    return Ok(file);
                }
            }
        }
        if project.base != project.root
            && let Some(target) = self.probe(&project.base.join(specifier))
        {
            return Ok(target);
        }
        if specifier.starts_with('#') {
            let (_, manifest) = self
                .nearest_manifest(directory)
                .ok_or("No package imports owner")?;
            let owner = manifest.0;
            let json = manifest.1;
            let target = Self::mapped(
                json.get("imports").ok_or("No imports map")?,
                specifier,
                &project.conditions,
            )
            .ok_or("Package import not exported")?;
            if !target.starts_with("./") {
                return Err("External package imports targets pending conformance".into());
            }
            return self
                .probe(&owner.join(target))
                .ok_or_else(|| "Package imports target absent".into());
        }
        let pieces: Vec<_> = specifier.split('/').collect();
        let count = if specifier.starts_with('@') { 2 } else { 1 };
        if pieces.len() < count {
            return Err("Invalid package specifier".into());
        }
        let package = pieces[..count].join("/");
        let subpath = if pieces.len() == count {
            ".".to_string()
        } else {
            format!("./{}", pieces[count..].join("/"))
        };
        let boundary = self.host.boundary(&file).ok_or("Missing source scope")?;
        let mut ancestor = directory;
        loop {
            let owner = ancestor.join("node_modules").join(&package);
            let manifest = owner.join("package.json");
            if let Some(source) = self.host.read(&manifest) {
                let json = Ordered::parse(&source)?;
                if let Some(exports) = json.get("exports") {
                    let target = Self::mapped(exports, &subpath, &project.conditions)
                        .ok_or("Package export blocked")?;
                    if !target.starts_with("./") {
                        return Err("Invalid package export target".into());
                    }
                    let normalized = self
                        .host
                        .normalized(&owner.join(&target))
                        .ok_or("Export target outside scope")?;
                    if !normalized.starts_with(&owner) {
                        return Err("Package export escapes package".into());
                    }
                    return self
                        .probe(&normalized)
                        .ok_or_else(|| "Package export target absent".into());
                }
                if subpath != "." {
                    return self
                        .probe(&owner.join(subpath.trim_start_matches("./")))
                        .ok_or_else(|| "Package subpath absent".into());
                }
                for field in ["types", "typings", "main"] {
                    if let Some(target) = json.get(field).and_then(Ordered::string)
                        && let Some(file) = self.probe(&owner.join(target))
                    {
                        return Ok(file);
                    }
                }
                return self
                    .probe(&owner.join("index"))
                    .ok_or_else(|| "Package entry absent".into());
            }
            if ancestor == boundary {
                break;
            }
            ancestor = ancestor.parent().ok_or("Missing package boundary")?;
        }
        Err("Package absent".into())
    }

    fn probe(&mut self, path: &Path) -> Option<PathBuf> {
        if self.host.file(path) {
            return path.canonicalize().ok();
        }
        for extension in ["ts", "tsx", "d.ts", "mts", "cts", "js", "jsx"] {
            let candidate = PathBuf::from(format!("{}.{extension}", path.to_string_lossy()));
            if self.host.file(&candidate) {
                return candidate.canonicalize().ok();
            }
        }
        for extension in ["ts", "tsx", "d.ts", "js"] {
            let candidate = path.join(format!("index.{extension}"));
            if self.host.file(&candidate) {
                return candidate.canonicalize().ok();
            }
        }
        None
    }

    fn nearest_manifest(&mut self, directory: &Path) -> Option<(PathBuf, (PathBuf, Ordered))> {
        let boundary = self.host.boundary(directory)?;
        let mut directory = directory;
        loop {
            let path = directory.join("package.json");
            if let Some(source) = self.host.read(&path) {
                return Some((
                    path,
                    (directory.to_path_buf(), Ordered::parse(&source).ok()?),
                ));
            }
            if directory == boundary {
                return None;
            }
            directory = directory.parent()?;
        }
    }

    fn mapped(value: &Ordered, key: &str, conditions: &[String]) -> Option<String> {
        if let Ordered::Object(object) = value
            && object
                .iter()
                .any(|(name, _)| name.starts_with('.') || name.starts_with('#'))
        {
            if let Some(exact) = value.get(key) {
                return Self::condition(exact, conditions);
            }
            let mut patterns: Vec<_> = object
                .iter()
                .filter_map(|(pattern, target)| {
                    let (prefix, suffix) = pattern.split_once('*')?;
                    Some((
                        prefix.len(),
                        suffix.len(),
                        key.strip_prefix(prefix)?.strip_suffix(suffix)?.to_string(),
                        target,
                    ))
                })
                .collect();
            patterns.sort_by_key(|item| std::cmp::Reverse((item.0, item.1)));
            let (_, _, middle, target) = patterns.first()?;
            return Some(Self::condition(target, conditions)?.replace('*', middle));
        }
        (key == ".")
            .then(|| Self::condition(value, conditions))
            .flatten()
    }

    fn condition(value: &Ordered, conditions: &[String]) -> Option<String> {
        match value {
            Ordered::String(target) => Some(target.clone()),
            Ordered::Array(array) => array
                .iter()
                .find_map(|target| Self::condition(target, conditions)),
            Ordered::Object(entries) => entries
                .iter()
                .find(|(key, _)| key == "default" || conditions.contains(key))
                .and_then(|(_, target)| Self::condition(target, conditions)),
            Ordered::Other => None,
        }
    }
}
