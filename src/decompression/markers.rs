// src/decompression/markers.rs
//
// Backward-compatible re-export shim. Marker expansion used to live in
// this file (`expand_marker`, `expand_markers_in_line`); in Phase 2
// those functions move to `crate::compression::markers` so that the
// decompressor and the construction side (`markers::build_marker`)
// share one source of truth.
//
// This shim re-exports `expand_markers_in_line` (the only function the
// `Decompressor` actually used) so the existing call site keeps
// compiling without modification.

pub(crate) use crate::compression::markers::expand_markers_in_line;
