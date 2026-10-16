//! Framework-free consumer proof of the private source-bound boundary.
#![cfg(all(feature = "typescript", feature = "csharp"))]

use super::*;
use crate::compression::Fidelity;
use crate::ir::CompiledIR;
use crate::ir::compiler::IRCompiler;
use crate::layers::meta::semantic::SemanticRelation;

#[derive(Debug, PartialEq, Eq)]
enum ReadCallee<'a> {
    Free(&'a str),
    Member {
        receiver: &'a str,
        terminal: &'a str,
    },
    Unsupported(Option<&'a str>),
}

#[derive(Debug, PartialEq, Eq)]
enum ReadQualifier<'a> {
    Plain,
    Spread,
    Named(&'a str),
}

#[derive(Debug, PartialEq, Eq)]
struct ReadCall<'a> {
    invocation: &'a str,
    callee: ReadCallee<'a>,
    arguments: Vec<(&'a str, ReadQualifier<'a>)>,
    type_arguments: Vec<&'a str>,
    parent: Option<(&'a str, ParentSite)>,
    declaration: Option<(DeclarationContextKind, &'a str, Option<&'a str>)>,
}

// The consumer accepts only a validated view. Its sole text authority is
// the view's checked byte slicing; no grammar or language-specific branch.
fn read<'a>(view: &InvocationEvidenceView<'a>) -> Result<Vec<ReadCall<'a>>, EvidenceSliceError> {
    view.occurrences()
        .iter()
        .map(|call| {
            let callee = match call.callee() {
                WrittenCallee::Free { terminal } => ReadCallee::Free(view.slice(*terminal)?),
                WrittenCallee::Member { receiver, terminal } => ReadCallee::Member {
                    receiver: view.slice(*receiver)?,
                    terminal: view.slice(*terminal)?,
                },
                WrittenCallee::Unsupported { function } => {
                    ReadCallee::Unsupported(function.map(|span| view.slice(span)).transpose()?)
                }
            };
            let arguments = call
                .arguments()
                .iter()
                .map(|argument| {
                    let qualifier = match argument.qualifier() {
                        ArgumentQualifier::Plain => ReadQualifier::Plain,
                        ArgumentQualifier::Spread => ReadQualifier::Spread,
                        ArgumentQualifier::Named { name } => {
                            ReadQualifier::Named(view.slice(*name)?)
                        }
                    };
                    Ok((view.slice(argument.span())?, qualifier))
                })
                .collect::<Result<Vec<_>, EvidenceSliceError>>()?;
            let declaration = call
                .declaration_context()
                .map(|context| -> Result<_, EvidenceSliceError> {
                    Ok((
                        context.kind(),
                        view.slice(context.declaration())?,
                        context
                            .written_name()
                            .map(|span| view.slice(span))
                            .transpose()?,
                    ))
                })
                .transpose()?;
            Ok(ReadCall {
                invocation: view.slice(call.span())?,
                callee,
                arguments,
                type_arguments: call
                    .type_arguments()
                    .iter()
                    .map(|span| view.slice(*span))
                    .collect::<Result<_, _>>()?,
                parent: call
                    .parent()
                    .map(|parent| {
                        view.slice(parent.invocation())
                            .map(|text| (text, parent.site()))
                    })
                    .transpose()?,
                declaration,
            })
        })
        .collect()
}

fn compile(source: &str, csharp: bool) -> (CompiledIR, InvocationEvidenceSet) {
    let mut compiler = IRCompiler::new();
    let (language, query) = if csharp {
        compiler.add_language_layer(Box::new(crate::ir::layers::csharp::CSharpLayer::new()));
        (
            crate::compression::language::safe_csharp_language().unwrap(),
            crate::queries::CS_QUERY,
        )
    } else {
        compiler.add_language_layer(Box::new(
            crate::ir::layers::typescript::TypeScriptLayer::new(),
        ));
        (
            crate::compression::language::safe_typescript_language().unwrap(),
            crate::queries::TS_QUERY,
        )
    };
    let ir = compiler
        .compile_focused(
            source,
            "alias",
            Some("workspace/Example"),
            language,
            query,
            Fidelity::High,
            None,
            None,
        )
        .unwrap();
    (ir, compiler.invocation_evidence.take().unwrap())
}

fn assert_canonical(ir: &CompiledIR) {
    let mut calls = ir
        .instructions
        .iter()
        .filter_map(|op| op.call_parts())
        .map(|(_, name, argc, spread)| (name, argc, spread))
        .collect::<Vec<_>>();
    calls.sort();
    assert_eq!(calls, [("Load", 0, false), ("Save", 1, false)]);
    let mut evidence = crate::ir::semantic_projection::project_generic_facts(
        &ir.instructions,
        "workspace/Example",
    )
    .into_iter()
    .filter(|edge| edge.relation == SemanticRelation::Calls)
    .map(|edge| {
        let evidence = edge
            .call_evidence
            .expect("Calls keeps its canonical evidence");
        (evidence.explicit_arg_count, evidence.has_spread)
    })
    .collect::<Vec<_>>();
    evidence.sort();
    assert_eq!(evidence, [(0, false), (1, false)]);
}

