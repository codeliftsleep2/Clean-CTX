use super::{
    model::{Candidate, Diagnostic},
    parse,
    profile::Trusted,
};
use crate::pcg::{Error, Result, identity, require, security};
use serde_json::json;
use std::{collections::BTreeMap, fs, io::Read, path::Path};

pub struct Capture {
    pub nodes: BTreeMap<String, Candidate>,
    pub diagnostics: Vec<Diagnostic>,
    witnesses: BTreeMap<String, (String, Vec<u8>)>,
    contents: BTreeMap<String, std::sync::Arc<Vec<u8>>>,
    bytes: u64,
    #[cfg(test)]
    boundary_work: (usize, usize, usize, usize, usize),
}
impl Capture {
    pub(super) fn source(&self, path: &str) -> Result<&[u8]> {
        self.contents
            .get(path)
            .map(|s| s.as_slice())
            .ok_or(Error("document_source_unavailable"))
    }
    #[cfg(test)]
    pub fn boundary_work(&self) -> (usize, usize, usize, usize, usize) {
        self.boundary_work
    }
    #[cfg(test)]
    pub fn read_work(&self) -> (u64, usize) {
        (self.bytes, self.witnesses.len())
    }
    fn read(&mut self, trusted: &Trusted, relative: &str) -> Result<std::sync::Arc<Vec<u8>>> {
        if let Some(bytes) = self.contents.get(relative) {
            return Ok(bytes.clone());
        }
        security::relative(relative)?;
        let path = trusted.profile.workspace_root.join(relative);
        security::no_links(&path, false)?;
        require(path.is_file(), "document_source_type")?;
        let mut file = fs::File::open(&path).map_err(|_| Error("document_source_unavailable"))?;
        let remaining = trusted.profile.max_bytes.saturating_sub(self.bytes);
        require(
            file.metadata()
                .map_err(|_| Error("document_source_unavailable"))?
                .len()
                <= remaining,
            "document_work_limit",
        )?;
        let mut bytes = Vec::new();
        file.by_ref()
            .take(remaining.saturating_add(1))
            .read_to_end(&mut bytes)
            .map_err(|_| Error("document_source_unavailable"))?;
        require(bytes.len() as u64 <= remaining, "document_work_limit")?;
        self.bytes += bytes.len() as u64;
        let permission = security::permissions(&path)?;
        self.witnesses
            .insert(relative.into(), (identity::sha(&bytes), permission));
        let bytes = std::sync::Arc::new(bytes);
        self.contents.insert(relative.into(), bytes.clone());
        Ok(bytes)
    }
    pub fn completion(&mut self, trusted: &Trusted) -> Result<()> {
        let expected = self.witnesses.clone();
        self.contents.clear();
        for (path, witness) in expected {
            self.read(trusted, &path)?;
            require(
                self.witnesses[&path].0 == witness.0 && self.witnesses[&path].1 == witness.1,
                "document_source_changed",
            )?;
        }
        trusted.recheck()
    }
}

fn relative(root: &Path, path: &Path) -> Result<String> {
    Ok(path
        .strip_prefix(root)
        .map_err(|_| Error("document_scope"))?
        .to_str()
        .ok_or(Error("document_path_encoding"))?
        .replace('\\', "/"))
}

