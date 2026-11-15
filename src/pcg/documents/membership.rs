//! Explicit source membership; no prose classification or operational profile registry.
use super::{model::Diagnostic, profile::Trusted, scan::Capture};
use crate::pcg::{Result, security};
use std::collections::{BTreeMap, BTreeSet};

pub(super) const REGISTRY: &str = "docs/architecture/knowledge-governance/corpus.csv";

/// Closed two-column CSV, allowing conventional quoted/escaped fields. Embedded
/// record newlines are refused: governed paths and identity lists are single-line.
pub(super) fn columns(line: &str) -> Option<Vec<String>> {
    let mut fields = Vec::new();
    let mut chars = line.chars().peekable();
    loop {
        let mut value = String::new();
        if chars.peek() == Some(&'"') {
            chars.next();
            loop {
                match chars.next()? {
                    '"' if chars.peek() == Some(&'"') => {
                        chars.next();
                        value.push('"');
                    }
                    '"' => break,
                    ch => value.push(ch),
                }
            }
            if chars.peek().is_some_and(|ch| *ch != ',') {
                return None;
            }
        } else {
            while chars.peek().is_some_and(|ch| *ch != ',') {
                let ch = chars.next()?;
                if ch == '"' {
                    return None;
                }
                value.push(ch);
            }
        }
        fields.push(value);
        if chars.next().is_none() {
            break;
        }
    }
    Some(fields)
}

pub(super) fn parse(
    bytes: &[u8],
    trusted: &Trusted,
) -> std::result::Result<BTreeMap<String, BTreeSet<String>>, String> {
    if bytes.starts_with(&[0xef, 0xbb, 0xbf]) {
        return Err("Registry must be UTF-8 without BOM".into());
    }
    let text = std::str::from_utf8(bytes).map_err(|_| "Registry must be strict UTF-8")?;
    let mut lines = text.lines();
    if columns(lines.next().unwrap_or_default())
        != Some(vec!["source_path".into(), "expected_ids".into()])
    {
        return Err("Expected exactly source_path,expected_ids columns".into());
    }
    let mut sources = BTreeMap::new();
    let mut owners = BTreeSet::new();
    for (offset, line) in lines.enumerate() {
        let error = |message: &str| format!("Registry line {}: {message}", offset + 2);
        let fields = columns(line).ok_or_else(|| error("Malformed CSV record"))?;
        let [path, ids] = fields.as_slice() else {
            return Err(error("Expected two fields"));
        };
        security::relative(path)
            .map_err(|_| error("Expected canonical repository-relative path"))?;
        if path.contains('\\')
            || path
                .split('/')
                .any(|part| part.is_empty() || part == "." || part == "..")
            || !trusted
                .profile
                .document_roots
                .iter()
                .any(|root| std::path::Path::new(path).starts_with(root))
        {
            return Err(error(
                "Path must be canonical and inside existing document roots",
            ));
        }
        let mut expected = BTreeSet::new();
        for id in ids.split(';') {
            if !super::validate::node_id(id)
                || !expected.insert(id.to_owned())
                || !owners.insert(id.to_owned())
            {
                return Err(error("Invalid identity or duplicate identity ownership"));
            }
        }
        if sources.insert(path.clone(), expected).is_some() {
            return Err(error("Duplicate source path"));
        }
    }
    Ok(sources)
}

pub(super) fn validate(capture: &mut Capture, trusted: &Trusted) -> Result<()> {
    if !trusted
        .profile
        .workspace_root
        .join(REGISTRY)
        .try_exists()
        .map_err(|_| crate::pcg::Error("document_source_unavailable"))?
    {
        // Legacy catalogs remain compatible until accepted governance state is
        // activated. Accepted-state non-evasion is a separate publication check.
        return Ok(());
    }
    let bytes = capture.read(trusted, REGISTRY)?;
    let membership = match parse(&bytes, trusted) {
        Ok(membership) => membership,
        Err(message) => {
            capture.diagnostics.push(Diagnostic::new(
                "E_GOVERNANCE_REGISTRY",
                REGISTRY,
                None,
                &message,
            ));
            return Ok(());
        }
    };
    let mut actual = BTreeMap::<&str, BTreeSet<&str>>::new();
    for (id, candidate) in &capture.nodes {
        actual.entry(&candidate.path).or_default().insert(id);
        if !membership.contains_key(&candidate.path) {
            capture.diagnostics.push(Diagnostic::new("E_UNREGISTERED_DECLARED_SOURCE", &candidate.path, Some(id),
                "Register this declared source and exact stable identities in corpus.csv before admission"));
        }
    }
    for (path, expected) in membership {
        let found = actual.get(path.as_str());
        for id in &expected {
            if !found.is_some_and(|found| found.contains(id.as_str())) {
                capture.diagnostics.push(Diagnostic::new("E_GOVERNED_DECLARATION_MISSING", &path, Some(id),
                    "Restore the expected governed declaration; membership changes require explicit owner authorization"));
            }
        }
        for id in found.into_iter().flatten() {
            if !expected.contains(*id) {
                capture.diagnostics.push(Diagnostic::new("E_UNEXPECTED_GOVERNED_IDENTITY", &path, Some(id),
                    "Use the registered stable identity set or prepare an authorized membership change"));
            }
        }
    }
    Ok(())
}
