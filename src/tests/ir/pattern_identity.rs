// src/tests/ir/pattern_identity.rs
//
// F2 — PATTERN IDENTITY PRESERVATION (producer boundary).
//
// Pattern recognition may CLASSIFY or summarize a method. It may NOT remove the
// identity-bearing declaration facts downstream consumers depend on:
//
//     DefMethod   → hierarchical MethodNode, rendered `M <name>`, semantic
//                   registration, `UnitTable` lookup, `Calls` caller subject
//     Param*      → signature reconstruction + the `UnitTable` fingerprint
//     Return      → rendered signature
//
// The consumptive recognizers used to consume all three into a single `PAT` op,
// so a recognized method vanished from every one of those representations while
// the pattern op still referred to it. The invariant now enforced at the
// producer boundary is:
//
//     A transformation must never delete an identity-bearing method
//     declaration while emitting another op that still refers to it.
//
// These regressions pin the producer-side contract: the classification is
// ADDITIVE (emitted after the declaration, never instead of it), the
// declaration keeps its original ids, and exactly one `DefMethod` per method id
// remains. The downstream consequences are pinned separately in
// `src/tests/ir/pattern_identity_downstream.rs`.
//
// Scope note: the ADDITIVE pre-`DefMethod` classification flags (`CTOR`,
// `OBSERVABLE`) and the additive accessor flags (`GETTER`/`SETTER`) belong to a
// different lifecycle and are deliberately NOT repaired or asserted here.

use crate::compression::Fidelity;
use crate::ir::compiler::IRCompiler;
use crate::ir::layers::PatternRecognizer;
use crate::ir::layers::patterns::CodePatternRecognizer;
use crate::ir::layers::typescript::TypeScriptLayer;
use crate::ir::opcodes::CoreOp;
use crate::ir::patterns::CompressingPatternRecognizer;

// ─ Fixtures (production shapes) ───────────────────────────────────────

/// A TypeScript constructor with a DI parameter property — the production
/// shape the CTOR classification recognizes.
const TS_CTOR: &str = r#"
export class Panel {
    constructor(private greeter: Greeter) {}

    greet(): string {
        return 'hi';
    }
}
"#;

/// A Promise-returning, non-`async`, zero-parameter method: the shape the
/// PROMISE classification recognizes (`DEF_M + Return(Promise-like)`). The body
/// carries no call fact on purpose — a call fact makes the IRPAT-001 decline
/// guard apply first, which is a separate, already-pinned contract.
///
/// Requires `Fidelity::Medium` or above: the recognizer matches the emitted
/// `Return` type, and at `Low` the TypeScript label compacts to `load()` so the
/// `Return` is the void alias.
const TS_PROMISE: &str = r#"
export class DataService {
    load(): Promise<string> {}
}
"#;

/// An Observable-returning, zero-parameter method: the shape the OBSERVABLE
/// classification recognizes (`DEF_M + Return(Observable-like)`).
///
/// Requires `Fidelity::Medium` or above for the same reason as `TS_PROMISE`,
/// while Edit fidelity may conservatively decline classification when a body
/// operation makes the pattern span non-adjacent.
const TS_OBSERVABLE: &str = r#"
export class StreamService {
    load(): Observable<string> {}
}
"#;

/// A parameter-less constructor: the shape the EMPTY_CTOR classification
/// recognizes (`DEF_M(constructor) + Return`).
const TS_EMPTY_CTOR: &str = r#"
export class EmptyPanel {
    constructor() {}
}
"#;

/// A Rust `fn new` in an inherent impl — the canonical Rust constructor and the
/// shape the CTOR classification recognizes.
#[cfg(feature = "rust")]
const RS_NEW: &str = r#"
pub struct Config {
    pub name: String,
}

impl Config {
    pub fn new(name: String) -> Self {
        Config { name }
    }
}
"#;

// ─ Helpers ────────────────────────────────────────────────────────────

