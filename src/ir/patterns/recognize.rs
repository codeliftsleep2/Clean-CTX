// src/ir/patterns/recognize.rs
//
// Consumptive pattern recognition: the flag-consumption helpers, the
// IRPAT-001 orphan guards, the centralized match dispatcher, and the
// individual pattern matchers.
//
// Split out of `src/ir/patterns.rs` along the representation/recognition
// boundary: the parent module owns the pattern TYPE surface (`PatternOp`,
// `MergeItem`, `CompressionStats`, `CompressingPatternRecognizer`), this
// module owns the code that decides whether a region of the instruction
// stream is a recognizable pattern.
//
// IRPAT-001: a consumptive transformation must never consume a `DefMethod(M)`
// while a surviving operation that references `M` would be orphaned. The
// guards live here (`op_is_unrepresentable_method_ref`,
// `trailing_region_references_method`, `trailing_region_references_call`, and
// the decline check inside `try_compress_pattern`).
//
// F2 — IDENTITY PRESERVATION: a recognized pattern CLASSIFIES a method; it
// never deletes it. Every matcher therefore reports how many of the ops it
// matched are the declaration's identity-bearing facts (`DefMethod`, its
// `Param*`, its `Return`) so the caller re-emits them unchanged before the
// classification op. Only genuinely redundant, non-identity ops — `Injects`
// for CTOR and `Flags(OVERRIDE)` for OVERRIDE. OBSERVABLE reads the typed
// `MethodModifiers(ASYNC)` fact without consuming it. The wrapper's
// leading/trailing `Flags(M)` runs are still summarized away.
// A method must never vanish from the hierarchical projection, the rendered
// `M` line, the `UnitTable`, the semantic registration, or a caller-side
// `Calls` subject merely because a pattern recognized it.

use super::PatternOp;
use crate::ir::opcodes::{CoreOp, DeclarationModifier};

/// A matcher's result: `(classification, retained, consumed)`.
///
/// `retained` is how many ops at the front of the matcher's slice are the
/// declaration's identity-bearing facts (`DefMethod`, `Param*`, `Return`) and
/// must be handed back to be re-emitted unchanged (F2). `consumed` is the full
/// span the matcher matched. See [`PatternMatch`].
type MatcherResult = (PatternOp, usize, usize);

/// `CoreOp::Call` first operand: the caller's `DefMethod` id.
fn op_references_method(op: &CoreOp, method_id: &str) -> bool {
    match op {
        CoreOp::Param(mid, _, _, _)
        | CoreOp::Return(mid, _)
        | CoreOp::Flags(mid, _)
        | CoreOp::MethodModifiers(mid, _)
        | CoreOp::ControlSummary(mid, _)
        | CoreOp::DataFlow(mid, _, _)
        | CoreOp::SideEffect(mid, _)
        | CoreOp::ExecutionContext(mid, _)
        | CoreOp::ControlFlow(mid, _, _)
        | CoreOp::Body(mid, _, _, _) => mid == method_id,
        CoreOp::Call(caller, _, _, _) => caller == method_id,
        _ => false,
    }
}

/// True when a surviving `CoreOp::Call` for `method_id` remains in the
/// caller's own trailing region after the consumed span.
///
/// The scan is bounded by the method's own op run: it walks forward while each
/// op still references `method_id` (annotation ops, params, returns, bodies,
/// and the method's own flags) and stops at the first op that belongs to
/// something else. A `CALL` inside that run would survive compression as an
/// orphan — the validator registers method identities from `DefMethod` only,
/// so no `PatternOp` can re-register it (E011) — therefore the caller must
/// decline.
///
/// The check is deliberately narrow: it can only fire on the NEW `Call` op
/// kind, so compression of every existing stream is bit-for-bit unchanged.
fn trailing_region_references_call(slice: &[CoreOp], offset: usize, method_id: &str) -> bool {
    let mut idx = offset;
    while idx < slice.len() {
        match &slice[idx] {
            CoreOp::Call(caller, _, _, _) if caller == method_id => return true,
            op if op_references_method(op, method_id) => idx += 1,
            _ => break,
        }
    }
    false
}

// ── Centralized flag consumption helpers ──────────────────────────────

/// Count consecutive pattern flags or authoritative typed facts for
/// `method_id`. Pattern flags may be summarized; typed facts are re-emitted.
fn count_trailing_annotations(slice: &[CoreOp], offset: usize, method_id: &str) -> usize {
    let mut count = 0;
    while offset + count < slice.len() {
        match &slice[offset + count] {
            CoreOp::Flags(mid, _)
            | CoreOp::MethodModifiers(mid, _)
            | CoreOp::ControlSummary(mid, _)
                if mid == method_id =>
            {
                count += 1
            }
            _ => break,
        }
    }
    count
}

