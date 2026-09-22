// Core MCP handlers, decomposed by responsibility.

mod common;
mod compress;
pub(crate) mod content;
mod delta;
mod delta_apply;
mod provide;
#[cfg(feature = "angular")]
mod provide_angular;
mod provide_persistence;
mod restore;

pub(crate) use common::projection_error_response;
#[cfg(all(test, feature = "rust"))]
pub(crate) use common::{contract_fields, contract_fields_focused};
pub(crate) use compress::handle_compress_code_context;
pub(crate) use delta::{handle_delta_code_context, handle_diff_code_context};
pub(crate) use delta_apply::handle_apply_delta;
pub(crate) use provide::handle_provide_code_context;
pub(crate) use restore::handle_restore_context;

#[cfg(test)]
#[path = "../../tests/mcp/projection_contract.rs"]
mod projection_contract_tests;

#[cfg(all(test, feature = "typescript"))]
#[path = "../../tests/mcp/control_full_content.rs"]
mod control_full_content_tests;
