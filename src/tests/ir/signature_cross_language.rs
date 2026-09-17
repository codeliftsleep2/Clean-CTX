// src/tests/ir/signature_cross_language.rs
//
// Cross-language probes for the SHARED declaration-identity boundary.
//
// The defect was not C#-specific in its mechanism: `compaction::method` (the
// flattened label) and `PassContext::parse_method_sig` are shared by every
// language that reaches the IR, so a type-parameter list containing `, `
// (`pair<A, B>`) destroyed the name wherever the flattening happened. These
// probes establish which languages were affected and prove that the renderer
// itself is innocent — it consumes the IR's structured fields and never
// re-parses text (see `ir::render_llm`).
//
// Per-language expectations follow each language's established compaction
// contract:
//   * Low carries the bare identifier (`pair`),
//   * Medium collapses `, ` for NAME-FIRST languages (`pair<A,B>`),
//   * High keeps the declaration verbatim (`pair<A, B>`),
//   * Java's method type-parameter list precedes the return type, so the
//     identifier that owns the parameter list is unchanged at every fidelity.

use super::*;

/// Compile TypeScript with the production compiler configuration.
#[cfg(feature = "typescript")]
fn compile_ts(source: &str, fidelity: Fidelity) -> CompiledIR {
    use crate::ir::layers::typescript::TypeScriptLayer;
    let language =
        crate::compression::language::safe_typescript_language().expect("typescript grammar");
    let mut compiler = IRCompiler::new();
    compiler.add_language_layer(Box::new(TypeScriptLayer::new()));
    compiler.add_pattern_recognizer(Box::new(CodePatternRecognizer::new()));
    compiler.add_pattern_recognizer(Box::new(CompressingPatternRecognizer::new()));
    compiler
        .compile(
            source,
            "Signature.ts",
            language,
            crate::queries::TS_QUERY,
            fidelity,
            None,
        )
        .expect("production compilation must succeed")
}

/// Compile Java with the production compiler configuration.
#[cfg(feature = "java")]
fn compile_java(source: &str, fidelity: Fidelity) -> CompiledIR {
    use crate::ir::layers::java::JavaLayer;
    let language = crate::compression::language::safe_java_language().expect("java grammar");
    let mut compiler = IRCompiler::new();
    compiler.add_language_layer(Box::new(JavaLayer::new()));
    compiler.add_pattern_recognizer(Box::new(CodePatternRecognizer::new()));
    compiler.add_pattern_recognizer(Box::new(CompressingPatternRecognizer::new()));
    compiler
        .compile(
            source,
            "Signature.java",
            language,
            crate::queries::JAVA_QUERY,
            fidelity,
            None,
        )
        .expect("production compilation must succeed")
}

/// Compile Rust with the production compiler configuration.
#[cfg(feature = "rust")]
fn compile_rs(source: &str, fidelity: Fidelity) -> CompiledIR {
    use crate::ir::layers::rust::RustLayer;
    let language = crate::compression::language::safe_rust_language().expect("rust grammar");
    let mut compiler = IRCompiler::new();
    compiler.add_language_layer(Box::new(RustLayer::new()));
    compiler.add_pattern_recognizer(Box::new(CodePatternRecognizer::new()));
    compiler.add_pattern_recognizer(Box::new(CompressingPatternRecognizer::new()));
    compiler
        .compile(
            source,
            "signature.rs",
            language,
            crate::queries::RS_QUERY,
            fidelity,
            None,
        )
        .expect("production compilation must succeed")
}

/// The name a two-type-parameter name-first declaration must carry at
/// `fidelity`.
#[cfg(any(feature = "typescript", feature = "rust"))]
fn expected_two_type_params<'a>(
    fidelity: Fidelity,
    spaced: &'a str,
    collapsed: &'a str,
) -> &'a str {
    match fidelity {
        Fidelity::Low => "pair",
        Fidelity::Medium => collapsed,
        _ => spaced,
    }
}

// ── TypeScript ─────────────────────────────────────────────────────

