use crate::compression::Fidelity;
use crate::ir::compiler::IRCompiler;
use crate::ir::csharp_resolution::{
    CSharpResolution, CSharpResolutionReason, CSharpTypeIdentity, CSharpUsingDirectiveKind,
    resolve_known_external_type,
};
use crate::ir::opcodes::CoreOp;

fn compile(source: &str) -> (crate::ir::compiler::CompiledIR, IRCompiler) {
    let (language, query) =
        crate::compression::language::language_for_extension("cs").expect("C# language support");
    let mut compiler = IRCompiler::new();
    let compiled = compiler
        .compile_focused(
            source,
            "r0-neutral-resolution",
            Some("C:/workspace/NeutralReferences.cs"),
            language,
            query,
            Fidelity::High,
            None,
            None,
        )
        .expect("C# compilation");
    (compiled, compiler)
}

#[test]
fn r0_csharp_resolution_exposes_neutral_identity_and_explicit_dispositions() {
    let source = r#"
namespace Consumer;

public class Qualified : Example.Namespace.BaseType {}
public class GlobalQualified : global::Example.Namespace.BaseType {}
public class DeferredSimple : BaseType {}
public class DeferredAlias : ExampleAlias::BaseType {}
public class MissingQualified : Missing.Namespace.BaseType {}
"#;
    let (compiled, compiler) = compile(source);

    let written: Vec<_> = compiled
        .instructions
        .iter()
        .filter_map(|instruction| match instruction {
            CoreOp::BaseTypeRef(_, written) => Some(written.as_str()),
            _ => None,
        })
        .collect();
    assert!(written.contains(&"Example.Namespace.BaseType"));
    assert!(written.contains(&"global::Example.Namespace.BaseType"));
    assert!(written.contains(&"BaseType"));
    assert!(written.contains(&"ExampleAlias::BaseType"));
    assert!(written.contains(&"Missing.Namespace.BaseType"));

    let authority = [CSharpTypeIdentity::known_external(
        "Example.Namespace.BaseType",
    )];
    let projection = written
        .iter()
        .map(|reference| {
            (
                *reference,
                resolve_known_external_type(reference, &authority),
            )
        })
        .collect::<Vec<_>>();
    let projection_debug = format!("{projection:#?}");
    assert_eq!(
        projection_debug
            .matches("qualified_name: \"Example.Namespace.BaseType\"")
            .count(),
        2,
        "full and global qualification must converge on one neutral identity: {projection_debug}"
    );
    assert_eq!(
        projection
            .iter()
            .filter(|(_, result)| matches!(result, CSharpResolution::Resolved { .. }))
            .count(),
        2,
        "both explicitly qualified spellings must resolve: {projection_debug}"
    );
    assert!(
        projection.iter().any(|(written, result)| {
            *written == "ExampleAlias::BaseType"
                && matches!(
                    result,
                    CSharpResolution::Unsupported(
                        CSharpResolutionReason::UnsupportedAliasQualifiedName
                    )
                )
        }),
        "deferred alias syntax must be explicitly unsupported: {projection_debug}"
    );
    assert!(
        projection.iter().any(|(written, result)| {
            *written == "Missing.Namespace.BaseType"
                && matches!(
                    result,
                    CSharpResolution::Unresolved(CSharpResolutionReason::MissingAuthority)
                )
        }),
        "missing authority must remain distinct from unsupported syntax: {projection_debug}"
    );

    assert!(
        compiler.semantic_edges.iter().all(|edge| {
            !matches!(
                edge.subject.entity_type,
                "DbContext" | "Controller" | "MapperProfile" | "Hub" | "TestClass"
            )
        }),
        "the neutral resolver must not manufacture a framework role"
    );
}

#[test]
fn r0_csharp_resolution_keeps_failure_kinds_distinct() {
    let authority = [
        CSharpTypeIdentity::known_external("Example.Namespace.BaseType"),
        CSharpTypeIdentity::known_external("Duplicate.Namespace.BaseType"),
        CSharpTypeIdentity::known_external("Duplicate.Namespace.BaseType"),
    ];

    assert_eq!(
        resolve_known_external_type("BaseType", &authority),
        CSharpResolution::Unsupported(CSharpResolutionReason::UnsupportedSimpleName)
    );
    assert_eq!(
        resolve_known_external_type("ExampleAlias::BaseType", &authority),
        CSharpResolution::Unsupported(CSharpResolutionReason::UnsupportedAliasQualifiedName)
    );
    assert_eq!(
        resolve_known_external_type("Example..BaseType", &authority),
        CSharpResolution::Unsupported(CSharpResolutionReason::MalformedTypeReference)
    );
    assert_eq!(
        resolve_known_external_type("Missing.Namespace.BaseType", &authority),
        CSharpResolution::Unresolved(CSharpResolutionReason::MissingAuthority)
    );
    assert_eq!(
        resolve_known_external_type("Duplicate.Namespace.BaseType", &authority),
        CSharpResolution::Ambiguous(CSharpResolutionReason::CompetingCandidates)
    );
}

#[test]
fn r0_csharp_compilation_publishes_neutral_syntax_facts() {
    let source = r#"
using Example.Namespace;
using Alias = Example.Namespace;
using static Example.Helpers;

namespace Consumer;

[Example.Marker]
public class Qualified : Example.Namespace.BaseType {}
public interface Contract {}
public struct Value {}
public enum Choice { One }
public record Payload();
"#;
    let (_, compiler) = compile(source);
    let facts = &compiler.csharp_syntax_facts;

    assert_eq!(facts.declarations.len(), 5, "{facts:#?}");
    assert!(
        facts
            .declarations
            .iter()
            .any(|fact| fact.name == "Qualified")
    );
    assert!(
        facts
            .declarations
            .iter()
            .any(|fact| fact.name == "Contract")
    );
    assert!(facts.declarations.iter().any(|fact| fact.name == "Value"));
    assert!(facts.declarations.iter().any(|fact| fact.name == "Choice"));
    assert!(facts.declarations.iter().any(|fact| fact.name == "Payload"));

    assert!(facts.type_references.iter().any(|fact| {
        fact.written == "Example.Namespace.BaseType"
            && source[fact.span.start_byte..fact.span.end_byte] == fact.written
    }));
    assert!(facts.attributes.iter().any(|fact| {
        fact.written == "Example.Marker"
            && source[fact.span.start_byte..fact.span.end_byte] == fact.written
    }));
    assert!(facts.using_directives.iter().any(|fact| {
        fact.kind == CSharpUsingDirectiveKind::Namespace
            && fact.written == "using Example.Namespace;"
    }));
    assert!(facts.using_directives.iter().any(|fact| {
        fact.kind == CSharpUsingDirectiveKind::Alias
            && fact.written == "using Alias = Example.Namespace;"
    }));
    assert!(facts.using_directives.iter().any(|fact| {
        fact.kind == CSharpUsingDirectiveKind::Static
            && fact.written == "using static Example.Helpers;"
    }));
}
