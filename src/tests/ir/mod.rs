// src/tests/ir/mod.rs
//
// IR module tests are loaded via #[cfg(test)] #[path] annotations
// in each source file (opcodes.rs, compiler.rs, render.rs, wire.rs,
// symbol_table.rs, delta.rs, replay.rs).
//
// Phase G integration tests are loaded here as a submodule.
#[path = "integration.rs"]
mod integration;

// Phase A (FAANG remediation F-01–F-03) integration tests:
// Verifies that the full 4-layer pipeline is wired through IRCompiler.
#[path = "layers_integration.rs"]
mod layers_integration;

// Rust language support integration tests:
// Verifies that Rust source files are correctly compiled through the
// full IR pipeline with the Rust language layer.
#[cfg(feature = "rust")]
#[path = "rust_integration.rs"]
mod rust_integration;

// Rust token tracking integration tests:
// Verifies that Rust files produce proper token savings through the
// analytics pipeline and session stats tracking.
#[cfg(feature = "rust")]
#[path = "rust_stats_integration.rs"]
mod rust_stats_integration;

// RED->GREEN regression: ctor-pattern consumption orphaning M-referencing
// annotations (Issue #36 / Addendum #37). The ctor patterns decline
// compression when the region after the span still references the method —
// PatternOp cannot represent those annotations, so the original valid
// sequence is preserved.
#[path = "regression_ctor_pattern_orphan.rs"]
mod regression_ctor_pattern_orphan;

// F2 (pattern identity preservation): pattern recognition must classify a
// method, never delete it. `DefMethod` / `Param*` / `Return` survive every
// consumptive pattern, so the classification is additive at the producer
// boundary and the declaration is never erased from downstream
// representations.
#[path = "pattern_identity.rs"]
mod pattern_identity;

// F2 downstream contracts: the method that carries a pattern classification
// must still exist in the hierarchical projection, the rendered `M` line, the
// `UnitTable`, the semantic registration, and as a `Calls` subject. These read
// the real production pipeline; they are the reason F2's severity was real.
#[path = "pattern_identity_downstream.rs"]
mod pattern_identity_downstream;
