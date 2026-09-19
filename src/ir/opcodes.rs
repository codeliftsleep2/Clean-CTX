// src/ir/opcodes.rs
//
// Core IR opcodes — the universal instruction set.
// Every language compiles down to these operations.
// Serialized as positional JSON arrays: [opcode, ...operands]
//
// Phase A: IR Core — instruction type definitions and constants.
//
// R-43a: Added 4 new CoreOp variants for execution semantics:
//   - DataFlow: tracks which symbols a method reads/writes
//   - ControlFlow: tracks control flow constructs (if, loop, match, try, await, return)
//   - SideEffect: annotates method side-effect type (pure, io, mutation, async, transaction)
//   - ExecutionContext: method execution context (sync, async, thread_bound, transaction_scope, realtime)
//
// Edit Mode: Added CoreOp::Body for verbatim method body transport.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Closed declaration-modifier vocabulary shared by language producers.
///
/// Serialized spellings intentionally match the established flag payloads;
/// the distinct `CoreOp` variants provide the semantic-family boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeclarationModifier {
    #[serde(rename = "ASYNC")]
    Async,
    #[serde(rename = "GEN")]
    Generator,
    #[serde(rename = "EXPORT")]
    Export,
    #[serde(rename = "STATIC")]
    Static,
    #[serde(rename = "PRIVATE")]
    Private,
    #[serde(rename = "PROTECTED")]
    Protected,
    #[serde(rename = "ABSTRACT")]
    Abstract,
    #[serde(rename = "UNSAFE")]
    Unsafe,
}

impl DeclarationModifier {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Async => FLAG_ASYNC,
            Self::Generator => FLAG_GEN,
            Self::Export => FLAG_EXPORT,
            Self::Static => FLAG_STATIC,
            Self::Private => FLAG_PRIVATE,
            Self::Protected => FLAG_PROTECTED,
            Self::Abstract => FLAG_ABSTRACT,
            Self::Unsafe => FLAG_UNSAFE,
        }
    }

    pub fn from_serialized(value: &str) -> Option<Self> {
        match value {
            FLAG_ASYNC => Some(Self::Async),
            FLAG_GEN => Some(Self::Generator),
            FLAG_EXPORT => Some(Self::Export),
            FLAG_STATIC => Some(Self::Static),
            FLAG_PRIVATE => Some(Self::Private),
            FLAG_PROTECTED => Some(Self::Protected),
            FLAG_ABSTRACT => Some(Self::Abstract),
            FLAG_UNSAFE => Some(Self::Unsafe),
            _ => None,
        }
    }
}

impl fmt::Display for DeclarationModifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Closed vocabulary for compact, method-level control summaries.
///
/// These facts summarize the presence of control constructs. They are distinct
/// from detailed `ControlFlow` edges and residual pattern classifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ControlSummary {
    #[serde(rename = "IF")]
    Branch,
    #[serde(rename = "LOOP")]
    Loop,
    #[serde(rename = "RET")]
    Return,
    #[serde(rename = "THROW")]
    Throw,
}

impl ControlSummary {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Branch => FLAG_IF,
            Self::Loop => FLAG_LOOP,
            Self::Return => FLAG_RET,
            Self::Throw => FLAG_THROW,
        }
    }

    pub fn from_serialized(value: &str) -> Option<Self> {
        match value {
            FLAG_IF => Some(Self::Branch),
            FLAG_LOOP => Some(Self::Loop),
            FLAG_RET => Some(Self::Return),
            FLAG_THROW => Some(Self::Throw),
            _ => None,
        }
    }
}