/// True when `op` is one of the annotation ops that reference `method_id`
/// and have NO equivalent representation inside a compressed `PatternOp`.
///
/// The CTOR patterns DECLINE compression for any region whose ops after the
/// consumed span still carry an M-reference that a `PatternOp` payload cannot
/// represent (DataFlow / SideEffect / ExecutionContext / ControlFlow / Body /
/// Call). See the note below for why this conservatism is retained after F2.
///
/// DIS-2026-003: `Body(M, ...)` is emitted by the language layer at
/// Edit+ fidelity, between a constructor's `Return(M)` and its trailing
/// `Flags(M, ["PRIVATE"])` (parameter property). The `Body` op breaks the
/// wrapper's adjacent trailing-Flags run, so `Flags(M)` is no longer
/// adjacent to the consumed span and cannot be consumed by the wrapper.
/// Treating `Body(M)` as unrepresentable makes the guard decline
/// compression for that region, preserving the full valid sequence.
///
/// Native call facts (`CoreOp::Call`): the FIRST operand is the caller's
/// `DefMethod` id, so a surviving `CALL(M, ...)` is an M-reference that the
/// `PatternOp` payload cannot represent and that the validator cannot
/// re-register (E011 registers identities from `DefMethod` only).
///
/// F2 note — why this guard is unchanged: before F2 the decline was the only
/// thing keeping such an op from being orphaned, because the match consumed
/// `DefMethod(M)`. F2 now RETAINS `DefMethod`/`Param*`/`Return`, so a surviving
/// `Body(M)`/`DataFlow(M)`/`Call(M, …)` would keep a registered owner even if
/// the region compressed. Weakening this guard would therefore newly compress
/// shapes that have never been compressed, i.e. a compression-policy change
/// with its own stream, wire and delta consequences — a separate decision. The
/// guard stays: it is a conservative decline, it keeps every previously
/// uncompressed region byte-identical, and it cannot orphan anything.
fn op_is_unrepresentable_method_ref(op: &CoreOp, method_id: &str) -> bool {
    match op {
        CoreOp::DataFlow(mid, _, _)
        | CoreOp::SideEffect(mid, _)
        | CoreOp::ExecutionContext(mid, _)
        | CoreOp::ControlFlow(mid, _, _)
        | CoreOp::Body(mid, _, _, _) => mid == method_id,
        CoreOp::Call(caller, _, _, _) => caller == method_id,
        _ => false,
    }
}

/// After a ctor pattern's consumed span, the wrapper consumes the run of
/// immediately adjacent `Flags(method_id)` ops (established contract). If any
/// op AFTER that flag run still references `method_id`, compression would
/// orphan it — the caller must decline.
fn trailing_region_references_method(slice: &[CoreOp], offset: usize, method_id: &str) -> bool {
    let mut idx = offset;
    while idx < slice.len() {
        match &slice[idx] {
            CoreOp::Flags(mid, _)
            | CoreOp::MethodModifiers(mid, _)
            | CoreOp::ControlSummary(mid, _)
                if mid == method_id =>
            {
                idx += 1
            }
            _ => break,
        }
    }
    slice
        .get(idx)
        .is_some_and(|op| op_is_unrepresentable_method_ref(op, method_id))
}

/// One successful consumptive-pattern match.
///
/// `retained`/`retained_start` carry the F2 identity contract: the ops in
/// `slice[retained_start .. retained_start + retained]` are the matched
/// declaration's identity-bearing facts — `DefMethod`, then its `Param*`, then
/// its `Return`, in that order — and the caller re-emits them VERBATIM before
/// the classification op. `consumed` is the whole matched span, i.e. the
/// identity ops plus the genuinely redundant ops the recognizer summarizes
/// into the pattern (`Injects` for CTOR and `Flags(OVERRIDE)` for OVERRIDE)
/// plus the wrapper's leading and trailing annotation runs. Typed declaration
/// modifiers inside the consumed span are re-emitted unchanged.
///
/// `retained <= consumed` always; the two are equal for the recognizers that
/// summarize nothing but identity (PROMISE, EMPTY_CTOR, GETTER, SETTER).
pub(super) struct PatternMatch {
    /// The classification op emitted after the retained declaration facts.
    pub(super) pattern: PatternOp,
    /// Offset, within the matched slice, of the first identity-bearing op.
    pub(super) retained_start: usize,
    /// How many ops from `retained_start` are re-emitted unchanged.
    pub(super) retained: usize,
    /// Total ops consumed from the start of the matched slice.
    pub(super) consumed: usize,
}

