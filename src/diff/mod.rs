// src/diff/mod.rs
//
// AST-level diff compression. Given two structural snapshots of a file
// (baseline and current), produce a compact change-set describing
// added / removed / modified classes, methods, fields, and imports.
//
// The output is dramatically smaller than re-emitting the full compressed
// skeleton for the current file when most of the file is unchanged — it
// only carries the deltas, prefixed with `+`, `-`, `~`, and `=` markers
// that LLM tokenizers typically encode as 1 token each.
//
// Module split:
//   - `snapshot`  : CapturedStructure, CapturedClass, CapturedMethod
//   - `action`    : DiffAction, DiffKind, DiffTarget
//   - `builder`   : build_snapshot + try_build_with (the tree-sitter walk)
//   - `differ`    : diff_snapshots + diff_class (the comparison logic)
//   - `formatter` : format_diff + diff_summary
//   - `keys`      : method_key, field_key, group_by_key, summarize_class

pub(crate) mod action;
pub(crate) mod builder;
pub(crate) mod differ;
pub(crate) mod formatter;
pub(crate) mod keys;
pub(crate) mod snapshot;

pub use action::{DiffAction, DiffKind, DiffTarget};
pub use builder::build_snapshot;
pub use differ::diff_snapshots;
pub use formatter::{diff_summary, format_diff};
pub use snapshot::{CapturedClass, CapturedMethod, CapturedStructure};
