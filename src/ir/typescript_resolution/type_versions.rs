//! Package version redirects share compiler witnesses and ordinary source probes.
use super::{Resolver, package::Ordered, project::Project};
use std::path::{Path, PathBuf};

impl Resolver {
    /// None: no applicable mapping. Some(None): matched mapping with no file.
    pub(super) fn type_version_target(
        &mut self,
        owner: &Path,
        json: &Ordered,
        requested: &str,
        project: &Project,
    ) -> Result<Option<Option<PathBuf>>, String> {
        if json.get("typesVersions").is_none() {
            return Ok(None);
        }
        let key = (owner.to_path_buf(), requested.to_string());
        if !self.type_version_active.insert(key.clone()) {
            return Err("Cyclic typesVersions redirect".into());
        }
        let result = self.type_version_target_inner(owner, json, requested, project);
        self.type_version_active.remove(&key);
        result
    }

    fn type_version_target_inner(
        &mut self,
        owner: &Path,
        json: &Ordered,
        requested: &str,
        project: &Project,
    ) -> Result<Option<Option<PathBuf>>, String> {
        if !super::host::lexical(&owner.join(requested)).starts_with(super::host::lexical(owner)) {
            return Ok(None);
        }
        let Some(versions) = json.get("typesVersions") else {
            return Ok(None);
        };
        let Ordered::Object(versions) = versions else {
            return Err("Invalid typesVersions".into());
        };
        let version = project
            .compiler_version
            .as_deref()
            .ok_or("typesVersions requires compiler-version evidence")?;
        let mut selected = None;
        for (range, paths) in versions {
            if super::version_ranges::matches(range, version)? {
                selected = Some(paths);
                break;
            }
        }
        let Some(Ordered::Object(paths)) = selected else {
            return if selected.is_some() {
                Err("Invalid typesVersions paths".into())
            } else {
                Ok(None)
            };
        };
        let mut matches = Vec::new();
        for (pattern, targets) in paths {
            if pattern == requested {
                matches.push((usize::MAX, "", targets));
                continue;
            }
            if let Some((prefix, suffix)) = pattern.split_once('*') {
                if suffix.contains('*') {
                    return Err("Invalid typesVersions pattern".into());
                }
                if let Some(middle) = requested
                    .strip_prefix(prefix)
                    .and_then(|rest| rest.strip_suffix(suffix))
                {
                    matches.push((prefix.len(), middle, targets));
                }
            }
        }
        matches.sort_by_key(|mapping| std::cmp::Reverse(mapping.0));
        let Some((_, middle, Ordered::Array(targets))) = matches.first() else {
            return if matches.is_empty() {
                Ok(None)
            } else {
                Err("Invalid typesVersions substitutions".into())
            };
        };
        for target in targets {
            let target = target.string().ok_or("Invalid typesVersions target")?;
            if target.matches('*').count() > 1 {
                return Err("Invalid typesVersions substitution".into());
            }
            let candidate = owner.join(target.replace('*', middle));
            if let Some(file) = self.probe(&candidate, project) {
                return Ok(Some(Some(file)));
            }
        }
        Ok(Some(None))
    }
}

pub(super) fn entry_request(json: &Ordered, subpath: &str) -> String {
    let entry = if subpath == "." {
        ["types", "typings", "main"]
            .iter()
            .find_map(|field| json.get(field).and_then(Ordered::string))
            .unwrap_or("index")
    } else {
        subpath
    };
    entry.strip_prefix("./").unwrap_or(entry).to_string()
}
