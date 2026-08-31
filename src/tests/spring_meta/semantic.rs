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
