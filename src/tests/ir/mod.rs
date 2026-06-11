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
#[path = "rust_integration.rs"]
mod rust_integration;

// Rust token tracking integration tests:
// Verifies that Rust files produce proper token savings through the
// analytics pipeline and session stats tracking.
#[path = "rust_stats_integration.rs"]
mod rust_stats_integration;
