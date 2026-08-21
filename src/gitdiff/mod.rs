// src/gitdiff/mod.rs
//
// R-12 Phase 1: Multi-file / git-commit diff.
//
// This module provides the building blocks for the `diff_commits` MCP
// tool: safe git subprocess execution, ref validation, and changed-file
// collection. The full diff engine (AST snapshots + `format_diff`
// rendering) is wired in Phase 2.
//
// Module split:
//   - `refs`     : validate_ref + resolve_ref (security-critical)
//   - `runner`   : run_git (safe subprocess execution)
//   - `workspace`: collect_changed_files + show_file + FileChange
//   - `engine`   : gitdiff_workspace orchestrator (Phase 2)

pub(crate) mod engine;
pub(crate) mod refs;
pub(crate) mod runner;
pub(crate) mod workspace;

pub use engine::{GitDiffSummary, gitdiff_workspace};
pub use refs::{resolve_ref, validate_ref};
pub use runner::{is_git_repo, run_git};
pub use workspace::{FileChange, collect_changed_files, show_file};
