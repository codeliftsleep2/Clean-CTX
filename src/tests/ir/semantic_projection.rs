// src/tests/ir/semantic_projection.rs
//
// The language-agnostic IR fact → `SemanticEdge` projection at the compile
// boundary.
//
// Covers RED-CALL19's identity premise (arity never enters an entity name or
// entity identity), the method-registration shape, unresolved callee
// truthfulness, and the edge evidence that later becomes WorkspaceIndex
// occurrence identity.

use crate::ir::opcodes::CoreOp;
use crate::ir::semantic_projection::{
    BUILTIN_DOMAIN, BUILTIN_LAYER, CLASS_ENTITY_TYPE, INTERFACE_ENTITY_TYPE, METHOD_ENTITY_TYPE,
    TYPE_REF_ENTITY_TYPE, project_calls, project_constructor_parameter_types,
    project_generic_facts, project_inheritance, project_method_declarations,
};
use crate::layers::meta::semantic::SemanticRelation;

fn stream() -> Vec<CoreOp> {
    vec![
        CoreOp::DefClass("C1".into(), "Example".into()),
        CoreOp::DefMethod("C1".into(), "M1".into(), "Process".into()),
        CoreOp::Call("M1".into(), "Save".into(), 1, false),
        CoreOp::Call("M1".into(), "Save".into(), 2, false),
        CoreOp::Call("M1".into(), "OrderBy".into(), 2, false),
    ]
}

#[test]
fn method_declarations_register_one_occurrence_per_callable() {
    let edges = project_method_declarations(&stream(), "C:/repo/Example.cs");
    assert_eq!(edges.len(), 1);

    let edge = &edges[0];
    assert_eq!(edge.relation, SemanticRelation::Defines);
    assert_eq!(edge.layer, BUILTIN_LAYER);
    assert_eq!(edge.subject.domain, BUILTIN_DOMAIN);
    assert_eq!(edge.subject.entity_type, METHOD_ENTITY_TYPE);
    assert_eq!(edge.subject.name, "Process");
    assert_eq!(edge.subject.file.as_deref(), Some("C:/repo/Example.cs"));
    assert_eq!(
        edge.subject, edge.object,
        "a self-referential Defines edge is the registration carrier"
    );
    assert!(
        edge.call_evidence.is_none(),
        "a declaration carries no call evidence"
    );
}

#[test]
fn calls_project_method_level_subject_and_object_with_evidence() {
    let edges = project_calls(&stream(), "C:/repo/Example.cs");
    assert_eq!(edges.len(), 3);

    let save_one = &edges[0];
    assert_eq!(save_one.relation, SemanticRelation::Calls);
    assert_eq!(save_one.subject.domain, BUILTIN_DOMAIN);
    assert_eq!(save_one.subject.entity_type, METHOD_ENTITY_TYPE);
    assert_eq!(save_one.subject.name, "Process");
    assert_eq!(save_one.object.domain, BUILTIN_DOMAIN);
    assert_eq!(save_one.object.entity_type, METHOD_ENTITY_TYPE);
    assert_eq!(save_one.object.name, "Save");
    let save_one_evidence = save_one.call_evidence.expect("call evidence");
    assert_eq!(save_one_evidence.explicit_arg_count, 1);
    assert!(
        !save_one_evidence.has_spread,
        "a call with no expanding argument is exact written-arity evidence"
    );
    let save_two_evidence = edges[1].call_evidence.expect("call evidence");
    assert_eq!(
        save_two_evidence.explicit_arg_count, 2,
        "same caller/callee with a different arity is a different fact"
    );
    assert!(!save_two_evidence.has_spread);
}

#[test]
fn spread_call_projects_the_qualifier_into_evidence() {
    let instructions = vec![
        CoreOp::DefMethod("C1".into(), "M1".into(), "Process".into()),
        CoreOp::Call("M1".into(), "Save".into(), 1, false),
        CoreOp::Call("M1".into(), "Save".into(), 1, true),
    ];
    let edges = project_calls(&instructions, "C:/repo/Example.ts");
    assert_eq!(edges.len(), 2, "both facts must project");
    let exact = edges[0].call_evidence.expect("exact evidence");
    let spread = edges[1].call_evidence.expect("spread evidence");
    assert_eq!(
        exact.explicit_arg_count, spread.explicit_arg_count,
        "both calls write one argument node"
    );
    assert!(!exact.has_spread);
    assert!(spread.has_spread, "the expanding fact must be qualified");
    assert_ne!(
        exact, spread,
        "one written argument is not the same evidence in both cases"
    );
}

