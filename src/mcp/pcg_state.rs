//! Explicit environment-selected PCG capability, independent of source persistence.
use crate::pcg::session::Session;
use std::path::PathBuf;

pub(crate) struct PcgState {
    pub session: Option<Session>,
    pub documents: Option<crate::pcg::documents::Session>,
}
impl PcgState {
    pub fn from_environment() -> Self {
        let mut state =
            Self::from_path(std::env::var_os("CLEAN_CTX_PCG_PROFILE").map(PathBuf::from));
        state.documents = std::env::var_os("CLEAN_CTX_PCG_DOCUMENT_PROFILE")
            .map(PathBuf::from)
            .map(crate::pcg::documents::Session::new);
        state
    }
    pub fn from_path(path: Option<PathBuf>) -> Self {
        Self {
            session: path.map(Session::new),
            documents: None,
        }
    }
    pub fn enabled(&self) -> bool {
        self.session.is_some()
    }
}
