//! Project-declared virtual roots affect lookup, never authorization.
use super::Resolver;
use super::host::lexical;
use super::project::Project;
use std::path::{Path, PathBuf};

impl Resolver {
    pub(super) fn relative_module(
        &mut self,
        directory: &Path,
        specifier: &str,
        project: &Project,
        directory_lookup: bool,
    ) -> Result<PathBuf, String> {
        let candidate = lexical(&directory.join(specifier));
        if let Some(file) = self.relative_candidate(&candidate, project, directory_lookup) {
            return Ok(file);
        }
        let Some(option) = project.options.get("rootDirs") else {
            return Err("Relative module absent".into());
        };
        let roots = option
            .value
            .as_array()
            .ok_or("Invalid rootDirs")?
            .iter()
            .map(|value| {
                let path = option
                    .directory
                    .join(value.as_str().ok_or("Invalid rootDirs entry")?);
                self.project_path(&path, project)
                    .ok_or("Virtual root outside authorized scope")
            })
            .collect::<Result<Vec<_>, _>>()?;
        let components = crate::workspace::path_identity::path_components(&candidate);
        let matched = roots
            .iter()
            .enumerate()
            .filter(|(_, root)| {
                let prefix = crate::workspace::path_identity::path_components(root);
                candidate.components().count() > root.components().count()
                    && crate::workspace::path_identity::components_are_within(&components, &prefix)
            })
            .max_by_key(|(_, root)| root.components().count());
        let Some((selected, root)) = matched else {
            return Err("Relative module absent from virtual roots".into());
        };
        let suffix: PathBuf = candidate
            .components()
            .skip(root.components().count())
            .collect();
        // These are compiler-declared ordered fallbacks, not competing symbol
        // candidates chosen by workspace discovery order.
        for (index, root) in roots.iter().enumerate() {
            if index == selected {
                continue;
            }
            if let Some(file) =
                self.relative_candidate(&root.join(&suffix), project, directory_lookup)
            {
                return Ok(file);
            }
        }
        Err("Relative module absent from virtual roots".into())
    }

    fn relative_candidate(
        &mut self,
        path: &Path,
        project: &Project,
        directory_lookup: bool,
    ) -> Option<PathBuf> {
        if directory_lookup {
            self.probe(path, project)
        } else {
            self.probe_file(path, project)
        }
    }
}
