// src/tests/ir/mod.rs
//
// IR module tests are loaded via #[cfg(test)] #[path] annotations
// in each source file (opcodes.rs, compiler.rs, render.rs, wire.rs,
// symbol_table.rs, delta.rs, replay.rs).
//
// Phase G integration tests are loaded here as a submodule.
#[path = "integration.rs"]
mod integration;
