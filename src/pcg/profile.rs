use super::{Error, Result, identity, model::Profile, require, security};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};

pub(super) struct Trusted {
    pub profile: Profile,
    pub path: PathBuf,
    pub root: PathBuf,
    bytes: Vec<u8>,
    permissions: Vec<u8>,
}

impl Trusted {
    pub fn load(path: &Path) -> Result<Self> {
        require(path.is_absolute(), "profile_path")?;
        security::no_links(path, false)?;
        let bytes = fs::read(path).map_err(|_| Error("profile_unavailable"))?;
        let value = identity::parse(&bytes)?;
        security::json_nonsecret(&value)?;
        let profile: Profile =
            serde_json::from_value(value).map_err(|_| Error("profile_schema"))?;
        require(profile.schema_version == 1, "profile_schema")?;
        for v in [
            &profile.partition_id,
            &profile.workspace_id,
            &profile.policy_revision,
        ] {
            identity::id(v)?;
        }
        require(!profile.owner_label.is_empty(), "profile_schema")?;
        identity::hex(&profile.approved_manifest_sha256, 64)?;
        let limits = &profile.limits;
        require(
            [
                limits.records,
                limits.edges,
                limits.evidence_bytes,
                limits.token_budget,
            ]
            .iter()
            .all(|n| *n > 0 && *n <= i64::MAX as u64),
            "profile_limits",
        )?;
        for p in [
            &profile.workspace_root,
            &profile.database_path,
            &profile.manifest_path,
        ] {
            require(Path::new(p).is_absolute(), "profile_path")?;
            security::no_links(Path::new(p), p == &profile.database_path)?;
        }
        let root = Path::new(&profile.workspace_root)
            .canonicalize()
            .map_err(|_| Error("profile_root"))?;
        require(root.is_dir(), "profile_root")?;
        let database = Path::new(&profile.database_path);
        let manifest = Path::new(&profile.manifest_path);
        require(
            database != path && database != manifest && path != manifest,
            "profile_isolation",
        )?;
        // WAL/SHM companions are part of the same derived store and cannot alias
        // profile/manifest inputs or follow a pre-existing reparse point.
        for suffix in ["-wal", "-shm", "-journal"] {
            let companion = PathBuf::from(format!("{}{suffix}", profile.database_path));
            require(
                companion != path && companion != manifest,
                "profile_isolation",
            )?;
            security::no_links(&companion, true)?;
        }
        let permissions = security::permissions(path)?;
        Ok(Self {
            profile,
            path: path.to_owned(),
            root,
            bytes,
            permissions,
        })
    }

    pub fn recheck(&self) -> Result<()> {
        security::no_links(&self.path, false)?;
        require(
            security::permissions(&self.path)? == self.permissions
                && fs::read(&self.path).map_err(|_| Error("profile_changed"))? == self.bytes,
            "profile_changed",
        )
    }

    pub fn manifest(&self, work: &mut Work) -> Result<Vec<u8>> {
        security::no_links(Path::new(&self.profile.manifest_path), false)?;
        let bytes = work.read(Path::new(&self.profile.manifest_path))?;
        require(
            identity::sha(&bytes) == self.profile.approved_manifest_sha256,
            "manifest_digest",
        )?;
        Ok(bytes)
    }
}

#[derive(Default, serde::Serialize)]
pub(super) struct Work {
    pub bytes: u64,
    pub current_sources: u64,
    pub current_sources_completion: u64,
    pub current_bytes: u64,
    pub git_bytes: u64,
    pub evidence_bytes: u64,
    pub definitions: u64,
    pub assertions_and_refs: u64,
    pub evidence_validations: u64,
    pub evidence_hash_microseconds: u128,
    pub generation_microseconds: u128,
    #[serde(skip)]
    pub ceiling: u64,
}
impl Work {
    pub fn new(ceiling: u64) -> Self {
        Self {
            ceiling,
            ..Self::default()
        }
    }
    pub fn debit(&mut self, bytes: u64) -> Result<()> {
        self.bytes = self.bytes.checked_add(bytes).ok_or(Error("work_limit"))?;
        require(self.bytes <= self.ceiling, "work_limit")
    }
    pub fn read(&mut self, path: &Path) -> Result<Vec<u8>> {
        let mut file = fs::File::open(path).map_err(|_| Error("source_inaccessible"))?;
        require(
            file.metadata()
                .map_err(|_| Error("source_inaccessible"))?
                .is_file(),
            "unsafe_path",
        )?;
        let mut result = Vec::new();
        let mut chunk = [0u8; 8192];
        loop {
            let remaining = self.ceiling.saturating_sub(self.bytes);
            if remaining == 0 {
                // Do not consume even a lookahead byte past the owner ceiling.
                // Like every filesystem observation, this length check is not
                // an atomic snapshot against an uncooperative concurrent writer.
                require(
                    file.metadata()
                        .map_err(|_| Error("source_inaccessible"))?
                        .len()
                        <= result.len() as u64,
                    "work_limit",
                )?;
                break;
            }
            let nmax = remaining.min(chunk.len() as u64) as usize;
            let n = file
                .read(&mut chunk[..nmax])
                .map_err(|_| Error("source_inaccessible"))?;
            if n == 0 {
                break;
            }
            self.debit(n as u64)?;
            result.extend_from_slice(&chunk[..n]);
        }
        Ok(result)
    }
}