#[test]
fn arity_never_enters_an_entity_name_or_identity() {
    let edges = project_calls(&stream(), "C:/repo/Example.cs");
    for edge in &edges {
        for name in [edge.subject.name.as_str(), edge.object.name.as_str()] {
            assert!(
                !name.contains('/') && !name.contains('$') && !name.contains(":2"),
                "arity must never be encoded into a semantic name: {name}"
            );
        }
    }
    // Both arities describe the same caller and the same callee entity.
    assert_eq!(edges[0].subject.name, edges[1].subject.name);
    assert_eq!(edges[0].object.name, edges[1].object.name);
}

#[test]
fn unresolved_callee_still_projects_as_the_edge_object() {
    let edges = project_calls(&stream(), "C:/repo/Example.cs");
    assert!(
        edges.iter().any(|edge| edge.object.name == "OrderBy"),
        "a callee with no declaration in the file must still be recorded"
    );
    assert!(
        !edges.iter().any(|edge| edge.object.name.is_empty()),
        "an unnamed callee must never be projected"
    );
}

#[test]
fn unknown_caller_projects_no_edge() {
    let instructions = vec![CoreOp::Call("M404".into(), "Save".into(), 1, false)];
    assert!(
        project_calls(&instructions, "C:/repo/Example.cs").is_empty(),
        "a caller that does not exist in the compiled IR projects nothing \
         (validator rule E011 owns the diagnostic)"
    );
}

#[test]
fn duplicate_identical_facts_project_equal_edges() {
    let instructions = vec![
        CoreOp::DefMethod("C1".into(), "M1".into(), "Process".into()),
        CoreOp::Call("M1".into(), "Save".into(), 1, false),
        CoreOp::Call("M1".into(), "Save".into(), 1, false),
    ];
    let edges = project_calls(&instructions, "C:/repo/Example.cs");
    assert_eq!(edges.len(), 2);
    // Per-file extraction keeps both; the WorkspaceIndex collapses identical
    // occurrences of one asserting file (RED-CALL6).
    assert_eq!(edges[0].object.name, edges[1].object.name);
    assert_eq!(edges[0].call_evidence, edges[1].call_evidence);
}

#[test]
fn generic_projection_orders_declarations_before_calls() {
    let edges = project_generic_facts(&stream(), "C:/repo/Example.cs");
    assert_eq!(edges.len(), 4);
    assert_eq!(edges[0].relation, SemanticRelation::Defines);
    assert!(
        edges[1..]
            .iter()
            .all(|edge| edge.relation == SemanticRelation::Calls),
        "declaration occurrences must precede call facts"
    );
}

#[test]
fn canonical_inheritance_projects_typed_names_and_written_external_targets() {
    let instructions = vec![
        CoreOp::DefClass("C1".into(), "Base".into()),
        CoreOp::DefClass("C2".into(), "Worker".into()),
        CoreOp::DefInterface("I1".into(), "Runnable".into()),
        CoreOp::DefInterface("I2".into(), "AdvancedRunnable".into()),
        CoreOp::Extends("C2".into(), "C1".into()),
        CoreOp::Implements("C2".into(), "I1".into()),
        CoreOp::InterfaceExtends("I2".into(), "I1".into()),
        CoreOp::Extends("C1".into(), "ExternalBase".into()),
        CoreOp::BaseTypeRef("C2".into(), "UnknownBaseOrContract".into()),
    ];

    let edges = project_inheritance(&instructions, "C:/repo/Worker.cs");
    assert_eq!(edges.len(), 4);
    let facts: Vec<_> = edges
        .iter()
        .map(|edge| {
            (
                edge.relation,
                edge.subject.entity_type,
                edge.subject.name.as_str(),
                edge.object.entity_type,
                edge.object.name.as_str(),
            )
        })
        .collect();
    assert_eq!(
        facts,
        vec![
            (
                SemanticRelation::Extends,
                CLASS_ENTITY_TYPE,
                "Worker",
                CLASS_ENTITY_TYPE,
                "Base"
            ),
            (
                SemanticRelation::Implements,
                CLASS_ENTITY_TYPE,
                "Worker",
                INTERFACE_ENTITY_TYPE,
                "Runnable"
            ),
            (
                SemanticRelation::Extends,
                INTERFACE_ENTITY_TYPE,
                "AdvancedRunnable",
                INTERFACE_ENTITY_TYPE,
                "Runnable"
            ),
            (
                SemanticRelation::Extends,
                CLASS_ENTITY_TYPE,
                "Base",
                CLASS_ENTITY_TYPE,
                "ExternalBase"
            ),
        ]
    );
    assert!(edges.iter().all(|edge| {
        edge.layer == BUILTIN_LAYER
            && edge.call_evidence.is_none()
            && edge.subject.file.as_deref() == Some("C:/repo/Worker.cs")
            && edge.object.file == edge.subject.file
    }));
    assert!(edges.iter().all(|edge| {
        edge.object.name != "UnknownBaseOrContract" && edge.subject.name != "UnknownBaseOrContract"
    }));
}

