// src/ir/patterns.rs
//
// Phase H: Positional Encoding & Advanced Compression — Pattern Compression.
//
// The Phase F `layers/patterns::CodePatternRecognizer` is **additive** —
// it emits a `FLAGS` op (CTOR / OBSERVABLE / GETTER / SETTER) alongside
// the original instructions. That's useful for context but it does not
// actually reduce wire size.
//
// The `CompressingPatternRecognizer` here is **consumptive**: when it
// recognises a pattern, it replaces N source instructions with a single
// compact `PAT_*` op. This is the Layer 4 "advanced compression" pass
// called for in §11 of the spec.
//
// Recognised patterns:
//   - **PAT_CTOR**   — `DEF_M(constructor) + SIG*(P:ServiceType) + RET + INJECTS` → 1 op
//   - **PAT_OBSERVABLE** — `DEF_M + RET($P) + FLAGS(ASYNC)` → 1 op
//   - **PAT_GETTER** / **PAT_SETTER** — `DEF_M(get X)` / `DEF_M(set X)` → 1 op
//   - **PAT_OVERRIDE** — `DEF_M + FLAGS(OVERRIDE)` → 1 op
//   - **PAT_PROMISE** — `DEF_M + RET($P)` (without ASYNC) → 1 op
//
// A pattern that doesn't match falls through unchanged (zero regression).

use super::opcodes::CoreOp;

/// A compressed pattern op.
///
/// The first operand is the pattern name (e.g. "PAT_CTOR"). Subsequent
/// operands are positional: the first method id, then any captured
/// metadata (e.g. the getter property name).
///
/// Serialised wire form: `["PAT", "CTOR", "C1", "M1", "S1", "S2"]`
///   - `"PAT"`      — opcode
///   - `"CTOR"`     — pattern name (variadic)
///   - `"C1"`       — class id (where applicable)
///   - `"M1"`       — method id
///   - `"S1"…"S2"`  — extra metadata (injected deps, property name, …)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PatternOp {
    /// `DEF_M(constructor) + SIG*(...) + RET + INJECTS` → single op.
    /// Wire: `["PAT", "CTOR", class_id, method_id, dep1, ...]`
    Constructor {
        class_id: String,
        method_id: String,
        deps: Vec<String>,
    },
    /// `DEF_M + RET($P/$O) + FLAGS(ASYNC)` → single op.
    /// Wire: `["PAT", "OBSERVABLE", class_id, method_id, return_type]`
    Observable {
        class_id: String,
        method_id: String,
        return_type: String,
    },
    /// `DEF_M + RET($P)` (no ASYNC flag) → single op.
    /// Wire: `["PAT", "PROMISE", class_id, method_id, return_type]`
    Promise {
        class_id: String,
        method_id: String,
        return_type: String,
    },
    /// `DEF_M(get X) [+ RET]` → single op.
    /// Wire: `["PAT", "GETTER", class_id, method_id, property]`
    Getter {
        class_id: String,
        method_id: String,
        property: String,
    },
    /// `DEF_M(set X) [+ SIG(value)]` → single op.
    /// Wire: `["PAT", "SETTER", class_id, method_id, property]`
    Setter {
        class_id: String,
        method_id: String,
        property: String,
    },
    /// `DEF_M + FLAGS(OVERRIDE)` → single op.
    /// Wire: `["PAT", "OVERRIDE", class_id, method_id]`
    Override {
        class_id: String,
        method_id: String,
    },
    /// A pattern that was recognised but the constructor had no params —
    /// still useful to flag so the LLM knows it's a ctor.
    /// Wire: `["PAT", "EMPTY_CTOR", class_id, method_id]`
    EmptyConstructor {
        class_id: String,
        method_id: String,
    },
}

