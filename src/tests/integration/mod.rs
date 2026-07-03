// src/tests/integration/mod.rs
//
// Integration tests for cross-module interactions.
// These tests verify that multiple subsystems work correctly together.

mod text_vs_ir;
mod cbm_meta;
mod delta_e2e;
mod persistence;
mod concurrency;
mod error_paths;
mod mcp_e2e;
mod workspace_perf;
