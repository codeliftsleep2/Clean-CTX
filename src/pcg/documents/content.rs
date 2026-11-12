//! Source-owned exact content, never heading inference or generated knowledge.
use super::model::Diagnostic;
use crate::pcg::{Error, Result, identity, require, security};
use std::{collections::BTreeMap, sync::Arc};

pub const ROLES: &[&str] = &[
    "statement",
    "subject",
    "scope",
    "outcome",
    "rationale",
    "alternative",
    "rejected_alternative",
    "rejection_reason",
    "consequence",
    "constraint",
    "verification_obligation",
    "observation",
    "counterexample",
    "qualification",
    "historical_applicability",
    "evidence",
];
pub const SCOPES: &[&str] = &[
    "proposition",
    "decision",
    "invariant",
    "constraint",
    "finding",
    "authority",
    "evidence",
    "multi_section",
];

#[derive(Clone, Debug)]
pub struct Spec {
    pub scope: String,
    pub components: Vec<Binding>,
}
#[derive(Clone, Debug)]
pub struct Binding {
    pub role: String,
    pub label: Option<String>,
    pub path: String,
    pub selector: String,
}
#[derive(Clone)]
pub struct Component {
    pub binding: Binding,
    pub source: Arc<Vec<u8>>,
    pub source_sha: String,
    pub start: usize,
    pub end: usize,
    pub content_sha: String,
}
#[derive(Clone)]
pub struct Knowledge {
    pub scope: String,
    pub components: Vec<Component>,
}

fn name(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
}
pub fn spec(scope: String, entries: Vec<String>) -> Result<Spec> {
    require(SCOPES.contains(&scope.as_str()), "knowledge_scope")?;
    let mut components = Vec::new();
    let mut unique = std::collections::BTreeSet::new();
    for entry in entries {
        require(
            unique.insert(entry.clone()),
            "knowledge_component_duplicate",
        )?;
        let (role, locator) = entry.split_once(' ').ok_or(Error("knowledge_binding"))?;
        let (role, label) = role
            .split_once(':')
            .map_or((role, None), |(a, b)| (a, Some(b)));
        require(
            ROLES.contains(&role) && label.is_none_or(name),
            "knowledge_role",
        )?;
        let (path, selector) = locator
            .rsplit_once('#')
            .ok_or(Error("knowledge_selector"))?;
        if path != "self" {
            security::relative(path)?;
        }
        require(name(selector), "knowledge_selector")?;
        components.push(Binding {
            role: role.into(),
            label: label.map(str::to_owned),
            path: path.into(),
            selector: selector.into(),
        });
    }
    require(
        components.iter().any(|c| c.role == "statement"),
        "knowledge_statement",
    )?;
    Ok(Spec { scope, components })
}

type MarkerIndex = std::result::Result<BTreeMap<String, (usize, usize)>, &'static str>;
#[derive(Default)]
pub struct Resolver {
    texts: BTreeMap<String, std::result::Result<Arc<str>, &'static str>>,
    indexes: BTreeMap<String, MarkerIndex>,
    hashes: BTreeMap<String, String>,
    content_hashes: BTreeMap<(String, usize, usize), String>,
    #[cfg(test)]
    pub extracted: usize,
    #[cfg(test)]
    work: (usize, usize, usize, usize),
}
impl Resolver {
    #[cfg(test)]
    pub fn work(&self) -> (usize, usize, usize, usize, usize) {
        (
            self.work.0,
            self.work.1,
            self.work.2,
            self.work.3,
            self.extracted,
        )
    }

