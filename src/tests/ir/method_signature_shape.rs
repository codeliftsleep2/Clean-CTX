// src/tests/ir/method_signature_shape.rs
//
// IR-level method-identity regressions (RED-SIG1..RED-SIG12) through the
// production compiler configuration.
//
// The IR is the canonical structured representation for a declaration's
// identity: `CoreOp::DefMethod` carries the name, `CoreOp::Param` the formal
// parameters, and `CoreOp::Return` the return type. Every downstream consumer
// — the hierarchical wire form, `render_llm` and its overload grouping,
// `UnitTable`, and the semantic projection into `WorkspaceIndex` — reads those
// values, so a corrupted identity here is projected silently into all of them.
// The assertions below therefore pin the STRUCTURED values, not only the
// rendered line.
//
// Child module: `signature_cross_language.rs` probes the other languages
// through the same shared boundary (the compaction/pipeline pair is shared
// across C#, TypeScript, Java, and Rust), so a shared regression cannot hide
// behind the C# fixtures.
#![cfg(feature = "csharp")]

use crate::compression::Fidelity;
use crate::ir::compiler::{CompiledIR, IRCompiler};
use crate::ir::hierarchical::ir_to_hierarchical;
use crate::ir::layers::csharp::CSharpLayer;
use crate::ir::layers::patterns::CodePatternRecognizer;
use crate::ir::opcodes::CoreOp;
use crate::ir::patterns::CompressingPatternRecognizer;
use crate::ir::render_llm::render_hierarchical_for_llm;
use crate::queries::CS_QUERY;
use std::collections::HashMap;

/// Compile C# with the production compiler configuration (the C# language
/// layer plus both production pattern recognizers), exactly as
/// `compile_file_ir_focused` does.
fn compile_cs(source: &str, fidelity: Fidelity) -> CompiledIR {
    let language =
        crate::compression::language::safe_csharp_language().expect("csharp grammar enabled");
    let mut compiler = IRCompiler::new();
    compiler.add_language_layer(Box::new(CSharpLayer::new()));
    compiler.add_pattern_recognizer(Box::new(CodePatternRecognizer::new()));
    compiler.add_pattern_recognizer(Box::new(CompressingPatternRecognizer::new()));
    compiler
        .compile(source, "Signature.cs", language, CS_QUERY, fidelity, None)
        .expect("production compilation must succeed")
}

/// Static class with `no` base list whose member body contains a ternary `:`
/// and a fluent call — the shape that fabricated an `extends` edge.
const CS_STATIC_CLASS_TERNARY_BODY: &str = r#"namespace Ordering;

public static class PairingHelpers
{
    public static int Pick(int[] values, bool ascending)
    {
        return values.Length == 0 ? 0 : values.OrderByDescending(v => v).First();
    }
}
"#;

/// Case A — the reported extension-method shape: two method type parameters,
/// a nested generic parameter type whose own argument list contains a comma,
/// and a multi-line parameter list.
const CS_EXTENSION_TWO_TYPE_PARAMS: &str = r#"using System.Linq;
using System.Linq.Expressions;

namespace Ordering;

public static class QueryablePairExtensions
{
    public static IOrderedQueryable<TFirst> Pair<TFirst, TSecond>(
        this IQueryable<TFirst> source,
        Expression<Func<TFirst, TSecond>> keySelector,
        ListSortDirection direction)
    {
        return source.OrderByDescending(keySelector);
    }
}
"#;

/// RED-SIG3 — the same generic shape WITHOUT extension-receiver syntax, as an
/// ordinary static method and as an instance method.
const CS_ORDINARY_GENERIC: &str = r#"namespace Pairs;

public class GenericPairer
{
    public static TResult Pair<TFirst, TSecond>(TFirst first, TSecond second)
    {
        return default;
    }

    public TValue Map<TKey, TValue>(TKey key)
    {
        return default;
    }
}
"#;

/// RED-SIG2 — three type parameters, so the fix cannot be hard-coded for two.
const CS_THREE_TYPE_PARAMS: &str = r#"namespace Pairs;

public static class TripleBuilder
{
    public static Mapper<A> Build<A, B, C>(A a, B b, C c)
    {
        return default;
    }
}
"#;

