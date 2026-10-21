use std::collections::BTreeMap;
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

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) enum Witness {
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
    pub(crate) fn current(&self) -> bool {
        match self {
            Self::Content { path, hash } => {
                std::fs::read(path).is_ok_and(|bytes| crate::cache::content_hash(&bytes) == *hash)
            }
            Self::Missing { path } => std::fs::symlink_metadata(path)
                .is_err_and(|error| error.kind() == std::io::ErrorKind::NotFound),
            Self::Kind {
                path,
                file,
                directory,
            } => std::fs::metadata(path).is_ok_and(|metadata| {
                metadata.is_file() == *file && metadata.is_dir() == *directory
            }),
            Self::Physical { path, target } => path
                .canonicalize()
                .is_ok_and(|current| shape(&current) == shape(target)),
            Self::Unreadable { .. } | Self::OutsideScope { .. } => false,
        }
    }
}

/// One operation owns immutable bytes and every observation used to resolve them.
pub(crate) struct Host {
    roots: Vec<PathBuf>,
    aliases: Vec<(PathBuf, PathBuf)>,
    sources: BTreeMap<PathBuf, Option<Arc<str>>>,
    pub witnesses: Vec<Witness>,
}

impl Host {
    pub(crate) fn seed(&mut self, path: &Path, source: &str) -> Result<(), String> {
        let path = self
            .normalized(path)
            .ok_or("Source snapshot outside scope")?;
        if let Some(existing) = self.sources.get(&path) {
            return if existing.as_deref() == Some(source) {
                Ok(())
            } else {
                Err("Cannot replace an already consumed TypeScript snapshot".into())
            };
        }
        self.sources.insert(path, Some(Arc::from(source)));
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
                    .map(|path| (shape(&route), shape(&path)))
                    .map_err(|error| error.to_string())
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
            witnesses: aliases
                .iter()
                .map(|(route, physical)| Witness::Physical {
                    path: route.clone(),
                    target: physical.clone(),
                })
                .collect(),
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
            .filter(|root| {
                crate::workspace::path_identity::components_are_within(
                    &crate::workspace::path_identity::path_components(&shape(file)),
                    &crate::workspace::path_identity::path_components(root),
                )
            })
            .max_by_key(|root| root.components().count())
            .cloned()
    }

    pub(crate) fn normalized(&mut self, path: &Path) -> Option<PathBuf> {
        let path = shape(path);
        if !path.is_absolute() {
            return None;
        }
        let mut normalized = PathBuf::new();
        for part in path.components() {
            match part {
                std::path::Component::ParentDir => {
                    normalized.pop();
                }
                std::path::Component::CurDir => {}
                _ => normalized.push(part.as_os_str()),
            }
        }
        if let Some((route, physical)) = self
            .aliases
            .iter()
            .filter(|(route, _)| normalized.starts_with(route))
            .max_by_key(|(route, _)| route.components().count())
        {
            normalized = physical.join(normalized.strip_prefix(route).ok()?);
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
                    if !self.admits(&target) {
                        self.witnesses.push(Witness::OutsideScope { path: target });
                        return None;
                    }
                    self.witnesses.push(Witness::Physical {
                        path: ancestor.to_path_buf(),
                        target: target.clone(),
                    });
                    break shape(&target).join(normalized.strip_prefix(ancestor).ok()?);
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
        let path = self.normalized(path)?;
        if let Some(source) = self.sources.get(&path) {
            return source.clone();
        }
        let source = match std::fs::read(&path) {
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
        self.sources.insert(path, source.clone());
        source
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
        self.witnesses.iter().all(Witness::current)
    }
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