#[test]
fn base_type_ref_projects_neither_extends_nor_implements() {
    let instructions = vec![
        CoreOp::DefClass("C1".into(), "AmbiguousOwner".into()),
        CoreOp::BaseTypeRef("C1".into(), "Shared".into()),
    ];

    let edges = project_inheritance(&instructions, "C:/repo/Ambiguous.cs");
    assert!(
        edges.is_empty(),
        "canonical uncertainty must not become an Extends or Implements semantic edge"
    );
}

#[test]
fn generic_projection_includes_structural_inheritance_after_existing_facts() {
    let mut instructions = stream();
    instructions.extend([
        CoreOp::DefClass("C2".into(), "Child".into()),
        CoreOp::Extends("C2".into(), "Example".into()),
    ]);
    let edges = project_generic_facts(&instructions, "C:/repo/Example.cs");
    assert_eq!(
        edges.last().map(|edge| edge.relation),
        Some(SemanticRelation::Extends)
    );
    assert_eq!(
        edges.last().map(|edge| edge.subject.name.as_str()),
        Some("Child")
    );
}

#[test]
fn constructor_parameter_projection_uses_canonical_ownership_and_type_refs() {
    let instructions = vec![
        CoreOp::DefClass("C1".into(), "BarController".into()),
        CoreOp::DefMethod("C1".into(), "M1".into(), "BarController".into()),
        CoreOp::Param(
            "M1".into(),
            "P1".into(),
            "IFooService".into(),
            "fooService".into(),
        ),
        CoreOp::DefMethod("C1".into(), "M2".into(), "Handle".into()),
        CoreOp::Param("M2".into(), "P2".into(), "IFooService".into(), "foo".into()),
    ];
    let edges = project_constructor_parameter_types(&instructions, "C:/repo/BarController.cs");
    assert_eq!(
        edges.len(),
        1,
        "ordinary method parameters must not project"
    );
    let edge = &edges[0];
    assert_eq!(edge.relation, SemanticRelation::HasConstructorParameterType);
    assert_eq!(edge.subject.domain, BUILTIN_DOMAIN);
    assert_eq!(edge.subject.entity_type, CLASS_ENTITY_TYPE);
    assert_eq!(edge.subject.name, "BarController");
    assert_eq!(edge.object.domain, BUILTIN_DOMAIN);
    assert_eq!(edge.object.entity_type, TYPE_REF_ENTITY_TYPE);
    assert_eq!(edge.object.name, "IFooService");
    assert_eq!(edge.layer, BUILTIN_LAYER);
    assert_eq!(
        edge.subject.file.as_deref(),
        Some("C:/repo/BarController.cs")
    );
    assert_eq!(edge.object.file, edge.subject.file);
}

