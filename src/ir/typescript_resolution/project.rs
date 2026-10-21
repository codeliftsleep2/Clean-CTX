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
        let boundary = host
            .boundary(file)
            .ok_or("Source outside authorized roots")?;
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
        // Explicit source-only projects preserve legacy resolution without
        // inferring an Angular version or package API from absent configuration.
        Ok(Self {
            root: boundary.clone(),
            config: None,
            mode: "node10".into(),
            base: boundary,
            paths: BTreeMap::new(),
            conditions: vec!["types".into(), "import".into(), "default".into()],
        })
    }

    fn load(host: &mut Host, path: &Path, active: &mut BTreeSet<PathBuf>) -> Result<Self, String> {
        let path = host
            .normalized(path)
            .ok_or("Configuration outside authorized roots")?;
        if !active.insert(path.clone()) {
            return Err("Cyclic tsconfig extends".into());
        }
        let result = (|| {
            let source = host.read(&path).ok_or("Cannot read tsconfig")?;
            let json = jsonc(&source)?;
            let directory = path
                .parent()
                .ok_or("Configuration has no parent")?
                .to_path_buf();
            let mut project = Self {
                root: directory.clone(),
                config: Some(path.clone()),
                mode: "node10".into(),
                base: directory.clone(),
                paths: BTreeMap::new(),
                conditions: vec!["types".into(), "import".into(), "default".into()],
            };
            if let Some(extends) = json.get("extends") {
                let parents: Vec<&str> = if let Some(name) = extends.as_str() {
                    vec![name]
                } else {
                    extends
                        .as_array()
                        .ok_or("Invalid extends")?
                        .iter()
                        .map(|name| name.as_str().ok_or("Invalid extends entry"))
                        .collect::<Result<_, _>>()?
                };
                for parent in parents {
                    if !parent.starts_with('.') {
                        return Err(
                            "Package tsconfig extends requires package-config resolution".into(),
                        );
                    }
                    let mut target = directory.join(parent);
                    if !host.file(&target) {
                        target = directory.join(format!("{parent}.json"));
                    }
                    project = Self::load(host, &target, active)?;
                }
                project.root = directory.clone();
                project.config = Some(path.clone());
            }
            let options = &json["compilerOptions"];
            if let Some(mode) = options["moduleResolution"].as_str() {
                project.mode = mode.to_ascii_lowercase();
            }
            if let Some(base) = options["baseUrl"].as_str() {
                project.base = directory.join(base);
            }
            if let Some(paths) = options["paths"].as_object() {
                let base = if options.get("baseUrl").is_some() {
                    project.base.clone()
                } else {
                    directory.clone()
                };
                project.paths.clear();
                for (name, targets) in paths {
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
            if let Some(conditions) = options["customConditions"].as_array() {
                for condition in conditions {
                    project
                        .conditions
                        .push(condition.as_str().ok_or("Invalid condition")?.to_string());
                }
            }
            if !matches!(
                project.mode.as_str(),
                "bundler" | "node10" | "node" | "classic" | "node16" | "nodenext"
            ) {
                return Err("Unknown moduleResolution".into());
            }
            Ok(project)
        })();
        active.remove(&path);
        result
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
