//! Compiler library options select real consumed declaration sources.
use super::Resolver;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct LibraryRequest {
    pub manifest: PathBuf,
    pub version: String,
    pub name: String,
    pub resolved: Result<PathBuf, String>,
    pub override_specifier: Option<String>,
}

impl Resolver {
    pub(super) fn library_source(
        &mut self,
        file: &Path,
        name: &str,
        default: bool,
    ) -> Result<PathBuf, String> {
        let project = self.request_project(file)?;
        let directory = project
            .config
            .as_deref()
            .and_then(Path::parent)
            .or_else(|| file.parent())
            .ok_or("Library project parent absent")?;
        let (manifest, version) =
            super::project_version::installed_manifest(&mut self.host, directory)?
                .ok_or("Compiler library authority absent")?;
        if super::project_version::known_release(&version) != Some((5, 9)) {
            return Err("Library policy requires reviewed compiler version".into());
        }
        let name = name.to_ascii_lowercase();
        let filename = if default {
            name.clone()
        } else {
            library_filename(&name).ok_or("Library option requires reviewed identifier mapping")?
        };
        let override_specifier = if default {
            None
        } else {
            let mapped = filename
                .strip_prefix("lib.")
                .and_then(|name| name.strip_suffix(".d.ts"))
                .ok_or("Invalid mapped library file")?;
            let (package, subpath) = mapped.split_once('.').unwrap_or((mapped, ""));
            let specifier = if subpath.is_empty() {
                format!("@typescript/lib-{package}")
            } else {
                format!("@typescript/lib-{package}/{}", subpath.replace('.', "-"))
            };
            Some(specifier)
        };
        let resolved = (|| {
            if let Some(specifier) = &override_specifier {
                let package_name = super::package_lookup::package_parts(specifier)?.0;
                let boundary = self
                    .host
                    .boundary(directory)
                    .ok_or("Library override boundary absent")?;
                let mut ancestor = directory;
                loop {
                    let candidate = ancestor.join("node_modules").join(&package_name);
                    if self.host.directory(&candidate)? {
                        // Library override resolution has fixed Node10 declaration
                        // mechanics; source paths/exports conditions do not retarget it.
                        let mut library = project.clone();
                        library.mode = "node10".into();
                        library.extensions = super::probe::FileExtensions::Declaration;
                        library.paths.clear();
                        library.options.clear();
                        library.conditions =
                            vec!["types".into(), "require".into(), "default".into()];
                        return self.package_reference(ancestor, specifier, &library, false);
                    }
                    if ancestor == boundary {
                        break;
                    }
                    ancestor = ancestor
                        .parent()
                        .ok_or("Library override boundary absent")?;
                }
            }
            let owner = manifest.parent().ok_or("Compiler package parent absent")?;
            let target = owner.join("lib").join(&filename);
            if self.host.file(&target) {
                self.host
                    .normalized(&target)
                    .ok_or_else(|| "Library source outside scope".to_string())
            } else {
                Err("Required compiler library absent or unavailable".into())
            }
        })();
        self.library_evidence.push(LibraryRequest {
            manifest,
            version,
            name,
            resolved: resolved.clone(),
            override_specifier,
        });
        resolved
    }

    pub(super) fn initial_libraries(&mut self, file: &Path) -> Result<Vec<PathBuf>, String> {
        let project = self.request_project(file)?;
        if let Some(option) = project.options.get("noLib") {
            if option.value.as_bool().ok_or("Invalid noLib")? {
                return Ok(Vec::new());
            }
        }
        if let Some(option) = project.options.get("lib") {
            let mut libraries = Vec::new();
            for name in option.value.as_array().ok_or("Invalid lib")? {
                libraries.push(self.library_source(
                    file,
                    name.as_str().ok_or("Invalid lib entry")?,
                    false,
                )?);
            }
            return Ok(libraries);
        }
        // No installed compiler cannot create built-in declaration authority.
        if project.compiler_version.is_none() {
            return Ok(Vec::new());
        }
        let target = if let Some(option) = project.options.get("target") {
            option
                .value
                .as_str()
                .ok_or("Invalid target")?
                .to_ascii_lowercase()
        } else {
            let module = super::project_version::effective_module(
                &project.options,
                project
                    .compiler_version
                    .as_deref()
                    .ok_or("Compiler target evidence absent")?,
            )
            .ok_or("Effective compiler module unproved")?;
            match module.as_str() {
                "nodenext" => "esnext",
                "node16" | "node18" => "es2022",
                "node20" => "es2023",
                _ => "es5",
            }
            .to_string()
        };
        let filename = match target.as_str() {
            "es3" | "es5" => "lib.d.ts".to_string(),
            "es6" | "es2015" => "lib.es6.d.ts".to_string(),
            "es2016" | "es2017" | "es2018" | "es2019" | "es2020" | "es2021" | "es2022"
            | "es2023" | "es2024" | "esnext" => format!("lib.{target}.full.d.ts"),
            _ => return Err("Default library target requires conformance".into()),
        };
        Ok(vec![self.library_source(file, &filename, true)?])
    }
}

// TypeScript 5.9 libMap identifiers and final alias values; no declarations are synthesized.
fn library_filename(name: &str) -> Option<String> {
    let mapped = match name {
        "es6" => "es2015",
        "es7" => "es2016",
        "esnext.symbol" => "es2019.symbol",
        "esnext.asynciterable" => "es2018.asynciterable",
        "esnext.bigint" => "es2020.bigint",
        "esnext.weakref" => "es2021.weakref",
        "esnext.object" => "es2024.object",
        "esnext.regexp" => "es2024.regexp",
        "esnext.string" => "es2024.string",
        other => other,
    };
    const NAMES: &str = "es5 es2015 es2016 es2017 es2018 es2019 es2020 es2021 es2022 es2023 es2024 esnext
        dom dom.iterable dom.asynciterable webworker webworker.importscripts webworker.iterable webworker.asynciterable scripthost
        es2015.core es2015.collection es2015.generator es2015.iterable es2015.promise es2015.proxy es2015.reflect es2015.symbol es2015.symbol.wellknown
        es2016.array.include es2016.intl es2017.arraybuffer es2017.date es2017.object es2017.sharedmemory es2017.string es2017.intl es2017.typedarrays
        es2018.asyncgenerator es2018.asynciterable es2018.intl es2018.promise es2018.regexp
        es2019.array es2019.object es2019.string es2019.symbol es2019.intl
        es2020.bigint es2020.date es2020.promise es2020.sharedmemory es2020.string es2020.symbol.wellknown es2020.intl es2020.number
        es2021.promise es2021.string es2021.weakref es2021.intl es2022.array es2022.error es2022.intl es2022.object es2022.string es2022.regexp
        es2023.array es2023.collection es2023.intl es2024.arraybuffer es2024.collection es2024.object es2024.promise es2024.regexp es2024.sharedmemory es2024.string
        esnext.collection esnext.intl esnext.disposable esnext.decorators esnext.array esnext.iterator esnext.promise esnext.float16 esnext.error esnext.sharedmemory decorators decorators.legacy";
    NAMES
        .split_ascii_whitespace()
        .any(|name| name == mapped)
        .then(|| format!("lib.{mapped}.d.ts"))
}
