//! Layer-agnostic parsing utilities shared across ALL meta-layers
//! (Angular, Spring, .NET).
//!
//! All scanners across the meta-layers MUST use these primitives instead of
//! hand-rolling their own string/depth awareness. This eliminates the defect
//! class where a fix in one layer is not propagated to duplicated logic in
//! another (Round-8 structural audit).
//!
//! This module is deliberately free of any Angular/Spring/.NET-specific
//! vocabulary so every meta-layer can depend on it without a layering
//! violation. `angular_meta::util` re-exports these for backward
//! compatibility with the Angular sub-layers.

mod class_source;
mod declaration;
mod scanner;

pub use class_source::{
    class_source_from_capture, find_class_source_start, find_decorator_inclusive_start,
};
pub use declaration::{extract_decl_name, extract_entity_type, extract_first_quoted};
pub use scanner::{
    collect_call_body, consume_call_expression, extract_quoted_value, find_enclosing_brace,
    find_first_top_level, find_matching_brace, is_inside_comment_or_string, skip_string,
    skip_template, split_top_level,
};

#[cfg(test)]
#[path = "tests/meta_util.rs"]
mod tests;
