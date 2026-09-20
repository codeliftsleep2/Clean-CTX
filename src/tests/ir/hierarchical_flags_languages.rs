// src/tests/ir/hierarchical_flags_languages.rs
//
// Cross-language pipeline probes for the method-flag projection fix
// (RED-FLAG7..RED-FLAG10). Child module of `hierarchical_flags.rs`: the
// synthetic projection regressions and the helper surface live in the parent
// and are visible here through `use super::*` (a private item of an ancestor
// module is visible in its descendants).
use super::*;

// ── Cross-language pipeline probes (production compiler config) ────────
//
// Each probe compiles ONE class with ONE method that carries a declaration
// modifier AND control flow, through the same compiler configuration the
// production path uses (language layer + both production pattern
// recognizers), and then asserts:
//
//   1. the flat stream keeps declaration modifiers and control summaries in
//      separate typed operations,
//   2. the hierarchical node preserves both occurrences, with the
//      declaration family FIRST (capture order is deterministic:
//      `run_capture_pipeline_nodes` sorts captures by start byte, and a
//      declaration's `method.root` capture precedes every control-flow
//      capture inside its body), and
//   3. the rendered line carries every value in the
//      compact `ctl:` form.
//
// The intra-family order of the control-flow values is not pinned positionally
// here (the synthetic regressions above pin first-seen ordering exactly); the
// probe pins the declaration family's prefix, the exact membership and
// the verbatim rendering.

#[cfg(any(
    feature = "csharp",
    feature = "typescript",
    feature = "java",
    feature = "rust"
))]
fn probe_single_method(ir: &CompiledIR, declaration: &[&str], control_flow: &[&str], label: &str) {
    let hir = ir_to_hierarchical(ir);
    let all = methods(&hir);
    assert_eq!(
        all.len(),
        1,
        "{label}: exactly one method expected: {all:?}"
    );
    let (mid, _name, merged) = &all[0];

    // Pattern-recognition protection: the pre-hierarchical flat stream still
    // carries one separate op per producer.
    let raw_modifiers = modifier_ops_for(ir, mid);
    let raw_flags = flag_ops_for(ir, mid);
    assert_eq!(
        raw_modifiers.len(),
        1,
        "{label}: exactly one declaration-modifier occurrence expected: {raw_modifiers:?}"
    );
    assert_eq!(
        raw_flags.len(),
        1,
        "{label}: one typed control-summary occurrence expected"
    );

    // Preservation: the declaration family's values come FIRST (capture order
    // is by start byte, so a declaration's own op precedes every control-flow
    // capture inside its body). The hierarchy must equal the flattened raw
    // occurrences exactly; no value may be removed or invented.
    let expected = raw_modifiers
        .iter()
        .flatten()
        .map(|modifier| modifier.as_str().to_string())
        .chain(raw_flags.iter().flatten().cloned())
        .collect::<Vec<_>>();
    assert_eq!(
        merged.len(),
        expected.len(),
        "{label}: no occurrences may be removed or invented: {merged:?}"
    );
    assert_eq!(
        &merged[..declaration.len()],
        declaration,
        "{label}: the declaration family must come first: {merged:?}"
    );
    for value in control_flow {
        assert!(
            merged[declaration.len()..].iter().any(|seen| seen == value),
            "{label}: missing control-flow flag {value}: {merged:?}"
        );
    }

    // The renderer exposes each semantic family through its distinct key.
    let rendered = render(&hir, Fidelity::Low);
    assert!(
        rendered.contains(&format!("mod:{}", declaration.join(","))),
        "{label}: rendered modifiers must mirror the typed field:\n{rendered}"
    );
    assert!(
        rendered.contains(&format!("ctl:{}", control_flow.join(","))),
        "{label}: compact summaries must preserve meaning:\n{rendered}"
    );
}

// ── RED-FLAG7: C# real pipeline ────────────────────────────────────────

/// `public static` method whose body contains `if` + `return`.
#[cfg(feature = "csharp")]
const CS_FLAG_PROBE: &str = r#"namespace Ordering;

