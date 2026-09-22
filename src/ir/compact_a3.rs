//! Research-only COMPACT-A3 sparse positional codec.

mod declarations;
pub mod document;
pub mod facts;

pub use declarations::{decode_declarations, encode_declarations, SCHEMA_VERSION};

pub const COLD_PREAMBLE: &str =
    "// COMPACT-A A3; sparse file context; workspace graph via workspace_query";
pub const COLD_LEGEND: &str = "S A3 |=columns q=JSON-string -=absent; C/I owner X=extends J=implements F=field M=method p=param; cm/cf/mo/cs/pf/lf occurrence-groups pt=pattern D=injection; V=behavior-owner fc/fd/se/ec facts; K=caller k=unresolved-call(argc,* spread); $=import T=alias; Y=body-count B=exact(method,start,end,bytes); Z=counts. Typed IDs, row order, duplicates, and empty groups are significant.";
