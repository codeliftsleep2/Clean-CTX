//! Root-file membership and explicit reference traversal, never workspace guessing.
use super::host::lexical;
use super::{
    Host,
    project_config::{Config, document},
};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct Ownership {
    pub selected_explicitly: bool,
    pub configurations: Vec<PathBuf>,
    pub root_candidates: Vec<PathBuf>,
    /// Exclusion from a root-file list does not forbid importing the source.
    pub root_member: bool,
}

pub(super) fn select(
    host: &mut Host,
    config: &Path,
    file: &Path,
) -> Result<(PathBuf, Ownership), String> {
    selection(host, config, file, false)
}

pub(super) fn explicit(
    host: &mut Host,
    config: &Path,
    file: &Path,
) -> Result<(PathBuf, Ownership), String> {
    selection(host, config, file, true)
}

fn selection(
    host: &mut Host,
    config: &Path,
    file: &Path,
    explicit: bool,
) -> Result<(PathBuf, Ownership), String> {
    let config = host
        .normalized(config)
        .ok_or("Project config outside scope")?;
    let mut seen = BTreeSet::new();
    let mut active = BTreeSet::new();
    let mut candidates = BTreeSet::new();
    let mut pending = vec![(config.to_path_buf(), false)];
    while let Some((path, exiting)) = pending.pop() {
        let path = host
            .normalized(&path)
            .ok_or("Project config outside scope")?;
        if exiting {
            active.remove(&path);
            seen.insert(path);
            continue;
        }
        if active.contains(&path) {
            return Err("Cyclic TypeScript project references".into());
        }
        if seen.contains(&path) {
            continue;
        }
        active.insert(path.clone());
        pending.push((path.clone(), true));
        let mut config = document(host, &path, &mut BTreeSet::new())?;
        config.options.retain(|_, option| !option.value.is_null());
        let directory = path.parent().ok_or("Config without directory")?;
        if super::project_config::has_config_dir(&config) {
            let version = super::project_version::installed(host, directory)?;
            super::project_config::substitute_config_dir(
                &mut config,
                directory,
                version.as_deref(),
            )?;
        }
        if member(
            host,
            &config,
            path.parent().ok_or("Config without directory")?,
            file,
        )? {
            candidates.insert(path);
        }
        pending.extend(config.references.into_iter().map(|path| (path, false)));
    }
    if !explicit && candidates.len() > 1 {
        return Err(format!(
            "Ambiguous TypeScript project ownership: {candidates:?}"
        ));
    }
    let selected = if explicit {
        config.clone()
    } else {
        candidates
            .iter()
            .next()
            .cloned()
            .unwrap_or_else(|| config.clone())
    };
    let root_member = if explicit {
        candidates.contains(&config)
    } else {
        !candidates.is_empty()
    };
    Ok((
        selected,
        Ownership {
            selected_explicitly: explicit,
            configurations: seen.into_iter().collect(),
            root_member,
            root_candidates: candidates.into_iter().collect(),
        },
    ))
}