/// Compile TypeScript with the real production compiler configuration.
/// `with_patterns = false` yields the PRE-pattern stream (the
/// `PatternRecognitionPass` runs but holds no recognizers, so it is a no-op),
/// which is how a test observes what the pattern pass changed.
fn compile_ts(source: &str, file_id: &str, fidelity: Fidelity, with_patterns: bool) -> Vec<CoreOp> {
    let (language, query_string) =
        crate::compression::language::language_for_extension("ts").expect("TS language");
    let mut compiler = IRCompiler::new();
    compiler.add_language_layer(Box::new(TypeScriptLayer::new()));
    if with_patterns {
        compiler.add_pattern_recognizer(Box::new(CodePatternRecognizer::new()));
        compiler.add_pattern_recognizer(Box::new(CompressingPatternRecognizer::new()));
    }
    compiler
        .compile(source, file_id, language, query_string, fidelity, None)
        .unwrap_or_else(|e| panic!("{file_id} should compile: {e}"))
        .instructions
}

#[cfg(feature = "rust")]
fn compile_rs(source: &str, file_id: &str) -> Vec<CoreOp> {
    use crate::ir::layers::rust::RustLayer;
    let (language, query_string) =
        crate::compression::language::language_for_extension("rs").expect("Rust language");
    let mut compiler = IRCompiler::new();
    compiler.add_language_layer(Box::new(RustLayer::new()));
    compiler.add_pattern_recognizer(Box::new(CodePatternRecognizer::new()));
    compiler.add_pattern_recognizer(Box::new(CompressingPatternRecognizer::new()));
    compiler
        .compile(source, file_id, language, query_string, Fidelity::Low, None)
        .unwrap_or_else(|e| panic!("{file_id} should compile: {e}"))
        .instructions
}

/// Every `(method_id, name)` a `DefMethod` registers, in stream order.
fn declared_methods(instructions: &[CoreOp]) -> Vec<(String, String)> {
    instructions
        .iter()
        .filter_map(|op| match op {
            CoreOp::DefMethod(_, mid, name) => Some((mid.clone(), name.clone())),
            _ => None,
        })
        .collect()
}

/// The method id a declaration named `name` was given, requiring exactly one.
fn sole_method_id(instructions: &[CoreOp], name: &str) -> String {
    let ids: Vec<String> = declared_methods(instructions)
        .into_iter()
        .filter(|(_, n)| n == name)
        .map(|(mid, _)| mid)
        .collect();
    assert_eq!(
        ids.len(),
        1,
        "expected exactly one `{name}` declaration: {:?}",
        declared_methods(instructions)
    );
    ids[0].clone()
}

/// Every `(name, args)` classification in the stream.
fn classifications(instructions: &[CoreOp]) -> Vec<(String, Vec<String>)> {
    instructions
        .iter()
        .filter_map(|op| match op {
            CoreOp::Pattern(name, args) => Some((name.clone(), args.clone())),
            _ => None,
        })
        .collect()
}

/// `(name, args)` for the classification named `name` that refers to `mid`.
fn classification_for(instructions: &[CoreOp], name: &str, mid: &str) -> Option<Vec<String>> {
    classifications(instructions)
        .into_iter()
        .find(|(n, args)| n == name && args.get(1).is_some_and(|a| a == mid))
        .map(|(_, args)| args)
}

/// The F2 invariant over a whole stream: a method-level classification op that
/// no `DefMethod` registers is exactly the defect this phase removes.
fn orphaned_classifications(instructions: &[CoreOp]) -> Vec<String> {
    let declared: Vec<String> = declared_methods(instructions)
        .into_iter()
        .map(|(mid, _)| mid)
        .collect();
    classifications(instructions)
        .into_iter()
        .filter(|(_, args)| {
            args.get(1)
                .is_some_and(|mid| mid.starts_with('M') && !declared.contains(mid))
        })
        .map(|(name, args)| format!("{name} {args:?}"))
        .collect()
}

/// How many `DefMethod` ops register `mid`.
fn declarations_of(instructions: &[CoreOp], mid: &str) -> usize {
    declared_methods(instructions)
        .into_iter()
        .filter(|(m, _)| m == mid)
        .count()
}

// ─ RED-F2-1 … RED-F2-5: production shapes ─────────────────────────────

#[test]
fn ts_constructor_keeps_its_declaration_and_gains_the_ctor_classification() {
    let pre = compile_ts(TS_CTOR, "ctor.ts", Fidelity::Low, false);
    let ctor_id = sole_method_id(&pre, "constructor");

    let post = compile_ts(TS_CTOR, "ctor.ts", Fidelity::Low, true);
    assert!(
        classification_for(&post, "CTOR", &ctor_id).is_some(),
        "CTOR classification must still be produced, got: {:?}",
        classifications(&post)
    );
    assert_eq!(
        declarations_of(&post, &ctor_id),
        1,
        "the constructor's identity must survive pattern recognition exactly \
         once: {post:?}"
    );
    assert!(
        orphaned_classifications(&post).is_empty(),
        "no classification may be orphaned from its declaration: {:?}",
        orphaned_classifications(&post)
    );
}

