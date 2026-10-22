//! Ordered package-map selection, shared by exports, imports and self references.
use super::{Resolver, package::Ordered, project::Project};

enum Selected {
    Unmatched,
    Blocked,
    Target(String),
}

impl Resolver {
    pub(super) fn mapped(
        value: &Ordered,
        key: &str,
        project: &Project,
    ) -> Result<Option<String>, String> {
        let mut replacement = None;
        let value = if let Ordered::Object(entries) = value
            && entries
                .iter()
                .any(|(name, _)| name.starts_with('.') || name.starts_with('#'))
        {
            let prefix = if key.starts_with('#') { '#' } else { '.' };
            if entries.iter().any(|(name, _)| !name.starts_with(prefix)) {
                return Ok(None);
            }
            if let Some(exact) = value.get(key) {
                exact
            } else {
                let mut patterns: Vec<_> = entries
                    .iter()
                    .filter_map(|(pattern, target)| {
                        let (prefix, suffix) = pattern.split_once('*')?;
                        Some((
                            prefix.len(),
                            suffix.len(),
                            key.strip_prefix(prefix)?.strip_suffix(suffix)?,
                            target,
                        ))
                    })
                    .collect();
                patterns.sort_by_key(|item| std::cmp::Reverse((item.0, item.1)));
                let Some((_, _, middle, target)) = patterns.first() else {
                    return Ok(None);
                };
                replacement = Some(*middle);
                *target
            }
        } else {
            if key != "." {
                return Ok(None);
            }
            value
        };
        Ok(match select(value, project)? {
            Selected::Target(target) => Some(if let Some(middle) = replacement {
                target.replace('*', middle)
            } else {
                target
            }),
            Selected::Unmatched | Selected::Blocked => None,
        })
    }
}

fn select(value: &Ordered, project: &Project) -> Result<Selected, String> {
    match value {
        Ordered::String(target) => Ok(Selected::Target(target.clone())),
        Ordered::Null => Ok(Selected::Blocked),
        Ordered::Other => Ok(Selected::Unmatched),
        Ordered::Array(array) => {
            if array.is_empty() {
                return Ok(Selected::Blocked);
            }
            for target in array {
                match select(target, project)? {
                    Selected::Unmatched => {}
                    selected => return Ok(selected),
                }
            }
            Ok(Selected::Unmatched)
        }
        Ordered::Object(entries) => {
            for (key, target) in entries {
                let applicable = if key == "default" || project.conditions.contains(key) {
                    true
                } else if let Some(range) = key.strip_prefix("types@") {
                    let version = project
                        .compiler_version
                        .as_deref()
                        .ok_or("Versioned types require compiler-version evidence")?;
                    super::version_ranges::matches(range, version)?
                } else {
                    false
                };
                if applicable {
                    match select(target, project)? {
                        Selected::Unmatched => {}
                        selected => return Ok(selected),
                    }
                }
            }
            Ok(Selected::Unmatched)
        }
    }
}