#[cfg(feature = "csharp")]
#[test]
fn compiled_csharp_constructor_projects_the_written_parameter_type() {
    let source = r#"
public interface IFooService {}

public sealed class BarController
{
    public BarController(IFooService fooService) {}
    public void Handle(IFooService foo) {}
}
"#;
    let instructions = compile_csharp(source);
    let edges = project_constructor_parameter_types(&instructions, "C:/repo/BarController.cs");
    assert_eq!(edges.len(), 1, "only the constructor parameter projects");
    let edge = &edges[0];
    assert_eq!(edge.subject.name, "BarController");
    assert_eq!(edge.object.name, "IFooService");
    assert_eq!(edge.object.entity_type, TYPE_REF_ENTITY_TYPE);
    assert_eq!(edge.relation, SemanticRelation::HasConstructorParameterType);
}

// ── Blast radius: a corrupted declaration identity is not rendering-only ──
//
// `CoreOp::DefMethod` names are produced once, by `emit_method_ir`, and are
// consumed twice: `render_llm` renders them, and THIS projection turns them
// into `builtin / Method` registration occurrences (and into the subject of
// every `Calls` edge from that method) which reach WorkspaceIndex and
// `workspace_query`. A structural identity defect therefore reached the
// semantic surface as well — and because both consumers read the same
// upstream value, no projection change was needed to correct it. This pins
// the corrected names at the projection boundary with a REAL C# compilation.

/// Compile C# with the production compiler configuration, exactly as
/// `compile_file_ir_focused` does (the declaration identity under test is
/// produced by that path).
#[cfg(feature = "csharp")]
fn compile_csharp(source: &str) -> Vec<CoreOp> {
    use crate::compression::Fidelity;
    use crate::ir::compiler::IRCompiler;
    use crate::ir::layers::csharp::CSharpLayer;
    use crate::ir::layers::patterns::CodePatternRecognizer;
    use crate::ir::patterns::CompressingPatternRecognizer;
    use crate::queries::CS_QUERY;

    let language =
        crate::compression::language::safe_csharp_language().expect("csharp grammar enabled");
    let mut compiler = IRCompiler::new();
    compiler.add_language_layer(Box::new(CSharpLayer::new()));
    compiler.add_pattern_recognizer(Box::new(CodePatternRecognizer::new()));
    compiler.add_pattern_recognizer(Box::new(CompressingPatternRecognizer::new()));
    compiler
        .compile(
            source,
            "Signature.cs",
            language,
            CS_QUERY,
            Fidelity::High,
            None,
        )
        .expect("production compilation must succeed")
        .instructions
}

#[cfg(feature = "csharp")]
#[test]
fn projected_declaration_names_are_the_structural_method_identities() {
    const SOURCE: &str = r#"namespace Pairs;

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
    let instructions = compile_csharp(SOURCE);
    let edges = project_method_declarations(&instructions, "C:/repo/Signature.cs");
    let registered: Vec<&str> = edges
        .iter()
        .map(|edge| edge.subject.name.as_str())
        .collect();

    assert!(registered.contains(&"GetPair"), "{registered:?}");
    assert!(registered.contains(&"Tenth"), "{registered:?}");
    assert!(
        !registered.contains(&"static"),
        "a modifier must never become a registered entity name: {registered:?}"
    );
}

#[cfg(feature = "csharp")]
#[test]
fn projected_generic_identity_survives_to_the_call_subject() {
    const SOURCE: &str = r#"using System.Linq;
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
    let instructions = compile_csharp(SOURCE);
    let edges = project_method_declarations(&instructions, "C:/repo/Signature.cs");
    let registered: Vec<&str> = edges
        .iter()
        .map(|edge| edge.subject.name.as_str())
        .collect();
    assert!(
        registered.contains(&"Pair<TFirst, TSecond>"),
        "{registered:?}"
    );
    assert!(!registered.contains(&"TSecond>"), "{registered:?}");

    // The callable's own body calls `OrderByDescending`: the caller side of
    // that `Calls` edge is the corrected declaration identity.
    let edges = project_calls(&instructions, "C:/repo/Signature.cs");
    let calls: Vec<(&str, &str)> = edges
        .iter()
        .map(|edge| (edge.subject.name.as_str(), edge.object.name.as_str()))
        .collect();
    assert!(
        calls
            .iter()
            .any(|(caller, callee)| *caller == "Pair<TFirst, TSecond>"
                && *callee == "OrderByDescending"),
        "{calls:?}"
    );
}
