use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub(crate) enum Namespace {
    Value,
    Type,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, serde::Serialize, serde::Deserialize)]
pub(crate) struct DeclarationId {
    pub source: PathBuf,
    pub source_hash: String,
    pub start: usize,
    pub end: usize,
    pub name: String,
    pub kind: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
pub(crate) enum Witness {
    Directory {
        path: PathBuf,
        entries: Vec<PathBuf>,
    },
    Content {
        path: PathBuf,
        hash: String,
    },
    Missing {
        path: PathBuf,
    },
    Kind {
        path: PathBuf,
        file: bool,
        directory: bool,
    },
    Physical {
        path: PathBuf,
        target: PathBuf,
    },
    Unreadable {
        path: PathBuf,
    },
    OutsideScope {
        path: PathBuf,
    },
}

impl Witness {
    /// Recheck every exact constraint once within this call; never retain results.
    pub(crate) fn all_current(witnesses: &[Self]) -> bool {
        let mut checked = std::collections::HashSet::new();
        witnesses
            .iter()
            .all(|witness| !checked.insert(witness) || witness.current())
    }

    pub(crate) fn current(&self) -> bool {
        #[cfg(test)]
        crate::work_observations_af_perf_002::witness(self);
        #[cfg(all(test, feature = "angular"))]
        crate::ops_r1_observation::record(10, 1);
        #[cfg(all(test, feature = "angular"))]
        crate::angular_meta::admission_work_metrics::witness();
        match self {
            Self::Directory { path, entries } => {
                #[cfg(test)]
                crate::work_observations_af_perf_002::update(|s| s.enumerations += 1);
                directory_entries(path).is_ok_and(|current| current == *entries)
            }
            Self::Content { path, hash } => {
                #[cfg(test)]
                crate::work_observations_af_perf_002::update(|s| s.reads += 1);
                std::fs::read(path).is_ok_and(|bytes| {
                    #[cfg(test)]
                    crate::work_observations_af_perf_002::update(|s| s.hashes += 1);
                    crate::cache::content_hash(&bytes) == *hash
                })
            }
            Self::Missing { path } => {
                #[cfg(test)]
                crate::work_observations_af_perf_002::update(|s| s.metadata += 1);
                std::fs::symlink_metadata(path)
                    .is_err_and(|error| error.kind() == std::io::ErrorKind::NotFound)
            }
            Self::Kind {
                path,
                file,
                directory,
            } => {
                #[cfg(test)]
                crate::work_observations_af_perf_002::update(|s| s.metadata += 1);
                std::fs::metadata(path).is_ok_and(|metadata| {
                    metadata.is_file() == *file && metadata.is_dir() == *directory
                })
            }
            Self::Physical { path, target } => {
                #[cfg(test)]
                crate::work_observations_af_perf_002::update(|s| s.canonicalizations += 1);
                path.canonicalize()
                    .is_ok_and(|current| shape(&current) == shape(target))
            }
            Self::Unreadable { .. } | Self::OutsideScope { .. } => false,
        }
    }
}

/// One operation owns immutable bytes and every observation used to resolve them.
pub(crate) struct Host {
    pub(super) roots: Vec<PathBuf>,
    pub(super) aliases: Vec<(PathBuf, PathBuf)>,
    pub(super) sources: BTreeMap<PathBuf, Option<Arc<str>>>,
    pub(super) witnesses: super::acquisition_inputs::InputLog,
    pub(super) source_inputs: BTreeMap<PathBuf, Arc<super::acquisition_inputs::SourceInput>>,
    source_receipts: BTreeMap<PathBuf, super::acquisition_inputs::Inputs>,
    pub(super) shared_inputs: Vec<super::shared_acquisition::BoundProgram>,
    pub(super) supplied_inputs: BTreeMap<PathBuf, Arc<str>>,
}

impl Host {
    pub(super) fn directory(&mut self, path: &Path) -> Result<bool, String> {
        let path = self
            .normalized(path)
            .ok_or("Directory outside authorized scope")?;
        match std::fs::metadata(&path) {
            Ok(metadata) => {
                let directory = metadata.is_dir();
                self.witnesses.push(Witness::Kind {
                    path,
                    file: metadata.is_file(),
                    directory,
                });
                Ok(directory)
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                self.witnesses.push(Witness::Missing { path });
                Ok(false)
            }
            Err(error) => {
                self.witnesses.push(Witness::Unreadable { path });
                Err(error.to_string())
            }
        }
    }
    pub(super) fn entries(&mut self, path: &Path) -> Result<Vec<PathBuf>, String> {
        let physical = self
            .normalized(path)
            .ok_or("Directory outside authorized scope")?;
        if !self.admits(&physical) {
            return Err("Directory outside authorized scope".into());
        }
        let entries = directory_entries(&physical).map_err(|error| error.to_string())?;
        if self.normalized(path).as_ref() != Some(&physical) {
            return Err("Directory route changed during inventory".into());
        }
        self.witnesses.push(Witness::Directory {
            path: physical,
            entries: entries.clone(),
        });
        Ok(entries)
    }
    pub(crate) fn seed(&mut self, path: &Path, source: &str) -> Result<(), String> {
        let scope = self.witnesses.begin();
        self.witnesses.roots();
        let path = self.normalized(path);
        let backing = self.witnesses.finish(scope);
        let path = path.ok_or("Source snapshot outside scope")?;
        if let Some(existing) = self.sources.get(&path) {
            if existing.as_deref() != Some(source) {
                return Err("Cannot replace an already consumed TypeScript snapshot".into());
            }
        } else {
            let bytes: Arc<str> = Arc::from(source);
            let input = Arc::new(super::acquisition_inputs::SourceInput {
                path: path.clone(),
                bytes: Arc::clone(&bytes),
                supplied: true,
                accounted: scope != usize::MAX,
                backing,
            });
            self.source_inputs.insert(path.clone(), input);
            let input = &self.source_inputs[&path];
            self.source_receipts.insert(
                path.clone(),
                super::acquisition_inputs::source_receipt(input),
            );
            self.sources.insert(path.clone(), Some(bytes));
        }
        self.supplied_inputs.insert(path, Arc::from(source));
        Ok(())
    }
    pub(crate) fn new(roots: &[PathBuf]) -> Result<Self, String> {
        let aliases: Vec<_> = roots
            .iter()
            .map(|root| {
                let route = if root.is_absolute() {
                    root.clone()
                } else {
                    std::env::current_dir()
                        .map_err(|error| error.to_string())?
                        .join(root)
                };
                route
                    .canonicalize()
                    .map_err(|error| error.to_string())
                    .and_then(|path| {
                        if path.to_str().is_none() || route.to_str().is_none() {
                            return Err(
                                "Authorized root has no lossless UTF-8 identity".to_string()
                            );
                        }
                        Ok((shape(&route), shape(&path)))
                    })
            })
            .collect::<Result<_, _>>()?;
        if aliases.is_empty() {
            return Err("TypeScript resolution requires explicit authorized roots".into());
        }
        Ok(Self {
            roots: aliases
                .iter()
                .map(|(_, physical)| physical.clone())
                .collect(),
            witnesses: super::acquisition_inputs::InputLog::new(
                aliases
                    .iter()
                    .map(|(route, physical)| Witness::Physical {
                        path: route.clone(),
                        target: physical.clone(),
                    })
                    .collect(),
            ),
            source_inputs: BTreeMap::new(),
            source_receipts: BTreeMap::new(),
            shared_inputs: Vec::new(),
            supplied_inputs: BTreeMap::new(),
            aliases,
            sources: BTreeMap::new(),
        })
    }