impl PatternOp {
    /// Convert to a positional `Vec<String>` for the wire.
    pub fn to_tuple(&self) -> Vec<String> {
        match self {
            PatternOp::Constructor { class_id, method_id, deps } => {
                let mut t = vec!["PAT".into(), "CTOR".into(), class_id.clone(), method_id.clone()];
                t.extend(deps.iter().cloned());
                t
            }
            PatternOp::EmptyConstructor { class_id, method_id } => {
                vec!["PAT".into(), "EMPTY_CTOR".into(), class_id.clone(), method_id.clone()]
            }
            PatternOp::Observable { class_id, method_id, return_type } => {
                vec![
                    "PAT".into(),
                    "OBSERVABLE".into(),
                    class_id.clone(),
                    method_id.clone(),
                    return_type.clone(),
                ]
            }
            PatternOp::Promise { class_id, method_id, return_type } => {
                vec![
                    "PAT".into(),
                    "PROMISE".into(),
                    class_id.clone(),
                    method_id.clone(),
                    return_type.clone(),
                ]
            }
            PatternOp::Getter { class_id, method_id, property } => {
                vec![
                    "PAT".into(),
                    "GETTER".into(),
                    class_id.clone(),
                    method_id.clone(),
                    property.clone(),
                ]
            }
            PatternOp::Setter { class_id, method_id, property } => {
                vec![
                    "PAT".into(),
                    "SETTER".into(),
                    class_id.clone(),
                    method_id.clone(),
                    property.clone(),
                ]
            }
            PatternOp::Override { class_id, method_id } => {
                vec![
                    "PAT".into(),
                    "OVERRIDE".into(),
                    class_id.clone(),
                    method_id.clone(),
                ]
            }
        }
    }

    /// Reconstruct a `PatternOp` from its wire tuple.
    pub fn from_tuple(tuple: &[String]) -> Option<Self> {
        if tuple.len() < 3 {
            return None;
        }
        if tuple[0] != "PAT" {
            return None;
        }
        let class_id = tuple.get(2)?.clone();
        let method_id = tuple.get(3)?.clone();
        match tuple[1].as_str() {
            "CTOR" => {
                let deps = if tuple.len() > 4 { tuple[4..].to_vec() } else { Vec::new() };
                Some(PatternOp::Constructor { class_id, method_id, deps })
            }
            "EMPTY_CTOR" => Some(PatternOp::EmptyConstructor { class_id, method_id }),
            "OBSERVABLE" => {
                let return_type = tuple.get(4)?.clone();
                Some(PatternOp::Observable { class_id, method_id, return_type })
            }
            "PROMISE" => {
                let return_type = tuple.get(4)?.clone();
                Some(PatternOp::Promise { class_id, method_id, return_type })
            }
            "GETTER" => {
                let property = tuple.get(4)?.clone();
                Some(PatternOp::Getter { class_id, method_id, property })
            }
            "SETTER" => {
                let property = tuple.get(4)?.clone();
                Some(PatternOp::Setter { class_id, method_id, property })
            }
            "OVERRIDE" => Some(PatternOp::Override { class_id, method_id }),
            _ => None,
        }
    }

    /// Pattern name (the second tuple element).
    pub fn name(&self) -> &'static str {
        match self {
            PatternOp::Constructor { .. } => "CTOR",
            PatternOp::EmptyConstructor { .. } => "EMPTY_CTOR",
            PatternOp::Observable { .. } => "OBSERVABLE",
            PatternOp::Promise { .. } => "PROMISE",
            PatternOp::Getter { .. } => "GETTER",
            PatternOp::Setter { .. } => "SETTER",
            PatternOp::Override { .. } => "OVERRIDE",
        }
    }

    /// Number of source `CoreOp`s this pattern consumed (useful for
    /// reporting compression statistics).
    pub fn consumed(&self) -> usize {
        match self {
            // CTOR: DEF_M + Param* + Return + INJECTS = 2 + 1 + 1 = 4 (typical)
            // Conservative: assume at least 3 (DEF_M + SIG + RET) + optional INJECTS
            PatternOp::Constructor { deps, .. } => 3 + deps.len().min(1),
            PatternOp::EmptyConstructor { .. } => 2, // DEF_M + RET
            // Observable: DEF_M + RET + FLAGS
            PatternOp::Observable { .. } | PatternOp::Override { .. } => 3,
            // Promise: DEF_M + RET
            PatternOp::Promise { .. } => 2,
            // Accessor: DEF_M + RET (or just DEF_M)
            PatternOp::Getter { .. } | PatternOp::Setter { .. } => 2,
        }
    }
}

