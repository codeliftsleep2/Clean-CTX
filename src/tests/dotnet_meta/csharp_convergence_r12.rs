use crate::compression::Fidelity;
use crate::ir::compiler::IRCompiler;
use crate::layers::meta::semantic::{SemanticEdge, SemanticRelation};
use std::collections::BTreeSet;

fn compile_edges(source: &str, file: &str) -> Vec<SemanticEdge> {
    let (language, query) =
        crate::compression::language::language_for_extension("cs").expect("C# language");
    let mut compiler = IRCompiler::new();
    compiler
        .compile_focused(
            source,
            "r12-cross-consumer-convergence",
            Some(file),
            language,
            query,
            Fidelity::High,
            None,
            None,
        )
        .expect("C# compilation");
    compiler.semantic_edges
}

#[test]
fn r12_every_resolved_framework_role_bypasses_textual_applicability() {
    let cases = [
        (
            "DbContext",
            "public class AppContext : global::Microsoft.EntityFrameworkCore.DbContext {}",
        ),
        (
            "AutoMapper",
            "public class Maps : global::AutoMapper.Profile {}",
        ),
        (
            "SignalR",
            "public class Events : global::Microsoft.AspNetCore.SignalR.Hub { public void Broadcast() {} }",
        ),
        (
            "ASP.NET base",
            "public class Endpoint : global::Microsoft.AspNetCore.Mvc.ControllerBase {}",
        ),
        (
            "ASP.NET attribute",
            "[global::Microsoft.AspNetCore.Mvc.ApiControllerAttribute] public class Endpoint {}",
        ),
        (
            "MSTest",
            "[global::Microsoft.VisualStudio.TestTools.UnitTesting.TestClassAttribute] public class Tests {}",
        ),
    ];

    for (name, source) in cases {
        assert!(
            !crate::dotnet_meta::detect::is_dotnet_file(source),
            "{name} fixture must remain outside textual candidate detection"
        );
        crate::dotnet_meta::reset_detection_count();
        crate::dotnet_meta::reset_evaluation_count();
        let _ = compile_edges(source, &format!("C:/repo/{name}.cs"));
        assert_eq!(
            crate::dotnet_meta::detection_count(),
            0,
            "resolved {name} authority must short-circuit textual detection"
        );
        assert_eq!(
            crate::dotnet_meta::evaluation_count(),
            1,
            "resolved {name} authority must positively admit .NET evaluation"
        );
    }
}

#[test]
fn r12_migrated_consumers_converge_in_one_compilation_generation() {
    let source = r#"
public class AppContext : global::Microsoft.EntityFrameworkCore.DbContext {}

public class Maps : global::AutoMapper.Profile {
    public Maps() { CreateMap<Source, Destination>(); }
}

public class Events : global::Microsoft.AspNetCore.SignalR.Hub {
    public void Broadcast() {}
}

[global::Microsoft.AspNetCore.Mvc.ApiControllerAttribute]
[global::Microsoft.AspNetCore.Mvc.RouteAttribute("orders")]
public class OrdersEndpoint {
    [global::Microsoft.AspNetCore.Mvc.HttpHeadAttribute]
    public void Head() {}
}

[global::Microsoft.VisualStudio.TestTools.UnitTesting.TestClassAttribute]
public class OrdersTests {}
"#;
    let edges = compile_edges(source, "C:/repo/AllConsumers.cs");
    let facts: BTreeSet<(String, String, String)> = edges
        .iter()
        .filter_map(|edge| {
            let relation = match edge.relation {
                SemanticRelation::Defines
                    if edge.subject.domain == "dotnet"
                        && matches!(
                            edge.subject.entity_type,
                            "DbContext" | "Controller" | "TestClass"
                        ) =>
                {
                    format!("{:?}", edge.relation)
                }
                SemanticRelation::MapsFrom
                | SemanticRelation::MapsTo
                | SemanticRelation::HubMethodTargets
                | SemanticRelation::HasRoute
                | SemanticRelation::ControllerAction => format!("{:?}", edge.relation),
                _ => return None,
            };
            Some((
                format!("{}/{}", edge.subject.entity_type, edge.subject.name),
                relation,
                format!("{}/{}", edge.object.entity_type, edge.object.name),
            ))
        })
        .collect();

    let expected = BTreeSet::from([
        (
            "DbContext/AppContext".to_string(),
            "Defines".to_string(),
            "DbContext/AppContext".to_string(),
        ),
        (
            "MapperProfile/Maps".to_string(),
            "MapsFrom".to_string(),
            "Entity/Source".to_string(),
        ),
        (
            "MapperProfile/Maps".to_string(),
            "MapsTo".to_string(),
            "Entity/Destination".to_string(),
        ),
        (
            "Hub/Events".to_string(),
            "HubMethodTargets".to_string(),
            "HubMethod/Broadcast".to_string(),
        ),
        (
            "Controller/OrdersEndpoint".to_string(),
            "Defines".to_string(),
            "Controller/OrdersEndpoint".to_string(),
        ),
        (
            "Controller/OrdersEndpoint".to_string(),
            "HasRoute".to_string(),
            "Route/orders".to_string(),
        ),
        (
            "Controller/OrdersEndpoint".to_string(),
            "ControllerAction".to_string(),
            "Action/Head".to_string(),
        ),
        (
            "TestClass/OrdersTests".to_string(),
            "Defines".to_string(),
            "TestClass/OrdersTests".to_string(),
        ),
    ]);
    assert_eq!(facts, expected);
}