/// Try to match a pattern at the start of `slice`.
///
/// Centralized wrapper that enforces the invariant:
/// > A pattern consuming `DefMethod(Mx)` must consume/handle all immediately
/// > adjacent `Flags(Mx, ...)` before and after its span.
///
/// This handles leading flags (emitted by the additive `CodePatternRecognizer`)
/// and trailing flags (emitted by language-layer passes) for EVERY consumptive
/// pattern, preventing orphaned Flags ops (E003) regardless of which pattern
/// matches.
///
/// It additionally enforces IRPAT-001 for native call facts: no consumptive
/// pattern (ctor, empty ctor, observable, promise, getter, setter, override)
/// may consume a `DefMethod(M)` while a surviving `CALL(M, ...)` — whose caller
/// id can never be re-registered from a `PatternOp` — would be orphaned.
///
/// F2: the returned match never includes the declaration's identity-bearing
/// ops in the part it replaces. Each matcher reports them as its `retained`
/// count, and they are re-emitted unchanged (see [`PatternMatch`]).
pub(super) fn try_compress_pattern(slice: &[CoreOp]) -> Option<PatternMatch> {
    if slice.is_empty() {
        return None;
    }

    // Step 1: Find the first non-Flags op to determine the method_id.
    // Leading Flags ops from the additive CodePatternRecognizer (e.g.
    // FLAGS(Mx, ["CTOR"])) may precede DefMethod.
    let first_non_flags = {
        let mut idx = 0;
        while idx < slice.len() {
            match &slice[idx] {
                CoreOp::Flags(_, _) => idx += 1,
                _ => break,
            }
        }
        idx
    };

    // If everything is Flags, there's no pattern to match.
    if first_non_flags >= slice.len() {
        return None;
    }

    // Extract method_id from the first non-Flags op (must be DefMethod for
    // any pattern to match).
    let method_id = match &slice[first_non_flags] {
        CoreOp::DefMethod(_, mid, _) => mid.clone(),
        _ => return None,
    };

    // Step 2: Verify all leading Flags ops reference this method_id.
    // If any leading flag belongs to a different method, do NOT consume it.
    for flag in slice.iter().take(first_non_flags) {
        if let CoreOp::Flags(mid, _) = flag {
            if mid != &method_id {
                return None;
            }
        }
    }

    // Step 3: Try each pattern on the slice starting after leading flags.
    let inner_slice = &slice[first_non_flags..];
    let result = try_ctor_pattern(inner_slice)
        .or_else(|| try_empty_ctor_pattern(inner_slice))
        .or_else(|| try_observable_pattern(inner_slice))
        .or_else(|| try_promise_pattern(inner_slice))
        .or_else(|| try_getter_pattern(inner_slice))
        .or_else(|| try_setter_pattern(inner_slice))
        .or_else(|| try_override_pattern(inner_slice));

    // Step 4: If a pattern matched, cover adjacent pattern flags and typed
    // declaration modifiers for the same method. The merge path re-emits the
    // authoritative modifiers and summarizes only eligible pattern flags.
    // IRPAT-001 decline: a surviving CALL for this method (see
    // `trailing_region_references_call`) must never be orphaned by consuming
    // its caller's `DefMethod`.
    if let Some((pat, inner_retained, inner_consumed)) = result {
        if trailing_region_references_call(slice, first_non_flags + inner_consumed, &method_id) {
            return None;
        }
        let trailing =
            count_trailing_annotations(slice, first_non_flags + inner_consumed, &method_id);
        Some(PatternMatch {
            pattern: pat,
            // F2: the declaration facts (DefMethod + Param* + Return) are
            // never part of what the classification replaces.
            retained_start: first_non_flags,
            retained: inner_retained,
            consumed: first_non_flags + inner_consumed + trailing,
        })
    } else {
        None
    }
}

/// Returns true if the method name is a recognized constructor name.
///
/// NF-08: Made `pub` so the additive `CodePatternRecognizer` in
/// `layers/patterns.rs` can also use it, ensuring both recognizers
/// match the same set of constructor names.
pub fn is_constructor_name(name: &str) -> bool {
    matches!(
        name,
        "constructor" | "new" | "__init__" | "initialize" | "ctor"
    )
}