/// Layer 4 advanced pattern recognizer.
///
/// Unlike `layers::patterns::CodePatternRecognizer`, this recognizer
/// **consumes** the matched instructions and emits a single compact
/// `PatternOp` per match. This is the wire-size-reducing pass.
#[derive(Debug, Clone, Default)]
pub struct CompressingPatternRecognizer;

impl CompressingPatternRecognizer {
    pub fn new() -> Self {
        Self
    }

    /// Compress an instruction stream by recognising and merging patterns.
    ///
    /// Returns `(compressed_ops, stats)` where `stats` reports how many
    /// source ops were consumed and how many compressed ops were emitted.
    pub fn compress(&self, instructions: &[CoreOp]) -> (Vec<PatternOp>, CompressionStats) {
        let mut output: Vec<PatternOp> = Vec::new();
        let mut source_count = 0usize;
        let mut output_count = 0usize;
        let mut i = 0;

        while i < instructions.len() {
            if let Some((pat, consumed)) = try_compress_pattern(&instructions[i..]) {
                source_count += consumed;
                output_count += 1;
                output.push(pat);
                i += consumed;
            } else {
                // Pass-through: the source op is not part of any recognised
                // pattern. We do NOT emit anything for it — the recognizer
                // is a *filter* that only emits pattern ops. The caller is
                // responsible for merging the original stream back in.
                source_count += 1;
                i += 1;
            }
        }

        let stats = CompressionStats {
            source_ops: source_count,
            output_ops: output_count,
            ratio: if output_count == 0 {
                0.0
            } else {
                source_count as f64 / output_count as f64
            },
        };
        (output, stats)
    }

    /// Compress and merge back into the original stream. Pass-through
    /// instructions are preserved in their original position relative to
    /// any preceding/following pattern ops.
    ///
    /// Returns a `Vec<CompressedItem>` which is either a passthrough
    /// `CoreOp` or a recognised `PatternOp`. The caller can decide how
    /// to serialise the merged stream.
    pub fn compress_merged(&self, instructions: &[CoreOp]) -> Vec<CompressedItem> {
        let mut output: Vec<CompressedItem> = Vec::new();
        let mut i = 0;

        while i < instructions.len() {
            if let Some((pat, consumed)) = try_compress_pattern(&instructions[i..]) {
                output.push(CompressedItem::Pattern(pat));
                i += consumed;
            } else {
                output.push(CompressedItem::Passthrough(instructions[i].clone()));
                i += 1;
            }
        }

        output
    }
}

/// Statistics from a `compress()` run.
#[derive(Debug, Clone, PartialEq)]
pub struct CompressionStats {
    /// Number of source `CoreOp`s examined.
    pub source_ops: usize,
    /// Number of `PatternOp`s emitted.
    pub output_ops: usize,
    /// `source_ops / output_ops` (∞ if output is 0). >1 means compression.
    pub ratio: f64,
}

/// Result of a `compress_merged()` call: a heterogeneous stream of
/// pass-through CoreOps and recognised PatternOps.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CompressedItem {
    /// An instruction that no pattern matched.
    Passthrough(CoreOp),
    /// A compressed pattern.
    Pattern(PatternOp),
}

// ── Pattern recognisers (consumptive) ─────────────────────────────────