/// RED-SIG5/6/7 — two unrelated tuple-returning methods in one class.
const CS_TUPLES: &str = r#"namespace Pairs;

public static class TupleFactory
{
    public static (int alpha, int beta) GetPair(int[] values)
    {
        return (values[0], values[1]);
    }

    public static (string name, int count) Tenth(string[] names)
    {
        return (names[0], names.Length);
    }
}
"#;

/// RED-SIG8 — the same shape with an UNNAMED tuple return type.
const CS_UNNAMED_TUPLE: &str = r#"namespace Pairs;

public static class UnnamedFactory
{
    public static (int, int) GetPair(int[] values)
    {
        return (values[0], values[1]);
    }
}
"#;

/// RED-SIG9 — the tuple return on an INSTANCE method: `static` was incidental,
/// never causal.
const CS_INSTANCE_TUPLE: &str = r#"namespace Pairs;

public class InstanceFactory
{
    public (int alpha, int beta) GetPair(int[] values)
    {
        return (values[0], values[1]);
    }
}
"#;

/// RED-SIG11 — the single-type-parameter control shape.
const CS_SINGLE_TYPE_PARAM: &str = r#"using System.Linq;

namespace Ordering;

public static class SingleTypeParamExtensions
{
    public static ToSet<T> ToSet<T>(this IEnumerable<T> source)
    {
        return default;
    }
}
"#;

/// RED-SIG12 — ordinary non-generic, non-tuple methods.
const CS_ORDINARY: &str = r#"namespace Pairs;

public class Ordinary
{
    public int Add(int a, int b)
    {
        return a + b;
    }

    public void Reset()
    {
    }

    public string Describe(string name)
    {
        return name;
    }
}
"#;

/// The structured facts of one declared method: identity, formal parameters,
/// return type, and flags.
#[derive(Debug, Clone, PartialEq, Eq)]
struct MethodFacts {
    name: String,
    params: Vec<String>,
    return_type: String,
    flags: Vec<String>,
}

/// Every method's structured facts, in declaration order.
fn methods(ir: &CompiledIR) -> Vec<MethodFacts> {
    let mut order: Vec<String> = Vec::new();
    let mut facts: HashMap<String, MethodFacts> = HashMap::new();
    for op in &ir.instructions {
        match op {
            CoreOp::DefMethod(_cid, mid, name) => {
                order.push(mid.clone());
                facts.insert(
                    mid.clone(),
                    MethodFacts {
                        name: name.clone(),
                        params: Vec::new(),
                        return_type: String::new(),
                        flags: Vec::new(),
                    },
                );
            }
            CoreOp::Param(mid, _pid, _ty, param_name) => {
                if let Some(facts) = facts.get_mut(mid) {
                    facts.params.push(param_name.clone());
                }
            }
            CoreOp::Return(mid, ty) => {
                if let Some(facts) = facts.get_mut(mid) {
                    facts.return_type = ty.clone();
                }
            }
            CoreOp::Flags(mid, flags) => {
                if let Some(facts) = facts.get_mut(mid) {
                    // `CoreOp::Flags` carries TWO producers writing separate ops
                    // for the same method id: the language layer's declaration
                    // modifiers (STATIC / ASYNC / PRIVATE / …) and the
                    // accumulated control-flow flags (IF / LOOP / RET / THROW)
                    // flushed at the end of the declaration. A faithful view of
                    // the instruction stream is therefore the union of that
                    // method's flag ops; keeping only the last one would
                    // silently drop the other producer's entire family.
                    for flag in flags.iter() {
                        if !facts.flags.iter().any(|existing| existing == flag) {
                            facts.flags.push(flag.to_string());
                        }
                    }
                }
            }
            CoreOp::MethodModifiers(mid, modifiers) => {
                if let Some(facts) = facts.get_mut(mid) {
                    for modifier in modifiers {
                        let name = modifier.as_str().to_string();
                        if !facts.flags.contains(&name) {
                            facts.flags.push(name);
                        }
                    }
                }
            }
            _ => {}
        }
    }
    order
        .iter()
        .filter_map(|mid| facts.get(mid).cloned())
        .collect()
}

/// The declared method names of a compilation.
fn method_names(ir: &CompiledIR) -> Vec<String> {
    methods(ir).into_iter().map(|m| m.name).collect()
}

