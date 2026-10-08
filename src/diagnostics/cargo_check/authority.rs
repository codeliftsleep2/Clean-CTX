use serde::Serialize;
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthoritySource {
    StartupOption,
    Environment,
    CliArgument,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FileIdentity {
    pub length: u64,
    pub modified_unix_nanos: Option<u128>,
    pub sha256: String,
    #[cfg(unix)]
    pub device: u64,
    #[cfg(unix)]
    pub inode: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AuthorityError {
    #[error("approved path must be absolute")]
    RelativePath,
    #[error("approved path could not be resolved")]
    UnresolvablePath,
    #[error("approved workspace is not a directory")]
    WorkspaceNotDirectory,
    #[error("approved workspace does not contain Cargo.toml")]
    MissingManifest,
    #[error("approved Cargo path is not a regular file")]
    CargoNotFile,
    #[error("approved Cargo path is not an executable for this platform")]
    CargoNotExecutable,
    #[error("approved authority changed after admission")]
    IdentityChanged,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovedWorkspaceRoot {
    canonical_root: PathBuf,
    manifest_identity: FileIdentity,
    source: AuthoritySource,
}

impl ApprovedWorkspaceRoot {
    pub fn admit(path: &Path, source: AuthoritySource) -> Result<Self, AuthorityError> {
        require_absolute(path)?;
        let canonical_root = path
            .canonicalize()
            .map_err(|_| AuthorityError::UnresolvablePath)?;
        if !canonical_root.is_dir() {
            return Err(AuthorityError::WorkspaceNotDirectory);
        }
        let manifest = canonical_root.join("Cargo.toml");
        if !manifest.is_file() {
            return Err(AuthorityError::MissingManifest);
        }
        let manifest_identity = file_identity(&manifest)?;
        Ok(Self {
            canonical_root,
            manifest_identity,
            source,
        })
    }

    pub fn canonical_root(&self) -> &Path {
        &self.canonical_root
    }

    pub fn source(&self) -> AuthoritySource {
        self.source
    }

    pub(crate) fn manifest_identity(&self) -> &FileIdentity {
        &self.manifest_identity
    }

    pub fn revalidate(&self) -> Result<(), AuthorityError> {
        let canonical = self
            .canonical_root
            .canonicalize()
            .map_err(|_| AuthorityError::IdentityChanged)?;
        if canonical != self.canonical_root || !canonical.is_dir() {
            return Err(AuthorityError::IdentityChanged);
        }
        let current = file_identity(&canonical.join("Cargo.toml"))
            .map_err(|_| AuthorityError::IdentityChanged)?;
        if current != self.manifest_identity {
            return Err(AuthorityError::IdentityChanged);
        }
        Ok(())
    }

    pub fn display_path(&self, path: &Path) -> DisplayPath {
        let normalized = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.canonical_root.join(path)
        };
        match relative_display_path(&self.canonical_root, &normalized) {
            Some(relative) => DisplayPath {
                value: relative,
                classification: PathClassification::InsideWorkspace,
            },
            _ => DisplayPath {
                value: path
                    .file_name()
                    .map(|name| format!("<external>/{}", name.to_string_lossy()))
                    .unwrap_or_else(|| "<external>".to_owned()),
                classification: PathClassification::External,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovedCargoExecutable {
    canonical_path: PathBuf,
    identity: FileIdentity,
    source: AuthoritySource,
}

impl ApprovedCargoExecutable {
    pub fn admit(path: &Path, source: AuthoritySource) -> Result<Self, AuthorityError> {
        require_absolute(path)?;
        let canonical_path = path
            .canonicalize()
            .map_err(|_| AuthorityError::UnresolvablePath)?;
        if !canonical_path.is_file() {
            return Err(AuthorityError::CargoNotFile);
        }
        if !is_platform_executable(&canonical_path)? {
            return Err(AuthorityError::CargoNotExecutable);
        }
        let identity = file_identity(&canonical_path)?;
        Ok(Self {
            canonical_path,
            identity,
            source,
        })
    }

    pub fn canonical_path(&self) -> &Path {
        &self.canonical_path
    }

    pub fn source(&self) -> AuthoritySource {
        self.source
    }

    pub fn identity(&self) -> &FileIdentity {
        &self.identity
    }

    pub fn revalidate(&self) -> Result<(), AuthorityError> {
        let canonical = self
            .canonical_path
            .canonicalize()
            .map_err(|_| AuthorityError::IdentityChanged)?;
        if canonical != self.canonical_path || !canonical.is_file() {
            return Err(AuthorityError::IdentityChanged);
        }
        if !is_platform_executable(&canonical).unwrap_or(false)
            || file_identity(&canonical).ok().as_ref() != Some(&self.identity)
        {
            return Err(AuthorityError::IdentityChanged);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PathClassification {
    InsideWorkspace,
    External,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DisplayPath {
    pub value: String,
    pub classification: PathClassification,
}

fn require_absolute(path: &Path) -> Result<(), AuthorityError> {
    if path.is_absolute() {
        Ok(())
    } else {
        Err(AuthorityError::RelativePath)
    }
}

fn file_identity(path: &Path) -> Result<FileIdentity, AuthorityError> {
    let mut file = std::fs::File::open(path).map_err(|_| AuthorityError::UnresolvablePath)?;
    let metadata = file
        .metadata()
        .map_err(|_| AuthorityError::UnresolvablePath)?;
    let modified_unix_nanos = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_nanos());
    let sha256 = hash_file(&mut file)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        Ok(FileIdentity {
            length: metadata.len(),
            modified_unix_nanos,
            sha256,
            device: metadata.dev(),
            inode: metadata.ino(),
        })
    }
    #[cfg(not(unix))]
    {
        Ok(FileIdentity {
            length: metadata.len(),
            modified_unix_nanos,
            sha256,
        })
    }
}

fn hash_file(file: &mut std::fs::File) -> Result<String, AuthorityError> {
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|_| AuthorityError::UnresolvablePath)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

#[cfg(windows)]
fn is_platform_executable(path: &Path) -> Result<bool, AuthorityError> {
    Ok(path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("exe")))
}

#[cfg(unix)]
fn is_platform_executable(path: &Path) -> Result<bool, AuthorityError> {
    use std::os::unix::fs::PermissionsExt;
    let mode = path
        .metadata()
        .map_err(|_| AuthorityError::UnresolvablePath)?
        .permissions()
        .mode();
    Ok(mode & 0o111 != 0)
}

#[cfg(not(any(windows, unix)))]
fn is_platform_executable(_path: &Path) -> Result<bool, AuthorityError> {
    Ok(false)
}

fn contains_parent_escape(path: &Path) -> bool {
    path.components()
        .any(|component| matches!(component, std::path::Component::ParentDir))
}

#[cfg(not(windows))]
fn relative_display_path(root: &Path, candidate: &Path) -> Option<String> {
    let relative = candidate.strip_prefix(root).ok()?;
    (!contains_parent_escape(relative)).then(|| relative.to_string_lossy().replace('\\', "/"))
}

#[cfg(windows)]
fn relative_display_path(root: &Path, candidate: &Path) -> Option<String> {
    if contains_parent_escape(candidate) {
        return None;
    }
    let root = windows_display_spelling(root);
    let candidate = windows_display_spelling(candidate);
    let root = root.trim_end_matches('/');
    let prefix = candidate.get(..root.len())?;
    if !prefix.eq_ignore_ascii_case(root) {
        return None;
    }
    let relative = &candidate[root.len()..];
    if !relative.is_empty() && !relative.starts_with('/') {
        return None;
    }
    Some(relative.trim_start_matches('/').to_owned())
}

#[cfg(windows)]
fn windows_display_spelling(path: &Path) -> String {
    let value = path.to_string_lossy().replace('\\', "/");
    value
        .strip_prefix("//?/UNC/")
        .map(|suffix| format!("//{suffix}"))
        .unwrap_or_else(|| value.strip_prefix("//?/").unwrap_or(&value).to_owned())
}
