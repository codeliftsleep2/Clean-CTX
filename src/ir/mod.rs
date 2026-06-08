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
// Phase B: Global Symbol Table — cross-stage symbol registry.
// Phase C: Delta Transport — instruction-level diff engine.
// Phase D: State Replay — client-side state machine for applying deltas.

pub mod opcodes;
pub mod compiler;
pub mod render;
pub mod wire;
pub mod symbol_table;
pub mod delta;
pub mod replay;
pub mod layers;

// Re-export public types for downstream consumers.
pub use opcodes::CoreOp;
pub use compiler::{CompiledIR, IRCompiler};
pub use render::ir_to_text;
pub use wire::{ir_to_wire, op_to_tuple};
pub use symbol_table::{GlobalSymbolTable, SymbolEntry, SymbolKind};
pub use delta::{IRDelta, DeltaOps, ModOp, DeltaComputer, primary_key_from_tuple, key_tuple_from_tuple};
pub use replay::{ContextState, FileState, DeltaError};

#[cfg(test)]
#[path = "../tests/ir/mod.rs"]
mod tests;
