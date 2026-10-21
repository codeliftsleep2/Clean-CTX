use super::Host;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub(super) struct Project {
    pub root: PathBuf,
    pub config: Option<PathBuf>,
    pub mode: String,
    pub base: PathBuf,
    pub paths: BTreeMap<String, Vec<PathBuf>>,
    pub conditions: Vec<String>,
}

impl Project {
    pub(super) fn discover(host: &mut Host, file: &Path) -> Result<Self, String> {
        let file = host
            .normalized(file)
            .ok_or("Source outside authorized roots")?;
        let boundary = host
            .boundary(&file)
            .ok_or("Source outside authorized roots")?;
        if boundary == file {
            return Ok(Self::source_only(boundary));
        }
        let mut directory = file.parent().ok_or("Source has no directory")?;
        loop {
            let config = directory.join("tsconfig.json");
            if host.file(&config) {
                return Self::load(host, &config, &mut BTreeSet::new());
            }
            if directory == boundary {
                break;
            }
            directory = directory.parent().ok_or("Missing project boundary")?;
        }
        Ok(Self::source_only(boundary))
    }

    fn source_only(boundary: PathBuf) -> Self {
        Self {
            root: boundary.clone(),
            config: None,
            mode: "unresolved".into(),
            base: boundary,
            paths: BTreeMap::new(),
            conditions: vec!["types".into(), "import".into(), "default".into()],
        }
    }

    fn load(host: &mut Host, path: &Path, active: &mut BTreeSet<PathBuf>) -> Result<Self, String> {
        let path = host
            .normalized(path)
            .ok_or("Configuration outside authorized roots")?;
        let directory = path
            .parent()
            .ok_or("Configuration has no parent")?
            .to_path_buf();
        let options = super::project_config::options(host, &path, active)?;
        let mut project = Self::source_only(directory.clone());
        project.config = Some(path);
        if let Some(option) = options.get("moduleResolution") {
            project.mode = option
                .value
                .as_str()
                .ok_or("Invalid moduleResolution")?
                .to_ascii_lowercase();
        }
        if let Some(option) = options.get("baseUrl") {
            project.base = option
                .directory
                .join(option.value.as_str().ok_or("Invalid baseUrl")?);
        }
        if let Some(option) = options.get("paths") {
            let base = if options.contains_key("baseUrl") {
                project.base.clone()
            } else {
                option.directory.clone()
            };
            for (name, targets) in option.value.as_object().ok_or("Invalid paths")? {
                let targets = targets
                    .as_array()
                    .ok_or("Invalid paths targets")?
                    .iter()
                    .map(|target| {
                        target
                            .as_str()
                            .map(|target| base.join(target))
                            .ok_or("Invalid paths target")
                    })
                    .collect::<Result<_, _>>()?;
                project.paths.insert(name.clone(), targets);
            }
        }
        if let Some(option) = options.get("customConditions") {
            for condition in option.value.as_array().ok_or("Invalid customConditions")? {
                project
                    .conditions
                    .push(condition.as_str().ok_or("Invalid condition")?.to_string());
            }
        }
        if !matches!(
            project.mode.as_str(),
            "unresolved" | "bundler" | "node10" | "node" | "classic" | "node16" | "nodenext"
        ) {
            return Err("Unknown moduleResolution".into());
        }
        Ok(project)
    }
}

/// Strip JSONC comments and trailing commas without reinterpreting string bytes.
pub(super) fn jsonc(source: &str) -> Result<Value, String> {
    serde_json::from_slice(&clean_jsonc(source)?).map_err(|error| error.to_string())
}

pub(super) fn clean_jsonc(source: &str) -> Result<Vec<u8>, String> {
    let bytes = source.as_bytes();
    let mut output = bytes.to_vec();
    let mut index = 0;
    let mut quoted = false;
    let mut escaped = false;
    while index < bytes.len() {
        if quoted {
            if escaped {
                escaped = false;
            } else if bytes[index] == b'\\' {
                escaped = true;
            } else if bytes[index] == b'"' {
                quoted = false;
            }
        } else if bytes[index] == b'"' {
            quoted = true;
        } else if bytes.get(index..index + 2) == Some(b"//") {
            while index < bytes.len() && bytes[index] != b'\n' {
                output[index] = b' ';
                index += 1;
            }
            continue;
        } else if bytes.get(index..index + 2) == Some(b"/*") {
            output[index] = b' ';
            output[index + 1] = b' ';
            index += 2;
            while index + 1 < bytes.len() && &bytes[index..index + 2] != b"*/" {
                output[index] = b' ';
                index += 1;
            }
            if index + 1 >= bytes.len() {
                return Err("Unterminated JSONC comment".into());
            }
            output[index] = b' ';
            output[index + 1] = b' ';
            index += 2;
            continue;
        }
        index += 1;
    }
    quoted = false;
    escaped = false;
    for index in 0..output.len() {
        let byte = output[index];
        if quoted {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                quoted = false;
            }
        } else if byte == b'"' {
            quoted = true;
        } else if byte == b',' {
            let next = output[index + 1..]
                .iter()
                .copied()
                .find(|byte| !byte.is_ascii_whitespace());
            if matches!(next, Some(b'}' | b']')) {
                output[index] = b' ';
            }
        }
    }
    Ok(output)
}
