// src/tests/dotnet_meta/semantic.rs
//
// Tests for .NET semantic edge extraction

use crate::compression::Fidelity;
use crate::dotnet_meta::DotNetMetaLayer;
use crate::layers::meta::MetaLayer;
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};

// ── Controller → ControllerAction ────────────────────────────────────

#[test]
fn controller_controller_action() {
    let source = r#"
using Microsoft.AspNetCore.Mvc;

[ApiController]
[Route("api/users")]
public class UsersController : ControllerBase
{
    [HttpGet]
    public IActionResult GetAll() { return Ok(); }

    [HttpPost]
    public IActionResult Create([FromBody] CreateUserDto dto) { return Ok(); }
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = DotNetMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::Medium, None);

    let actions: Vec<&SemanticEdge> = edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::ControllerAction)
        .collect();
    assert!(!actions.is_empty(), "should have ControllerAction edges");

    let has_get_all = actions
        .iter()
        .any(|e| e.object == EntityRef::new("dotnet", "Action", "GetAll"));
    assert!(has_get_all, "should have GetAll action");
}

// ── Controller → HasRoute ────────────────────────────────────────────

#[test]
fn controller_has_route() {
    let source = r#"
[ApiController]
[Route("api/users")]
public class UsersController : ControllerBase { }
"#;
    let class_captures = vec![source.to_string()];
    let layer = DotNetMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::Low, None);

    let routes: Vec<&SemanticEdge> = edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::HasRoute)
        .collect();
    assert!(!routes.is_empty(), "should have HasRoute edge");

    let has_route = routes
        .iter()
        .any(|e| e.object == EntityRef::new("dotnet", "Route", "api/users"));
    assert!(has_route, "should route to api/users");
}

// ── DbContext → HasEntity ────────────────────────────────────────────

#[test]
fn dbcontext_has_entity() {
    let source = r#"
using Microsoft.EntityFrameworkCore;

public class AppDbContext : DbContext
{
    public DbSet<User> Users { get; set; }
    public DbSet<Order> Orders { get; set; }
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = DotNetMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::Low, None);

    let entities: Vec<&SemanticEdge> = edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::HasEntity)
        .collect();
    assert_eq!(entities.len(), 2, "should have two HasEntity edges");
}

// ── Φ Output Unchanged ───────────────────────────────────────────────

#[test]
fn semantic_extraction_does_not_alter_phi_output() {
    let source = r#"
using Microsoft.AspNetCore.Mvc;

[ApiController]
[Route("api/users")]
public class UsersController : ControllerBase
{
    [HttpGet]
    public IActionResult GetAll() { return Ok(); }
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = DotNetMetaLayer::new();

    let _edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::Medium, None);

    let output = layer.enrich(source, &class_captures, Fidelity::Medium, None);
    let rendered = output.map(|o| o.rendered).unwrap_or_default();

    assert!(
        rendered.contains("Φctrl:"),
        "enrich() must still produce Φctrl: markers after semantic extraction"
    );
}
