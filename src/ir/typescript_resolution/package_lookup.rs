//! Package targets use the same project, source host and export authority.
use super::Resolver;
use super::host::lexical;
use super::package::Ordered;
use super::project::Project;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

impl Resolver {
    pub(super) fn package_document(&mut self, path: &Path) -> Result<Option<Ordered>, String> {
        if let Some(source) = self.host.read(path) {
            return Ordered::parse(&source).map(Some);
        }
        if self.host.file(path)
            || matches!(
                self.host.witnesses.last(),
                Some(
                    super::Witness::Unreadable { .. }
                        | super::Witness::OutsideScope { .. }
                        | super::Witness::Kind {
                            directory: true,
                            ..
                        }
                )
            )
        {
            return Err("Package manifest is unreadable or is not a file".into());
        }
        Ok(None)
    }

    pub(super) fn package_import(
        &mut self,
        directory: &Path,
        specifier: &str,
        project: &Project,
        active: &mut BTreeSet<(PathBuf, String)>,
    ) -> Result<PathBuf, String> {
        if specifier == "#" || specifier.starts_with("#/") {
            return Err("Invalid package import specifier".into());
        }
        let (_, (owner, json)) = self
            .nearest_manifest(directory)?
            .ok_or("No package imports owner")?;
        let key = (owner.clone(), specifier.to_string());
        if !active.insert(key.clone()) {
            return Err("Cyclic package imports mapping".into());
        }
        let result = (|| {
            let target = Self::mapped(
                json.get("imports").ok_or("No imports map")?,
                specifier,
                project,
            )?
            .ok_or("Package import not exported")?;
            if target.starts_with("./") {
                return self.package_target(&owner, &target, project);
            }
            if target.starts_with('#') {
                return self.package_import(&owner, &target, project, active);
            }
            let exports_enabled = project
                .options
                .get("resolvePackageJsonExports")
                .and_then(|option| option.value.as_bool())
                .unwrap_or(matches!(
                    project.mode.as_str(),
                    "bundler" | "node16" | "nodenext"
                ));
            self.package_reference(&owner, &target, project, exports_enabled)
        })();
        active.remove(&key);
        result
    }

    pub(super) fn package_reference(
        &mut self,
        directory: &Path,
        specifier: &str,
        project: &Project,
        exports_enabled: bool,
    ) -> Result<PathBuf, String> {
        let (package, subpath) = package_parts(specifier)?;
        if exports_enabled
            && let Some((_, (owner, json))) = self.nearest_manifest(directory)?
            && json.get("name").and_then(Ordered::string) == Some(package.as_str())
            && let Some(exports) = json
                .get("exports")
                .filter(|value| !matches!(value, Ordered::Null))
        {
            // A matching package self-reference is authoritative, including
            // blocked exports. Do not fall through to another installed copy.
            let target = Self::mapped(exports, &subpath, project)?.ok_or("Self export blocked")?;
            return self.package_target(&owner, &target, project);
        }
        let boundary = self
            .host
            .boundary(directory)
            .ok_or("Missing package scope")?;
        let mut ancestor = directory;
        loop {
            let owner = ancestor.join("node_modules").join(&package);
            if let Some(json) = self.package_document(&owner.join("package.json"))? {
                if let Some(exports) = json
                    .get("exports")
                    .filter(|value| exports_enabled && !matches!(value, Ordered::Null))
                {
                    let target = Self::mapped(exports, &subpath, project)?
                        .ok_or("Package export blocked")?;
                    return self.package_target(&owner, &target, project);
                }
                let request = super::type_versions::entry_request(&json, &subpath);
                if let Some(target) = self.type_version_target(&owner, &json, &request, project)? {
                    return target.ok_or_else(|| "Versioned package target absent".into());
                }
                if subpath != "." {
                    return self
                        .probe(&owner.join(&subpath[2..]), project)
                        .ok_or_else(|| "Package subpath absent".into());
                }
                for field in ["types", "typings", "main"] {
                    if let Some(target) = json.get(field).and_then(Ordered::string)
                        && let Some(file) = self.probe(&owner.join(target), project)
                    {
                        return Ok(file);
                    }
                }
                return self
                    .probe(&owner.join("index"), project)
                    .ok_or_else(|| "Package entry absent".into());
            }
            let candidate = if subpath == "." {
                owner
            } else {
                owner.join(&subpath[2..])
            };
            if let Some(file) = self.probe(&candidate, project) {
                return Ok(file);
            }
            if ancestor == boundary {
                break;
            }
            ancestor = ancestor.parent().ok_or("Missing package boundary")?;
        }
        Err("Package absent".into())
    }

    pub(super) fn package_target(
        &mut self,
        owner: &Path,
        target: &str,
        project: &Project,
    ) -> Result<PathBuf, String> {
        let suffix = target
            .strip_prefix("./")
            .ok_or("Invalid package-local target")?;
        if suffix
            .split(['/', '\\'])
            .any(|part| matches!(part, "." | "..") || part.eq_ignore_ascii_case("node_modules"))
            || suffix.contains('%')
            || suffix.contains('\\')
            || suffix.contains(':')
            || suffix.starts_with('/')
        {
            return Err("Invalid or unresolved encoded package target".into());
        }
        let candidate = lexical(&owner.join(suffix));
        if let Some(input) = self.local_package_input(owner, &candidate, project)? {
            return Ok(input);
        }
        // Export/import maps identify files, not directory-module entry points.
        // The Host validates physical destinations before any source read.
        self.probe_file(&candidate, project)
            .ok_or_else(|| "Package target absent".into())
    }
}

pub(super) fn package_parts(specifier: &str) -> Result<(String, String), String> {
    if specifier.is_empty()
        || specifier.starts_with(['.', '/', '\\', '#'])
        || specifier.contains(['\\', ':', '%'])
    {
        return Err("Invalid or unresolved package specifier".into());
    }
    let pieces: Vec<_> = specifier.split('/').collect();
    let count = if specifier.starts_with('@') { 2 } else { 1 };
    if pieces.len() < count
        || pieces
            .iter()
            .any(|piece| piece.is_empty() || matches!(*piece, "." | ".."))
    {
        return Err("Invalid package specifier".into());
    }
    Ok((
        pieces[..count].join("/"),
        if pieces.len() == count {
            ".".into()
        } else {
            format!("./{}", pieces[count..].join("/"))
        },
    ))
}
