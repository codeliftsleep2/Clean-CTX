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

// ── DI Bindings: implementation → abstraction/token ────────────────────

#[test]
fn di_registration_emits_binds_add_scoped() {
    let source = r#"
using Microsoft.Extensions.DependencyInjection;

public class OrderService : IOrderService
{
    public void Register(IServiceCollection services)
    {
        services.AddScoped<IOrderService, OrderService>();
    }
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = DotNetMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::Low, None);

    let binds_edges: Vec<&SemanticEdge> = edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::Binds)
        .collect();
    assert_eq!(binds_edges.len(), 1, "should have one Binds edge");

    let binds = binds_edges[0];
    assert_eq!(
        binds.subject,
        EntityRef::new("dotnet", "Implementation", "OrderService")
    );
    assert_eq!(
        binds.object,
        EntityRef::new("dotnet", "Token", "IOrderService")
    );
    assert_eq!(binds.layer, "dotnet");
}

#[test]
fn di_registration_emits_binds_add_singleton() {
    let source = r#"
using Microsoft.Extensions.DependencyInjection;

public class ConfigService
{
    public void Register(IServiceCollection services)
    {
        services.AddSingleton<IConfig, ConfigService>();
    }
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = DotNetMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::Low, None);

    let binds_edges: Vec<&SemanticEdge> = edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::Binds)
        .collect();
    assert_eq!(binds_edges.len(), 1);

    let binds = binds_edges[0];
    assert_eq!(
        binds.subject,
        EntityRef::new("dotnet", "Implementation", "ConfigService")
    );
    assert_eq!(binds.object, EntityRef::new("dotnet", "Token", "IConfig"));
}

#[test]
fn di_registration_emits_binds_add_transient() {
    let source = r#"
using Microsoft.Extensions.DependencyInjection;

public class EmailService
{
    public void Register(IServiceCollection services)
    {
        services.AddTransient<IEmailService, EmailService>();
    }
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = DotNetMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::Low, None);

    let binds_edges: Vec<&SemanticEdge> = edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::Binds)
        .collect();
    assert_eq!(binds_edges.len(), 1);

    let binds = binds_edges[0];
    assert_eq!(
        binds.subject,
        EntityRef::new("dotnet", "Implementation", "EmailService")
    );
    assert_eq!(
        binds.object,
        EntityRef::new("dotnet", "Token", "IEmailService")
    );
}

#[test]
fn di_registration_binds_direction_is_implementation_to_token() {
    // Verify direction: implementation → token (NOT reversed)
    let source = r#"
using Microsoft.Extensions.DependencyInjection;

public class PaymentService
{
    public void Register(IServiceCollection services)
    {
        services.AddScoped<IPaymentService, PaymentService>();
    }
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = DotNetMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::Low, None);

    let binds_edges: Vec<&SemanticEdge> = edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::Binds)
        .collect();
    assert_eq!(binds_edges.len(), 1);

    // Subject must be implementation, object must be token
    assert_eq!(binds_edges[0].subject.entity_type, "Implementation");
    assert_eq!(binds_edges[0].object.entity_type, "Token");
}

#[test]
fn di_registration_add_db_context_no_binds() {
    // AddDbContext<T>() is a single-type registration: no Binds edge
    let source = r#"
using Microsoft.EntityFrameworkCore;

public class AppDbContext : DbContext
{
    public DbSet<User> Users { get; set; }
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = DotNetMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::Low, None);

    let binds_edges: Vec<&SemanticEdge> = edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::Binds)
        .collect();
    assert!(
        binds_edges.is_empty(),
        "AddDbContext<T>() must NOT produce Binds edges (no abstraction/token endpoint)"
    );
}

#[test]
fn di_registration_multiple_binds() {
    // Multiple DI registrations in the same class
    let source = r#"
using Microsoft.Extensions.DependencyInjection;

public class ServiceRegistration
{
    public void Register(IServiceCollection services)
    {
        services.AddScoped<IOrderService, OrderService>();
        services.AddSingleton<IConfig, ConfigService>();
        services.AddTransient<IEmailService, EmailService>();
    }
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = DotNetMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::Low, None);

    let binds_edges: Vec<&SemanticEdge> = edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::Binds)
        .collect();
    assert_eq!(binds_edges.len(), 3, "should have three Binds edges");
}

#[test]
fn di_registration_binds_does_not_duplicate_phi() {
    // Verify that Binds edges are additive: Φdi markers remain unchanged
    let source = r#"
using Microsoft.Extensions.DependencyInjection;

public class OrderService
{
    public void Register(IServiceCollection services)
    {
        services.AddScoped<IOrderService, OrderService>();
    }
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = DotNetMetaLayer::new();

    // Get semantic edges (includes Binds)
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::Low, None);
    let binds_count = edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::Binds)
        .count();
    assert_eq!(binds_count, 1);

    // Verify Φdi markers are still produced
    let output = layer.enrich(source, &class_captures, Fidelity::Low, None);
    let rendered = output.map(|o| o.rendered).unwrap_or_default();
    assert!(
        rendered.contains("Φdi:"),
        "Φdi markers must still be produced alongside Binds edges"
    );
}

#[test]
fn di_registration_binds_is_not_inferred_from_implements() {
    // Binds must come from explicit DI registration, NOT from Implements
    let source = r#"
public class OrderService : IOrderService
{
    public void DoWork() { }
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = DotNetMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::Low, None);

    let binds_edges: Vec<&SemanticEdge> = edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::Binds)
        .collect();
    assert!(
        binds_edges.is_empty(),
        "Implements must NOT imply Binds (no DI registration present)"
    );
}