pub(super) fn member(
    host: &mut Host,
    config: &Config,
    directory: &Path,
    file: &Path,
) -> Result<bool, String> {
    if let Some(files) = config.roots.get("files") {
        for name in files.value.as_array().ok_or("Invalid files")? {
            let target = host
                .normalized(&files.directory.join(name.as_str().ok_or("Invalid file")?))
                .ok_or("Root file outside authorized scope")?;
            if target == file {
                return Ok(true);
            }
        }
    }
    let extension = file.extension().and_then(|ext| ext.to_str()).unwrap_or("");
    let boolean = |name| {
        config
            .options
            .get(name)
            .and_then(|option| option.value.as_bool())
    };
    let allow_js = boolean("allowJs").unwrap_or_else(|| boolean("checkJs") == Some(true));
    let source_extension = matches!(extension, "ts" | "tsx" | "mts" | "cts")
        || (allow_js && matches!(extension, "js" | "jsx" | "mjs" | "cjs"));
    if !source_extension {
        return Ok(false);
    }
    let includes = config.roots.get("include");
    let included = if let Some(includes) = includes {
        patterns(&includes.directory, &includes.value, file, false)?
    } else {
        !config.roots.contains_key("files")
            && patterns(directory, &serde_json::json!(["**/*"]), file, false)?
    };
    if !included {
        return Ok(false);
    }
    if let Some(excludes) = config.roots.get("exclude") {
        return Ok(!patterns(&excludes.directory, &excludes.value, file, true)?);
    }
    for key in ["outDir", "declarationDir"] {
        if let Some(output) = config.options.get(key) {
            let output = output.value.as_str().ok_or("Invalid output directory")?;
            if file.starts_with(lexical(&config.options[key].directory.join(output))) {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

fn patterns(
    directory: &Path,
    value: &serde_json::Value,
    file: &Path,
    exclude: bool,
) -> Result<bool, String> {
    for pattern in value.as_array().ok_or("Invalid patterns")? {
        let pattern = pattern
            .as_str()
            .ok_or("Invalid pattern")?
            .replace('\\', "/");
        if pattern.is_empty() {
            continue;
        }
        let parts: Vec<_> = pattern.split('/').collect();
        let mut recursive = false;
        for part in &parts {
            if *part == ".." && recursive {
                return Err("Parent directory after recursive wildcard".into());
            }
            recursive |= *part == "**";
        }
        if !exclude && parts.last() == Some(&"**") {
            return Err("Include ends in recursive wildcard".into());
        }
        let mut pattern = lexical(&directory.join(&pattern))
            .to_string_lossy()
            .replace('\\', "/");
        if !pattern
            .rsplit('/')
            .next()
            .unwrap_or("")
            .contains(['.', '*', '?'])
        {
            pattern.push_str("/**/*");
        }
        let file = file.to_string_lossy().replace('\\', "/");
        #[cfg(windows)]
        let (pattern, file) = (pattern.to_lowercase(), file.to_lowercase());
        if glob(&pattern, &file, exclude) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn glob(pattern: &str, value: &str, exclude: bool) -> bool {
    let pattern: Vec<_> = pattern.split('/').collect();
    let value: Vec<_> = value.split('/').collect();
    let mut reachable = vec![false; value.len() + 1];
    reachable[0] = true;
    for component in pattern {
        let mut next = vec![false; value.len() + 1];
        if component == "**" {
            for index in 0..=value.len() {
                next[index] = reachable[index]
                    || (index > 0
                        && next[index - 1]
                        && (exclude || !implicitly_excluded(value[index - 1])));
            }
        } else {
            for index in 0..value.len() {
                next[index + 1] = reachable[index]
                    && segment(component, value[index], exclude, index + 1 == value.len());
            }
        }
        reachable = next;
    }
    if exclude {
        reachable.into_iter().any(|matches| matches)
    } else {
        reachable[value.len()]
    }
}

fn implicitly_excluded(value: &str) -> bool {
    value.starts_with('.') || matches!(value, "node_modules" | "bower_components" | "jspm_packages")
}

fn segment(pattern: &str, value: &str, exclude: bool, last: bool) -> bool {
    if !exclude
        && ((value.starts_with('.') && !pattern.starts_with('.'))
            || (pattern.contains(['*', '?'])
                && matches!(value, "node_modules" | "bower_components" | "jspm_packages")))
    {
        return false;
    }
    let minified = !exclude && last && value.ends_with(".min.js");
    let value: Vec<_> = value.chars().collect();
    let min_dot = minified.then(|| value.len() - 7);
    let mut reachable = vec![false; value.len() + 1];
    reachable[0] = true;
    for character in pattern.chars() {
        let mut next = vec![false; value.len() + 1];
        for index in 0..=value.len() {
            next[index] = if character == '*' {
                reachable[index] || (index > 0 && next[index - 1] && min_dot != Some(index - 1))
            } else {
                index > 0
                    && reachable[index - 1]
                    && (character == '?' || character == value[index - 1])
            };
        }
        reachable = next;
    }
    reachable[value.len()]
}
