// Core MCP handlers, decomposed by responsibility.

mod common;
mod compress;
mod delta;
mod provide;
#[cfg(feature = "angular")]
mod provide_angular;
mod restore;

#[cfg(test)]
pub(crate) use common::{contract_fields, contract_fields_focused, maybe_economics_fallback};
pub(crate) use compress::handle_compress_code_context;
pub(crate) use delta::{handle_apply_delta, handle_delta_code_context, handle_diff_code_context};
pub(crate) use provide::handle_provide_code_context;
pub(crate) use restore::handle_restore_context;

#[cfg(test)]
#[path = "../../tests/mcp/projection_contract.rs"]
mod projection_contract_tests;
