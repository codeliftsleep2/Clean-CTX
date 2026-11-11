//! Explicit environment-selected PCG capability, independent of source persistence.
use crate::pcg::session::Session;
use std::path::PathBuf;

pub(crate) struct PcgState {
    pub session: Option<Session>,
}
impl PcgState {
    pub fn from_environment() -> Self {
        Self::from_path(std::env::var_os("CLEAN_CTX_PCG_PROFILE").map(PathBuf::from))
    }
    pub fn from_path(path: Option<PathBuf>) -> Self {
        Self {
            session: path.map(Session::new),
        }
    }
    pub fn enabled(&self) -> bool {
        self.session.is_some()
    }
}
