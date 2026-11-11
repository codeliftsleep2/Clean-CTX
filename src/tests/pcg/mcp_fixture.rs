//! Test-only bridge: reuse the accepted fixture without exposing private PCG owners.
use serde_json::Value;
use std::{fs, path::PathBuf};

pub(crate) struct Fixture {
    pub dir: tempfile::TempDir,
    pub root: PathBuf,
    pub profile_path: PathBuf,
    pub profile: Value,
    pub generation: String,
}
impl Fixture {
    pub fn new() -> Self {
        let f = super::support::Fixture::new();
        let generation = f.run(false, false).unwrap().generation;
        Self {
            dir: f.dir,
            root: f.root,
            profile_path: f.profile_path,
            profile: f.profile,
            generation,
        }
    }
    pub fn database(&self) -> PathBuf {
        self.dir.path().join("projection.db")
    }
    pub fn write_profile(&self) {
        fs::write(
            &self.profile_path,
            serde_json::to_vec(&self.profile).unwrap(),
        )
        .unwrap();
    }
}
