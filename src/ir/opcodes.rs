// src/ir/opcodes.rs
//
// Core IR opcodes — the universal instruction set.
// Every language compiles down to these operations.
// Serialized as positional JSON arrays: [opcode, ...operands]
//
// Phase A: IR Core — instruction type definitions and constants.

use std::fmt;

/// Core IR opcodes — the universal instruction set.
/// Every language compiles down to these operations.
/// Serialized as positional JSON arrays: [opcode, ...operands]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CoreOp {
    // ── Structural Definitions ──────────────────────────
    /// ["DEF_C", class_id, original_name]
    DefClass(String, String),

    /// ["DEF_M", class_id, method_id, original_name]
    DefMethod(String, String, String),

    /// ["DEF_F", class_id, field_id, original_name]
    DefField(String, String, String),

    /// ["DEF_I", interface_id, original_name]
    DefInterface(String, String),

    // ── Signatures & Types ──────────────────────────────
    /// ["SIG", method_id, param_id, type_opcode, param_name]
    Param(String, String, String, String),

    /// ["RET", method_id, type_opcode]
    Return(String, String),

    /// ["FIELD_T", field_id, type_opcode]
    FieldType(String, String),

    // ── Control Flow & Behavior ─────────────────────────
    /// ["FLAGS", target_id, flag1, flag2, ...]
    /// Replaces ⊕guard, ⊕loop, ⊕⇒, ⊕! markers
    Flags(String, Vec<String>),

    /// ["FLAGS_C", class_id, flag1, flag2, ...]
    /// Class-level flags: EXPORT, ABSTRACT, etc.
    ClassFlags(String, Vec<String>),

    // ── Relationships ───────────────────────────────────
    /// ["EXT", child_id, parent_id]
    Extends(String, String),

    /// ["IMPL", class_id, interface_id]
    Implements(String, String),

    /// ["INJECTS", class_id, dep1, dep2, ...]
    Injects(String, Vec<String>),

    // ── Imports ─────────────────────────────────────────
    /// ["IMP", alias, module, named_export]
    Import(String, String, String),

    // ── Type Aliases (runtime-assigned) ─────────────────
    /// ["TYPE", alias, original_type]
    TypeAlias(String, String),

    // ── Compressed Patterns (Phase H consumptive) ───────
    /// ["PAT", pattern_name, class_id, method_id, ...metadata]
    /// Consumptive pattern op that replaces N source instructions
    /// with a single compact op. Produced by `CompressingPatternRecognizer`.
    Pattern(String, Vec<String>),
}

impl fmt::Display for CoreOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreOp::DefClass(id, name) => write!(f, "DEF_C {} {}", id, name),
            CoreOp::DefMethod(cid, mid, name) => write!(f, "DEF_M {} {} {}", cid, mid, name),
            CoreOp::DefField(cid, fid, name) => write!(f, "DEF_F {} {} {}", cid, fid, name),
            CoreOp::DefInterface(id, name) => write!(f, "DEF_I {} {}", id, name),
            CoreOp::Param(mid, pid, ty, name) => write!(f, "SIG {} {} {} {}", mid, pid, ty, name),
            CoreOp::Return(mid, ty) => write!(f, "RET {} {}", mid, ty),
            CoreOp::FieldType(fid, ty) => write!(f, "FIELD_T {} {}", fid, ty),
            CoreOp::Flags(tid, flags) => write!(f, "FLAGS {} {}", tid, flags.join(" ")),
            CoreOp::ClassFlags(cid, flags) => write!(f, "FLAGS_C {} {}", cid, flags.join(" ")),
            CoreOp::Extends(child, parent) => write!(f, "EXT {} {}", child, parent),
            CoreOp::Implements(cid, iid) => write!(f, "IMPL {} {}", cid, iid),
            CoreOp::Injects(cid, deps) => write!(f, "INJECTS {} {}", cid, deps.join(" ")),
            CoreOp::Import(alias, module, named) => {
                write!(f, "IMP {} {} {}", alias, module, named)
            }
            CoreOp::TypeAlias(alias, original) => write!(f, "TYPE {} {}", alias, original),
            CoreOp::Pattern(name, args) => {
                write!(f, "PAT {} {}", name, args.join(" "))
            }
        }
    }
}

