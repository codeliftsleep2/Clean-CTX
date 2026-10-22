//! Automatic type names are acquired from project policy, then canonically resolved.
use super::Resolver;
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct AutomaticTypeRequest {
    pub name: String,
    pub resolved: Result<PathBuf, String>,
}

impl Resolver {
    pub(super) fn automatic_type_sources(&mut self, file: &Path) -> Result<Vec<PathBuf>, String> {
        let project = self.request_project(file)?;
        let mut names = BTreeSet::new();
        if let Some(types) = project.options.get("types") {
            for name in types.value.as_array().ok_or("Invalid automatic types")? {
                names.insert(
                    name.as_str()
                        .ok_or("Invalid automatic type name")?
                        .to_string(),
                );
            }
        } else {
            let release = project
                .compiler_version
                .as_deref()
                .and_then(super::project_version::known_release);
            // Reviewed TS6 default is empty; explicit directives still resolve.
            if release == Some((6, 0)) {
                return Ok(Vec::new());
            }
            let mut roots = Vec::new();
            if let Some(option) = project.options.get("typeRoots") {
                for root in option.value.as_array().ok_or("Invalid typeRoots")? {
                    roots.push(
                        option
                            .directory
                            .join(root.as_str().ok_or("Invalid typeRoots entry")?),
                    );
                }
            } else {
                let boundary = self
                    .host
                    .boundary(file)
                    .ok_or("Type roots outside authorized scope")?;
                let mut directory = file.parent().ok_or("Type root parent absent")?;
                loop {
                    roots.push(directory.join("node_modules/@types"));
                    if directory == boundary {
                        break;
                    }
                    directory = directory.parent().ok_or("Type root boundary absent")?;
                }
            }
            for root in roots {
                if !self.host.directory(&root)? {
                    continue;
                }
                for entry in self.host.entries(&root)? {
                    let name = entry
                        .file_name()
                        .and_then(|name| name.to_str())
                        .ok_or("Nonlossless type package name")?;
                    if name.starts_with('.') || !self.host.directory(&entry)? {
                        continue;
                    }
                    names.insert(name.to_string());
                }
            }
            if !names.is_empty() && release != Some((5, 9)) {
                return Err("Automatic type default requires reviewed compiler evidence".into());
            }
        }
        let mut sources = Vec::new();
        for name in names {
            let resolved = self.type_directive(file, &name);
            self.automatic_type_evidence.push(AutomaticTypeRequest {
                name,
                resolved: resolved.clone(),
            });
            sources.push(resolved?);
        }
        Ok(sources)
    }
}