    pub(crate) fn admits(&self, path: &Path) -> bool {
        self.roots.iter().any(|root| {
            crate::workspace::path_identity::components_are_within(
                &crate::workspace::path_identity::path_components(&shape(path)),
                &crate::workspace::path_identity::path_components(root),
            )
        })
    }

    pub(crate) fn boundary(&self, file: &Path) -> Option<PathBuf> {
        self.roots
            .iter()
            .chain(self.aliases.iter().map(|(route, _)| route))
            .filter(|root| {
                crate::workspace::path_identity::components_are_within(
                    &crate::workspace::path_identity::path_components(&shape(file)),
                    &crate::workspace::path_identity::path_components(root),
                )
            })
            .max_by_key(|root| root.components().count())
            .cloned()
    }

    pub(super) fn logical(&mut self, path: &Path) -> Option<PathBuf> {
        self.normalized(path)?; // Physical scope and route witnesses remain mandatory.
        Some(lexical(&shape(path)))
    }

    pub(crate) fn normalized(&mut self, path: &Path) -> Option<PathBuf> {
        if path.to_str().is_none() {
            self.witnesses.push(Witness::Unreadable {
                path: path.to_path_buf(),
            });
            return None;
        }
        let path = shape(path);
        if !path.is_absolute() {
            return None;
        }
        let mut normalized = lexical(&path);
        if let Some((route, physical)) = self
            .aliases
            .iter()
            .filter(|(route, _)| normalized.starts_with(route))
            .max_by_key(|(route, _)| route.components().count())
        {
            let suffix = normalized.strip_prefix(route).ok()?;
            normalized = if suffix.as_os_str().is_empty() {
                physical.clone()
            } else {
                physical.join(suffix)
            };
        }
        if !self.admits(&normalized) {
            self.witnesses
                .push(Witness::OutsideScope { path: normalized });
            return None;
        }
        // Check the deepest existing ancestor before probing a nonexistent leaf.
        // This prevents a symlinked directory from allowing out-of-root reads.
        let mut ancestor = normalized.as_path();
        let physical = loop {
            match std::fs::symlink_metadata(ancestor) {
                Ok(_) => {
                    let target = match ancestor.canonicalize() {
                        Ok(target) => target,
                        Err(_) => {
                            self.witnesses.push(Witness::Unreadable {
                                path: ancestor.to_path_buf(),
                            });
                            return None;
                        }
                    };
                    if target.to_str().is_none() {
                        self.witnesses.push(Witness::Unreadable { path: target });
                        return None;
                    }
                    if !self.admits(&target) {
                        self.witnesses.push(Witness::OutsideScope { path: target });
                        return None;
                    }
                    self.witnesses.push(Witness::Physical {
                        path: ancestor.to_path_buf(),
                        target: target.clone(),
                    });
                    let suffix = normalized.strip_prefix(ancestor).ok()?;
                    let target = shape(&target);
                    // Joining an empty path adds a trailing separator. An
                    // existing file must remain a file path, not `file.ts/`.
                    break if suffix.as_os_str().is_empty() {
                        target
                    } else {
                        target.join(suffix)
                    };
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    self.witnesses.push(Witness::Missing {
                        path: ancestor.to_path_buf(),
                    });
                    ancestor = ancestor.parent()?;
                }
                Err(_) => {
                    self.witnesses.push(Witness::Unreadable {
                        path: ancestor.to_path_buf(),
                    });
                    return None;
                }
            }
        };
        Some(physical)
    }