pub fn scan(trusted: &Trusted) -> Result<Capture> {
    let mut result = Capture {
        nodes: BTreeMap::new(),
        diagnostics: Vec::new(),
        witnesses: BTreeMap::new(),
        contents: BTreeMap::new(),
        bytes: 0,
        #[cfg(test)]
        boundary_work: (0, 0, 0, 0, 0),
    };
    let root = &trusted.profile.workspace_root;
    let mut stack: Vec<_> = trusted
        .profile
        .document_roots
        .iter()
        .map(|p| root.join(p))
        .collect();
    let mut paths = std::collections::BTreeSet::new();
    let mut examined = 0;
    while let Some(path) = stack.pop() {
        examined += 1;
        require(examined <= trusted.profile.max_files, "document_work_limit")?;
        security::no_links(&path, false)?;
        if path.is_dir() {
            for entry in fs::read_dir(&path).map_err(|_| Error("document_source_unavailable"))? {
                let entry = entry.map_err(|_| Error("document_source_unavailable"))?;
                if entry.file_name() != ".git" {
                    require(
                        stack.len().saturating_add(examined) < trusted.profile.max_files,
                        "document_work_limit",
                    )?;
                    stack.push(entry.path());
                }
            }
        } else if path.extension().is_some_and(|ext| {
            ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("markdown")
        }) {
            paths.insert(relative(root, &path)?);
        }
    }
    for path in paths {
        let bytes = match result.read(trusted, &path) {
            Ok(bytes) => bytes,
            Err(Error("document_work_limit")) => return Err(Error("document_work_limit")),
            Err(_) => {
                result.diagnostics.push(Diagnostic::new(
                    "E_DOCUMENT_SOURCE",
                    &path,
                    None,
                    "Current document must be readable; retained revisions are historical only",
                ));
                continue;
            }
        };
        let declarations = match parse::declarations(&path, &bytes) {
            Ok(values) => values,
            Err(error) => {
                result.diagnostics.push(error);
                continue;
            }
        };
        if declarations.is_empty() {
            continue;
        }
        security::nonsecret(std::str::from_utf8(&bytes).map_err(|_| Error("invalid_utf8"))?)?;
        for declaration in declarations {
            require(
                result.nodes.len() < trusted.profile.max_nodes,
                "document_work_limit",
            )?;
            let metadata =
                serde_json::to_value(&declaration).map_err(|_| Error("document_metadata"))?;
            security::json_nonsecret(&metadata)?;
            let source_sha = result.witnesses[&path].0.clone();
            let mut evidence =
                vec![json!({"kind":"declaration_source","path":path,"sha256":source_sha})];
            if let Some(artifact) = &declaration.artifact {
                match result.read(trusted, artifact) {
                    Ok(source) => {
                        let hash = result.witnesses[artifact].0.clone();
                        let text = if artifact == &path {
                            None
                        } else {
                            std::str::from_utf8(&source).ok()
                        };
                        if let Some(text) = text {
                            security::nonsecret(text)?;
                        }
                        evidence.push(
                            json!({"kind":"artifact","path":artifact,"sha256":hash,"text":text}),
                        );
                    }
                    Err(Error("document_work_limit")) => return Err(Error("document_work_limit")),
                    Err(_) => result.diagnostics.push(Diagnostic::new(
                        "E_MISSING_ARTIFACT",
                        &path,
                        Some(&declaration.id),
                        "Artifact must be a readable repository file without links or traversal",
                    )),
                }
            }
            let revision = identity::hash(
                &json!({"path":path,"source_sha256":source_sha,"declaration":metadata,"evidence":evidence}),
            );
            let id = declaration.id.clone();
            if let std::collections::btree_map::Entry::Vacant(entry) =
                result.nodes.entry(id.clone())
            {
                entry.insert(Candidate {
                    path: path.clone(),
                    source_sha,
                    bytes: bytes.clone(),
                    declaration,
                    revision,
                    evidence,
                    knowledge: None,
                });
            } else {
                result.diagnostics.push(Diagnostic::new(
                    "E_DUPLICATE_ID",
                    &path,
                    Some(&id),
                    "Correct duplicate stable identity; source metadata is never rewritten",
                ));
            }
        }
    }
    let mut resolver = super::content::Resolver::default();
    for (id, mut node) in std::mem::take(&mut result.nodes) {
        if let Some(spec) = &node.declaration.knowledge {
            match resolver.resolve(spec, &node.path, |path| result.read(trusted, path)) {
                Ok(knowledge) => {
                    for component in &knowledge.components {
                        if !node
                            .evidence
                            .iter()
                            .any(|e| e["path"].as_str() == Some(&component.binding.path))
                        {
                            let text = std::str::from_utf8(&component.source)
                                .map_err(|_| Error("invalid_utf8"))?;
                            node.evidence
                                .push(json!({"kind":"artifact","path":component.binding.path,
                                "sha256":component.source_sha,"text":text}));
                        }
                    }
                    node.revision = identity::hash(
                        &json!({"path":node.path,"source_sha256":node.source_sha,
                        "declaration":serde_json::to_value(&node.declaration).map_err(|_| Error("document_metadata"))?,
                        "evidence":node.evidence}),
                    );
                    node.knowledge = Some(knowledge);
                }
                Err(error) => result
                    .diagnostics
                    .push(super::content::diagnostic(&node.path, &id, error)),
            }
        }
        result.nodes.insert(id, node);
    }
    #[cfg(test)]
    {
        result.boundary_work = resolver.work();
    }
    require(
        result
            .nodes
            .values()
            .map(|n| n.declaration.relationships.len())
            .sum::<usize>()
            <= trusted.profile.max_edges,
        "document_work_limit",
    )?;
    Ok(result)
}
