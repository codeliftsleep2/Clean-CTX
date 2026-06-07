// src/decompression/mod.rs
//
// Inverse of the compression pipeline. Given a compressed string produced by
// `crate::compressor::compress_file` (with all the path-alias and symbol
// footers), rebuild a human-readable form.
//
// Module split:
//   - `decompressor` : the `Decompressor` struct + public entry points
//   - `opcodes`      : the 32-entry primitive opcode table
//   - `markers`      : marker expansion (⊕guard → "", ⊕⇒ → "→ ", etc.)
//   - `walker`       : line-by-line section walker

pub(crate) mod decompressor;
pub(crate) mod markers;
pub(crate) mod opcodes;
pub(crate) mod walker;

pub use decompressor::Decompressor;