#[cfg(feature = "rust")]
#[test]
fn rust_fn_new_keeps_its_declaration_and_gains_the_ctor_classification() {
    let pre = compile_rs(RS_NEW, "ctor.rs");
    let new_id = sole_method_id(&pre, "new");

    let post = compile_rs(RS_NEW, "ctor.rs");
    assert!(
        classification_for(&post, "CTOR", &new_id).is_some(),
        "Rust `pub fn new(..) -> Self` must still be classified CTOR, got: {:?}",
        classifications(&post)
    );
    assert_eq!(
        declarations_of(&post, &new_id),
        1,
        "the `new` declaration must survive pattern recognition exactly once: {post:?}"
    );
    assert!(orphaned_classifications(&post).is_empty());
}

#[test]
fn promise_method_keeps_its_declaration_and_gains_the_promise_classification() {
    // `Fidelity::Medium`: the fidelity at which the TypeScript compacted label
    // still carries the declared return type (`Promise<string>`), which is the
    // shape `try_promise_pattern` matches. At `Low` the label is `load()` and
    // the emitted `Return` is the void alias, so no promise-like return exists
    // to match — the identity contract for that case is pinned separately by
    // `identity_survives_where_a_classification_cannot_fire`.
    let pre = compile_ts(TS_PROMISE, "promise.ts", Fidelity::Medium, false);
    let load_id = sole_method_id(&pre, "load");

    let post = compile_ts(TS_PROMISE, "promise.ts", Fidelity::Medium, true);
    assert!(
        classification_for(&post, "PROMISE", &load_id).is_some(),
        "PROMISE classification must still be produced, got: {:?}",
        classifications(&post)
    );
    assert_eq!(
        declarations_of(&post, &load_id),
        1,
        "the promise-returning method must survive pattern recognition: {post:?}"
    );
}

#[test]
fn observable_method_keeps_its_declaration_and_gains_the_observable_classification() {
    // `Fidelity::Medium` again: OBSERVABLE derives from the declared return.
    let pre = compile_ts(TS_OBSERVABLE, "observable.ts", Fidelity::Medium, false);
    let load_id = sole_method_id(&pre, "load");

    let post = compile_ts(TS_OBSERVABLE, "observable.ts", Fidelity::Medium, true);
    assert!(
        classification_for(&post, "OBSERVABLE", &load_id).is_some(),
        "OBSERVABLE classification must still be produced, got: {:?}",
        classifications(&post)
    );
    assert_eq!(
        declarations_of(&post, &load_id),
        1,
        "the Observable-returning method must survive pattern recognition: {post:?}"
    );
}

/// Identity preservation is UNCONDITIONAL: it must not depend on a
/// classification being produced.
///
/// This is a **non-RED invariant guard** (unlike RED-F2-1…12): before the F2
/// fix no classification fired for these shapes either, so it was green then
/// and stays green now. It exists to fail loudly if a later change makes
/// identity conditional on a pattern match.
#[test]
fn identity_survives_where_a_classification_cannot_fire() {
    // F2 is unconditional: identity preservation must NOT depend on a
    // classification being produced. The real pipeline reports that at
    // `Fidelity::Low` the TypeScript method label is compacted to `load()`, so
    // the emitted `Return` is the void alias and the Promise/Observable
    // recognizers have no qualifying return contract to match. The declaration must
    // still survive intact and no classification may be orphaned.
    for (file_id, source) in [
        ("DataService.ts", TS_PROMISE),
        ("StreamService.ts", TS_OBSERVABLE),
    ] {
        let pre = compile_ts(source, file_id, Fidelity::Low, false);
        let mid = sole_method_id(&pre, "load");

        let post = compile_ts(source, file_id, Fidelity::Low, true);
        assert_eq!(
            declarations_of(&post, &mid),
            1,
            "{file_id} @ Low: the declaration must survive even when no \
             classification fires: {post:?}"
        );
        assert!(
            orphaned_classifications(&post).is_empty(),
            "{file_id} @ Low: {:?}",
            orphaned_classifications(&post)
        );
    }
}