/// One named method's structured facts.
fn method(ir: &CompiledIR, name: &str) -> MethodFacts {
    let all = methods(ir);
    all.iter()
        .find(|m| m.name == name)
        .cloned()
        .unwrap_or_else(|| {
            panic!(
                "no method named {name}; declared: {:?}",
                all.iter().map(|m| m.name.as_str()).collect::<Vec<_>>()
            )
        })
}

/// The rendered skeleton for a compilation.
fn render(ir: &CompiledIR, fidelity: Fidelity) -> String {
    render_hierarchical_for_llm(&ir_to_hierarchical(ir), fidelity)
}

// ── RED-SIG1: two type parameters preserve method identity ─────────

#[test]
fn red_sig1_two_type_parameters_preserve_method_identity() {
    // Per-fidelity expectations are the established compaction contracts:
    // Low carries the bare identifier only (the same contract a
    // single-type-parameter method already had), Medium/High keep the type
    // parameters. Under the defect EVERY fidelity gave `TSecond>`.
    for (fidelity, expected) in [
        (Fidelity::Low, "Pair"),
        (Fidelity::Medium, "Pair<TFirst, TSecond>"),
        (Fidelity::High, "Pair<TFirst, TSecond>"),
    ] {
        let ir = compile_cs(CS_EXTENSION_TWO_TYPE_PARAMS, fidelity);
        let names = method_names(&ir);
        assert!(
            names.iter().any(|n| n == expected),
            "{fidelity:?}: expected {expected}, declared {names:?}"
        );
        assert!(
            !names.iter().any(|n| n == "TSecond>"),
            "{fidelity:?}: the generic name must never degrade to its own type parameter: {names:?}"
        );
    }
}

/// The parameter list is part of the same declaration shape: a comma inside a
/// generic argument list (`Expression<Func<TFirst, TSecond>>`) declares ONE
/// parameter, not two.
#[test]
fn red_sig1_parameter_list_is_not_inflated_by_nested_generic_commas() {
    for fidelity in [Fidelity::Medium, Fidelity::High] {
        let ir = compile_cs(CS_EXTENSION_TWO_TYPE_PARAMS, fidelity);
        let facts = method(&ir, "Pair<TFirst, TSecond>");
        assert_eq!(
            facts.params.len(),
            3,
            "{fidelity:?}: the declaration writes three parameters, got {:?}",
            facts.params
        );
        assert!(facts.params[0].contains("source"), "{:?}", facts.params);
        assert!(
            facts.params[1].contains("keySelector") && facts.params[1].contains("TSecond"),
            "{:?}",
            facts.params
        );
        assert!(facts.params[2].contains("direction"), "{:?}", facts.params);
    }
}

// ── RED-SIG2: three type parameters ───────────────────────────────

#[test]
fn red_sig2_three_type_parameters_are_structural() {
    for (fidelity, expected) in [
        (Fidelity::Low, "Build"),
        (Fidelity::Medium, "Build<A, B, C>"),
        (Fidelity::High, "Build<A, B, C>"),
    ] {
        let ir = compile_cs(CS_THREE_TYPE_PARAMS, fidelity);
        assert_eq!(
            method(&ir, expected).params.len(),
            3,
            "{fidelity:?}: A, B and C are three formal parameters"
        );
    }
}

// ── RED-SIG3: non-extension generic methods ───────────────────────

#[test]
fn red_sig3_non_extension_generic_methods_keep_their_identity() {
    for (fidelity, pair, map) in [
        (Fidelity::Low, "Pair", "Map"),
        (
            Fidelity::Medium,
            "Pair<TFirst, TSecond>",
            "Map<TKey, TValue>",
        ),
        (Fidelity::High, "Pair<TFirst, TSecond>", "Map<TKey, TValue>"),
    ] {
        let ir = compile_cs(CS_ORDINARY_GENERIC, fidelity);
        let names = method_names(&ir);
        assert!(names.iter().any(|n| n == pair), "{fidelity:?}: {names:?}");
        assert!(names.iter().any(|n| n == map), "{fidelity:?}: {names:?}");
        assert!(
            !names.iter().any(|n| n == "TSecond>" || n == "TValue>"),
            "{fidelity:?}: receiver syntax is not the trigger: {names:?}"
        );
    }
}

// ── RED-SIG4: extension generic method (the original failure shape) ──

