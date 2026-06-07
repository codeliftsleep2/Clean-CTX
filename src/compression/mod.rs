// src/compression/mod.rs
//
// Shared building blocks for the compression pipeline, plus the two
// orchestrators (`compress_file` and `compress_file_streaming`).
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
//   - `streaming`         : Streaming variant with progress callbacks

pub(crate) mod capture_pipeline;
pub(crate) mod markers;
pub(crate) mod opcodes;
pub(crate) mod pipeline;
pub(crate) mod report;
pub(crate) mod streaming;
pub(crate) mod symbol_compression;
pub mod language;
pub mod fidelity;

// Re-export shared types for downstream callers.
//
// `Fidelity` is `pub` (not `pub(crate)`) because the historical
// `crate::compressor::Fidelity` import path needs to remain public, and
// downstream consumers (tests, the MCP server) use the same type.
pub(crate) use capture_pipeline::CapEntry;
pub use fidelity::Fidelity;
pub use language::{detect_language, language_for_extension, looks_like_csharp};
// These are re-exported for convenience but the main consumers
// (`pipeline.rs`, `streaming.rs`, consumers of `dictionary/`) import
// directly from the submodules.

// Re-export the orchestrator entry points and the progress type.
pub use pipeline::compress_file;
pub use streaming::{compress_file_streaming, CompressionProgress};
