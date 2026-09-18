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
// recognises a pattern, it replaces the redundant source instructions with a
// single compact `PAT_*` classification op. This is the Layer 4 "advanced
// compression" pass called for in §11 of the spec.
//
// F2 — IDENTITY PRESERVATION: a pattern classifies a declaration; it never
// deletes it. `DefMethod`, its `Param*`, and its `Return` are identity-bearing
// facts every downstream consumer depends on (hierarchical `MethodNode`,
// rendered `M <name>`, `UnitTable`/`apply_edit` targeting, semantic method
// registration, caller-side `Calls` identity), so they are re-emitted
// unchanged and the `PAT` op is added alongside them. Only genuinely
// redundant, non-identity ops are summarized away (`Injects` for CTOR,
// `Flags(ASYNC)` for OBSERVABLE, `Flags(OVERRIDE)` for OVERRIDE, and the
// additive/trailing `Flags(M)` runs the wrapper consumes). For PROMISE,
// EMPTY_CTOR, GETTER and SETTER nothing but identity is matched, so those
// classifications are purely additive.
//
// Recognised patterns:
//   - **PAT_CTOR**   — `DEF_M(constructor) + SIG*(P:ServiceType) + RET + INJECTS`
//   - **PAT_OBSERVABLE** — `DEF_M + RET($P) + FLAGS(ASYNC)`
//   - **PAT_GETTER** / **PAT_SETTER** — `DEF_M(get X)` / `DEF_M(set X)`
//   - **PAT_OVERRIDE** — `DEF_M + FLAGS(OVERRIDE)`
//   - **PAT_PROMISE** — `DEF_M + RET($P)` (without ASYNC)
//
// A pattern that doesn't match falls through unchanged (zero regression).

use super::layers::PatternRecognizer;
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
    Override { class_id: String, method_id: String },
    /// A pattern that was recognised but the constructor had no params —
    /// still useful to flag so the LLM knows it's a ctor.
    /// Wire: `["PAT", "EMPTY_CTOR", class_id, method_id]`
    EmptyConstructor { class_id: String, method_id: String },
}

