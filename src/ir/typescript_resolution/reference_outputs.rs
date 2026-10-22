//! Explicit referenced-project output policy at the shared file-probe boundary.
use super::{
    Resolver,
    project::Project,
    project_config::{Config, OptionValue, document},
};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

fn option_path(option: &OptionValue) -> Result<PathBuf, String> {
    let value = option
        .value
        .as_str()
        .ok_or("Invalid referenced output path option")?;
    if value.contains("${") {
        return Err("Referenced output path template requires interpretation".into());
    }
    Ok(super::host::lexical(&option.directory.join(value)))
}

impl Resolver {
    pub(super) fn referenced_output(
        &mut self,
        source: &Path,
        project: &Project,
    ) -> Result<Option<PathBuf>, String> {
        match project
            .options
            .get("disableSourceOfProjectReferenceRedirect")
        {
            None => return Ok(None),
            Some(option) if option.value == serde_json::Value::Bool(false) => return Ok(None),
            Some(option) if option.value == serde_json::Value::Bool(true) => {}
            Some(_) => return Err("Invalid disableSourceOfProjectReferenceRedirect".into()),
        }
        let Some(config) = &project.config else {
            return Ok(None);
        };
        let name = source
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or("Non-UTF8 source filename")?;
        if name.ends_with(".d.ts") || name.ends_with(".d.mts") || name.ends_with(".d.cts") {
            return Ok(None);
        }
        let suffix = match source.extension().and_then(|extension| extension.to_str()) {
            Some("mts" | "mjs") => "d.mts",
            Some("cts" | "cjs") => "d.cts",
            Some("ts" | "tsx" | "js" | "jsx") => "d.ts",
            _ => return Ok(None),
        };
        let config_inputs = document(&mut self.host, config, &mut BTreeSet::new())?;
        let mut pending: Vec<_> = config_inputs
            .references
            .into_iter()
            .map(|path| (path, false))
            .collect();
        let mut active = BTreeSet::new();
        let mut seen = BTreeSet::new();
        let mut selected = None;
        while let Some((path, exiting)) = pending.pop() {
            let path = self
                .host
                .normalized(&path)
                .ok_or("Referenced config outside scope")?;
            if exiting {
                active.remove(&path);
                seen.insert(path);
                continue;
            }
            if active.contains(&path) {
                return Err("Cyclic output project references".into());
            }
            if seen.contains(&path) {
                continue;
            }
            active.insert(path.clone());
            pending.push((path.clone(), true));
            let mut config = document(&mut self.host, &path, &mut BTreeSet::new())?;
            config.options.retain(|_, option| !option.value.is_null());
            let directory = path.parent().ok_or("Referenced config directory absent")?;
            if super::project_membership::member(&mut self.host, &config, directory, source)? {
                if selected.is_some() {
                    return Err("Competing referenced source owners".into());
                }
                if config
                    .options
                    .get("composite")
                    .and_then(|option| option.value.as_bool())
                    != Some(true)
                {
                    return Err("Referenced output requires composite project authority".into());
                }
                selected = Some(self.declaration_output(source, suffix, directory, &config)?);
            }
            pending.extend(config.references.into_iter().map(|path| (path, false)));
        }
        Ok(selected)
    }

    fn declaration_output(
        &mut self,
        source: &Path,
        suffix: &str,
        directory: &Path,
        config: &Config,
    ) -> Result<PathBuf, String> {
        if config.options.contains_key("outFile") {
            return Err("Bundled referenced output requires conformance".into());
        }
        let root = config
            .options
            .get("rootDir")
            .map(option_path)
            .transpose()?
            .unwrap_or_else(|| directory.to_path_buf());
        let root = self
            .host
            .normalized(&root)
            .ok_or("Referenced rootDir outside scope")?;
        let relative = source
            .strip_prefix(&root)
            .map_err(|_| "Referenced source is outside rootDir")?;
        let output = config
            .options
            .get("declarationDir")
            .or_else(|| config.options.get("outDir"))
            .map(option_path)
            .transpose()?;
        let target = if let Some(output) = output {
            output.join(relative)
        } else {
            source.to_path_buf()
        }
        .with_extension(suffix);
        self.host
            .normalized(&target)
            .ok_or("Referenced declaration output outside authorized scope".into())
    }
}