public static class FlagProbe
{
    public static int Pick(int[] values)
    {
        if (values.Length == 0)
        {
            return 0;
        }

        return values.Length;
    }
}
"#;

#[cfg(feature = "csharp")]
fn compile_cs(source: &str) -> CompiledIR {
    use crate::ir::compiler::IRCompiler;
    use crate::ir::layers::csharp::CSharpLayer;
    use crate::ir::layers::patterns::CodePatternRecognizer;
    use crate::ir::patterns::CompressingPatternRecognizer;

    let language =
        crate::compression::language::safe_csharp_language().expect("csharp grammar enabled");
    let mut compiler = IRCompiler::new();
    compiler.add_language_layer(Box::new(CSharpLayer::new()));
    compiler.add_pattern_recognizer(Box::new(CodePatternRecognizer::new()));
    compiler.add_pattern_recognizer(Box::new(CompressingPatternRecognizer::new()));
    compiler
        .compile(
            source,
            "FlagProbe.cs",
            language,
            crate::queries::CS_QUERY,
            Fidelity::Low,
            None,
        )
        .expect("production compilation must succeed")
}

#[cfg(feature = "csharp")]
#[test]
fn red_flag7_csharp_pipeline_keeps_modifier_and_flow_flags() {
    let ir = compile_cs(CS_FLAG_PROBE);
    probe_single_method(&ir, &["STATIC"], &["IF", "RET"], "csharp");
}

/// The representative C# case: BEFORE the hierarchical encode the flat stream
/// the pattern recognizers and the wire consume still holds two separate ops.
#[cfg(feature = "csharp")]
#[test]
fn pattern_recognition_input_holds_two_separate_flag_ops_csharp() {
    let ir = compile_cs(CS_FLAG_PROBE);
    let hir = ir_to_hierarchical(&ir);
    let all = methods(&hir);
    assert_eq!(all.len(), 1, "exactly one method expected: {all:?}");
    let (mid, _, merged) = &all[0];

    let raw_modifiers = modifier_ops_for(&ir, mid);
    let raw_flags = flag_ops_for(&ir, mid);
    assert_eq!(
        raw_modifiers,
        vec![vec![DeclarationModifier::Static]],
        "the declaration family has its own typed op"
    );
    assert!(
        raw_flags[0].contains(&"IF".to_string()) && raw_flags[0].contains(&"RET".to_string()),
        "the control-flow family has its own typed op: {raw_flags:?}"
    );
    let expected = raw_modifiers
        .iter()
        .flatten()
        .map(|modifier| modifier.as_str().to_string())
        .chain(raw_flags.iter().flatten().cloned())
        .collect::<Vec<_>>();
    assert_eq!(
        merged, &expected,
        "declaration and every control occurrence must survive"
    );
}

// ── RED-FLAG8: TypeScript ──────────────────────────────────────────────

/// `static` method whose body contains `if` + `return` (a declaration
/// modifier that the TypeScript layer really produces, and a genuine
/// control-flow family).
#[cfg(feature = "typescript")]
const TS_FLAG_PROBE: &str = r#"export class FlagProbe {
    static pick(values: string[]): boolean {
        if (values.length === 0) {
            return false;
        }

        return true;
    }
}
"#;

#[cfg(feature = "typescript")]
fn compile_ts(source: &str) -> CompiledIR {
    use crate::ir::compiler::IRCompiler;
    use crate::ir::layers::patterns::CodePatternRecognizer;
    use crate::ir::layers::typescript::TypeScriptLayer;
    use crate::ir::patterns::CompressingPatternRecognizer;

    let language = crate::compression::language::safe_typescript_language()
        .expect("typescript grammar enabled");
    let mut compiler = IRCompiler::new();
    compiler.add_language_layer(Box::new(TypeScriptLayer::new()));
    compiler.add_pattern_recognizer(Box::new(CodePatternRecognizer::new()));
    compiler.add_pattern_recognizer(Box::new(CompressingPatternRecognizer::new()));
    compiler
        .compile(
            source,
            "FlagProbe.ts",
            language,
            crate::queries::TS_QUERY,
            Fidelity::Low,
            None,
        )
        .expect("production compilation must succeed")
}

