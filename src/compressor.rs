// src/compressor.rs
//
// Backward-compatible re-export shim for Phase 3. The three public entry
// points and the `Fidelity` enum now live in `crate::compression::*`:
//
//   - `compress_file`          → `crate::compression::pipeline::compress_file`
//   - `compress_file_streaming` → `crate::compression::streaming::compress_file_streaming`
//   - `CompressionProgress`    → `crate::compression::streaming::CompressionProgress`
//   - `Fidelity`               → `crate::compression::fidelity::Fidelity`
//
// External consumers importing from `crate::compressor` or
// `clean_ctx::compressor` see no breaking change.

pub use crate::compression::compress_file;
pub use crate::compression::compress_file_streaming;
pub use crate::compression::CompressionProgress;
pub use crate::compression::Fidelity;