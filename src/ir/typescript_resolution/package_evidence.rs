//! Package ownership and public-export comparison for captured endpoints.
use super::{DeclarationId, Namespace, Resolution, Resolver};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct PackageExportEvidence {
    pub declaration: DeclarationId,
    pub manifest: PathBuf,
    pub manifest_hash: String,
    pub name: String,
    pub version: Option<String>,
    pub public_export: Resolution,
}

pub(super) fn capture(
    resolver: &mut Resolver,
    request: &Path,
    sites: &[(usize, usize, Resolution)],
) -> Vec<PackageExportEvidence> {
    let ids: BTreeSet<_> = sites
        .iter()
        .flat_map(|(_, _, result)| match result {
            Resolution::Resolved(id) => std::slice::from_ref(id),
            Resolution::Merged(ids) => ids.as_slice(),
            _ => &[],
        })
        .cloned()
        .collect();
    resolver.in_source_project(request, |resolver| {
        ids.into_iter()
            .filter_map(|declaration| {
                let boundary = resolver.host.boundary(&declaration.source)?;
                let mut directory = declaration.source.parent()?;
                let (manifest, text) = loop {
                    let manifest = directory.join("package.json");
                    if !resolver.host.admits(&manifest) {
                        return None;
                    }
                    if resolver.host.file(&manifest) {
                        let text = resolver.host.read(&manifest)?;
                        break (resolver.host.normalized(&manifest)?, text);
                    }
                    if resolver.host.directory(&manifest).ok()? {
                        return None;
                    }
                    if directory == boundary || !directory.starts_with(&boundary) {
                        return None;
                    }
                    directory = directory.parent()?;
                };
                // Reuse the duplicate-key-refusing package parser and the same Host.
                let package = super::package::Ordered::parse(&text).ok()?;
                let name = package.get("name")?.string()?.to_string();
                let version = package
                    .get("version")
                    .and_then(|value| value.string())
                    .map(str::to_string);
                let public_export = match resolver.module(request, &name) {
                    Ok(entry) => {
                        let (ids, complete) = resolver.export_set(
                            &entry,
                            &declaration.name,
                            Namespace::Value,
                            &mut BTreeSet::new(),
                        );
                        resolver.outcome(ids, complete)
                    }
                    Err(error) => Resolution::Unresolved(error),
                };
                Some(PackageExportEvidence {
                    declaration,
                    manifest,
                    manifest_hash: crate::cache::content_hash(text.as_bytes()),
                    name,
                    version,
                    public_export,
                })
            })
            .collect()
    })
}