/// Try to match and consume a pattern at the start of `slice`.
/// Returns `(pattern, instructions_consumed)` on success.
fn try_compress_pattern(slice: &[CoreOp]) -> Option<(PatternOp, usize)> {
    if slice.is_empty() {
        return None;
    }

    // CTOR + injectable deps
    if let Some(result) = try_ctor_pattern(slice) {
        return Some(result);
    }
    // Empty constructor
    if let Some(result) = try_empty_ctor_pattern(slice) {
        return Some(result);
    }
    // Observable / Promise
    if let Some(result) = try_observable_pattern(slice) {
        return Some(result);
    }
    if let Some(result) = try_promise_pattern(slice) {
        return Some(result);
    }
    // Accessors
    if let Some(result) = try_getter_pattern(slice) {
        return Some(result);
    }
    if let Some(result) = try_setter_pattern(slice) {
        return Some(result);
    }
    // Override
    if let Some(result) = try_override_pattern(slice) {
        return Some(result);
    }
    None
}

/// Returns true if the method name is a recognized constructor name.
fn is_constructor_name(name: &str) -> bool {
    matches!(name, "constructor" | "new" | "__init__" | "initialize" | "ctor")
}

/// CTOR pattern: `DEF_M(constructor) + Param* + Return + INJECTS` → 1 op.
fn try_ctor_pattern(slice: &[CoreOp]) -> Option<(PatternOp, usize)> {
    if slice.is_empty() {
        return None;
    }
    let (class_id, method_id) = match &slice[0] {
        CoreOp::DefMethod(cid, mid, name) if is_constructor_name(name) => {
            (cid.clone(), mid.clone())
        }
        _ => return None,
    };

    // Walk forward, collecting params + return + (optional) INJECTS
    let mut idx = 1;
    let mut param_count = 0;
    let mut saw_return = false;
    let mut saw_injects = false;
    let mut deps: Vec<String> = Vec::new();

    while idx < slice.len() {
        match &slice[idx] {
            CoreOp::Param(mid, _, _, _) if mid == &method_id => {
                param_count += 1;
                idx += 1;
            }
            CoreOp::Return(mid, _) if mid == &method_id => {
                saw_return = true;
                idx += 1;
                break; // RET terminates the method body
            }
            _ => break, // unrelated op — stop
        }
    }

    // Check for trailing INJECTS
    if idx < slice.len() {
        if let CoreOp::Injects(cid, inj_deps) = &slice[idx] {
            if cid == &class_id {
                saw_injects = true;
                deps = inj_deps.clone();
                idx += 1;
            }
        }
    }

    // We require: at least 1 param OR an INJECTS op to qualify as a
    // constructor-injection pattern. Otherwise it's an empty ctor.
    if !saw_injects && param_count == 0 {
        return None;
    }
    if !saw_return && !saw_injects {
        return None;
    }

    let consumed = idx;
    Some((
        PatternOp::Constructor { class_id, method_id, deps },
        consumed,
    ))
}

/// Empty-ctor pattern: `DEF_M(constructor) + Return` (no params, no injects).
fn try_empty_ctor_pattern(slice: &[CoreOp]) -> Option<(PatternOp, usize)> {
    if slice.len() < 2 {
        return None;
    }
    let (class_id, method_id) = match &slice[0] {
        CoreOp::DefMethod(cid, mid, name) if is_constructor_name(name) => {
            (cid.clone(), mid.clone())
        }
        _ => return None,
    };
    if let CoreOp::Return(mid, _) = &slice[1] {
        if mid == &method_id {
            return Some((
                PatternOp::EmptyConstructor { class_id, method_id },
                2,
            ));
        }
    }
    None
}

/// Observable pattern: `DEF_M + Return($P|$O) + Flags(ASYNC)` → 1 op.
fn try_observable_pattern(slice: &[CoreOp]) -> Option<(PatternOp, usize)> {
    if slice.len() < 3 {
        return None;
    }
    let (class_id, method_id) = match &slice[0] {
        CoreOp::DefMethod(cid, mid, _) => (cid.clone(), mid.clone()),
        _ => return None,
    };
    let return_type = match &slice[1] {
        CoreOp::Return(mid, ty) if mid == &method_id => ty.clone(),
        _ => return None,
    };
    // Must be Promise-like and have an ASYNC flag
    let is_promise_like =
        return_type == "$P" || return_type.contains("Promise") || return_type.contains("Observable");
    if !is_promise_like {
        return None;
    }
    match &slice[2] {
        CoreOp::Flags(mid, flags)
            if mid == &method_id
                && flags.iter().any(|f| f == "ASYNC") =>
        {
            Some((
                PatternOp::Observable { class_id, method_id, return_type },
                3,
            ))
        }
        _ => None,
    }
}