#[test]
fn empty_constructor_keeps_its_declaration_and_gains_the_empty_ctor_classification() {
    let pre = compile_ts(TS_EMPTY_CTOR, "empty-ctor.ts", Fidelity::Low, false);
    let ctor_id = sole_method_id(&pre, "constructor");

    let post = compile_ts(TS_EMPTY_CTOR, "empty-ctor.ts", Fidelity::Low, true);
    assert!(
        classification_for(&post, "EMPTY_CTOR", &ctor_id).is_some(),
        "EMPTY_CTOR classification must still be produced, got: {:?}",
        classifications(&post)
    );
    assert_eq!(
        declarations_of(&post, &ctor_id),
        1,
        "the parameter-less constructor must survive pattern recognition: {post:?}"
    );
}

// ─ RED-F2-6 / RED-F2-12: the complete consumptive surface ─────────────

/// One recognizer-boundary fixture per consumptive classification.
///
/// Two of the shapes have no production producer today:
///   * `OVERRIDE` needs `Flags(M, ["OVERRIDE"])`, which no language layer emits
///     (the C# layer documents `override` but publishes no such flag);
///   * `GETTER`/`SETTER` need a written method name containing a space, which
///     `parse_method_sig` never produces (the additive accessor path is the F4
///     lifecycle, deliberately untouched here).
///     The shapes that DO have production producers are covered by RED-F2-1…5;
///     these fixtures pin the same identity contract on the remaining recognizers.
fn boundary_class() -> CoreOp {
    CoreOp::DefClass("C1".into(), "Example".into())
}

fn boundary_cases() -> Vec<(&'static str, &'static str, Vec<CoreOp>, &'static str)> {
    vec![
        (
            "ctor",
            "CTOR",
            vec![
                boundary_class(),
                CoreOp::DefMethod("C1".into(), "M1".into(), "constructor".into()),
                CoreOp::Param("M1".into(), "P1".into(), "SV1".into(), "dep".into()),
                CoreOp::Return("M1".into(), "$v".into()),
                CoreOp::Injects("C1".into(), vec!["S1".into()]),
            ],
            "M1",
        ),
        (
            "empty-ctor",
            "EMPTY_CTOR",
            vec![
                boundary_class(),
                CoreOp::DefMethod("C1".into(), "M1".into(), "constructor".into()),
                CoreOp::Return("M1".into(), "$v".into()),
            ],
            "M1",
        ),
        (
            "observable",
            "OBSERVABLE",
            vec![
                boundary_class(),
                CoreOp::DefMethod("C1".into(), "M1".into(), "load".into()),
                CoreOp::Return("M1".into(), "$P".into()),
                CoreOp::MethodModifiers(
                    "M1".into(),
                    vec![crate::ir::opcodes::DeclarationModifier::Async],
                ),
            ],
            "M1",
        ),
        (
            "promise",
            "PROMISE",
            vec![
                boundary_class(),
                CoreOp::DefMethod("C1".into(), "M1".into(), "load".into()),
                CoreOp::Return("M1".into(), "$P".into()),
            ],
            "M1",
        ),
        (
            "override",
            "OVERRIDE",
            vec![
                boundary_class(),
                CoreOp::DefMethod("C1".into(), "M1".into(), "toString".into()),
                CoreOp::Flags("M1".into(), vec!["OVERRIDE".into()]),
            ],
            "M1",
        ),
        (
            "getter",
            "GETTER",
            vec![
                boundary_class(),
                CoreOp::DefMethod("C1".into(), "M1".into(), "get name".into()),
                CoreOp::Return("M1".into(), "$s".into()),
            ],
            "M1",
        ),
        (
            "setter",
            "SETTER",
            vec![
                boundary_class(),
                CoreOp::DefMethod("C1".into(), "M1".into(), "set name".into()),
                CoreOp::Param("M1".into(), "P1".into(), "$s".into(), "value".into()),
                CoreOp::Return("M1".into(), "$v".into()),
            ],
            "M1",
        ),
    ]
}