#[test]
fn red_sig4_extension_generic_method_preserves_every_field() {
    let ir = compile_cs(CS_EXTENSION_TWO_TYPE_PARAMS, Fidelity::High);
    let facts = method(&ir, "Pair<TFirst, TSecond>");
    assert_eq!(facts.params.len(), 3);
    assert_eq!(
        facts.return_type, "IOrderedQueryable<TFirst>",
        "the declared return type, not a fabricated one"
    );
    assert!(
        facts.flags.iter().any(|flag| flag == "STATIC"),
        "the method is declared static: {:?}",
        facts.flags
    );

    let out = render(&ir, Fidelity::High);
    assert!(out.contains("M Pair<TFirst, TSecond>"), "{out}");
    assert!(!out.contains("M TSecond>"), "{out}");
}

// ── RED-SIG5..RED-SIG9: tuple return types ────────────────────────

#[test]
fn red_sig5_named_tuple_return_preserves_every_field() {
    // HIGH: the declaration's own parameter list and its tuple return type.
    let ir = compile_cs(CS_TUPLES, Fidelity::High);
    let facts = method(&ir, "GetPair");
    assert_eq!(facts.params, vec!["int[] values".to_string()]);
    assert_eq!(facts.return_type, "(int alpha, int beta)");

    // MEDIUM: the Medium label contract drops declared return types, so the
    // tuple members must NOT reappear as the name or as parameters.
    let ir = compile_cs(CS_TUPLES, Fidelity::Medium);
    let facts = method(&ir, "GetPair");
    assert_eq!(facts.params, vec!["int[] values".to_string()]);
    assert_eq!(facts.return_type, "$v");

    // LOW: bare parameter names only.
    let ir = compile_cs(CS_TUPLES, Fidelity::Low);
    assert_eq!(method(&ir, "GetPair").params, vec!["values".to_string()]);

    // The defect's field shift, at every fidelity: the tuple members are never
    // a name and the modifiers are never a name.
    for fidelity in [Fidelity::Low, Fidelity::Medium, Fidelity::High] {
        let names = method_names(&compile_cs(CS_TUPLES, fidelity));
        assert!(
            !names.iter().any(|n| n == "static" || n == "public"),
            "{fidelity:?}: a modifier is not a method identity: {names:?}"
        );
    }
}

#[test]
fn red_sig6_second_tuple_method_stays_distinct() {
    for fidelity in [Fidelity::Low, Fidelity::Medium, Fidelity::High] {
        let names = method_names(&compile_cs(CS_TUPLES, fidelity));
        assert!(
            names.iter().any(|n| n == "GetPair"),
            "{fidelity:?}: {names:?}"
        );
        assert!(
            names.iter().any(|n| n == "Tenth"),
            "{fidelity:?}: {names:?}"
        );
    }

    let ir = compile_cs(CS_TUPLES, Fidelity::High);
    assert_eq!(
        method(&ir, "Tenth").params,
        vec!["string[] names".to_string()]
    );
    assert_eq!(method(&ir, "Tenth").return_type, "(string name, int count)");
    assert_ne!(
        method(&ir, "GetPair").return_type,
        method(&ir, "Tenth").return_type
    );
}

#[test]
fn red_sig7_no_fabricated_overload_group_for_distinct_methods() {
    for fidelity in [Fidelity::Medium, Fidelity::High] {
        let out = render(&compile_cs(CS_TUPLES, fidelity), fidelity);
        assert!(out.contains("M GetPair"), "{fidelity:?}:\n{out}");
        assert!(out.contains("M Tenth"), "{fidelity:?}:\n{out}");
        assert!(
            !out.contains("static(+2)"),
            "{fidelity:?}: distinct methods must not group as overloads of a fabricated identity:\n{out}"
        );
        assert!(!out.contains("M static"), "{fidelity:?}:\n{out}");
    }
}

#[test]
fn red_sig8_unnamed_tuple_return_preserves_every_field() {
    let ir = compile_cs(CS_UNNAMED_TUPLE, Fidelity::High);
    let facts = method(&ir, "GetPair");
    assert_eq!(facts.params, vec!["int[] values".to_string()]);
    assert_eq!(facts.return_type, "(int, int)");
    assert!(
        !method_names(&ir).iter().any(|n| n == "static"),
        "tuple element names are not the trigger"
    );
}