/// CTOR pattern: `DEF_M(constructor) + Param* + Return + INJECTS` → 1 op.
///
/// NOTE: Leading/trailing Flags ops are handled by the centralized
/// `try_compress_pattern` wrapper. This function receives a slice that
/// starts at `DefMethod` and returns the identity-bearing prefix
/// (`DefMethod` + `Param*` + `Return`) unchanged alongside the
/// classification, so the only op the pattern actually replaces here is
/// `Injects` — a class-level DI list whose payload the pattern carries.
fn try_ctor_pattern(slice: &[CoreOp]) -> Option<MatcherResult> {
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
    let mut idx = 1; // skip DEF_M
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

    // F2: everything matched so far — `DefMethod`, its `Param*` and its
    // `Return` — is identity-bearing and is handed back for re-emission.
    // Only a trailing `Injects` is genuinely replaced by the classification.
    let identity_end = idx;

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

    // Orphan guard: if anything after the consumed span (past the wrapper's
    // trailing-Flags run) still references this method, the region cannot be
    // compressed without orphaning the reference — decline and leave the
    // original valid sequence in place.
    if trailing_region_references_method(slice, idx, &method_id) {
        return None;
    }

    Some((
        PatternOp::Constructor {
            class_id,
            method_id,
            deps,
        },
        identity_end,
        idx,
    ))
}

/// Empty-ctor pattern: `DEF_M(constructor) + Return` (no params, no injects).
///
/// F2: both matched ops are identity-bearing, so this classification is purely
/// additive — it replaces nothing.
fn try_empty_ctor_pattern(slice: &[CoreOp]) -> Option<MatcherResult> {
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
            // Orphan guard (same contract as try_ctor_pattern): decline when
            // the region after DEF_M + RET still references this method.
            if trailing_region_references_method(slice, 2, &method_id) {
                return None;
            }
            return Some((
                PatternOp::EmptyConstructor {
                    class_id,
                    method_id,
                },
                2,
                2,
            ));
        }
    }
    None
}

/// Observable pattern: `DEF_M + Return($P|$O) + MethodModifiers(ASYNC)`.
///
/// `DefMethod`, `Return`, and the authoritative modifier fact are retained;
/// the pattern is additive classification.
fn try_observable_pattern(slice: &[CoreOp]) -> Option<MatcherResult> {
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
    // Must be Promise-like and have an ASYNC declaration modifier.
    let is_promise_like = return_type == "$P"
        || return_type.contains("Promise")
        || return_type.contains("Observable");
    if !is_promise_like {
        return None;
    }
    match &slice[2] {
        CoreOp::MethodModifiers(mid, modifiers)
            if mid == &method_id && modifiers.contains(&DeclarationModifier::Async) =>
        {
            Some((
                PatternOp::Observable {
                    class_id,
                    method_id,
                    return_type,
                },
                2,
                2,
            ))
        }
        _ => None,
    }
}

/// Promise pattern: `DEF_M + Return($P)` (no ASYNC) → 1 op.
/// Only triggers if the observable pattern did not match
///
/// F2: both matched ops are identity-bearing, so this classification is purely
/// additive — it replaces nothing.
fn try_promise_pattern(slice: &[CoreOp]) -> Option<MatcherResult> {
    if slice.len() < 2 {
        return None;
    }
    let (class_id, method_id) = match &slice[0] {
        CoreOp::DefMethod(cid, mid, _) => (cid.clone(), mid.clone()),
        _ => return None,
    };
    match &slice[1] {
        CoreOp::Return(mid, ty) if mid == &method_id => {
            let is_promise_like = ty == "$P" || ty.contains("Promise") || ty.contains("Observable");
            if is_promise_like {
                Some((
                    PatternOp::Promise {
                        class_id,
                        method_id,
                        return_type: ty.clone(),
                    },
                    2,
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
///
/// F2: the accessor's `DefMethod` (and its `Return`, when matched) are
/// identity-bearing and are retained; this classification replaces nothing.
fn try_getter_pattern(slice: &[CoreOp]) -> Option<MatcherResult> {
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
            if mid == &method_id { 2 } else { 1 }
        } else {
            1
        }
    } else {
        1
    };
    Some((
        PatternOp::Getter {
            class_id,
            method_id,
            property,
        },
        consumed,
        consumed,
    ))
}

/// Setter pattern: `DEF_M(set X) [+ Param(value)]` → 1 op.
///
/// F2: the setter's `DefMethod` (and the `Param`/`Return` it matched) are
/// identity-bearing and are retained; this classification replaces nothing.
fn try_setter_pattern(slice: &[CoreOp]) -> Option<MatcherResult> {
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
        PatternOp::Setter {
            class_id,
            method_id,
            property,
        },
        idx,
        idx,
    ))
}

/// Override pattern: `DEF_M + Flags(OVERRIDE)` → 1 op.
///
/// F2: the `DefMethod` is retained; the `Flags(OVERRIDE)` op is the redundant
/// op the classification replaces (the OVERRIDE pattern name carries it).
fn try_override_pattern(slice: &[CoreOp]) -> Option<MatcherResult> {
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
                PatternOp::Override {
                    class_id,
                    method_id,
                },
                1,
                2,
            ))
        }
        _ => None,
    }
}
