//! Config inheritance retains each option's declaring directory before merging.
use super::Host;
use super::project::jsonc;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub(super) struct OptionValue {
    pub value: Value,
    pub directory: PathBuf,
}

pub(super) fn options(
    host: &mut Host,
    path: &Path,
    active: &mut BTreeSet<PathBuf>,
) -> Result<BTreeMap<String, OptionValue>, String> {
    let path = host.normalized(path).ok_or("Configuration outside scope")?;
    if !active.insert(path.clone()) {
        return Err("Cyclic tsconfig extends".into());
    }
    let result = (|| {
        let source = host.read(&path).ok_or("Cannot read tsconfig")?;
        let json = jsonc(&source)?;
        let directory = path.parent().ok_or("Configuration has no parent")?;
        let mut merged = BTreeMap::new();
        if let Some(extends) = json.get("extends") {
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
                merged.extend(options(host, &parent, active)?);
            }
        }
        if let Some(options) = json.get("compilerOptions") {
            for (name, value) in options.as_object().ok_or("Invalid compilerOptions")? {
                merged.insert(
                    name.clone(),
                    OptionValue {
                        value: value.clone(),
                        directory: directory.to_path_buf(),
                    },
                );
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
