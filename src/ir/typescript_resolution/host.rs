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
            Self::Physical { path, target } => {
                path.canonicalize().is_ok_and(|current| current == *target)
            }
            Self::Unreadable { .. } | Self::OutsideScope { .. } => false,
        }
    }
}

/// One operation owns immutable bytes and every observation used to resolve them.
pub(crate) struct Host {
    roots: Vec<PathBuf>,
    sources: BTreeMap<PathBuf, Option<Arc<str>>>,
    pub witnesses: Vec<Witness>,
}

impl Host {
    pub(crate) fn seed(&mut self, path: &Path, source: &str) -> Result<(), String> {
        let path = self
            .normalized(path)
            .ok_or("Source snapshot outside scope")?;
        self.sources.insert(path, Some(Arc::from(source)));
        Ok(())
    }
    pub(crate) fn new(roots: &[PathBuf]) -> Result<Self, String> {
        let roots: Vec<_> = roots
            .iter()
            .map(|root| {
                root.canonicalize()
                    .map(|path| shape(&path))
                    .map_err(|error| error.to_string())
            })
            .collect::<Result<_, _>>()?;
        if roots.is_empty() {
            return Err("TypeScript resolution requires explicit authorized roots".into());
        }
        Ok(Self {
            roots,
            sources: BTreeMap::new(),
            witnesses: Vec::new(),
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
        if !self.admits(&normalized) {
            self.witnesses
                .push(Witness::OutsideScope { path: normalized });
            return None;
        }
        // Check the deepest existing ancestor before probing a nonexistent leaf.
        // This prevents a symlinked directory from allowing out-of-root reads.
        let mut ancestor = normalized.as_path();
        loop {
            match std::fs::symlink_metadata(ancestor) {
                Ok(_) => {
                    let Ok(target) = ancestor.canonicalize() else {
                        return None;
                    };
                    if !self.admits(&target) {
                        self.witnesses.push(Witness::OutsideScope { path: target });
                        return None;
                    }
                    self.witnesses.push(Witness::Physical {
                        path: ancestor.to_path_buf(),
                        target,
                    });
                    break;
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
        }
        Some(normalized)
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
                String::from_utf8(bytes).ok().map(Arc::<str>::from)
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