#[test]
fn red_sig9_instance_tuple_method_proves_static_is_incidental() {
    let ir = compile_cs(CS_INSTANCE_TUPLE, Fidelity::High);
    let facts = method(&ir, "GetPair");
    assert_eq!(facts.params, vec!["int[] values".to_string()]);
    assert_eq!(facts.return_type, "(int alpha, int beta)");
    assert!(
        !method_names(&ir)
            .iter()
            .any(|n| n == "public" || n == "static"),
        "{:?}",
        method_names(&ir)
    );
}

// ── RED-SIG10: no false `extends` from a member body ──────────────

#[test]
fn red_sig10_static_class_has_no_false_extends_edge() {
    for fidelity in [Fidelity::Low, Fidelity::Medium, Fidelity::High] {
        let ir = compile_cs(CS_STATIC_CLASS_TERNARY_BODY, fidelity);
        let parents: Vec<&str> = ir
            .instructions
            .iter()
            .filter_map(|op| match op {
                CoreOp::Extends(_child, parent) => Some(parent.as_str()),
                _ => None,
            })
            .collect();
        assert!(
            parents.is_empty(),
            "{fidelity:?}: a class with no base list must have no extends edge, got {parents:?}"
        );

        let out = render(&ir, fidelity);
        assert!(
            !out.lines().any(|line| line.trim_start().starts_with("X ")),
            "{fidelity:?}: a member body expression must never render as a base type:\n{out}"
        );
        assert!(
            !out.contains("OrderByDescending"),
            "{fidelity:?}: the skeleton must not carry a method-body call:\n{out}"
        );
    }
}

/// The class head is still read correctly: a real base list is preserved.
#[test]
fn red_sig10_real_base_list_is_still_detected() {
    const WITH_BASE: &str = r#"namespace Pairs;

public class Derived : BaseType
{
    public int Value()
    {
        return 1;
    }
}
"#;
    let ir = compile_cs(WITH_BASE, Fidelity::Medium);
    let parents: Vec<&str> = ir
        .instructions
        .iter()
        .filter_map(|op| match op {
            CoreOp::Extends(_child, parent) => Some(parent.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(
        parents,
        vec!["BaseType"],
        "a declared base type is still read"
    );
}

// ── RED-SIG11/RED-SIG12: nonregressions ──────────────────────────

#[test]
fn red_sig11_single_type_parameter_shape_is_unchanged() {
    for (fidelity, expected) in [
        (Fidelity::Low, "ToSet"),
        (Fidelity::Medium, "ToSet<T>"),
        (Fidelity::High, "ToSet<T>"),
    ] {
        let ir = compile_cs(CS_SINGLE_TYPE_PARAM, fidelity);
        let facts = method(&ir, expected);
        assert_eq!(facts.params.len(), 1, "{fidelity:?}: {:?}", facts.params);
    }

    let ir = compile_cs(CS_SINGLE_TYPE_PARAM, Fidelity::High);
    assert_eq!(method(&ir, "ToSet<T>").return_type, "ToSet<T>");
}

#[test]
fn red_sig12_ordinary_methods_are_unchanged() {
    for fidelity in [Fidelity::Low, Fidelity::Medium, Fidelity::High] {
        let names = method_names(&compile_cs(CS_ORDINARY, fidelity));
        for expected in ["Add", "Reset", "Describe"] {
            assert!(
                names.iter().any(|n| n == expected),
                "{fidelity:?}: {names:?}"
            );
        }
    }

    let ir = compile_cs(CS_ORDINARY, Fidelity::Medium);
    assert_eq!(
        method(&ir, "Add").params,
        vec!["int a".to_string(), "int b".to_string()]
    );
    assert!(method(&ir, "Reset").params.is_empty());

    // High fidelity: the declared return type is the one the declaration wrote
    // (this is the field a torn tuple used to occupy with a method name).
    let ir = compile_cs(CS_ORDINARY, Fidelity::High);
    assert_eq!(method(&ir, "Add").return_type, "int");
    assert_eq!(method(&ir, "Reset").return_type, "void");
    assert_eq!(method(&ir, "Describe").return_type, "string");
}

#[path = "signature_cross_language.rs"]
mod signature_cross_language;
