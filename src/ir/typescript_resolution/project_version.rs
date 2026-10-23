//! Installed compiler evidence is data; no workspace compiler is executed.
use super::{Host, project::jsonc, project_config::OptionValue};
use std::{collections::BTreeMap, path::Path};

pub(super) fn installed(host: &mut Host, directory: &Path) -> Result<Option<String>, String> {
    Ok(installed_manifest(host, directory)?.map(|(_, version)| version))
}

pub(super) fn installed_manifest(
    host: &mut Host,
    directory: &Path,
) -> Result<Option<(std::path::PathBuf, String)>, String> {
    let boundary = host
        .boundary(directory)
        .ok_or("Compiler lookup outside scope")?;
    let mut directory = directory;
    loop {
        let manifest = directory.join("node_modules/typescript/package.json");
        if let Some(source) = host.read(&manifest) {
            let manifest = jsonc(&source)?;
            if manifest.get("name").and_then(serde_json::Value::as_str) != Some("typescript") {
                return Err("Compiler manifest does not identify TypeScript".into());
            }
            return manifest
                .get("version")
                .and_then(serde_json::Value::as_str)
                .map(|version| {
                    Some((
                        directory.join("node_modules/typescript/package.json"),
                        version.to_string(),
                    ))
                })
                .ok_or_else(|| "Invalid compiler version".into());
        }
        if directory == boundary {
            break;
        }
        directory = directory
            .parent()
            .ok_or("Missing compiler lookup boundary")?;
    }
    Ok(None)
}

/// Known versioned rules only. Unproved defaults stay unresolved.
pub(super) fn detection_default(
    options: &BTreeMap<String, OptionValue>,
    version: Option<&str>,
) -> Option<&'static str> {
    let version = version?;
    // Reviewed against the pinned 5.9 compiler's computed moduleDetection.
    // Other release rules require their own conformance evidence.
    if known_release(version)? != (5, 9) {
        return None;
    }
    let module = effective_module(options, version)?;
    Some(
        if matches!(module.as_str(), "node16" | "node18" | "node20" | "nodenext") {
            "force"
        } else {
            "auto"
        },
    )
}

/// Known versioned rules only. Unproved defaults stay unresolved.
pub(super) fn resolution_default(
    options: &BTreeMap<String, OptionValue>,
    version: Option<&str>,
) -> Option<&'static str> {
    let (major, minor) = known_release(version?)?;
    let modern = (major, minor) == (6, 0);
    let module = effective_module(options, version?)?;
    if modern {
        return match module.as_str() {
            "none" | "amd" | "umd" | "system" => Some("classic"),
            "nodenext" => Some("nodenext"),
            "node16" | "node18" | "node20" => Some("node16"),
            "commonjs" | "es6" | "es2015" | "es2020" | "es2022" | "esnext" | "preserve" => {
                Some("bundler")
            }
            _ => None,
        };
    }
    match module.as_str() {
        "commonjs" => Some("node10"),
        "node16" if (major, minor) >= (4, 7) => Some("node16"),
        "nodenext" if (major, minor) >= (4, 7) => Some("nodenext"),
        "node18" if (major, minor) >= (5, 8) => Some("node16"),
        "node20" if (major, minor) >= (5, 9) => Some("node16"),
        "preserve" if (major, minor) >= (5, 4) => Some("bundler"),
        "none" | "amd" | "umd" | "system" | "es6" | "es2015" | "es2020" | "es2022" | "esnext" => {
            Some("classic")
        }
        _ => None,
    }
}

// Kept in one place so compatibility and default derivation use identical evidence.
pub(super) fn known_release(version: &str) -> Option<(u32, u32)> {
    if version.split('.').any(|part| {
        part.is_empty()
            || !part.bytes().all(|byte| byte.is_ascii_digit())
            || (part.len() > 1 && part.starts_with('0'))
    }) {
        return None;
    }
    let parts = version
        .split('.')
        .map(str::parse::<u32>)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    let [major, minor, _patch] = parts.as_slice() else {
        return None;
    };
    let known = (matches!(*major, 4 | 5) && *minor <= 9) || (*major, *minor) == (6, 0);
    known.then_some((*major, *minor))
}

pub(super) fn effective_module(
    options: &BTreeMap<String, OptionValue>,
    version: &str,
) -> Option<String> {
    let modern = known_release(version)? == (6, 0);
    let module = if let Some(module) = options.get("module") {
        module.value.as_str()?.to_ascii_lowercase()
    } else if modern {
        let target = options
            .get("target")
            .map(|option| option.value.as_str())
            .unwrap_or(Some("es2025"))?
            .to_ascii_lowercase();
        match target.as_str() {
            "es3" | "es2022" | "es2023" | "es2024" | "es2025" => "es2022".to_string(),
            "esnext" => "esnext".to_string(),
            "es2020" | "es2021" => "es2020".to_string(),
            "es6" | "es2015" | "es2016" | "es2017" | "es2018" | "es2019" => "es2015".to_string(),
            "es5" => "commonjs".to_string(),
            _ => return None,
        }
    } else {
        let target = match options.get("target") {
            Some(target) => Some(target.value.as_str()?.to_ascii_lowercase()),
            None => None,
        };
        match target.as_deref() {
            None | Some("es3" | "es5") => "commonjs".to_string(),
            Some(
                "es6" | "es2015" | "es2016" | "es2017" | "es2018" | "es2019" | "es2020" | "es2021"
                | "es2022" | "es2023" | "es2024" | "esnext",
            ) => "es2015".to_string(),
            _ => return None,
        }
    };
    Some(module)
}