impl fmt::Display for ControlSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

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

    /// ["MOD_M", method_id, modifier1, modifier2, ...]
    MethodModifiers(String, Vec<DeclarationModifier>),

    /// ["MOD_C", class_id, modifier1, modifier2, ...]
    ClassModifiers(String, Vec<DeclarationModifier>),

    // ── Control Flow & Behavior ─────────────────────────
    /// ["CTRL_SUM", method_id, summary1, summary2, ...]
    ControlSummary(String, Vec<ControlSummary>),

    /// ["FLAGS", target_id, flag1, flag2, ...]
    /// Residual method pattern facts; modifiers and control summaries are typed.
    Flags(String, Vec<String>),

    /// ["FLAGS_C", class_id, flag1, flag2, ...]
    /// Residual class metadata such as Rust CFG and generic-parameter summaries.
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

    // ── Edit Mode: Verbatim Method Bodies ────────────────
    ///
    /// Body: ["BODY", method_id, verbatim_text]
    ///   or:  ["BODY", method_id, verbatim_text, start_byte, end_byte]
    /// Carries the raw, byte-exact method body text from the source.
    /// Only emitted when `Fidelity::Edit` is active. The text is the
    /// verbatim source slice — no transformation, no compression.
    /// Used by the LLM renderer to emit bodies that are safe for
    /// `replace_in_file` SEARCH blocks.
    ///
    /// apply_edit plan Phase 1: when the span fields are `Some`, they hold
    /// the absolute byte range `[start_byte, end_byte)` of the body slice
    /// within the ORIGINAL source file that produced this IR, making the op
    /// splice-addressable by the `apply_edit` write path. Both span fields
    /// must be `Some` or both `None` (pairing invariant). Span-less bodies
    /// arise from legacy wire tuples decoded by `tuple_to_op`; they remain
    /// fully valid IR but are not eligible for span-based splicing until the
    /// file is recompressed.
    Body(String, String, Option<u64>, Option<u64>),

    // ── R-43a: Execution Semantics ──────────────────────
    ///
    /// Dataflow: ["DATAFLOW", method_id, "reads"|"writes", target_symbol]
    /// Tracks which symbols a method reads from or writes to.
    /// Extracted from tree-sitter captures (confidence = 1.0).
    DataFlow(String, String, String),

    ///
    /// Control flow: ["CTRL", method_id, kind, target]
    /// kind: "if" | "loop" | "match" | "try" | "await" | "return"
    /// target: the target symbol or expression
    /// Extracted from tree-sitter captures (confidence = 1.0).
    ControlFlow(String, String, String),

    ///
    /// Side-effect annotation: ["EFFECT", method_id, effect_type]
    /// effect_type: "pure" | "io" | "mutation" | "async" | "transaction"
    /// Extracted from tree-sitter captures (confidence = 1.0).
    SideEffect(String, String),

    ///
    /// Execution context: ["CTX", method_id, context_type]
    /// context_type: "sync" | "async" | "thread_bound" | "transaction_scope" | "realtime"
    /// Extracted from tree-sitter captures (confidence = 1.0).
    ExecutionContext(String, String),

    // ── Structural Invocations (native call graph) ──────
    ///
    /// Invocation: `["CALL", caller_method_id, callee_name, explicit_arg_count]`,
    /// plus an ADDITIVE fifth operand `"spread"` when the call site expands a
    /// written argument:
    ///
    /// ```text
    /// exact  (has_spread == false): ["CALL", caller, callee, argc]
    /// spread (has_spread == true) : ["CALL", caller, callee, argc, "spread"]
    /// ```
    ///
    /// One structural invocation fact extracted from the existing language
    /// parse. `caller_method_id` is the `DefMethod` id of the innermost
    /// callable that contains the invocation in the same file;
    /// `callee_name` is the textual name written at the call site; and
    /// `explicit_arg_count` is the number of argument nodes written at the
    /// call site (structural, from the argument list node — never from text
    /// splitting).
    ///
    /// The callee is deliberately a NAME and not a resolved declaration
    /// identity: a call site carries no type information, so overload
    /// resolution is never claimed. Two calls that differ only in argument
    /// count remain distinguishable; two calls that differ only in argument
    /// type are honestly indistinguishable. Extension-method receivers are
    /// NOT added to the count, and optional/`params` compatibility is NOT
    /// evaluated here (that belongs to later resolution logic, if any).
    ///
    /// `has_spread` keeps `explicit_arg_count` HONEST as a written-node count.
    /// When it is `true`, at least one written argument expands at run time
    /// (TypeScript `foo(...args)`, `foo(a, ...rest)`), so the count is the
    /// written-argument count and never the runtime/declared arity: a
    /// consumer must not read it as exact-arity evidence. When it is `false`,
    /// every written argument is exactly one argument. C# and Java call syntax
    /// has no caller-side expansion operator, so their facts are always
    /// `false`; the qualifier is language-neutral evidence, never a
    /// language-specific opcode.
    Call(String, String, usize, bool),
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
            CoreOp::MethodModifiers(mid, modifiers) => write!(
                f,
                "MOD_M {} {}",
                mid,
                modifiers
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
            CoreOp::ClassModifiers(cid, modifiers) => write!(
                f,
                "MOD_C {} {}",
                cid,
                modifiers
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
            CoreOp::ControlSummary(mid, summaries) => write!(
                f,
                "CTRL_SUM {} {}",
                mid,
                summaries
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
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
            // Edit Mode: Verbatim Method Bodies
            // Spans are transport metadata for the write path; the rendered
            // form stays byte-identical to the pre-span format.
            CoreOp::Body(mid, text, ..) => {
                write!(f, "BODY {} {}", mid, text)
            }
            // R-43a: Execution Semantics
            CoreOp::DataFlow(mid, direction, target) => {
                write!(f, "DATAFLOW {} {} {}", mid, direction, target)
            }
            CoreOp::ControlFlow(mid, kind, target) => {
                write!(f, "CTRL {} {} {}", mid, kind, target)
            }
            CoreOp::SideEffect(mid, effect_type) => {
                write!(f, "EFFECT {} {}", mid, effect_type)
            }
            CoreOp::ExecutionContext(mid, context_type) => {
                write!(f, "CTX {} {}", mid, context_type)
            }
            // Native call facts. The spread qualifier is ADDITIVE in the
            // rendered form too: an exact call renders byte-identically to the
            // pre-qualifier form, and a spread call is labelled so its written
            // count can never be read as an exact arity.
            CoreOp::Call(caller, callee, argc, has_spread) => {
                if *has_spread {
                    write!(f, "CALL {} {} {} spread", caller, callee, argc)
                } else {
                    write!(f, "CALL {} {} {}", caller, callee, argc)
                }
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
        "MOD_M" => Some(-1),    // method_id, declaration modifiers...
        "MOD_C" => Some(-1),    // class_id, declaration modifiers...
        "CTRL_SUM" => Some(-1), // method_id, control summaries...
        "FLAGS" => Some(-1),    // target_id, flags...
        "FLAGS_C" => Some(-1),  // class_id, flags...
        "EXT" => Some(3),       // child_id, parent_id
        "IMPL" => Some(3),      // class_id, iface_id
        "INJECTS" => Some(-1),  // class_id, deps...
        "IMP" => Some(4),       // alias, module, named
        "TYPE" => Some(3),      // alias, original
        "PAT" => Some(-1),      // pattern_name, args...
        // Edit Mode: Verbatim Method Bodies
        // Dual shape: legacy 3-tuple when span-less, 5-tuple when spanned.
        "BODY" => Some(-1), // method_id, text [, start_byte, end_byte]
        // R-43a: Execution Semantics
        "DATAFLOW" => Some(4), // method_id, direction, target
        "CTRL" => Some(4),     // method_id, kind, target
        "EFFECT" => Some(3),   // method_id, effect_type
        "CTX" => Some(3),      // method_id, context_type
        // Dual shape (like BODY below): a 4-tuple for an exact call and a
        // 5-tuple when the spread qualifier is present, so the table reports
        // variadic and the tuple decoder performs the strict shape check.
        "CALL" => Some(-1), // caller_method_id, callee_name, explicit_arg_count [, "spread"]
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
        CoreOp::MethodModifiers(..) => "MOD_M",
        CoreOp::ClassModifiers(..) => "MOD_C",
        CoreOp::ControlSummary(..) => "CTRL_SUM",
        CoreOp::Flags(..) => "FLAGS",
        CoreOp::ClassFlags(..) => "FLAGS_C",
        CoreOp::Extends(..) => "EXT",
        CoreOp::Implements(..) => "IMPL",
        CoreOp::Injects(..) => "INJECTS",
        CoreOp::Import(..) => "IMP",
        CoreOp::TypeAlias(..) => "TYPE",
        CoreOp::Pattern(..) => "PAT",
        // Edit Mode: Verbatim Method Bodies
        CoreOp::Body(..) => "BODY",
        // R-43a: Execution Semantics
        CoreOp::DataFlow(..) => "DATAFLOW",
        CoreOp::ControlFlow(..) => "CTRL",
        CoreOp::SideEffect(..) => "EFFECT",
        CoreOp::ExecutionContext(..) => "CTX",
        CoreOp::Call(..) => "CALL",
    }
}

impl CoreOp {
    /// Returns the byte span `[start_byte, end_byte)` of a `Body` op when
    /// both span fields are present (the pairing invariant makes partial
    /// spans unrepresentable). `None` for span-less bodies decoded from
    /// legacy wire tuples.
    pub fn body_span(&self) -> Option<(u64, u64)> {
        match self {
            CoreOp::Body(_, _, Some(start), Some(end)) => Some((*start, *end)),
            _ => None,
        }
    }

    /// Returns
    /// `(caller_method_id, callee_name, explicit_arg_count, has_spread)` for an
    /// invocation fact, `None` for every other op. Mirrors `body_span`:
    /// callers that need to reason about invocations should not re-match
    /// the variant themselves.
    ///
    /// The qualifier is part of this accessor rather than a second lookup on
    /// purpose: a consumer can never read the written count while silently
    /// ignoring whether that count is exact.
    pub fn call_parts(&self) -> Option<(&str, &str, usize, bool)> {
        match self {
            CoreOp::Call(caller, callee, argc, has_spread) => {
                Some((caller, callee, *argc, *has_spread))
            }
            _ => None,
        }
    }
}

// ── Execution Semantics Constants ──────────────────────────────

/// Dataflow direction: read
pub const DATAFLOW_READ: &str = "reads";
/// Dataflow direction: write
pub const DATAFLOW_WRITE: &str = "writes";

/// Control flow kind: if/conditional
pub const CTRL_IF: &str = "if";
/// Control flow kind: loop
pub const CTRL_LOOP: &str = "loop";
/// Control flow kind: match/switch
pub const CTRL_MATCH: &str = "match";
/// Control flow kind: try/catch
pub const CTRL_TRY: &str = "try";
/// Control flow kind: await
pub const CTRL_AWAIT: &str = "await";
/// Control flow kind: return
pub const CTRL_RETURN: &str = "return";

/// Side-effect type: pure (no side effects)
pub const EFFECT_PURE: &str = "pure";
/// Side-effect type: io (input/output)
pub const EFFECT_IO: &str = "io";
/// Side-effect type: mutation (state mutation)
pub const EFFECT_MUTATION: &str = "mutation";
/// Side-effect type: async (asynchronous)
pub const EFFECT_ASYNC: &str = "async";
/// Side-effect type: transaction (database/atomic)
pub const EFFECT_TRANSACTION: &str = "transaction";

/// Execution context: sync
pub const CTX_SYNC: &str = "sync";
/// Execution context: async
pub const CTX_ASYNC: &str = "async";
/// Execution context: thread_bound
pub const CTX_THREAD_BOUND: &str = "thread_bound";
/// Execution context: transaction_scope
pub const CTX_TRANSACTION_SCOPE: &str = "transaction_scope";
/// Execution context: realtime
pub const CTX_REALTIME: &str = "realtime";

#[cfg(test)]
#[path = "../tests/ir/opcodes.rs"]
mod tests;

#[cfg(test)]
#[path = "../tests/ir/declaration_modifiers.rs"]
mod declaration_modifier_tests;

#[cfg(test)]
#[path = "../tests/ir/control_summaries.rs"]
mod control_summary_tests;