impl PatternOp {
    /// Convert to a positional `Vec<String>` for the wire.
    pub fn to_tuple(&self) -> Vec<String> {
        match self {
            PatternOp::Constructor {
                class_id,
                method_id,
                deps,
            } => {
                let mut t = vec![
                    "PAT".into(),
                    "CTOR".into(),
                    class_id.clone(),
                    method_id.clone(),
                ];
                t.extend(deps.iter().cloned());
                t
            }
            PatternOp::EmptyConstructor {
                class_id,
                method_id,
            } => {
                vec![
                    "PAT".into(),
                    "EMPTY_CTOR".into(),
                    class_id.clone(),
                    method_id.clone(),
                ]
            }
            PatternOp::Observable {
                class_id,
                method_id,
                return_type,
            } => {
                vec![
                    "PAT".into(),
                    "OBSERVABLE".into(),
                    class_id.clone(),
                    method_id.clone(),
                    return_type.clone(),
                ]
            }
            PatternOp::Promise {
                class_id,
                method_id,
                return_type,
            } => {
                vec![
                    "PAT".into(),
                    "PROMISE".into(),
                    class_id.clone(),
                    method_id.clone(),
                    return_type.clone(),
                ]
            }
            PatternOp::Getter {
                class_id,
                method_id,
                property,
            } => {
                vec![
                    "PAT".into(),
                    "GETTER".into(),
                    class_id.clone(),
                    method_id.clone(),
                    property.clone(),
                ]
            }
            PatternOp::Setter {
                class_id,
                method_id,
                property,
            } => {
                vec![
                    "PAT".into(),
                    "SETTER".into(),
                    class_id.clone(),
                    method_id.clone(),
                    property.clone(),
                ]
            }
            PatternOp::Override {
                class_id,
                method_id,
            } => {
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
                let deps = if tuple.len() > 4 {
                    tuple[4..].to_vec()
                } else {
                    Vec::new()
                };
                Some(PatternOp::Constructor {
                    class_id,
                    method_id,
                    deps,
                })
            }
            "EMPTY_CTOR" => Some(PatternOp::EmptyConstructor {
                class_id,
                method_id,
            }),
            "OBSERVABLE" => {
                let return_type = tuple.get(4)?.clone();
                Some(PatternOp::Observable {
                    class_id,
                    method_id,
                    return_type,
                })
            }
            "PROMISE" => {
                let return_type = tuple.get(4)?.clone();
                Some(PatternOp::Promise {
                    class_id,
                    method_id,
                    return_type,
                })
            }
            "GETTER" => {
                let property = tuple.get(4)?.clone();
                Some(PatternOp::Getter {
                    class_id,
                    method_id,
                    property,
                })
            }
            "SETTER" => {
                let property = tuple.get(4)?.clone();
                Some(PatternOp::Setter {
                    class_id,
                    method_id,
                    property,
                })
            }
            "OVERRIDE" => Some(PatternOp::Override {
                class_id,
                method_id,
            }),
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

    /// The source span this classification recognises, as a statistic.
    ///
    /// This is the historical span shape of each pattern family (`DEF_M` +
    /// signature ops + `RET` [+ `INJECTS`] and so on). It is NOT how many ops
    /// the merged stream loses: since F2 the identity-bearing ops
    /// (`DefMethod`, `Param*`, `Return`) are retained, and the authoritative
    /// retained/consumed split for a match is reported by
    /// `recognize::PatternMatch`. Kept unchanged for callers that report
    /// pattern-recognition statistics.
    pub fn consumed(&self) -> usize {
        match self {
            // CTOR: DEF_M + Param* + Return + INJECTS
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
/// **classifies** the matched region and emits a single compact `PatternOp`
/// per match. The declaration's identity-bearing ops (`DefMethod`, `Param*`,
/// `Return`) are never part of what the classification replaces; they are
/// reported by `PatternMatch::retained` and re-emitted by `compress_merged`
/// (see `F2` in the module header). Because `compress` is a *pattern-only*
/// extraction view, it emits classifications alone — it is `compress_merged`
/// that produces the merged stream every production consumer sees.
#[derive(Debug, Clone, Default)]
pub struct CompressingPatternRecognizer;

impl CompressingPatternRecognizer {
    pub fn new() -> Self {
        Self
    }

    /// Extract the pattern stream by recognising patterns.
    ///
    /// Returns `(patterns, stats)` where `stats.source_ops` counts the source
    /// ops the scan covered (including the identity-bearing ops each match
    /// retains, which are still part of the matched span) and `stats.output_ops`
    /// counts the classifications emitted. This API does not produce a merged
    /// stream: use `compress_merged` for that.
    pub fn compress(&self, instructions: &[CoreOp]) -> (Vec<PatternOp>, CompressionStats) {
        let mut output: Vec<PatternOp> = Vec::new();
        let mut source_count = 0usize;
        let mut output_count = 0usize;
        let mut i = 0;

        while i < instructions.len() {
            if let Some(matched) = try_compress_pattern(&instructions[i..]) {
                source_count += matched.consumed;
                output_count += 1;
                output.push(matched.pattern);
                i += matched.consumed;
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
    /// Returns a `Vec<MergeItem>` which is either a passthrough
    /// `CoreOp` or a recognised `PatternOp`. The caller can decide how
    /// to serialise the merged stream.
    ///
    /// F2 (identity preservation): a match's identity-bearing ops —
    /// `DefMethod`, then its `Param*`, then its `Return` — are re-emitted as
    /// passthrough items, in their original order, BEFORE the classification
    /// op. A pattern therefore ANNOTATES the declaration instead of replacing
    /// it, so every downstream consumer (hierarchical projection, renderer,
    /// `UnitTable`, semantic projection, `Calls` attribution) still receives a
    /// method it can attach the classification to. Only the redundant
    /// non-identity ops the recognizer summarized are dropped.
    ///
    /// F-32: Renamed from `CompressedItem` to `MergeItem` to clarify
    /// that this is a merge-result enum, not a compressed instruction.
    pub fn compress_merged(&self, instructions: &[CoreOp]) -> Vec<MergeItem> {
        let mut output: Vec<MergeItem> = Vec::new();
        let mut i = 0;

        while i < instructions.len() {
            if let Some(matched) = try_compress_pattern(&instructions[i..]) {
                let retained_start = i + matched.retained_start;
                let retained_end = retained_start + matched.retained;
                for op in &instructions[retained_start..retained_end] {
                    output.push(MergeItem::Passthrough(op.clone()));
                }
                output.push(MergeItem::Pattern(matched.pattern));
                i += matched.consumed;
            } else {
                output.push(MergeItem::Passthrough(instructions[i].clone()));
                i += 1;
            }
        }

        output
    }
}

/// NF-06: Implement the `PatternRecognizer` trait for `CompressingPatternRecognizer`.
///
/// This bridges the consumptive pattern recognizer into the production compile
/// path (Layer 4). The `recognize` method calls `compress_merged` and maps
/// each `MergeItem` to a `CoreOp`:
///   - `Passthrough` ops are forwarded as-is.
///   - `Pattern` ops are encoded as `CoreOp::Pattern(name, args)` where name
///     is the pattern's canonical name (e.g., "CTOR") and args contains the
///     tuple payload exluding the "PAT" prefix.
///
/// This replaces the additive `CodePatternRecognizer`'s flag-based approach
/// with real compression: the pattern's redundant source ops become one `PAT`
/// op, while the declaration's identity-bearing ops (`DefMethod`, `Param*`,
/// `Return`) are forwarded unchanged and simply precede that op (F2).
impl PatternRecognizer for CompressingPatternRecognizer {
    fn recognize(&self, instructions: &[CoreOp]) -> Vec<CoreOp> {
        let merged = self.compress_merged(instructions);
        merged
            .into_iter()
            .map(|item| match item {
                MergeItem::Passthrough(op) => op,
                MergeItem::Pattern(pat) => {
                    let tuple = pat.to_tuple();
                    // tuple[0] is "PAT", tuple[1] is pattern name, rest are args
                    let name = tuple.get(1).cloned().unwrap_or_default();
                    let args = tuple.into_iter().skip(2).collect();
                    CoreOp::Pattern(name, args)
                }
            })
            .collect()
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
/// F-32: Renamed from `CompressedItem` to `MergeItem` to clarify that
/// this enum represents the result of merging patterns into the instruction
/// stream — either a passthrough (unchanged instruction) or a compressed
/// pattern op. The name `CompressedItem` was confusing because `PatternOp`
/// is also "compressed" but lives at a different abstraction level.
pub enum MergeItem {
    /// An instruction that no pattern matched.
    Passthrough(CoreOp),
    /// A recognised pattern that replaces one or more instructions.
    Pattern(PatternOp),
}

// Consumptive pattern recognition: matchers, flag consumption, and the
// IRPAT-001 orphan guards. Split along the representation/recognition
// boundary so this module stays inside the active-file size ceiling.
mod recognize;
use recognize::try_compress_pattern;

// NF-08: the additive `CodePatternRecognizer` in `layers/patterns.rs` matches
// the same constructor names, so this stays on the module's public surface even
// though its definition now lives in the `recognize` child module.
pub use recognize::is_constructor_name;

#[cfg(test)]
#[path = "../tests/ir/patterns.rs"]
mod tests;

// IRPAT-001 for native call facts (RED-CALL21): a consumptive pattern must not
// orphan a surviving `CoreOp::Call`.
#[cfg(test)]
#[path = "../tests/ir/call_pattern_orphan.rs"]
mod call_pattern_orphan_tests;
