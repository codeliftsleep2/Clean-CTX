//! Shared parsing utilities for the Angular meta-layer.
//!
//! All scanners across the Angular meta-layers (RxJS, NgRx, Signals, Routing)
//! MUST use these primitives instead of hand-rolling their own string/depth
//! awareness. This eliminates the defect class where a fix in one layer is
//! not propagated to duplicated logic in another.
//!
//! The implementations live in the layer-agnostic [`crate::meta_util`] module
//! so the Spring and .NET meta-layers can share the SAME primitives without
//! depending on Angular-specific code (Round-8 structural audit). This module
//! re-exports them for the Angular sub-layers.

pub use crate::meta_util::{
    collect_call_body, consume_call_expression, extract_decl_name, extract_entity_type,
    extract_first_quoted, extract_quoted_value, find_enclosing_brace, find_first_top_level,
    find_matching_brace, skip_string, skip_template, split_top_level,
};

#[cfg(test)]
#[path = "../tests/angular_meta/util.rs"]
mod tests;