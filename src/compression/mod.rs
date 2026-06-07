// src/compression/mod.rs
//
// Shared building blocks for the compression pipeline. Before Phase 2 these
// concepts were duplicated across `compressor.rs`, `diff/builder.rs`,
// `dictionary/symbol.rs`, `decompression/*`, and `compaction/*`. After Phase 2
// they live here as the single source of truth.
//
// Module split:
//   - `fidelity`          : the `Fidelity` enum + `from_str` parser
//   - `language`          : centralized language detection (extension + heuristic)
//   - `opcodes`           : SHARED primitive opcode table (consumed by symbol
//                           dictionary and decompressor)
//   - `markers`           : SHARED marker construction (`build_marker`) and
//                           expansion (`expand_markers_in_line`)
//   - `capture_pipeline`  : SHARED tree-sitter capture-walk that yields a
//                           sorted `Vec<CapEntry>` from a parsed tree
//
// The orchestrators (`compress_file`, `compress_file_streaming`,
// `build_snapshot`) call into these modules rather than reimplementing them.

pub(crate) mod capture_pipeline;
pub(crate) mod markers;
pub(crate) mod opcodes;
pub mod language;
pub mod fidelity;

// Re-export shared types for downstream callers. These were previously
// defined inside `compressor.rs` and `diff/builder.rs`; they now live in
// the `compression` namespace so both modules can depend on them.
//
// `Fidelity` is `pub` (not `pub(crate)`) because the historical
// `crate::compressor::Fidelity` import path needs to remain public, and
// downstream consumers (tests, the MCP server) use the same type.
pub(crate) use capture_pipeline::CapEntry;
pub use fidelity::Fidelity;
pub use language::{detect_language, language_for_extension, looks_like_csharp};
pub(crate) use markers::{build_marker, expand_markers_in_line};
pub(crate) use opcodes::{builtin_opcode_map, is_primitive_opcode, PRIMITIVE_OPCODES};
