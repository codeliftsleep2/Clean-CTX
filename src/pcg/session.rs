//! Pins the trusted profile for an MCP session; never imports or writes a database.
use super::{Error, Result, profile::Trusted, projection, query};
use std::path::PathBuf;

pub(crate) struct Session {
    path: PathBuf,
    trusted: Result<Trusted>,
}
impl Session {
    pub fn new(path: PathBuf) -> Self {
        let trusted = Trusted::load(&path);
        Self { path, trusted }
    }
    pub fn project(
        &self,
        request: &query::Request,
        tokenizer: &str,
        budget: u64,
    ) -> Result<projection::Response> {
        let trusted = self
            .trusted
            .as_ref()
            .map_err(|_| Error("profile_unavailable"))?;
        trusted.recheck().map_err(|_| Error("profile_changed"))?;
        let result = projection::project(&self.path, request, tokenizer, budget);
        trusted.recheck().map_err(|_| Error("profile_changed"))?;
        result
    }
}