    pub(crate) fn read(&mut self, path: &Path) -> Option<Arc<str>> {
        use super::acquisition_inputs::{SourceInput, source_receipt};
        let scope = self.witnesses.begin();
        self.witnesses.roots();
        let normalized = self.normalized(path);
        let Some(path) = normalized else {
            self.witnesses.finish(scope);
            return None;
        };
        if let Some(source) = self.sources.get(&path).cloned() {
            if let Some(inputs) = self.source_receipts.get(&path) {
                self.witnesses.attribute(inputs);
            } else {
                self.witnesses.attribute(&Arc::from([]));
            }
            self.witnesses.finish(scope);
            return source;
        }
        let shared = self
            .shared_inputs
            .iter()
            .find_map(|bound| {
                bound
                    .sources
                    .get(&path)
                    .or_else(|| bound.proof.sources.get(&path))
            })
            .cloned();
        if let Some(input) = shared {
            self.witnesses.attribute(&source_receipt(&input));
            let source = Arc::clone(&input.bytes);
            self.source_inputs.insert(path.clone(), input);
            self.source_receipts
                .insert(path.clone(), source_receipt(&self.source_inputs[&path]));
            self.sources.insert(path, Some(Arc::clone(&source)));
            self.witnesses.finish(scope);
            return Some(source);
        }
        #[cfg(all(test, feature = "angular"))]
        crate::ops_r1_observation::record(8, 1);
        let source = match self.read_bytes(&path) {
            Ok(bytes) => {
                self.witnesses.push(Witness::Content {
                    path: path.clone(),
                    hash: crate::cache::content_hash(&bytes),
                });
                match String::from_utf8(bytes) {
                    Ok(source) => Some(Arc::<str>::from(source)),
                    Err(_) => {
                        self.witnesses
                            .push(Witness::Unreadable { path: path.clone() });
                        None
                    }
                }
            }
            Err(error) => {
                self.witnesses
                    .push(if error.kind() == std::io::ErrorKind::NotFound {
                        Witness::Missing { path: path.clone() }
                    } else {
                        Witness::Unreadable { path: path.clone() }
                    });
                None
            }
        };
        let backing = self.witnesses.finish(scope);
        self.source_receipts
            .insert(path.clone(), Arc::clone(&backing));
        if let Some(bytes) = source.as_ref() {
            let input = Arc::new(SourceInput {
                path: path.clone(),
                bytes: Arc::clone(bytes),
                supplied: false,
                accounted: scope != usize::MAX,
                backing,
            });
            self.witnesses.source(&input);
            self.source_inputs.insert(path.clone(), input);
            self.source_receipts
                .insert(path.clone(), source_receipt(&self.source_inputs[&path]));
        }
        self.sources.insert(path, source.clone());
        source
    }

