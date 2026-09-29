//! Research-only COMPACT-A3 sparse positional codec.

mod declarations;
pub mod document;
pub mod facts;
mod renumber;

pub use declarations::{SCHEMA_VERSION, decode_declarations, encode_declarations};

pub const COLD_PREAMBLE: &str =
    "// COMPACT-A A3; sparse file context; workspace graph via workspace_query";
pub const COLD_LEGEND: &str = "S A3 |=columns s=bare-string(JSON only when empty/reserved) -=absent; C/I owner X=extends J=implements F=field M=method p=param; cm/cf/mo groups(1 value omits count); $=import T=alias; Y/B exact bodies; Z=counts. Typed IDs, order, duplicates, empty groups matter.";
