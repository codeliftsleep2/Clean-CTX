//! Declaration packages share canonical export, version and file selection.
use super::{Resolver, project::Project};
use std::path::{Path, PathBuf};

impl Resolver {
    pub(super) fn type_reference_package(
        &mut self,
        owner: &Path,
        subpath: &str,
        project: &Project,
    ) -> Result<Option<PathBuf>, String> {
        let modern = matches!(project.mode.as_str(), "bundler" | "node16" | "nodenext");
        let exports_enabled = project
            .options
            .get("resolvePackageJsonExports")
            .and_then(|option| option.value.as_bool())
            .unwrap_or(modern);
        if exports_enabled {
            if let Some(json) = self.package_document(&owner.join("package.json"))? {
                if let Some(exports) = json
                    .get("exports")
                    .filter(|value| !matches!(value, super::package::Ordered::Null))
                {
                    let target = Self::mapped(exports, subpath, project)?
                        .ok_or("Type package export blocked")?;
                    return self.package_target(owner, &target, project).map(Some);
                }
            }
        }
        // Without an applicable exports map, use shared legacy package entry fields.
        if subpath == "." {
            if let Some(file) = self.probe_file(owner, project) {
                return Ok(Some(file));
            }
            if let Some(json) = self.package_document(&owner.join("package.json"))? {
                let request = super::type_versions::entry_request(&json, ".");
                if let Some(target) = self.type_version_target(owner, &json, &request, project)? {
                    return Ok(target);
                }
                for field in ["types", "typings", "main"] {
                    if let Some(target) = json.get(field).and_then(super::package::Ordered::string)
                        && let Some(file) = self.probe(&owner.join(target), project)
                    {
                        return Ok(Some(file));
                    }
                }
            }
            Ok(self.probe_file(&owner.join("index"), project))
        } else {
            Ok(self.probe(&owner.join(&subpath[2..]), project))
        }
    }
}
