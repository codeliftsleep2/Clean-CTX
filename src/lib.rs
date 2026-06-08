// src/lib.rs
//
// Public crate root.
//
// Phase 1: the four large source files have been split into focused
// submodules. The new module structure is the authoritative location of
// all logic:
//   - `compaction`    (was: helpers)   — split into class/method/field/import/expression
//   - `diff`          (split into 6 submodules)
//   - `dictionary`    (was: a single file; now split into `path` and `symbol`)
//   - `decompression` (was: decompressor; now split into 4 submodules)
//
// Phase 2: the cross-cutting duplications called out in the audit have
// been consolidated into a new `compression` module:
//   - `compression/opcodes`         — shared primitive opcode table
//   - `compression/fidelity`        — shared Fidelity enum
//   - `compression/markers`         — shared marker construction & expansion
//   - `compression/capture_pipeline`— shared tree-sitter capture walk
//   - `compression/language`        — shared language detection
//

pub mod cache;
pub mod compressor;
pub mod config;
pub mod mcp;
pub mod protocol;
pub mod queries;
pub mod analytics;

// New (authoritative) module paths — public so they can be re-exported.
pub mod compaction;
pub mod compression;
pub mod decompression;
pub mod dictionary;
pub mod diff;
pub mod angular_meta;

