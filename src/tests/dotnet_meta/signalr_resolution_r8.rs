use crate::compression::Fidelity;
use crate::ir::compiler::IRCompiler;
use crate::ir::opcodes::{CoreOp, ExecutionContextKind};
use crate::layers::meta::semantic::{SemanticEdge, SemanticRelation};
use std::collections::BTreeSet;

fn compile(source: &str) -> (Vec<CoreOp>, Vec<SemanticEdge>) {
    let (language, query) =
        crate::compression::language::language_for_extension("cs").expect("C# language");
    let mut compiler = IRCompiler::new();
    compiler.add_language_layer(Box::new(crate::ir::layers::csharp::CSharpLayer::new()));
    let compiled = compiler
        .compile_focused(
            source,
            "r8-signalr-resolution",
            Some("C:/repo/Hubs.cs"),
            language,
            query,
            Fidelity::High,
            None,
            None,
        )
        .expect("C# compilation");
    (compiled.instructions, compiler.semantic_edges)
}

fn targets(edges: &[SemanticEdge]) -> BTreeSet<(String, String)> {
    edges
        .iter()
        .filter(|edge| {
            edge.subject.domain == "dotnet"
                && edge.subject.entity_type == "Hub"
                && edge.relation == SemanticRelation::HubMethodTargets
        })
        .map(|edge| (edge.subject.name.clone(), edge.object.name.clone()))
        .collect()
}

fn realtime_methods(instructions: &[CoreOp]) -> BTreeSet<String> {
    let realtime_ids = instructions
        .iter()
        .filter_map(|instruction| match instruction {
            CoreOp::ExecutionContext(method, ExecutionContextKind::Realtime) => {
                Some(method.as_str())
            }
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    instructions
        .iter()
        .filter_map(|instruction| match instruction {
            CoreOp::DefMethod(_, id, name) if realtime_ids.contains(id.as_str()) => {
                Some(name.clone())
            }
            _ => None,
        })
        .collect()
}

#[test]
fn r8_signalr_hubs_use_shared_resolution_for_arity_spelling_ancestry_and_partials() {
    let (instructions, edges) = compile(
        r#"
using Microsoft.AspNetCore.SignalR;
using SignalR = Microsoft.AspNetCore.SignalR;
using HubBase = Microsoft.AspNetCore.SignalR.Hub;

namespace Application
{
    public class ImportedHub : Hub { public async Task ImportedCall() {} }
    public class TypedHub : Hub<IClient> { public void TypedCall() {} }
    public class QualifiedHub : Microsoft.AspNetCore.SignalR.Hub { public void QualifiedCall() {} }
    public class GlobalHub : global::Microsoft.AspNetCore.SignalR.Hub<IClient> { public void GlobalCall() {} }
    public class NamespaceAliasHub : SignalR.Hub { public void NamespaceAliasCall() {} }
    public class TypeAliasHub : HubBase { public void TypeAliasCall() {} }
    public class IntermediateHub : Hub { }
    public class DerivedHub : IntermediateHub { public void DerivedCall() {} }
    public partial class PartialHub : Hub<IClient> { }
    public partial class PartialHub { public void PartialCall() {} }
    public class EmptyHub : Hub { }
    public class WrongArityHub : Hub<IClient, IOther> { public void BadArityCall() {} }

    namespace Shadow
    {
        public class Hub { }
        public class FakeHub : Hub { public void ShadowCall() {} }
    }
}
"#,
    );

    assert_eq!(
        targets(&edges),
        BTreeSet::from([
            ("DerivedHub".into(), "DerivedCall".into()),
            ("GlobalHub".into(), "GlobalCall".into()),
            ("ImportedHub".into(), "ImportedCall".into()),
            ("NamespaceAliasHub".into(), "NamespaceAliasCall".into()),
            ("PartialHub".into(), "PartialCall".into()),
            ("QualifiedHub".into(), "QualifiedCall".into()),
            ("TypeAliasHub".into(), "TypeAliasCall".into()),
            ("TypedHub".into(), "TypedCall".into()),
        ])
    );
    assert_eq!(
        realtime_methods(&instructions),
        BTreeSet::from([
            "DerivedCall".into(),
            "GlobalCall".into(),
            "ImportedCall".into(),
            "NamespaceAliasCall".into(),
            "PartialCall".into(),
            "QualifiedCall".into(),
            "TypeAliasCall".into(),
            "TypedCall".into(),
        ]),
        "resolved Hub authority must drive canonical realtime contexts"
    );
    assert!(
        instructions.iter().any(|instruction| {
            matches!(
                instruction,
                CoreOp::ExecutionContext(method, ExecutionContextKind::Async)
                    if instructions.iter().any(|candidate| matches!(
                        candidate,
                        CoreOp::DefMethod(_, id, name)
                            if id == method && name == "ImportedCall"
                    ))
            )
        }),
        "independent async context must remain present"
    );
}

#[test]
fn r8_signalr_competing_imports_fail_closed() {
    let (instructions, edges) = compile(
        r#"
using Microsoft.AspNetCore.SignalR;
using Other;
namespace Other { public class Hub { } }
namespace Application
{
    public class AmbiguousHub : Hub { public void BadCall() {} }
}
"#,
    );

    assert!(
        targets(&edges).is_empty(),
        "competing source and known-external imports cannot authorize hub methods"
    );
    assert!(
        realtime_methods(&instructions).is_empty(),
        "ambiguous Hub identity cannot authorize realtime contexts"
    );
}
