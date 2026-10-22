//! Config inheritance retains each option's declaring directory before merging.
use super::Host;
use super::project::jsonc;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct OptionValue {
    pub value: Value,
    pub directory: PathBuf,
}

#[derive(Clone, Debug, Default)]
pub(super) struct Config {
    pub options: BTreeMap<String, OptionValue>,
    pub roots: BTreeMap<String, OptionValue>,
    // References belong to this config; TypeScript does not inherit them.
    pub references: Vec<PathBuf>,
}

const TEMPLATE_OPTIONS: &[&str] = &[
    "baseUrl",
    "paths",
    "rootDir",
    "rootDirs",
    "outDir",
    "declarationDir",
    "typeRoots",
];

pub(super) fn has_config_dir(config: &Config) -> bool {
    fn contains(value: &Value) -> bool {
        match value {
            Value::String(value) => starts_config_dir(value),
            Value::Array(values) => values.iter().any(contains),
            Value::Object(values) => values.values().any(contains),
            _ => false,
        }
    }
    config.roots.values().any(|option| contains(&option.value))
        || TEMPLATE_OPTIONS.iter().any(|name| {
            config
                .options
                .get(*name)
                .is_some_and(|option| contains(&option.value))
        })
}

pub(super) fn substitute_config_dir(
    config: &mut Config,
    directory: &Path,
    version: Option<&str>,
) -> Result<(), String> {
    fn substitute(value: &mut Value, directory: &Path) {
        match value {
            Value::String(value) if starts_config_dir(value) => {
                *value =
                    super::host::lexical(&directory.join(value.replacen("${configDir}", "./", 1)))
                        .to_string_lossy()
                        .into_owned();
            }
            Value::Array(values) => values
                .iter_mut()
                .for_each(|value| substitute(value, directory)),
            Value::Object(values) => values
                .values_mut()
                .for_each(|value| substitute(value, directory)),
            _ => {}
        }
    }
    if !has_config_dir(config) {
        return Ok(());
    }
    let version = version
        .and_then(super::project_version::known_release)
        .ok_or("Config directory template needs compiler version evidence")?;
    if version < (5, 5) {
        return Ok(());
    } // Older compilers interpret the string literally.
    if directory.to_str().is_none() {
        return Err("Config directory is not UTF-8".into());
    }
    for option in config.roots.values_mut() {
        substitute(&mut option.value, directory);
    }
    for name in TEMPLATE_OPTIONS {
        if let Some(option) = config.options.get_mut(*name) {
            substitute(&mut option.value, directory);
        }
    }
    Ok(())
}

fn starts_config_dir(value: &str) -> bool {
    value
        .get(.."${configDir}".len())
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("${configDir}"))
}

