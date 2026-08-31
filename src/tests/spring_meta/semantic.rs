// src/tests/spring_meta/semantic.rs
//
// Tests for Spring semantic edge extraction

use crate::compression::Fidelity;
use crate::layers::meta::MetaLayer;
use crate::layers::meta::SpringBootMetaLayer;
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};

// ── Controller → EndpointMapsTo ──────────────────────────────────────

#[test]
fn controller_endpoint_maps_to() {
    let source = r#"
import org.springframework.web.bind.annotation.*;

@RestController
@RequestMapping("/api/users")
public class UserController {

    @GetMapping
    public List<User> getAll() { return null; }

    @PostMapping
    public User create(@RequestBody User dto) { return null; }
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::Medium, None);

    let endpoints: Vec<&SemanticEdge> = edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::EndpointMapsTo)
        .collect();
    assert!(!endpoints.is_empty(), "should have EndpointMapsTo edges");

    let has_get = endpoints
        .iter()
        .any(|e| e.subject == EntityRef::new("spring", "Controller", "UserController"));
    assert!(has_get, "UserController should have endpoint mappings");
}

// ── Configuration → BeanProduces ─────────────────────────────────────

#[test]
fn configuration_bean_produces() {
    let source = r#"
import org.springframework.context.annotation.*;

@Configuration
public class AppConfig {

    @Bean
    public DataSource dataSource() {
        return null;
    }
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::Low, None);

    let beans: Vec<&SemanticEdge> = edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::BeanProduces)
        .collect();
    assert!(!beans.is_empty(), "should have BeanProduces edges");
}

// ── ConfigurationProperties ──────────────────────────────────────────

#[test]
fn configuration_properties_binding() {
    let source = r#"
import org.springframework.boot.context.properties.*;

@ConfigurationProperties(prefix = "app")
public class AppProperties {
    private String name;
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::Low, None);

    let has_config_props = edges
        .iter()
        .any(|e| e.relation == SemanticRelation::ConfigurationProperties);
    assert!(has_config_props, "should have ConfigurationProperties edge");
}

// ── Class-level @RequestMapping path ─────────────────────────────────

#[test]
fn class_level_request_mapping_path() {
    let source = r#"
import org.springframework.web.bind.annotation.*;

@RestController
@RequestMapping("/api")
public class ApiController {

    @GetMapping("/items")
    public List<Item> getItems() { return null; }
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::Medium, None);

    let endpoints: Vec<&SemanticEdge> = edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::EndpointMapsTo)
        .collect();
    assert!(!endpoints.is_empty(), "should have EndpointMapsTo edges");

    // Class-level @RequestMapping("/api") produces an edge with path "/api"
    let has_class_mapping = endpoints.iter().any(|e| {
        e.subject == EntityRef::new("spring", "Controller", "ApiController")
            && e.object.name == "/api"
    });
    assert!(has_class_mapping, "should have class-level /api mapping");

    // Method-level @GetMapping("/items") produces an edge with "GET /items"
    let has_method_mapping = endpoints.iter().any(|e| {
        e.subject == EntityRef::new("spring", "Controller", "ApiController")
            && e.object.name == "GET /items"
    });
    assert!(
        has_method_mapping,
        "should have method-level GET /items mapping"
    );
}

// ── Method-level @GetMapping with explicit path ─────────────────────

#[test]
fn method_level_get_mapping_with_path() {
    let source = r#"
import org.springframework.web.bind.annotation.*;

@RestController
public class HealthController {

    @GetMapping("/health")
    public String health() { return "ok"; }

    @PostMapping("/report")
    public String report() { return "done"; }
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::Medium, None);

    let endpoints: Vec<&SemanticEdge> = edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::EndpointMapsTo)
        .collect();
    assert!(!endpoints.is_empty(), "should have EndpointMapsTo edges");

    let has_health = endpoints.iter().any(|e| e.object.name == "GET /health");
    assert!(has_health, "should have GET /health endpoint");

    let has_report = endpoints.iter().any(|e| e.object.name == "POST /report");
    assert!(has_report, "should have POST /report endpoint");
}

// ── Class-level + method-level path composition ─────────────────────

#[test]
fn class_and_method_path_composition() {
    let source = r#"
import org.springframework.web.bind.annotation.*;

@RestController
@RequestMapping("/api/users")
public class UserController {

    @GetMapping("/list")
    public List<User> list() { return null; }

    @PostMapping("/create")
    public User create(@RequestBody User dto) { return null; }
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::Medium, None);

    let endpoints: Vec<&SemanticEdge> = edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::EndpointMapsTo)
        .collect();
    assert!(!endpoints.is_empty(), "should have EndpointMapsTo edges");

    let controller = EntityRef::new("spring", "Controller", "UserController");

    // Class-level @RequestMapping("/api/users") produces an edge
    let has_class_mapping = endpoints
        .iter()
        .any(|e| e.subject == controller && e.object.name == "/api/users");
    assert!(
        has_class_mapping,
        "should have class-level /api/users mapping"
    );

    // Method-level @GetMapping("/list") produces an edge
    let has_list = endpoints
        .iter()
        .any(|e| e.subject == controller && e.object.name == "GET /list");
    assert!(has_list, "should have GET /list endpoint");

    // Method-level @PostMapping("/create") produces an edge
    let has_create = endpoints
        .iter()
        .any(|e| e.subject == controller && e.object.name == "POST /create");
    assert!(has_create, "should have POST /create endpoint");
}

// ── Φ Output Unchanged ───────────────────────────────────────────────

#[test]
fn semantic_extraction_does_not_alter_phi_output() {
    let source = r#"
import org.springframework.web.bind.annotation.*;

@RestController
public class HealthController {

    @GetMapping("/health")
    public String health() { return "ok"; }
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();

    let _edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::Medium, None);

    let output = layer.enrich(source, &class_captures, Fidelity::Medium, None);
    let rendered = output.map(|o| o.rendered).unwrap_or_default();

    assert!(
        rendered.contains("Φrest:"),
        "enrich() must still produce Φrest: markers after semantic extraction"
    );
}
