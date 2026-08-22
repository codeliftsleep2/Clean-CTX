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

pub mod binary_wire;
pub mod compiler;
pub(crate) mod compiler_methods;
pub mod delta;
pub mod hierarchical;
pub mod layers;
pub mod opcodes;
pub mod patterns;
pub mod positional;
pub mod render;
pub mod render_llm;
pub mod replay;
pub mod string_table;
pub mod symbol_table;
pub mod wire;
// R-43b: IR Evolution Phases 2-6
pub mod inference_layer;
pub mod pipeline;
pub mod program_graph;
pub mod query;
pub mod validator;
// R-02 Phase 3: Type-aware compression for the IR path.
pub mod type_aliases;

// Re-export public types for downstream consumers.
pub use binary_wire::{
    BinaryDecodeError, binary_wire_json_to_ir, decode, encode, estimate_savings as binary_savings,
    ir_to_binary_wire_json, is_binary_wire,
};
pub use compiler::{CompileError, CompiledIR, IRCompiler};
pub use delta::{
    CompactDelta, CompactOps, DeltaComputer, DeltaOps, FieldPatch, IRDelta, ModOp, SemanticIntent,
    compact_decode, compact_encode, compute_field_patches, key_tuple_from_tuple,
    primary_key_from_tuple,
};
pub use hierarchical::{
    ClassNode, FieldNode, HierarchicalIR, MethodNode, PatternEntry,
    estimate_savings as hierarchical_savings, hierarchical_to_ir, ir_to_hierarchical,
    ir_to_hierarchical_wire, wire_to_ir as hierarchical_wire_to_ir,
};
pub use opcodes::CoreOp;
pub use patterns::{CompressingPatternRecognizer, CompressionStats, MergeItem, PatternOp};
pub use positional::{
    PositionalConfig, decode_op, encode_op, encode_stream, estimate_savings, ir_to_positional_wire,
    positional_char_count, verify_round_trip,
};
pub use render::{ir_to_text, ir_to_text_ops};
pub use render_llm::{render_hierarchical_for_llm, render_hierarchical_for_llm_focused};
pub use replay::{ContextState, DeltaError, FileState};
pub use string_table::{
    StringTable, estimate_savings as string_table_savings, ir_to_string_table_wire,
};
pub use symbol_table::{GlobalSymbolTable, SymbolEntry, SymbolKind};
pub use wire::{ir_to_wire, op_to_tuple};

#[cfg(test)]
#[path = "../tests/ir/mod.rs"]
mod tests;