pub(super) fn document(
    host: &mut Host,
    path: &Path,
    active: &mut BTreeSet<PathBuf>,
) -> Result<Config, String> {
    let path = host.normalized(path).ok_or("Configuration outside scope")?;
    if !active.insert(path.clone()) {
        return Err("Cyclic tsconfig extends".into());
    }
    let result = (|| {
        let source = host.read(&path).ok_or("Cannot read tsconfig")?;
        let json = jsonc(&source)?;
        let directory = path.parent().ok_or("Configuration has no parent")?;
        let mut merged = Config::default();
        if let Some(extends) = json.get("extends").filter(|value| !value.is_null()) {
            let parents = match extends {
                Value::String(parent) => vec![parent.as_str()],
                Value::Array(parents) => parents
                    .iter()
                    .map(|parent| parent.as_str().ok_or("Invalid extends entry"))
                    .collect::<Result<Vec<_>, _>>()?,
                _ => return Err("Invalid extends".into()),
            };
            for parent in parents {
                let parent = resolve_parent(host, directory, parent)?;
                // Later bases override only keys they actually contribute.
                let base = document(host, &parent, active)?;
                merged.options.extend(base.options);
                merged.roots.extend(base.roots);
            }
        }
        // jsconfig owns these defaults, overriding inherited defaults/options
        // before its explicit compiler options are applied.
        if path.file_name().is_some_and(|name| name == "jsconfig.json") {
            for (name, value) in [
                ("allowJs", Value::Bool(true)),
                ("maxNodeModuleJsDepth", Value::from(2)),
                ("allowSyntheticDefaultImports", Value::Bool(true)),
                ("skipLibCheck", Value::Bool(true)),
                ("noEmit", Value::Bool(true)),
            ] {
                merged.options.insert(
                    name.to_string(),
                    OptionValue {
                        value,
                        directory: directory.to_path_buf(),
                    },
                );
            }
        }
        if let Some(options) = json.get("compilerOptions").filter(|value| !value.is_null()) {
            for (name, value) in options.as_object().ok_or("Invalid compilerOptions")? {
                // Keep null as an explicit reset across later extends merges.
                // Effective consumers remove resets only after all bases merge.
                merged.options.insert(
                    name.clone(),
                    OptionValue {
                        value: value.clone(),
                        directory: directory.to_path_buf(),
                    },
                );
            }
        }
        for name in ["files", "include", "exclude"] {
            if let Some(value) = json.get(name).filter(|value| !value.is_null()) {
                let entries = value.as_array().ok_or("Invalid root file list")?;
                if entries.iter().any(|entry| !entry.is_string()) {
                    return Err("Invalid root file entry".into());
                }
                merged.roots.insert(
                    name.to_string(),
                    OptionValue {
                        value: value.clone(),
                        directory: directory.to_path_buf(),
                    },
                );
            }
        }
        if let Some(references) = json.get("references").filter(|value| !value.is_null()) {
            for reference in references.as_array().ok_or("Invalid project references")? {
                let target = reference
                    .get("path")
                    .and_then(Value::as_str)
                    .ok_or("Invalid project reference path")?;
                let target = directory.join(target);
                // A reference is a config file or a directory containing tsconfig.json.
                let target = if target.extension().is_some_and(|ext| ext == "json") {
                    target
                } else {
                    target.join("tsconfig.json")
                };
                merged.references.push(target);
            }
        }
        Ok(merged)
    })();
    active.remove(&path);
    result
}

fn config_candidate(host: &mut Host, path: &Path) -> Option<PathBuf> {
    if host.file(path) {
        return Some(path.to_path_buf());
    }
    let json = PathBuf::from(format!("{}.json", path.to_string_lossy()));
    if host.file(&json) {
        return Some(json);
    }
    let index = path.join("tsconfig.json");
    host.file(&index).then_some(index)
}

fn resolve_parent(host: &mut Host, directory: &Path, specifier: &str) -> Result<PathBuf, String> {
    if specifier.starts_with('.') || Path::new(specifier).is_absolute() {
        return config_candidate(host, &directory.join(specifier))
            .ok_or_else(|| "Extended configuration absent".into());
    }
    let parts: Vec<_> = specifier.split('/').collect();
    let count = if specifier.starts_with('@') { 2 } else { 1 };
    if parts.len() < count || parts.iter().any(|part| part.is_empty() || *part == "..") {
        return Err("Invalid configuration package specifier".into());
    }
    let package = parts[..count].join("/");
    let boundary = host
        .boundary(directory)
        .ok_or("Configuration outside scope")?;
    let mut ancestor = directory;
    loop {
        let owner = ancestor.join("node_modules").join(&package);
        if parts.len() > count {
            if let Some(path) = config_candidate(host, &owner.join(parts[count..].join("/"))) {
                return Ok(path);
            }
        } else {
            let manifest = owner.join("package.json");
            if let Some(source) = host.read(&manifest) {
                let json = jsonc(&source)?;
                if let Some(target) = json.get("tsconfig") {
                    let target = target.as_str().ok_or("Invalid package tsconfig entry")?;
                    return config_candidate(host, &owner.join(target))
                        .ok_or_else(|| "Package tsconfig entry absent".into());
                }
            }
            let config = owner.join("tsconfig.json");
            if host.file(&config) {
                return Ok(config);
            }
        }
        if ancestor == boundary {
            break;
        }
        ancestor = ancestor
            .parent()
            .ok_or("Missing configuration package boundary")?;
    }
    Err("Configuration package absent".into())
}
