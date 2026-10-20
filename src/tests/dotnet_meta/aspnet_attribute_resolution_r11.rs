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
            "r11-aspnet-attribute-resolution",
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

fn roles(edges: &[SemanticEdge]) -> BTreeSet<String> {
    edges
        .iter()
        .filter(|edge| {
            edge.relation == SemanticRelation::Defines
                && edge.subject.domain == "dotnet"
                && edge.subject.entity_type == "Controller"
        })
        .map(|edge| edge.subject.name.clone())
        .collect()
}

fn facts(edges: &[SemanticEdge]) -> BTreeSet<(String, &'static str, String)> {
    edges
        .iter()
        .filter_map(|edge| {
            let kind = match edge.relation {
                SemanticRelation::HasRoute => "route",
                SemanticRelation::ControllerAction => "action",
                _ => return None,
            };
            (edge.subject.domain == "dotnet" && edge.subject.entity_type == "Controller")
                .then(|| (edge.subject.name.clone(), kind, edge.object.name.clone()))
        })
        .collect()
}

#[test]
fn r11_aspnet_attributes_resolve_role_route_and_actions_for_every_supported_spelling() {
    let edges = compile_edges(
        r#"
using Microsoft.AspNetCore.Mvc;
using Mvc = Microsoft.AspNetCore.Mvc;
using ControllerMarker = Microsoft.AspNetCore.Mvc.ControllerAttribute;
using RouteMarker = Microsoft.AspNetCore.Mvc.RouteAttribute;
using GetMarker = Microsoft.AspNetCore.Mvc.HttpGetAttribute;
namespace Application {
    [ApiController] [Route("imported")] public class ImportedEndpoint { [HttpGet] public void Imported() {} }
    [Microsoft.AspNetCore.Mvc.ApiController] [Microsoft.AspNetCore.Mvc.Route("qualified")]
    public class QualifiedEndpoint { [Microsoft.AspNetCore.Mvc.HttpPost] public void Qualified() {} }
    [global::Microsoft.AspNetCore.Mvc.ApiControllerAttribute]
    [global::Microsoft.AspNetCore.Mvc.RouteAttribute("global")]
    public class GlobalEndpoint { [global::Microsoft.AspNetCore.Mvc.HttpPutAttribute] public void Global() {} }
    [Mvc.ApiController] [Mvc.Route("namespace-alias")]
    public class NamespaceAliasEndpoint { [Mvc.HttpDelete] public void NamespaceAlias() {} }
    [ControllerMarker] [RouteMarker("type-alias")]
    public class TypeAliasEndpoint { [GetMarker] public void TypeAlias() {} }
    [ApiController] public partial class PartialEndpoint {}
    [Route("partial")] public partial class PartialEndpoint { [HttpPatch] public void Partial() {} }
}
"#,
    );

    assert_eq!(
        roles(&edges),
        BTreeSet::from([
            "GlobalEndpoint".into(),
            "ImportedEndpoint".into(),
            "NamespaceAliasEndpoint".into(),
            "PartialEndpoint".into(),
            "QualifiedEndpoint".into(),
            "TypeAliasEndpoint".into(),
        ])
    );
    assert_eq!(
        facts(&edges),
        BTreeSet::from([
            ("GlobalEndpoint".into(), "action", "Global".into()),
            ("GlobalEndpoint".into(), "route", "global".into()),
            ("ImportedEndpoint".into(), "action", "Imported".into()),
            ("ImportedEndpoint".into(), "route", "imported".into()),
            (
                "NamespaceAliasEndpoint".into(),
                "action",
                "NamespaceAlias".into()
            ),
            (
                "NamespaceAliasEndpoint".into(),
                "route",
                "namespace-alias".into()
            ),
            ("PartialEndpoint".into(), "action", "Partial".into()),
            ("PartialEndpoint".into(), "route", "partial".into()),
            ("QualifiedEndpoint".into(), "action", "Qualified".into()),
            ("QualifiedEndpoint".into(), "route", "qualified".into()),
            ("TypeAliasEndpoint".into(), "action", "TypeAlias".into()),
            ("TypeAliasEndpoint".into(), "route", "type-alias".into()),
        ])
    );
}

#[test]
fn r11_aspnet_attribute_shadows_and_competing_imports_fail_closed() {
    let edges = compile_edges(
        r#"
using Microsoft.AspNetCore.Mvc;
using Other;
namespace Other {
    public class ApiControllerAttribute : System.Attribute {}
    public class RouteAttribute : System.Attribute { public RouteAttribute(string value) {} }
    public class HttpGetAttribute : System.Attribute {}
}
namespace Application {
    [ApiController] [Route("bad-role")] public class AmbiguousRoleEndpoint { [HttpGet] public void Bad() {} }
    namespace Shadow {
        public class RouteAttribute : System.Attribute { public RouteAttribute(string value) {} }
        public class HttpGetAttribute : System.Attribute {}
        [Route("bad-payload")] public class ShadowPayloadEndpoint : Microsoft.AspNetCore.Mvc.ControllerBase {
            [HttpGet] public void Bad() {}
        }
    }
}
"#,
    );

    assert!(!roles(&edges).contains("AmbiguousRoleEndpoint"));
    assert!(
        facts(&edges).is_empty(),
        "unresolved or ambiguous ASP.NET attributes cannot authorize route/action facts: {:?}",
        facts(&edges)
    );
}
