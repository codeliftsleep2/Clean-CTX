// src/compression/mod.rs
//
// Shared building blocks for the compression pipeline, plus the
// orchestrator (`compress_file`).
//
// # Phase 2 foundations (shared single-source modules)
//
//   - `fidelity`          : the `Fidelity` enum + `from_str` parser
//   - `language`          : centralized language detection (extension + heuristic)
//   - `opcodes`           : SHARED primitive opcode table (consumed by symbol
//                           dictionary and decompressor)
//   - `markers`           : SHARED marker construction (`build_marker`) and
//                           expansion (`expand_markers_in_line`)
//   - `capture_pipeline`  : SHARED tree-sitter capture-walk that yields a
//                           sorted `Vec<CapEntry>` from a parsed tree
//
// # Phase 3 orchestrators (extracted from the 601-line `compressor.rs`)
//
//   - `symbol_compression`: Low-fidelity opcode pass
//   - `report`            : Final optimisation-report formatting
//   - `pipeline`          : Non-streaming `compress_file` + shared helpers

pub(crate) mod capture_pipeline;
pub(crate) mod graph_utils;
pub(crate) mod markers;
pub(crate) mod micro_opcodes;
pub(crate) mod opcodes;
pub(crate) mod pipeline;
pub(crate) mod report;
pub(crate) mod symbol_compression;
// R-02: Type-aware compression — replaces configured type names with
// short alias tokens (`UserId` → `$uid`) and emits a reversible `§TA`
// footer. Wired into `compress_file_with_source`, `compress_text`, and
// `compress_source` in `pipeline.rs`.
pub mod fidelity;
pub mod language;
pub(crate) mod type_aliases;

// Re-export shared types for downstream callers.
//
// `Fidelity` is `pub` (not `pub(crate)`) because MCP tool handlers
// (tests, the MCP server) use the same type from `crate::compression::Fidelity`.
pub(crate) use capture_pipeline::CapEntry;
pub use fidelity::Fidelity;
pub use language::{detect_language, language_for_extension, looks_like_csharp};
// These are re-exported for convenience but the main consumers
// (`pipeline.rs`, consumers of `dictionary/`) import
// directly from the submodules.

// Re-export the orchestrator entry point.
pub use pipeline::compress_file;