#[cfg(feature = "typescript")]
#[test]
fn red_flag8_typescript_pipeline_keeps_modifier_and_flow_flags() {
    let ir = compile_ts(TS_FLAG_PROBE);
    probe_single_method(&ir, &["STATIC"], &["IF", "RET"], "typescript");
}

// ── RED-FLAG9: Java ────────────────────────────────────────────────────

/// `public static` method whose body contains `if` + `return` (the Java layer
/// emits EXPORT for `public` and STATIC for `static`).
#[cfg(feature = "java")]
const JAVA_FLAG_PROBE: &str = r#"public class FlagProbe {
    public static int pick(int[] values) {
        if (values.length == 0) {
            return 0;
        }

        return values.length;
    }
}
"#;

#[cfg(feature = "java")]
fn compile_java(source: &str) -> CompiledIR {
    use crate::ir::compiler::IRCompiler;
    use crate::ir::layers::java::JavaLayer;
    use crate::ir::layers::patterns::CodePatternRecognizer;
    use crate::ir::patterns::CompressingPatternRecognizer;

    let language =
        crate::compression::language::safe_java_language().expect("java grammar enabled");
    let mut compiler = IRCompiler::new();
    compiler.add_language_layer(Box::new(JavaLayer::new()));
    compiler.add_pattern_recognizer(Box::new(CodePatternRecognizer::new()));
    compiler.add_pattern_recognizer(Box::new(CompressingPatternRecognizer::new()));
    compiler
        .compile(
            source,
            "FlagProbe.java",
            language,
            crate::queries::JAVA_QUERY,
            Fidelity::Low,
            None,
        )
        .expect("production compilation must succeed")
}

#[cfg(feature = "java")]
#[test]
fn red_flag9_java_pipeline_keeps_modifier_and_flow_flags() {
    let ir = compile_java(JAVA_FLAG_PROBE);
    probe_single_method(&ir, &["EXPORT", "STATIC"], &["IF", "RET"], "java");
}

// ─ RED-FLAG10: Rust ──────────────────────────────────────────────────

/// `pub fn` inside an `impl` whose body contains `if` + `return` (the Rust
/// layer emits EXPORT for `pub`).
#[cfg(feature = "rust")]
const RS_FLAG_PROBE: &str = r#"pub struct FlagProbe;

impl FlagProbe {
    pub fn pick(values: &[i32]) -> usize {
        if values.is_empty() {
            return 0;
        }

        values.len()
    }
}
"#;

#[cfg(feature = "rust")]
fn compile_rs(source: &str) -> CompiledIR {
    use crate::ir::compiler::IRCompiler;
    use crate::ir::layers::patterns::CodePatternRecognizer;
    use crate::ir::layers::rust::RustLayer;
    use crate::ir::patterns::CompressingPatternRecognizer;

    let language =
        crate::compression::language::safe_rust_language().expect("rust grammar enabled");
    let mut compiler = IRCompiler::new();
    compiler.add_language_layer(Box::new(RustLayer::new()));
    compiler.add_pattern_recognizer(Box::new(CodePatternRecognizer::new()));
    compiler.add_pattern_recognizer(Box::new(CompressingPatternRecognizer::new()));
    compiler
        .compile(
            source,
            "flag_probe.rs",
            language,
            crate::queries::RS_QUERY,
            Fidelity::Low,
            None,
        )
        .expect("production compilation must succeed")
}

#[cfg(feature = "rust")]
#[test]
fn red_flag10_rust_pipeline_keeps_modifier_and_flow_flags() {
    let ir = compile_rs(RS_FLAG_PROBE);
    probe_single_method(&ir, &["EXPORT"], &["IF", "RET"], "rust");
}
