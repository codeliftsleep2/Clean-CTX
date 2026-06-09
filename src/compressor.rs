// src/compressor.rs
//
// DEPRECATED: This is a re-export shim only. Add new code to
// `crate::compression::*` (the canonical location). Kept for backward
// compatibility with external consumers importing from
// `crate::compressor` or `clean_ctx::compressor`.
//
// F-FINAL-03: The audit flagged this file as having a stale comment
// that did not warn future maintainers that it is a shim, not a
// primary source. The DEPRECATED notice above prevents well-meaning
// contributors from adding new compression logic to this file (which
// would then be missed by the canonical location).
//
// Phase 3 re-export map (do not modify without updating all callers):
//   - `compress_file`          → `crate::compression::pipeline::compress_file`
//   - `compress_file_streaming` → `crate::compression::streaming::compress_file_streaming`
//   - `CompressionProgress`    → `crate::compression::streaming::CompressionProgress`
//   - `Fidelity`               → `crate::compression::fidelity::Fidelity`

pub use crate::compression::compress_file;
pub use crate::compression::compress_file_streaming;
pub use crate::compression::CompressionProgress;
pub use crate::compression::Fidelity;