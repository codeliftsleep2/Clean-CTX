//! Validate options consumed by the authority host, not unrelated compiler flags.
use super::project_config::OptionValue;
use std::collections::BTreeMap;

pub(super) fn validate(
    options: &BTreeMap<String, OptionValue>,
    mode: &str,
    version: Option<&str>,
) -> Result<(), String> {
    let release = version.and_then(super::project_version::known_release);
    if release.is_some_and(|release| release < (5, 0)) && mode == "bundler" {
        return Err("Bundler resolution requires TypeScript 5 or newer".into());
    }
    if release.is_some_and(|release| release < (4, 7)) && matches!(mode, "node16" | "nodenext") {
        return Err("Stable Node16/NodeNext resolution requires TypeScript 4.7 or newer".into());
    }
    if release.is_some_and(|release| release < (5, 0)) && options.contains_key("customConditions") {
        return Err("customConditions requires TypeScript 5 or newer".into());
    }
    if release.is_some_and(|release| release < (4, 7)) && options.contains_key("moduleSuffixes") {
        return Err("moduleSuffixes requires TypeScript 4.7 or newer".into());
    }
    for name in [
        "moduleSuffixes",
        "rootDirs",
        "customConditions",
        "typeRoots",
    ] {
        if let Some(option) = options.get(name) {
            let values = option
                .value
                .as_array()
                .ok_or_else(|| format!("Invalid array compiler option: {name}"))?;
            if values.iter().any(|value| !value.is_string()) {
                return Err(format!("Invalid string array compiler option: {name}"));
            }
        }
    }
    for name in [
        "allowJs",
        "checkJs",
        "preserveSymlinks",
        "resolveJsonModule",
        "resolvePackageJsonImports",
        "resolvePackageJsonExports",
    ] {
        if options
            .get(name)
            .is_some_and(|option| !option.value.is_boolean())
        {
            return Err(format!("Invalid boolean compiler option: {name}"));
        }
    }
    for name in ["resolvePackageJsonExports", "resolvePackageJsonImports"] {
        if options.get(name).and_then(|option| option.value.as_bool()) == Some(true)
            && !matches!(mode, "bundler" | "node16" | "nodenext" | "unresolved")
        {
            return Err(format!("{name} requires modern module resolution"));
        }
    }
    if let Some(paths) = options.get("paths") {
        for (pattern, targets) in paths.value.as_object().ok_or("Invalid paths")? {
            if pattern.matches('*').count() > 1 {
                return Err("Multiple wildcards in paths key".into());
            }
            let targets = targets.as_array().ok_or("Invalid paths targets")?;
            if targets.is_empty() {
                return Err("Empty paths target list".into());
            }
            for target in targets {
                if target
                    .as_str()
                    .ok_or("Invalid paths target")?
                    .matches('*')
                    .count()
                    > 1
                {
                    return Err("Multiple wildcards in paths target".into());
                }
            }
        }
    }
    if options.contains_key("customConditions")
        && !matches!(mode, "bundler" | "node16" | "nodenext" | "unresolved")
    {
        return Err("customConditions requires modern module resolution".into());
    }
    let module = options
        .get("module")
        .map(|option| {
            option
                .value
                .as_str()
                .ok_or("Invalid module")
                .map(str::to_ascii_lowercase)
        })
        .transpose()?
        .or_else(|| {
            version.and_then(|version| super::project_version::effective_module(options, version))
        });
    if let Some(module) = module.as_deref() {
        let introduced = match module {
            "node16" | "nodenext" => Some((4, 7)),
            "node18" => Some((5, 8)),
            "node20" => Some((5, 9)),
            "preserve" => Some((5, 4)),
            "es2022" => Some((4, 5)),
            _ => None,
        };
        if introduced
            .zip(release)
            .is_some_and(|(introduced, installed)| installed < introduced)
        {
            return Err("Module setting predates installed compiler support".into());
        }
        if !matches!(
            module,
            "none"
                | "commonjs"
                | "amd"
                | "umd"
                | "system"
                | "es6"
                | "es2015"
                | "es2020"
                | "es2022"
                | "esnext"
                | "node16"
                | "node18"
                | "node20"
                | "nodenext"
                | "preserve"
        ) {
            return Err("Unknown module setting".into());
        }
        let required = match module {
            "node16" | "node18" | "node20" => Some("node16"),
            "nodenext" => Some("nodenext"),
            _ => None,
        };
        if required.is_some_and(|required| mode != "unresolved" && mode != required) {
            return Err("Node module and resolution settings conflict".into());
        }
        if matches!(mode, "node16" | "nodenext") && required != Some(mode) {
            return Err("Node resolution requires its matching module setting".into());
        }
        let ts6 = version.and_then(super::project_version::known_release) == Some((6, 0));
        if mode == "bundler" && matches!(module, "none" | "amd" | "umd" | "system") {
            return Err("Bundler resolution requires an ESM-capable module setting".into());
        }
        if mode == "bundler" && module == "commonjs" && !ts6 {
            return Err("CommonJS with bundler resolution requires TypeScript 6 evidence".into());
        }
    }
    Ok(())
}