#[test]
fn every_consumptive_classification_is_attached_to_a_surviving_declaration() {
    let recognizer = CompressingPatternRecognizer::new();
    for (label, pattern, input, mid) in boundary_cases() {
        let output = recognizer.recognize(&input);

        assert!(
            classification_for(&output, pattern, mid).is_some(),
            "{label}: the classification must still be produced, got: {:?}",
            classifications(&output)
        );
        assert_eq!(
            declarations_of(&output, mid),
            1,
            "{label}: the declaration must survive exactly once — a classification \
             may not disable or duplicate identity: {output:?}"
        );
        assert!(
            orphaned_classifications(&output).is_empty(),
            "{label}: classification without a declaration: {:?}",
            orphaned_classifications(&output)
        );
    }
}

#[test]
fn production_streams_never_orphan_a_classification_at_any_fidelity() {
    // The fidelity matrix. The invariant must hold whether or not a recognizer
    // fires at that fidelity and whether or not the Edit-only `Body` op makes
    // the IRPAT-001 guard decline the region. Nothing here asserts that a
    // classification is ACTIVE at a given fidelity — only that a classification
    // can never outlive the declaration it refers to.
    let cases: [(&str, &str); 3] = [
        ("Panel.ts", TS_CTOR),
        ("DataService.ts", TS_PROMISE),
        ("StreamService.ts", TS_OBSERVABLE),
    ];
    for fidelity in [
        Fidelity::Low,
        Fidelity::Medium,
        Fidelity::High,
        Fidelity::Edit,
    ] {
        for (file_id, source) in cases {
            for with_patterns in [false, true] {
                let stream = compile_ts(source, file_id, fidelity, with_patterns);
                let orphans = orphaned_classifications(&stream);
                assert!(
                    orphans.is_empty(),
                    "{file_id} @ {fidelity:?} (patterns={with_patterns}): \
                     orphaned classifications {orphans:?}"
                );
                if with_patterns {
                    // `declared_methods` reports `(method_id, name)` pairs: the
                    // id proves the classification's `args[1]` really points at
                    // the declaration, not merely at a same-named one.
                    println!(
                        "{file_id} @ {fidelity:?}: classifications={:?} methods={:?}",
                        classifications(&stream),
                        declared_methods(&stream)
                    );
                }
            }
        }
    }
}

// ─ Compression impact ─────────────────────────────────────────────────

#[test]
fn retaining_identity_ops_costs_exactly_the_declaration_ops() {
    // Structural (not wall-clock) impact, on the smallest streams that show it.
    let recognizer = CompressingPatternRecognizer::new();

    // CTOR: 4 declaration + DI ops in. Pre-F2 the whole span became one `PAT`;
    // post-F2 the three declaration ops are re-emitted and only the `Injects`
    // op is summarized into the classification.
    let ctor_in = boundary_cases()[0].2.clone();
    let ctor_out = recognizer.recognize(&ctor_in);
    println!(
        "CTOR   : pre-pattern {} ops -> post-pattern {} ops ({ctor_out:?})",
        ctor_in.len(),
        ctor_out.len()
    );
    assert_eq!(
        ctor_out,
        vec![
            CoreOp::DefClass("C1".into(), "Example".into()),
            CoreOp::DefMethod("C1".into(), "M1".into(), "constructor".into()),
            CoreOp::Param("M1".into(), "P1".into(), "SV1".into(), "dep".into()),
            CoreOp::Return("M1".into(), "$v".into()),
            CoreOp::Pattern("CTOR".into(), vec!["C1".into(), "M1".into(), "S1".into()]),
        ],
        "the classification must follow the retained declaration, in order"
    );

    // PROMISE matches nothing but identity, so the classification is purely
    // additive: the merged stream is one op LARGER than the pattern's span.
    // That is the honest cost of keeping identity — correctness wins.
    let promise_in = boundary_cases()[3].2.clone();
    let promise_out = recognizer.recognize(&promise_in);
    println!(
        "PROMISE: pre-pattern {} ops -> post-pattern {} ops ({promise_out:?})",
        promise_in.len(),
        promise_out.len()
    );
    assert_eq!(
        promise_out,
        vec![
            CoreOp::DefClass("C1".into(), "Example".into()),
            CoreOp::DefMethod("C1".into(), "M1".into(), "load".into()),
            CoreOp::Return("M1".into(), "$P".into()),
            CoreOp::Pattern(
                "PROMISE".into(),
                vec!["C1".into(), "M1".into(), "$P".into()]
            ),
        ]
    );
}