/// Promise pattern: `DEF_M + Return($P)` (no ASYNC) → 1 op.
/// Only triggers if the observable pattern did not match
fn try_promise_pattern(slice: &[CoreOp]) -> Option<(PatternOp, usize)> {
    if slice.len() < 2 {
        return None;
    }
    let (class_id, method_id) = match &slice[0] {
        CoreOp::DefMethod(cid, mid, _) => (cid.clone(), mid.clone()),
        _ => return None,
    };
    match &slice[1] {
        CoreOp::Return(mid, ty) if mid == &method_id => {
            let is_promise_like =
                ty == "$P" || ty.contains("Promise") || ty.contains("Observable");
            if is_promise_like {
                Some((
                    PatternOp::Promise {
                        class_id,
                        method_id,
                        return_type: ty.clone(),
                    },
                    2,
                ))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Getter pattern: `DEF_M(get X) [+ Return]` → 1 op.
fn try_getter_pattern(slice: &[CoreOp]) -> Option<(PatternOp, usize)> {
    if slice.is_empty() {
        return None;
    }
    let (class_id, method_id, name) = match &slice[0] {
        CoreOp::DefMethod(cid, mid, name) => (cid.clone(), mid.clone(), name.clone()),
        _ => return None,
    };
    if !name.to_lowercase().starts_with("get ") {
        return None;
    }
    let property = name[4..].trim().to_string();
    if property.is_empty() {
        return None;
    }
    // Optionally consume a trailing Return
    let consumed = if slice.len() >= 2 {
        if let CoreOp::Return(mid, _) = &slice[1] {
            if mid == &method_id {
                2
            } else {
                1
            }
        } else {
            1
        }
    } else {
        1
    };
    Some((
        PatternOp::Getter { class_id, method_id, property },
        consumed,
    ))
}

/// Setter pattern: `DEF_M(set X) [+ Param(value)]` → 1 op.
fn try_setter_pattern(slice: &[CoreOp]) -> Option<(PatternOp, usize)> {
    if slice.is_empty() {
        return None;
    }
    let (class_id, method_id, name) = match &slice[0] {
        CoreOp::DefMethod(cid, mid, name) => (cid.clone(), mid.clone(), name.clone()),
        _ => return None,
    };
    if !name.to_lowercase().starts_with("set ") {
        return None;
    }
    let property = name[4..].trim().to_string();
    if property.is_empty() {
        return None;
    }
    // Walk forward over an optional Param(value) + optional Return
    let mut idx = 1;
    if idx < slice.len() {
        if let CoreOp::Param(mid, _, _, _) = &slice[idx] {
            if mid == &method_id {
                idx += 1;
            }
        }
    }
    if idx < slice.len() {
        if let CoreOp::Return(mid, _) = &slice[idx] {
            if mid == &method_id {
                idx += 1;
            }
        }
    }
    Some((
        PatternOp::Setter { class_id, method_id, property },
        idx,
    ))
}

/// Override pattern: `DEF_M + Flags(OVERRIDE)` → 1 op.
fn try_override_pattern(slice: &[CoreOp]) -> Option<(PatternOp, usize)> {
    if slice.len() < 2 {
        return None;
    }
    let (class_id, method_id) = match &slice[0] {
        CoreOp::DefMethod(cid, mid, _) => (cid.clone(), mid.clone()),
        _ => return None,
    };
    match &slice[1] {
        CoreOp::Flags(mid, flags) if mid == &method_id && flags.iter().any(|f| f == "OVERRIDE") => {
            Some((
                PatternOp::Override { class_id, method_id },
                2,
            ))
        }
        _ => None,
    }
}

#[cfg(test)]
#[path = "../tests/ir/patterns.rs"]
mod tests;
