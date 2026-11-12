use crate::pcg::{Error, Result, identity, require, security};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub schema_version: u32,
    pub workspace_root: PathBuf,
    pub database_path: PathBuf,
    pub document_roots: Vec<String>,
    pub max_files: usize,
    pub max_nodes: usize,
    pub max_edges: usize,
    pub max_bytes: u64,
    /// Legacy profile metadata; retained without limiting architectural knowledge.
    #[serde(default)]
    pub token_budget: u64,
}
pub struct Trusted {
    pub profile: Profile,
    pub fingerprint: String,
    pub scope_fingerprint: String,
    path: PathBuf,
    bytes: Vec<u8>,
    permissions: Vec<u8>,
}
impl Trusted {
    pub fn load(path: &Path) -> Result<Self> {
        require(path.is_absolute(), "document_profile_path")?;
        security::no_links(path, false)?;
        let bytes = fs::read(path).map_err(|_| Error("document_profile_unavailable"))?;
        let value = identity::parse(&bytes)?;
        security::json_nonsecret(&value)?;
        let mut profile: Profile =
            serde_json::from_value(value).map_err(|_| Error("document_profile_schema"))?;
        require(
            profile.schema_version == 1
                && profile.max_files > 0
                && profile.max_nodes > 0
                && profile.max_edges > 0
                && profile.max_bytes > 0
                && !profile.document_roots.is_empty(),
            "document_profile_schema",
        )?;
        require(
            profile.workspace_root.is_absolute() && profile.database_path.is_absolute(),
            "document_profile_path",
        )?;
        security::no_links(&profile.workspace_root, false)?;
        security::no_links(&profile.database_path, true)?;
        let root = profile
            .workspace_root
            .canonicalize()
            .map_err(|_| Error("document_profile_root"))?;
        require(root.is_dir(), "document_profile_root")?;
        profile.workspace_root = root.clone();
        profile.database_path = if profile.database_path.exists() {
            profile
                .database_path
                .canonicalize()
                .map_err(|_| Error("document_profile_path"))?
        } else {
            let parent = profile
                .database_path
                .parent()
                .ok_or(Error("document_profile_path"))?
                .canonicalize()
                .map_err(|_| Error("document_profile_path"))?;
            parent.join(
                profile
                    .database_path
                    .file_name()
                    .ok_or(Error("document_profile_path"))?,
            )
        };
        let profile_identity = path
            .canonicalize()
            .map_err(|_| Error("document_profile_path"))?;
        let mut unique = BTreeSet::new();
        for dir in &profile.document_roots {
            security::relative(dir)?;
            let dir = root.join(dir);
            security::no_links(&dir, false)?;
            require(
                dir.is_dir() && unique.insert(dir.clone()),
                "document_profile_roots",
            )?;
            require(
                !profile.database_path.starts_with(&dir) && !profile_identity.starts_with(&dir),
                "document_profile_isolation",
            )?;
        }
        for suffix in ["-wal", "-shm", "-journal"] {
            security::no_links(
                &PathBuf::from(format!("{}{suffix}", profile.database_path.display())),
                true,
            )?;
        }
        Ok(Self {
            scope_fingerprint: identity::hash(
                &serde_json::json!({"workspace_root":profile.workspace_root,"document_roots":profile.document_roots,"database_path":profile.database_path}),
            ),
            profile,
            fingerprint: identity::sha(&bytes),
            path: path.into(),
            permissions: security::permissions(path)?,
            bytes,
        })
    }
    pub fn recheck(&self) -> Result<()> {
        security::no_links(&self.path, false)?;
        require(
            fs::read(&self.path).map_err(|_| Error("document_profile_changed"))? == self.bytes
                && security::permissions(&self.path)? == self.permissions,
            "document_profile_changed",
        )
    }
}
