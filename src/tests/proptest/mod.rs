// src/tests/proptest/mod.rs
//
// A-07: Property-based tests using `proptest`.
//
// These tests use fuzz-style random input generation to find edge cases
// that hand-written tests miss. Each module focuses on one function or
// area that has been a source of regressions:
//
// - decompressor: word_boundary_replace and quick_decompress
// - glob_matcher: glob_match in config.rs
// - modifier_stripper: strip_modifiers in compaction/modifiers.rs

pub mod decompressor;
pub mod glob_matcher;
pub mod modifier_stripper;