#[cfg(feature = "typescript")]
#[test]
fn typescript_function_with_two_type_params_keeps_its_identity() {
    // A top-level `function_declaration` is a `func.root` capture. The IR path
    // receives that capture as the RAW declaration (only `method.root` is
    // routed through the per-fidelity label compaction — see `CoreIRPass`'s
    // capture closure), so the identity here is the declaration's own text at
    // EVERY fidelity, spacing included. That is asserted exactly, together
    // with the written parameters and the trailing tuple annotation, rather
    // than through a per-fidelity label expectation that does not apply.
    const SRC: &str = "export function pair<A, B>(a: A, b: B): [A, B] {\n  return [a, b];\n}\n";
    for fidelity in [Fidelity::Low, Fidelity::Medium, Fidelity::High] {
        let ir = compile_ts(SRC, fidelity);
        let names = method_names(&ir);
        assert!(
            names.iter().any(|n| n == "pair<A, B>"),
            "{fidelity:?}: expected the declared identity, declared {names:?}"
        );
        assert!(
            !names.iter().any(|n| n == "pair" || n == "B>" || n == "B"),
            "{fidelity:?}: a type parameter is not a method identity: {names:?}"
        );
        let facts = method(&ir, "pair<A, B>");
        assert_eq!(
            facts.params,
            vec!["a".to_string(), "b".to_string()],
            "{fidelity:?}: the two written parameters stay two"
        );
        assert_eq!(
            facts.return_type, "[A, B]",
            "{fidelity:?}: the declared tuple return type"
        );
    }
}

#[cfg(feature = "typescript")]
#[test]
fn typescript_method_with_two_type_params_keeps_its_identity() {
    const SRC: &str =
        "export class Pairer {\n  pair<A, B>(a: A, b: B): [A, B] {\n    return [a, b];\n  }\n}\n";
    for fidelity in [Fidelity::Low, Fidelity::Medium, Fidelity::High] {
        let ir = compile_ts(SRC, fidelity);
        let names = method_names(&ir);
        let expected = expected_two_type_params(fidelity, "pair<A, B>", "pair<A,B>");
        assert!(
            names.iter().any(|n| n == expected),
            "{fidelity:?}: expected {expected}, declared {names:?}"
        );
    }
    // A name-first language's tuple return is its trailing annotation, so it
    // was never part of this defect: it stays the declared tuple.
    let ir = compile_ts(SRC, Fidelity::High);
    assert_eq!(method(&ir, "pair<A, B>").return_type, "[A, B]");
    assert_eq!(
        method(&ir, "pair<A, B>").params,
        vec!["a".to_string(), "b".to_string()]
    );
}

// ── Rust ───────────────────────────────────────────────────────────

#[cfg(feature = "rust")]
#[test]
fn rust_fn_with_two_type_params_and_tuple_return_keeps_its_identity() {
    // A free `fn` has no owning type, so the probe uses the module's own impl
    // block — exactly how Rust methods reach the IR in this repository.
    const SRC: &str = "pub struct Pairer;\n\nimpl Pairer {\n    pub fn pair<A, B>(a: A, b: B) -> (i32, i32) {\n        (1, 2)\n    }\n}\n";
    for fidelity in [Fidelity::Low, Fidelity::Medium, Fidelity::High] {
        let ir = compile_rs(SRC, fidelity);
        let names = method_names(&ir);
        let expected = expected_two_type_params(fidelity, "pair<A, B>", "pair<A,B>");
        assert!(
            names.iter().any(|n| n == expected),
            "{fidelity:?}: expected {expected}, declared {names:?}"
        );
        assert!(
            !names.iter().any(|n| n == "B>" || n == "B"),
            "{fidelity:?}: a type parameter is not a method identity: {names:?}"
        );
        assert_eq!(
            method(&ir, expected).params,
            vec!["a".to_string(), "b".to_string()],
            "{fidelity:?}: the two written parameters stay two"
        );
    }

    // Both halves of the combination survive at verbatim fidelity.
    let ir = compile_rs(SRC, Fidelity::High);
    assert_eq!(method(&ir, "pair<A, B>").return_type, "-> (i32, i32)");
}

// ── Java (control) ─────────────────────────────────────────────────

/// Java's method type-parameter list precedes the return type, so the
/// identifier that owns the parameter list was never affected. This is the
/// control that isolates the defect to the shared flattening rule rather than
/// to the renderer.
#[cfg(feature = "java")]
#[test]
fn java_method_with_two_type_params_keeps_its_identity() {
    const SRC: &str = "public class Pairer {\n    public <A, B> Result<A> pair(A a, B b) {\n        return null;\n    }\n}\n";
    for fidelity in [Fidelity::Low, Fidelity::Medium, Fidelity::High] {
        let ir = compile_java(SRC, fidelity);
        let names = method_names(&ir);
        assert!(
            names.iter().any(|n| n == "pair"),
            "{fidelity:?}: expected pair, declared {names:?}"
        );
        assert!(
            !names.iter().any(|n| n == "B>" || n == "pair<A, B>"),
            "{fidelity:?}: {names:?}"
        );
        assert_eq!(
            method(&ir, "pair").params,
            if fidelity == Fidelity::Low {
                vec!["a".to_string(), "b".to_string()]
            } else {
                // A colon-free parameter list keeps the declaration text (the
                // established C#/Java parameter shape), never an inflated list.
                vec!["A a".to_string(), "B b".to_string()]
            },
            "{fidelity:?}: the two written parameters stay two"
        );
    }
}
