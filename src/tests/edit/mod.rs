// src/tests/edit/mod.rs
//
// Test aggregator for the src/edit module tree. Individual test files are
// wired via `#[cfg(test)] #[path = "../tests/edit/<file>.rs"]` blocks in
// their corresponding production modules.

pub(crate) mod apply;
pub(crate) mod locate;
pub(crate) mod ops;
