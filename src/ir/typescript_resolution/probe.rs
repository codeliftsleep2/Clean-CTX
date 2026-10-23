//! File extension substitution and directory lookup share the consumed host.
use super::Resolver;
use super::package::Ordered;
use super::project::Project;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug)]
pub(super) enum FileExtensions {
    All,
    TypeScript,
    JavaScript,
    Declaration,
}

impl Resolver {
    pub(super) fn probe(&mut self, path: &Path, project: &Project) -> Option<PathBuf> {
        self.probe_directory(path, project, &mut BTreeSet::new())
            .ok()
            .flatten()
    }

    pub(super) fn probe_directory(
        &mut self,
        path: &Path,
        project: &Project,
        active: &mut BTreeSet<PathBuf>,
    ) -> Result<Option<PathBuf>, ()> {
        let path = self.project_path(path, project).ok_or(())?;
        if !active.insert(path.clone()) {
            return Ok(None);
        }
        let result = (|| {
            if let Some(file) = self.probe_file_result(&path, project)? {
                return Ok(Some(file));
            }
            let manifest = path.join("package.json");
            if let Some(source) = self.host.read(&manifest) {
                let json = Ordered::parse(&source).map_err(|_| ())?;
                let request = super::type_versions::entry_request(&json, ".");
                if let Some(target) = self
                    .type_version_target(&path, &json, &request, project)
                    .map_err(|_| ())?
                {
                    return Ok(target);
                }
                // Directory modules use legacy entry fields, not package exports.
                for field in ["types", "typings", "main"] {
                    if let Some(target) = json.get(field).and_then(Ordered::string)
                        && let Some(file) =
                            self.probe_directory(&path.join(target), project, active)?
                    {
                        return Ok(Some(file));
                    }
                }
            }
            self.probe_file_result(&path.join("index"), project)
        })();
        active.remove(&path);
        result
    }

    pub(super) fn probe_file(&mut self, path: &Path, project: &Project) -> Option<PathBuf> {
        self.probe_file_result(path, project).ok().flatten()
    }

    // Ok(None) permits ordinary fallback. Err blocks a selected/unproved target.
    pub(super) fn probe_file_result(
        &mut self,
        path: &Path,
        project: &Project,
    ) -> Result<Option<PathBuf>, ()> {
        let written = path.to_str().ok_or(())?;
        let (stem, extensions): (&str, &[&str]) = if let Some(stem) = written.strip_suffix(".d.mts")
        {
            (stem, &[".d.mts"])
        } else if let Some(stem) = written.strip_suffix(".d.cts") {
            (stem, &[".d.cts"])
        } else if let Some(stem) = written.strip_suffix(".d.ts") {
            (stem, &[".d.ts"])
        } else if let Some(stem) = written
            .strip_suffix(".js")
            .or_else(|| written.strip_suffix(".jsx"))
        {
            (stem, &[".ts", ".tsx", ".d.ts", ".js", ".jsx"])
        } else if let Some(stem) = written.strip_suffix(".mjs") {
            (stem, &[".mts", ".d.mts", ".mjs"])
        } else if let Some(stem) = written.strip_suffix(".cjs") {
            (stem, &[".cts", ".d.cts", ".cjs"])
        } else if let Some(stem) = written.strip_suffix(".tsx") {
            (stem, &[".tsx"])
        } else if let Some(stem) = written.strip_suffix(".mts") {
            (stem, &[".mts"])
        } else if let Some(stem) = written.strip_suffix(".cts") {
            (stem, &[".cts"])
        } else if let Some(stem) = written.strip_suffix(".ts") {
            (stem, &[".ts", ".tsx", ".d.ts"])
        } else {
            // Extensionless lookup never invents .mts/.cts candidates.
            (written, &[".ts", ".tsx", ".d.ts", ".js", ".jsx"])
        };
        let suffixes = match project.options.get("moduleSuffixes") {
            Some(option) => option
                .value
                .as_array()
                .ok_or(())?
                .iter()
                .map(|value| value.as_str())
                .collect::<Option<Vec<_>>>()
                .ok_or(())?,
            None => vec![""],
        };
        for extension in extensions {
            let declaration = extension.starts_with(".d.");
            let typescript = declaration || matches!(*extension, ".ts" | ".tsx" | ".mts" | ".cts");
            let admitted = match project.extensions {
                FileExtensions::All => true,
                FileExtensions::TypeScript => typescript,
                FileExtensions::JavaScript => !typescript,
                FileExtensions::Declaration => declaration,
            };
            if !admitted {
                continue;
            }
            for suffix in &suffixes {
                let candidate = PathBuf::from(format!("{stem}{suffix}{extension}"));
                if self.host.file(&candidate) {
                    let selected = self.project_path(&candidate, project).ok_or(())?;
                    match self.referenced_output(&selected, project) {
                        Ok(Some(output)) => {
                            return if self.host.file(&output) {
                                Ok(Some(output))
                            } else {
                                Err(())
                            };
                        }
                        Ok(None) => return Ok(Some(selected)),
                        Err(_) => return Err(()),
                    }
                }
            }
        }
        Ok(None)
    }
}