    pub fn resolve(
        &mut self,
        spec: &Spec,
        owner_path: &str,
        mut read: impl FnMut(&str) -> Result<Arc<Vec<u8>>>,
    ) -> Result<Knowledge> {
        let mut components = Vec::new();
        for binding in &spec.components {
            let path = if binding.path == "self" {
                owner_path
            } else {
                &binding.path
            };
            let source = read(path)?;
            if !self.texts.contains_key(path) {
                #[cfg(test)]
                {
                    self.work.0 += 1;
                }
                let text = (|| {
                    let text = String::from_utf8(source.as_ref().clone())
                        .map_err(|_| Error("invalid_utf8"))?;
                    security::nonsecret(&text)?;
                    Ok::<Arc<str>, Error>(Arc::from(text))
                })()
                .map_err(|e| e.0);
                self.texts.insert(path.into(), text);
            }
            let text = self.texts[path].as_ref().map_err(|e| Error(e))?.clone();
            let (start, end) = if binding.selector == "whole" {
                (0, source.len())
            } else {
                if !self.indexes.contains_key(path) {
                    #[cfg(test)]
                    {
                        self.work.1 += 1;
                    }
                    self.indexes
                        .insert(path.into(), index(&text).map_err(|e| e.0));
                }
                *self.indexes[path]
                    .as_ref()
                    .map_err(|e| Error(e))?
                    .get(&binding.selector)
                    .ok_or(Error("knowledge_boundary_missing"))?
            };
            require(
                start < end && text.is_char_boundary(start) && text.is_char_boundary(end),
                "knowledge_boundary_empty",
            )?;
            let mut binding = binding.clone();
            binding.path = path.into();
            if !self.hashes.contains_key(path) {
                #[cfg(test)]
                {
                    self.work.2 += 1;
                }
                self.hashes.insert(path.into(), identity::sha(&source));
            }
            let key = (path.to_string(), start, end);
            if !self.content_hashes.contains_key(&key) {
                #[cfg(test)]
                {
                    self.work.3 += 1;
                }
                self.content_hashes
                    .insert(key.clone(), identity::sha(&source[start..end]));
            }
            let source_sha = self.hashes[path].clone();
            components.push(Component {
                binding,
                source_sha,
                content_sha: self.content_hashes[&key].clone(),
                source,
                start,
                end,
            });
            #[cfg(test)]
            {
                self.extracted += 1;
            }
        }
        Ok(Knowledge {
            scope: spec.scope.clone(),
            components,
        })
    }
}

/// Index once per source/operation. Marker lines inside fenced examples are inert.
fn index(text: &str) -> Result<BTreeMap<String, (usize, usize)>> {
    let mut result = BTreeMap::new();
    let mut opened = Vec::<(String, usize)>::new();
    let mut seen = std::collections::BTreeSet::new();
    let mut fence: Option<(char, usize)> = None;
    let mut offset = 0;
    for raw in text.split_inclusive('\n') {
        let line = raw.trim_end_matches(['\r', '\n']);
        let marker = line.chars().next().filter(|c| *c == '`' || *c == '~');
        let count = marker.map_or(0, |c| line.chars().take_while(|x| *x == c).count());
        if let Some((ch, n)) = fence {
            if marker == Some(ch) && count >= n && line[count..].trim().is_empty() {
                fence = None;
            }
        } else if count >= 3 {
            fence = Some((marker.expect("fence marker"), count));
        } else if let Some(tail) = line.strip_prefix("<!-- pcg:") {
            let tail = tail
                .strip_suffix(" -->")
                .ok_or(Error("knowledge_boundary_malformed"))?;
            let (id, operation) = tail
                .rsplit_once(':')
                .ok_or(Error("knowledge_boundary_malformed"))?;
            require(name(id) && id != "whole", "knowledge_boundary_malformed")?;
            match operation {
                "begin" => {
                    require(seen.insert(id.to_owned()), "knowledge_boundary_ambiguous")?;
                    opened.push((id.into(), offset + raw.len()));
                }
                "end" => {
                    let (expected, start) =
                        opened.pop().ok_or(Error("knowledge_boundary_malformed"))?;
                    require(
                        expected == id && start < offset,
                        "knowledge_boundary_malformed",
                    )?;
                    result.insert(id.into(), (start, offset));
                }
                _ => return Err(Error("knowledge_boundary_malformed")),
            }
        }
        offset += raw.len();
    }
    require(opened.is_empty(), "knowledge_boundary_unclosed")?;
    Ok(result)
}

pub fn diagnostic(path: &str, id: &str, error: Error) -> Diagnostic {
    Diagnostic::new("E_KNOWLEDGE_BOUNDARY", path, Some(id), error.0)
}
