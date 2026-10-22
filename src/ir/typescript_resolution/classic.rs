//! Classic lookup shares snapshots/probes, but does not use Node directory imports.
use super::probe::FileExtensions;
use super::project::Project;
use super::{Resolver, host::lexical};
use std::path::{Path, PathBuf};

impl Resolver {
    pub(super) fn classic_module(
        &mut self,
        directory: &Path,
        specifier: &str,
        project: &Project,
    ) -> Result<PathBuf, String> {
        let boundary = self
            .host
            .boundary(directory)
            .ok_or("Classic source outside scope")?;
        let relative = specifier.starts_with('.') || Path::new(specifier).is_absolute();
        for extensions in [FileExtensions::TypeScript, FileExtensions::JavaScript] {
            let mut phase = project.clone();
            phase.extensions = extensions;
            if relative {
                if let Ok(file) = self.relative_module(directory, specifier, &phase, false) {
                    return Ok(file);
                }
                continue;
            }
            if let Some(file) = self.optional_module(specifier, &phase, false) {
                return Ok(file);
            }
            let mut ancestor = directory;
            loop {
                if let Some(file) = self.probe_file(&lexical(&ancestor.join(specifier)), &phase) {
                    return Ok(file);
                }
                if ancestor == boundary {
                    break;
                }
                ancestor = ancestor.parent().ok_or("Missing Classic boundary")?;
            }
            if matches!(extensions, FileExtensions::TypeScript) {
                let (package, subpath) = super::package_lookup::package_parts(specifier)?;
                let mangled = package
                    .strip_prefix('@')
                    .unwrap_or(&package)
                    .replace('/', "__");
                let mut ancestor = directory;
                loop {
                    let owner = ancestor.join("node_modules").join("@types").join(&mangled);
                    if let Some(file) = self.type_reference_package(&owner, &subpath, &phase)? {
                        return Ok(file);
                    }
                    if ancestor == boundary {
                        break;
                    }
                    ancestor = ancestor.parent().ok_or("Missing Classic type boundary")?;
                }
                if let Some(option) = project.options.get("typeRoots") {
                    let roots = option.value.as_array().ok_or("Invalid typeRoots")?;
                    let mut declarations = phase.clone();
                    declarations.extensions = FileExtensions::Declaration;
                    for root in roots {
                        let owner = option
                            .directory
                            .join(root.as_str().ok_or("Invalid typeRoots entry")?)
                            .join(&package);
                        if let Some(file) =
                            self.type_reference_package(&owner, &subpath, &declarations)?
                        {
                            return Ok(file);
                        }
                    }
                }
            }
        }
        Err("Classic module absent".into())
    }
}
