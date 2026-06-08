// src/ir/mod.rs
//
// Compiler IR — Structured State Protocol.
//
// This module defines the intermediate representation (IR) for compiled
// source code, replacing text-based compression with a structured
// instruction stream. The IR enables delta-based state transport so
// clients can apply incremental updates instead of re-parsing on every
// interaction.
//
// Phase A: IR Core — instruction types, compiler, render, wire format.

pub mod opcodes;
pub mod compiler;
pub mod render;
pub mod wire;

// Re-export public types for downstream consumers.
pub use opcodes::CoreOp;
pub use compiler::{CompiledIR, IRCompiler};
pub use render::ir_to_text;
pub use wire::{ir_to_wire, op_to_tuple};

#[cfg(test)]
#[path = "../tests/ir/mod.rs"]
mod tests;