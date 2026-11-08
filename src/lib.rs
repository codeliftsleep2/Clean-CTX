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

pub mod analytics;
pub mod cache;
pub mod cbm;
pub mod config;
pub mod edit;
pub mod intelligence;
pub mod mcp;
pub mod protocol;
pub mod queries;

// New (authoritative) module paths — public so they can be re-exported.
pub mod angular_meta;
pub mod compaction;
pub mod compression;
pub mod dictionary;
pub mod diff;
pub mod dotnet_meta;
pub mod error;
pub mod gitdiff;
pub mod ir;
pub mod layers;
pub mod meta_util;
pub mod observability;
pub mod proxy_spawner;
pub mod spring_meta;
pub mod tokenizer;
pub mod workspace;

// Integration tests module
#[cfg(test)]
mod tests;

#[cfg(all(test, feature = "rust", feature = "typescript", feature = "angular"))]
#[path = "tests/architecture/semantic_work_static.rs"]
mod semantic_work_static;

#[cfg(test)]
#[path = "tests/architecture/work_observations_af_perf_002.rs"]
pub(crate) mod work_observations_af_perf_002;

#[cfg(test)]
#[path = "tests/architecture/default_operation_observations_af_perf_002.rs"]
pub(crate) mod default_operation_observations_af_perf_002;

#[cfg(test)]
#[path = "tests/ir/binary_clone_equivalence_af_perf_002.rs"]
pub(crate) mod binary_clone_equivalence_af_perf_002;

#[cfg(test)]
#[path = "tests/ir/binary_clone_protection_af_perf_002.rs"]
mod binary_clone_protection_af_perf_002;

#[cfg(test)]
#[path = "tests/ir/binary_reverse_design_af_perf_002.rs"]
mod binary_reverse_design_af_perf_002;

#[cfg(test)]
#[path = "tests/ir/binary_reverse_equivalence_af_perf_002.rs"]
mod binary_reverse_equivalence_af_perf_002;

#[cfg(test)]
#[path = "tests/ir/binary_reverse_integrated_af_perf_002.rs"]
mod binary_reverse_integrated_af_perf_002;

#[cfg(test)]
#[path = "tests/workspace/index_complexity_af_perf_002.rs"]
mod index_complexity_af_perf_002;

#[cfg(test)]
#[path = "tests/workspace/index_membership_budget_af_perf_002.rs"]
mod index_membership_budget_af_perf_002;

#[cfg(test)]
#[path = "tests/workspace/index_enclosing_af_perf_002.rs"]
mod index_enclosing_af_perf_002;

#[cfg(all(test, feature = "rust"))]
#[path = "tests/architecture/membership_census_af_perf_002.rs"]
mod membership_census_af_perf_002;

#[cfg(test)]
#[path = "tests/workspace/index_ngrx_noop_af_perf_002.rs"]
mod index_ngrx_noop_af_perf_002;

#[cfg(all(test, feature = "csharp"))]
#[path = "tests/mcp/artifact_operation_work_af_perf_002.rs"]
mod artifact_operation_work_af_perf_002;
