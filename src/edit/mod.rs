// src/edit/mod.rs
//
// apply_edit write path core (docs/plans/APPLY_EDIT_PLAN.md Phase 2).
//
// This module is deliberately pure: it transforms source text against a
// caller-supplied unit table and never touches `McpState`, the filesystem,
// or session caches. The MCP handler (`mcp/tool_handlers/edit.rs`) is the
// thin adapter that wires this core to session state (hash registry, IR
// context, persistence).
//
// Layout:
//   ops.rs    — EditOperation model (ReplaceBody / InsertAfter /
//               InsertBefore / Delete) and per-op outcome records.
//   locate.rs — Unit relocation: build a splice-addressable unit table
//               from compiled IR `CoreOp::Body` spans, keyed on qualified
//               name + structural fingerprint (plan Risk Analysis #1).
//   apply.rs  — Verification, splicing, overlap checks, and the in-memory
//               tree-sitter syntax gate (hard pre-commit check, plan step 4).

pub mod apply;
pub mod locate;
pub mod ops;

#[cfg(test)]
#[path = "../tests/edit/ops.rs"]
mod ops_tests;

#[cfg(test)]
#[path = "../tests/edit/locate.rs"]
mod locate_tests;

#[cfg(test)]
#[path = "../tests/edit/apply.rs"]
mod apply_tests;

#[cfg(test)]
#[path = "../tests/edit/spans.rs"]
mod spans_tests;
