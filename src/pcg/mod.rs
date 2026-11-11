//! Owner-enrolled projection and explicit read-only acquisition. No runtime approval API.
pub mod cli;
mod closure;
mod enrollment;
mod git;
mod identity;
mod import;
mod model;
mod optional;
mod profile;
pub mod projection;
mod provenance;
pub mod query;
mod query_rows;
mod query_witnesses;
mod security;
mod selectors;
pub(crate) mod session;
mod store;
mod validation;
mod wire;

use std::fmt;

/// Fixed, non-disclosing diagnostics: never include source bytes, paths or SQL errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error(pub &'static str);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "pcg/{}", self.0)
    }
}
impl std::error::Error for Error {}
type Result<T> = std::result::Result<T, Error>;

fn require(condition: bool, code: &'static str) -> Result<()> {
    if condition { Ok(()) } else { Err(Error(code)) }
}

#[cfg(test)]
#[path = "../tests/pcg/enrollment.rs"]
mod enrollment_tests;
#[cfg(test)]
#[path = "../tests/pcg/persistence.rs"]
mod persistence_tests;
#[cfg(test)]
#[path = "../tests/pcg/process.rs"]
mod process_tests;
#[cfg(test)]
#[path = "../tests/pcg/security.rs"]
mod security_tests;
#[cfg(test)]
#[path = "../tests/pcg/support.rs"]
mod support;

#[cfg(test)]
#[path = "../tests/pcg/mcp_fixture.rs"]
pub(crate) mod mcp_fixture;

#[cfg(test)]
#[path = "../tests/pcg/query.rs"]
mod query_tests;
