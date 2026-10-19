use crate::compression::Fidelity;
use crate::ir::compiler::IRCompiler;
use crate::layers::meta::semantic::{SemanticEdge, SemanticRelation};
use std::collections::BTreeSet;

fn compile_edges(source: &str) -> Vec<SemanticEdge> {
    let (language, query) =
        crate::compression::language::language_for_extension("cs").expect("C# language");
    let mut compiler = IRCompiler::new();
    compiler
        .compile_focused(
            source,
            "r9-aspnet-resolution",
            Some("C:/repo/Controllers.cs"),
            language,
            query,
            Fidelity::High,
            None,
            None,
        )
        .expect("C# compilation");
    compiler.semantic_edges
}

fn controller_facts(edges: &[SemanticEdge]) -> BTreeSet<(String, &'static str, String)> {
    edges
        .iter()
        .filter(|edge| {
            edge.subject.domain == "dotnet"
                && edge.subject.entity_type == "Controller"
                && matches!(
                    edge.relation,
                    SemanticRelation::HasRoute | SemanticRelation::ControllerAction
                )
        })
        .map(|edge| {
            (
                edge.subject.name.clone(),
                match edge.relation {
                    SemanticRelation::HasRoute => "route",
                    SemanticRelation::ControllerAction => "action",
                    _ => unreachable!("controller filter admits only route/action relations"),
                },
                edge.object.name.clone(),
            )
        })
        .collect()
}

#[test]
fn r9_aspnet_bases_use_shared_resolution_for_spelling_ancestry_and_partials() {
    let edges = compile_edges(
        r#"
using Microsoft.AspNetCore.Mvc;
using Mvc = Microsoft.AspNetCore.Mvc;
using ControllerRoot = Microsoft.AspNetCore.Mvc.ControllerBase;

namespace Application
{
    [Route("imported")] public class ImportedController : ControllerBase { [HttpGet] public void Imported() {} }
    [Route("qualified")] public class QualifiedController : Microsoft.AspNetCore.Mvc.Controller { [HttpGet] public void Qualified() {} }
    [Route("global")] public class GlobalController : global::Microsoft.AspNetCore.Mvc.ControllerBase { [HttpGet] public void Global() {} }
    [Route("namespace-alias")] public class NamespaceAliasController : Mvc.ControllerBase { [HttpGet] public void NamespaceAlias() {} }
    [Route("type-alias")] public class TypeAliasController : ControllerRoot { [HttpGet] public void TypeAlias() {} }
    public class IntermediateController : ControllerBase { }
    [Route("derived")] public class DerivedController : IntermediateController { [HttpGet] public void Derived() {} }
    public partial class PartialController : ControllerBase { }
    [Route("partial")] public partial class PartialController { [HttpGet] public void Partial() {} }

    namespace Shadow
    {
        public class ControllerBase { }
        [Route("bad")] public class FakeController : ControllerBase { [HttpGet] public void Bad() {} }
    }
}
"#,
    );

    assert_eq!(
        controller_facts(&edges),
        BTreeSet::from([
            ("DerivedController".into(), "action", "Derived".into()),
            ("DerivedController".into(), "route", "derived".into()),
            ("GlobalController".into(), "action", "Global".into()),
            ("GlobalController".into(), "route", "global".into()),
            ("ImportedController".into(), "action", "Imported".into()),
            ("ImportedController".into(), "route", "imported".into()),
            (
                "NamespaceAliasController".into(),
                "action",
                "NamespaceAlias".into(),
            ),
            (
                "NamespaceAliasController".into(),
                "route",
                "namespace-alias".into(),
            ),
            ("PartialController".into(), "action", "Partial".into()),
            ("PartialController".into(), "route", "partial".into()),
            ("QualifiedController".into(), "action", "Qualified".into()),
            ("QualifiedController".into(), "route", "qualified".into()),
            ("TypeAliasController".into(), "action", "TypeAlias".into()),
            ("TypeAliasController".into(), "route", "type-alias".into()),
        ])
    );
}

#[test]
fn r9_aspnet_competing_imports_fail_closed() {
    let edges = compile_edges(
        r#"
using Microsoft.AspNetCore.Mvc;
using Other;
namespace Other { public class ControllerBase { } }
namespace Application
{
    [Route("bad")] public class AmbiguousController : ControllerBase
    {
        [HttpGet] public void Bad() {}
    }
}
"#,
    );

    assert!(
        controller_facts(&edges).is_empty(),
        "competing source and known-external imports cannot authorize controller facts"
    );
}
