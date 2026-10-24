//! Package ownership and public-export comparison for captured endpoints.
use super::{DeclarationId, Namespace, Resolution, Resolver};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct ConsumedModuleEntry {
    pub request: PathBuf,
    pub specifier: String,
    pub kind: super::module_resolution::ModuleRequestKind,
    pub resolution: Result<PathBuf, String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct PublicEntryEvidence {
    pub route: ConsumedModuleEntry,
    pub request_hash: String,
    pub entry_hash: String,
    pub namespace: Namespace,
    pub export_name: String,
    pub public_export: Resolution,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct PackageExportEvidence {
    pub declaration: DeclarationId,
    pub manifest: PathBuf,
    pub manifest_hash: String,
    pub name: String,
    pub version: Option<String>,
    pub public_export: Resolution,
    #[serde(default)]
    pub public_entries: Vec<PublicEntryEvidence>,
}

pub(super) fn capture(
    resolver: &mut Resolver,
    request: &Path,
    sites: &[(usize, usize, Resolution)],
) -> Vec<PackageExportEvidence> {
    capture_in_namespace(resolver, request, sites, Namespace::Value)
}

pub(super) fn capture_in_namespace(
    resolver: &mut Resolver,
    request: &Path,
    sites: &[(usize, usize, Resolution)],
    namespace: Namespace,
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
                // Only already-consumed module requests supply entry demands.
                // Do not derive entries from declaration paths or enumerate exports.
                let mut routes = Vec::new();
                for route in &resolver.consumed_entries {
                    if (route.specifier == name || route.specifier.starts_with(&format!("{name}/")))
                        && !routes.contains(route)
                    {
                        routes.push(route.clone());
                    }
                }
                let mut public_entries = Vec::new();
                for route in routes {
                    let Ok(entry) = &route.resolution else {
                        continue;
                    };
                    let Some(request_text) = resolver.host.read(&route.request) else {
                        continue;
                    };
                    let Some(entry_text) = resolver.host.read(entry) else {
                        continue;
                    };
                    let (ids, complete) = resolver.export_set(
                        entry,
                        &declaration.name,
                        namespace,
                        &mut BTreeSet::new(),
                    );
                    public_entries.push(PublicEntryEvidence {
                        request_hash: crate::cache::content_hash(request_text.as_bytes()),
                        entry_hash: crate::cache::content_hash(entry_text.as_bytes()),
                        route,
                        namespace,
                        export_name: declaration.name.clone(),
                        public_export: resolver.outcome(ids, complete),
                    });
                }
                let public_export = match resolver.module(request, &name) {
                    Ok(entry) => {
                        let (ids, complete) = resolver.export_set(
                            &entry,
                            &declaration.name,
                            namespace,
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
                    public_entries,
                })
            })
            .collect()
    })
}