#[test]
fn invocation_generic_consumer_cross_language_parity_and_canonical_preservation() {
    let sources = [
        "class C { run() { service.Save<User>(Load()); } }",
        "class C { void run() { service.Save<User>(Load()); } }",
    ];
    let mut results = Vec::new();
    let compiled = sources
        .iter()
        .enumerate()
        .map(|(index, source)| compile(source, index == 1))
        .collect::<Vec<_>>();
    for (csharp, source) in sources.iter().enumerate() {
        let (ir, set) = &compiled[csharp];
        assert_canonical(ir);
        let view = set.view("workspace/Example", source).unwrap();
        let calls = read(&view).unwrap();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].invocation, "service.Save<User>(Load())");
        assert_eq!(
            calls[0].callee,
            ReadCallee::Member {
                receiver: "service",
                terminal: "Save"
            }
        );
        assert_eq!(calls[0].arguments, [("Load()", ReadQualifier::Plain)]);
        assert_eq!(calls[0].type_arguments, ["User"]);
        assert_eq!(calls[0].parent, None);
        assert_eq!(calls[1].callee, ReadCallee::Free("Load"));
        assert!(calls[1].arguments.is_empty());
        assert!(calls[1].type_arguments.is_empty());
        assert_eq!(
            calls[1].parent,
            Some((calls[0].invocation, ParentSite::Argument { index: 0 }))
        );
        for call in &calls {
            let (kind, declaration, name) = call.declaration.unwrap();
            assert_eq!(kind, DeclarationContextKind::Callable);
            assert_eq!(name, Some("run"));
            assert_eq!(
                declaration,
                if csharp == 1 {
                    "void run() { service.Save<User>(Load()); }"
                } else {
                    "run() { service.Save<User>(Load()); }"
                }
            );
        }
        // Declaration syntax differs; all shared invocation fields are equal.
        results.push(
            calls
                .into_iter()
                .map(|mut call| {
                    call.declaration = None;
                    call
                })
                .collect::<Vec<_>>(),
        );
    }
    assert_eq!(results[0], results[1]);
}

#[test]
fn invocation_generic_consumer_requires_current_source_identity_and_hash() {
    for (source, csharp) in [
        ("class C { run() { service.Save<User>(Load()); } }", false),
        (
            "class C { void run() { service.Save<User>(Load()); } }",
            true,
        ),
    ] {
        let (_, old) = compile(source, csharp);
        assert_eq!(
            old.view("alias", source).err(),
            Some(EvidenceMismatch::SourceIdentityMismatch)
        );
        let edited = source.replace("Save", "Send"); // Same length cannot hide a stale span.
        assert_eq!(
            old.view("workspace/Example", &edited).err(),
            Some(EvidenceMismatch::SourceHashMismatch)
        );
        assert_eq!(
            old.view("workspace/Example", "").err(),
            Some(EvidenceMismatch::SourceHashMismatch)
        );
        let (_, fresh) = compile(&edited, csharp);
        let view = fresh.view("workspace/Example", &edited).unwrap();
        assert_eq!(
            read(&view).unwrap()[0].callee,
            ReadCallee::Member {
                receiver: "service",
                terminal: "Send"
            }
        );
        // A detached set remains a valid snapshot only for its original bytes.
        assert_eq!(
            read(&old.view("workspace/Example", source).unwrap()).unwrap()[0].callee,
            ReadCallee::Member {
                receiver: "service",
                terminal: "Save"
            }
        );
    }
}

#[test]
fn invocation_generic_consumer_slices_qualifiers_and_initializer_names() {
    for (source, csharp, qualifier, name) in [
        (
            "const café = service.Save<User>(...items);",
            false,
            ReadQualifier::Spread,
            "café",
        ),
        (
            "class C { object café = service.Save<User>(value: dto); }",
            true,
            ReadQualifier::Named("value"),
            "café",
        ),
    ] {
        let (_, set) = compile(source, csharp);
        let view = set.view("workspace/Example", source).unwrap();
        let calls = read(&view).unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].arguments[0].1, qualifier);
        assert_eq!(
            calls[0].arguments[0].0,
            if csharp { "value: dto" } else { "...items" }
        );
        let context = calls[0].declaration.unwrap();
        assert_eq!(
            context.0,
            if csharp {
                DeclarationContextKind::FieldInitializer
            } else {
                DeclarationContextKind::VariableInitializer
            }
        );
        assert_eq!(context.2, Some(name));
    }
}
