use crate::compression::Fidelity;
use crate::ir::pipeline::{PassContext, PassPipeline};
use crate::layers::meta::semantic::{SemanticEdge, SemanticRelation};

// Ingestion-boundary regression: entity occurrences unique per (identity, file).
//
// This was born RED when one controller occurrence was registered once per
// participating edge. It intentionally crosses the production pipeline and
// WorkspaceIndex ingestion boundary because registration is derived there.
#[test]
fn producer_registers_each_entity_identity_once_per_file() {
    const ACTION_COUNT: usize = 11;
    let actions: [(&str, &str); ACTION_COUNT] = [
        ("GetAll", "[HttpGet]"),
        ("GetById", "[HttpGet(\"{id}\")]"),
        ("Search", "[HttpGet(\"search\")]"),
        ("Create", "[HttpPost]"),
        ("BulkCreate", "[HttpPost(\"bulk\")]"),
        ("Update", "[HttpPut(\"{id}\")]"),
        ("Patch", "[HttpPatch(\"{id}\")]"),
        ("Delete", "[HttpDelete(\"{id}\")]"),
        ("Archive", "[HttpPost(\"{id}/archive\")]"),
        ("Restore", "[HttpPost(\"{id}/restore\")]"),
        ("Export", "[HttpGet(\"export\")]"),
    ];
    let mut source = String::from(
        "using Microsoft.AspNetCore.Mvc;\n\nnamespace Api.Controllers;\n\n[ApiController]\n[Route(\"api/orders\")]\npublic class OrdersController : ControllerBase\n{\n",
    );
    for (name, attribute) in actions {
        source.push_str(&format!(
            "    {attribute}\n    public IActionResult {name}(int id) {{ return Ok(); }}\n\n"
        ));
    }
    source.push_str("}\n");

    let mut context = PassContext::new(source, "OrdersController.cs".into(), Fidelity::Medium);
    context.canonical_path = Some("C:/repo/OrdersController.cs".into());
    context.language =
        Some(crate::compression::language::safe_csharp_language().expect("csharp grammar enabled"));
    context.query_string = crate::queries::CS_QUERY.to_string();
    PassPipeline::default_production()
        .run(&mut context)
        .expect("production pipeline should succeed");

    let edges = context.semantic_edges;
    let controller_actions: Vec<&SemanticEdge> = edges
        .iter()
        .filter(|edge| edge.relation == SemanticRelation::ControllerAction)
        .collect();
    assert_eq!(controller_actions.len(), ACTION_COUNT);
    let distinct_actions: std::collections::BTreeSet<&str> = controller_actions
        .iter()
        .map(|edge| edge.object.name.as_str())
        .collect();
    assert_eq!(distinct_actions.len(), ACTION_COUNT);
    assert_eq!(
        edges
            .iter()
            .filter(|edge| edge.relation == SemanticRelation::HasRoute)
            .count(),
        1
    );

    let canonical = "C:/repo/OrdersController.cs";
    let mut index = crate::workspace::index::WorkspaceIndex::new();
    index.add_edges(canonical, edges);
    assert_eq!(index.edge_count(), ACTION_COUNT + 1);
    assert_eq!(
        index
            .forward_edges_by_identity("dotnet", "Controller", "OrdersController")
            .len(),
        ACTION_COUNT + 1
    );

    let mut known: Vec<(&str, &str, String)> = vec![
        ("dotnet", "Controller", "OrdersController".to_string()),
        ("dotnet", "Route", "api/orders".to_string()),
        ("builtin", "Class", "OrdersController".to_string()),
    ];
    for (name, _) in actions {
        known.push(("dotnet", "Action", name.to_string()));
    }
    for (domain, entity_type, name) in &known {
        let occurrences = index.entities_by_identity(domain, entity_type, name);
        assert_eq!(
            occurrences.len(),
            1,
            "occurrence ({domain}, {entity_type}, {name}) must be registered once"
        );
        assert_eq!(occurrences[0].file.as_deref(), Some(canonical));
    }
    assert_eq!(index.entity_occurrence_count(), known.len());
    assert_eq!(
        index
            .entities_by_identity("dotnet", "Controller", "OrdersController")
            .len(),
        1
    );
}
