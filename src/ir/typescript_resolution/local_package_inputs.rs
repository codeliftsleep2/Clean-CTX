//! Local package map outputs may identify current inputs, never dependency sources.
use super::{CompilerOption, Resolver, project::Project};
use std::path::{Path, PathBuf};

fn option_path(option: &CompilerOption) -> Result<PathBuf, String> {
    let value = option
        .value
        .as_str()
        .ok_or("Invalid local package output path option")?;
    if value.contains("${") {
        return Err("Local package output template requires interpretation".into());
    }
    Ok(super::host::lexical(&option.directory.join(value)))
}

impl Resolver {
    pub(super) fn local_package_input(
        &mut self,
        owner: &Path,
        target: &Path,
        project: &Project,
    ) -> Result<Option<PathBuf>, String> {
        let Some(config) = &project.config else {
            return Ok(None);
        };
        let owner = self
            .host
            .normalized(owner)
            .ok_or("Local package owner outside scope")?;
        if owner.components().any(|part| {
            part.as_os_str()
                .to_str()
                .is_some_and(|part| part.eq_ignore_ascii_case("node_modules"))
        }) {
            return Ok(None);
        }
        let config = self
            .host
            .normalized(config)
            .ok_or("Local package config outside scope")?;
        let directory = config
            .parent()
            .ok_or("Local package config directory absent")?;
        if !directory.starts_with(&owner) {
            return Ok(None);
        }
        let written = target.to_str().ok_or("Non-UTF8 local package target")?;
        let allow_js = project
            .options
            .get("allowJs")
            .and_then(|option| option.value.as_bool())
            .unwrap_or_else(|| {
                project
                    .options
                    .get("checkJs")
                    .and_then(|option| option.value.as_bool())
                    == Some(true)
            });
        let (stem, extensions, declaration): (&str, &[&str], bool) =
            if let Some(stem) = written.strip_suffix(".d.mts") {
                (
                    stem,
                    if allow_js {
                        &[".mts", ".mjs"]
                    } else {
                        &[".mts"]
                    },
                    true,
                )
            } else if let Some(stem) = written.strip_suffix(".d.cts") {
                (
                    stem,
                    if allow_js {
                        &[".cts", ".cjs"]
                    } else {
                        &[".cts"]
                    },
                    true,
                )
            } else if let Some(stem) = written.strip_suffix(".d.ts") {
                (
                    stem,
                    if allow_js {
                        &[".ts", ".tsx", ".js", ".jsx"]
                    } else {
                        &[".ts", ".tsx"]
                    },
                    true,
                )
            } else if let Some(stem) = written.strip_suffix(".mjs") {
                (
                    stem,
                    if allow_js {
                        &[".mts", ".mjs"]
                    } else {
                        &[".mts"]
                    },
                    false,
                )
            } else if let Some(stem) = written.strip_suffix(".cjs") {
                (
                    stem,
                    if allow_js {
                        &[".cts", ".cjs"]
                    } else {
                        &[".cts"]
                    },
                    false,
                )
            } else if let Some(stem) = written
                .strip_suffix(".js")
                .or_else(|| written.strip_suffix(".jsx"))
            {
                (
                    stem,
                    if allow_js {
                        &[".ts", ".tsx", ".js", ".jsx"]
                    } else {
                        &[".ts", ".tsx"]
                    },
                    false,
                )
            } else {
                return Ok(None);
            };
        let mut outputs = Vec::new();
        if declaration && let Some(option) = project.options.get("declarationDir") {
            outputs.push(option_path(option)?);
        }
        if let Some(option) = project.options.get("outDir") {
            outputs.push(option_path(option)?);
        }
        if outputs.is_empty() {
            return Ok(None);
        }
        let stem = self
            .host
            .normalized(Path::new(stem))
            .ok_or("Local package output outside scope")?;
        for output in outputs {
            let output = self
                .host
                .normalized(&output)
                .ok_or("Local package output directory outside scope")?;
            let Ok(relative) = stem.strip_prefix(&output) else {
                continue;
            };
            let root = if let Some(option) = project.options.get("rootDir") {
                option_path(option)?
            } else if project
                .options
                .get("composite")
                .and_then(|option| option.value.as_bool())
                == Some(true)
            {
                directory.to_path_buf()
            } else {
                return Err("Local package remapping requires proved rootDir".into());
            };
            let root = self
                .host
                .normalized(&root)
                .ok_or("Local package source root outside scope")?;
            let base = root.join(relative);
            for extension in extensions {
                let input = PathBuf::from(format!(
                    "{}{extension}",
                    base.to_str().ok_or("Non-UTF8 local source input")?
                ));
                if self.host.file(&input) {
                    // As in the compiler, establish the possible original input
                    // first, then run the shared suffix/extension loader.
                    if let Some(selected) = self.probe_file(&input, project) {
                        return Ok(Some(selected));
                    }
                }
            }
        }
        Ok(None)
    }
}
