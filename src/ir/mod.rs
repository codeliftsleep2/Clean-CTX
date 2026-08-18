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
// Phase E: IR / Pretty Separation — fidelity-aware rendering.
// Phase F: Layered Encoding — language + meta + pattern layers.
// Phase G: Integration & MCP Tools — wire IR into the MCP tool surface.
// Phase H: Positional Encoding & Advanced Compression — key stripping + pattern compression.

pub mod opcodes;
pub mod compiler;
pub(crate) mod compiler_methods;
pub mod render;
pub mod wire;
pub mod symbol_table;
pub mod delta;
pub mod replay;
pub mod layers;
pub mod positional;
pub mod patterns;
pub mod string_table;
pub mod hierarchical;
pub mod binary_wire;
pub mod render_llm;
// R-43b: IR Evolution Phases 2-6
pub mod program_graph;
pub mod inference_layer;
pub mod pipeline;
pub mod validator;
pub mod query;
// R-02 Phase 3: Type-aware compression for the IR path.
pub mod type_aliases;

// Re-export public types for downstream consumers.
pub use opcodes::CoreOp;
pub use compiler::{CompiledIR, CompileError, IRCompiler};
pub use compiler_methods::MethodSig;
pub use render::{ir_to_text, ir_to_text_ops};
pub use wire::{ir_to_wire, op_to_tuple};
pub use symbol_table::{GlobalSymbolTable, SymbolEntry, SymbolKind};
pub use delta::{
    IRDelta, DeltaOps, ModOp, FieldPatch, DeltaComputer, SemanticIntent,
    compute_field_patches, primary_key_from_tuple, key_tuple_from_tuple,
    CompactDelta, CompactOps, compact_encode, compact_decode,
};
pub use replay::{ContextState, FileState, DeltaError};
pub use positional::{
    PositionalConfig, encode_op, decode_op, encode_stream, ir_to_positional_wire,
    estimate_savings, positional_char_count, verify_round_trip,
};
pub use patterns::{
    PatternOp, CompressingPatternRecognizer, CompressionStats, MergeItem,
};
pub use string_table::{
    StringTable, ir_to_string_table_wire, estimate_savings as string_table_savings,
};
pub use hierarchical::{
    HierarchicalIR, ClassNode, MethodNode, FieldNode, PatternEntry,
    ir_to_hierarchical, hierarchical_to_ir,
    ir_to_hierarchical_wire, wire_to_ir as hierarchical_wire_to_ir,
    estimate_savings as hierarchical_savings,
};
pub use render_llm::{render_hierarchical_for_llm, render_hierarchical_for_llm_focused};
pub use binary_wire::{
    BinaryDecodeError, encode, decode, estimate_savings as binary_savings,
    is_binary_wire, ir_to_binary_wire_json, binary_wire_json_to_ir,
};

#[cfg(test)]
#[path = "../tests/ir/mod.rs"]
mod tests;