    fn read_bytes(&mut self, path: &Path) -> std::io::Result<Vec<u8>> {
        let mut file = std::fs::File::open(path)?;
        #[cfg(windows)]
        {
            let target = super::host_windows::opened_path(&file)?;
            if target.to_str().is_none() {
                self.witnesses.push(Witness::Unreadable { path: target });
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Opened source has no lossless UTF-8 identity",
                ));
            }
            if !self.admits(&target) {
                self.witnesses.push(Witness::OutsideScope { path: target });
                return Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "Opened source outside authorized roots",
                ));
            }
            self.witnesses.push(Witness::Physical {
                path: path.to_path_buf(),
                target,
            });
        }
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        Ok(bytes)
    }

    pub(crate) fn file(&mut self, path: &Path) -> bool {
        let Some(path) = self.normalized(path) else {
            return false;
        };
        match std::fs::metadata(&path) {
            Ok(metadata) => {
                self.witnesses.push(Witness::Kind {
                    path,
                    file: metadata.is_file(),
                    directory: metadata.is_dir(),
                });
                metadata.is_file()
            }
            Err(error) => {
                self.witnesses
                    .push(if error.kind() == std::io::ErrorKind::NotFound {
                        Witness::Missing { path }
                    } else {
                        Witness::Unreadable { path }
                    });
                false
            }
        }
    }

    pub(crate) fn current(&self) -> bool {
        Witness::all_current(&self.witnesses)
    }
}

pub(super) fn lexical(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for part in path.components() {
        match part {
            std::path::Component::ParentDir => {
                result.pop();
            }
            std::path::Component::CurDir => {}
            _ => result.push(part.as_os_str()),
        }
    }
    result
}

fn directory_entries(path: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut entries = std::fs::read_dir(path)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<std::io::Result<Vec<_>>>()?;
    entries.sort();
    Ok(entries)
}

fn shape(path: &Path) -> PathBuf {
    #[cfg(windows)]
    if let Some(text) = path.to_str() {
        if let Some(rest) = text.strip_prefix("\\\\?\\UNC\\") {
            return PathBuf::from(format!("\\\\{rest}"));
        }
        if let Some(rest) = text.strip_prefix("\\\\?\\") {
            return PathBuf::from(rest);
        }
    }
    path.to_path_buf()
}