// ── Flag Constants ──────────────────────────────────────────────

/// Conditional branch (if/switch)
pub const FLAG_IF: &str = "IF";
/// Loop construct (for/while/do)
pub const FLAG_LOOP: &str = "LOOP";
/// Return statement
pub const FLAG_RET: &str = "RET";
/// Throw/exception
pub const FLAG_THROW: &str = "THROW";
/// Async function
pub const FLAG_ASYNC: &str = "ASYNC";
/// Generator function
pub const FLAG_GEN: &str = "GEN";
/// Exported symbol
pub const FLAG_EXPORT: &str = "EXPORT";
/// Static member
pub const FLAG_STATIC: &str = "STATIC";
/// Private visibility
pub const FLAG_PRIVATE: &str = "PRIVATE";
/// Protected visibility
pub const FLAG_PROTECTED: &str = "PROTECTED";
/// Abstract class/method
pub const FLAG_ABSTRACT: &str = "ABSTRACT";
/// Unsafe function/trait/impl (Rust-specific)
pub const FLAG_UNSAFE: &str = "UNSAFE";

// ── Built-in Type Opcodes ──────────────────────────────────────

/// string type
pub const TYPE_STRING: &str = "$s";
/// number type
pub const TYPE_NUMBER: &str = "$n";
/// boolean type
pub const TYPE_BOOLEAN: &str = "$b";
/// void type
pub const TYPE_VOID: &str = "$v";
/// true literal type
pub const TYPE_TRUE: &str = "$T";
/// false literal type
pub const TYPE_FALSE: &str = "$F";
/// null type
pub const TYPE_NULL: &str = "$nl";
/// undefined type
pub const TYPE_UNDEFINED: &str = "$ud";

// ── Arity Table ────────────────────────────────────────────────

/// Arity for each opcode: fixed arities are positive, variadic are -1.
/// Used for schema validation and positional decoding.
pub fn arity(opcode: &str) -> Option<i32> {
    match opcode {
        "DEF_C" => Some(3),     // id, name
        "DEF_M" => Some(4),     // class_id, id, name
        "DEF_F" => Some(4),     // class_id, id, name
        "DEF_I" => Some(3),     // id, name
        "SIG" => Some(5),       // method_id, param_id, type, name
        "RET" => Some(3),       // method_id, type
        "FIELD_T" => Some(3),   // field_id, type
        "FLAGS" => Some(-1),    // target_id, flags...
        "FLAGS_C" => Some(-1),  // class_id, flags...
        "EXT" => Some(3),       // child_id, parent_id
        "IMPL" => Some(3),      // class_id, iface_id
        "INJECTS" => Some(-1),  // class_id, deps...
        "IMP" => Some(4),       // alias, module, named
        "TYPE" => Some(3),      // alias, original
        "PAT" => Some(-1),      // pattern_name, args...
        _ => None,
    }
}

/// Get the opcode string from a CoreOp variant.
pub fn opcode_name(op: &CoreOp) -> &'static str {
    match op {
        CoreOp::DefClass(..) => "DEF_C",
        CoreOp::DefMethod(..) => "DEF_M",
        CoreOp::DefField(..) => "DEF_F",
        CoreOp::DefInterface(..) => "DEF_I",
        CoreOp::Param(..) => "SIG",
        CoreOp::Return(..) => "RET",
        CoreOp::FieldType(..) => "FIELD_T",
        CoreOp::Flags(..) => "FLAGS",
        CoreOp::ClassFlags(..) => "FLAGS_C",
        CoreOp::Extends(..) => "EXT",
        CoreOp::Implements(..) => "IMPL",
        CoreOp::Injects(..) => "INJECTS",
        CoreOp::Import(..) => "IMP",
        CoreOp::TypeAlias(..) => "TYPE",
        CoreOp::Pattern(..) => "PAT",
    }
}

#[cfg(test)]
#[path = "../tests/ir/opcodes.rs"]
mod tests;