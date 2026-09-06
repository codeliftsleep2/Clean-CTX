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
// ── Phase 19: @Autowired declared-type repair ─────────────────────────

fn autowired(edges: &[SemanticEdge]) -> Vec<&SemanticEdge> {
    edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::Autowired)
        .collect()
}

#[test]
fn autowired_field_multiline_targets_declared_type() {
    // Canonical shape from src/test_files/java/UserController.java.
    let source = r#"
import org.springframework.web.bind.annotation.RestController;
import org.springframework.beans.factory.annotation.Autowired;

@RestController
public class UserController {

    @Autowired
    private UserService userService;
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::High, None);
    let autowired = autowired(&edges);
    assert_eq!(autowired.len(), 1);
    assert_eq!(
        autowired[0].subject,
        EntityRef::new("spring", "Controller", "UserController")
    );
    assert_eq!(
        autowired[0].object,
        EntityRef::new("spring", "Service", "UserService")
    );
    assert_eq!(autowired[0].relation, SemanticRelation::Autowired);
    assert_eq!(autowired[0].layer, "spring");
}

#[test]
fn autowired_field_inline_targets_declared_type() {
    let source = r#"
import org.springframework.beans.factory.annotation.Autowired;

@RestController
public class UserController {
    @Autowired private UserService userService;
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::High, None);
    let autowired = autowired(&edges);
    assert_eq!(autowired.len(), 1);
    // The declared type — not the modifier token `private` — is the identity.
    assert_eq!(
        autowired[0].object,
        EntityRef::new("spring", "Service", "UserService")
    );
}

#[test]
fn autowired_field_qualified_type_preserved() {
    let source = r#"
import org.springframework.beans.factory.annotation.Autowired;

@RestController
public class UserController {
    @Autowired
    private com.example.UserService userService;
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::High, None);
    let autowired = autowired(&edges);
    assert_eq!(autowired.len(), 1);
    assert_eq!(
        autowired[0].object,
        EntityRef::new("spring", "Service", "com.example.UserService")
    );
}
#[test]
fn autowired_field_generic_type_preserved() {
    let source = r#"
import org.springframework.beans.factory.annotation.Autowired;

@RestController
public class UserController {
    @Autowired
    private Repository<Customer> repository;

    @Autowired
    private Repository<Customer, Order> dualRepository;
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::High, None);
    let autowired = autowired(&edges);
    assert_eq!(autowired.len(), 2);
    let types: Vec<&str> = autowired.iter().map(|e| e.object.name.as_str()).collect();
    assert!(types.contains(&"Repository<Customer>"));
    assert!(
        types.contains(&"Repository<Customer, Order>"),
        "commas inside generic arguments must not split the type"
    );
}

#[test]
fn autowired_field_with_qualifier_uses_declared_type() {
    let source = r#"
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;

@RestController
public class UserController {
    @Autowired
    @Qualifier("specialUserService")
    private UserService userService;
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::High, None);
    let autowired = autowired(&edges);
    assert_eq!(autowired.len(), 1);
    // The qualifier value is not semantic data in this phase; the declared
    // dependency type is the identity.
    assert_eq!(
        autowired[0].object,
        EntityRef::new("spring", "Service", "UserService")
    );
    assert!(!edges.iter().any(|e| e.object.name == "specialUserService"));
}

#[test]
fn autowired_malformed_declarations_fail_closed() {
    // Missing field name → no edge. Missing type → no edge.
    // Setter/method `@Autowired` → no edge.
    let source = r#"
import org.springframework.beans.factory.annotation.Autowired;

@RestController
public class UserController {
    @Autowired
    private;

    @Autowired
    UserService;

    @Autowired
    public void setUserService(UserService svc) {}
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::High, None);
    assert!(
        autowired(&edges).is_empty(),
        "malformed declarations must fail closed"
    );
    assert!(!edges.iter().any(|e| e.object.name == "?"));
    assert!(!edges.iter().any(|e| e.object.name == "private"));
    assert!(!edges.iter().any(|e| e.object.name == "public"));
}

#[test]
fn autowired_multiple_fields_each_typed() {
    let source = r#"
import org.springframework.beans.factory.annotation.Autowired;

@RestController
public class UserController {
    @Autowired
    private UserService userService;

    @Autowired
    private UserRepository userRepository;
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::High, None);
    let autowired = autowired(&edges);
    assert_eq!(autowired.len(), 2);
    let types: Vec<&str> = autowired.iter().map(|e| e.object.name.as_str()).collect();
    assert!(types.contains(&"UserService"));
    assert!(types.contains(&"UserRepository"));
}

// ============================================================================
// Phase 21: Spring DI Provision Semantics (@Service, @Repository, @Bean)
// ============================================================================

// -- Helpers -----------------------------------------------------------------

fn binds(edges: &[SemanticEdge]) -> Vec<&SemanticEdge> {
    edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::Binds)
        .collect()
}

fn bean_produces(edges: &[SemanticEdge]) -> Vec<&SemanticEdge> {
    edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::BeanProduces)
        .collect()
}

// -- @Service ----------------------------------------------------------------

#[test]
fn service_default_class_name_token() {
    let source = r#"
import org.springframework.stereotype.Service;

@Service
public class UserService {
    public List<String> findAll() { return null; }
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::Medium, None);
    let b = binds(&edges);
    assert_eq!(b.len(), 1, "should have exactly one Binds edge");
    assert_eq!(
        b[0].subject,
        EntityRef::new("spring", "Service", "UserService")
    );
    assert_eq!(
        b[0].object,
        EntityRef::new("spring", "Token", "UserService")
    );
}

#[test]
fn service_explicit_name_token() {
    let source = r#"
import org.springframework.stereotype.Service;

@Service("userService")
public class UserService {
    public List<String> findAll() { return null; }
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::Medium, None);
    let b = binds(&edges);
    assert_eq!(b.len(), 1);
    assert_eq!(
        b[0].subject,
        EntityRef::new("spring", "Service", "UserService")
    );
    assert_eq!(
        b[0].object,
        EntityRef::new("spring", "Token", "userService")
    );
}

#[test]
fn service_multiple_classes_independent() {
    // In the real pipeline, each class capture is a separate class.root span.
    // Multiple classes are processed as separate captures, not one combined capture.
    let source1 = r#"
import org.springframework.stereotype.Service;

@Service
public class UserService {}
"#;
    let source2 = r#"
import org.springframework.stereotype.Service;

@Service
public class OrderService {}
"#;
    let layer = SpringBootMetaLayer::new();

    let edges1 =
        layer.extract_semantic_edges(source1, &[source1.to_string()], Fidelity::Medium, None);
    let edges2 =
        layer.extract_semantic_edges(source2, &[source2.to_string()], Fidelity::Medium, None);

    let b1 = binds(&edges1);
    let b2 = binds(&edges2);
    assert_eq!(b1.len(), 1);
    assert_eq!(b2.len(), 1);
    assert_eq!(
        b1[0].object,
        EntityRef::new("spring", "Token", "UserService")
    );
    assert_eq!(
        b2[0].object,
        EntityRef::new("spring", "Token", "OrderService")
    );

    // Each class is its own implementation identity
    assert_eq!(
        b1[0].subject,
        EntityRef::new("spring", "Service", "UserService")
    );
    assert_eq!(
        b2[0].subject,
        EntityRef::new("spring", "Service", "OrderService")
    );
}

// -- @Repository -------------------------------------------------------------

#[test]
fn repository_default_class_name_token() {
    let source = r#"
import org.springframework.stereotype.Repository;

@Repository
public class UserRepository {
    public User findById(Long id) { return null; }
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::Medium, None);
    let b = binds(&edges);
    assert_eq!(b.len(), 1, "should have exactly one Binds edge");
    assert_eq!(
        b[0].subject,
        EntityRef::new("spring", "Repository", "UserRepository")
    );
    assert_eq!(
        b[0].object,
        EntityRef::new("spring", "Token", "UserRepository")
    );
}

#[test]
fn repository_explicit_name_token() {
    let source = r#"
import org.springframework.stereotype.Repository;

@Repository("userRepository")
public class UserRepository {
    public User findById(Long id) { return null; }
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::Medium, None);
    let b = binds(&edges);
    assert_eq!(b.len(), 1);
    assert_eq!(
        b[0].subject,
        EntityRef::new("spring", "Repository", "UserRepository")
    );
    assert_eq!(
        b[0].object,
        EntityRef::new("spring", "Token", "userRepository")
    );
}

// -- @Bean (fail closed on Binds) -------------------------------------------

#[test]
fn bean_preserves_bean_produces_no_binds() {
    // The current extraction path does NOT capture the @Bean method's return
    // type, so the Binds projection must fail closed. BeanProduces (the
    // factory declaration fact) remains present.
    let source = r#"
import org.springframework.context.annotation.*;

@Configuration
public class AppConfig {
    @Bean
    public UserService userService() {
        return new UserService();
    }
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::Medium, None);

    // BeanProduces (factory fact) is preserved
    let bp = bean_produces(&edges);
    assert!(!bp.is_empty(), "BeanProduces should remain present");

    // No Binds edge is created (return type not authoritative)
    let b = binds(&edges);
    assert!(
        b.is_empty(),
        "@Bean must NOT produce a Binds edge (return type not captured)"
    );
}

#[test]
fn bean_explicit_name_no_binds() {
    // Even with an explicit @Bean("name"), the Binds projection fails closed
    // because the implementation type (return type) is not captured.
    let source = r#"
import org.springframework.context.annotation.*;

@Configuration
public class AppConfig {
    @Bean("specialUserService")
    public UserService userService() {
        return new UserService();
    }
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::Medium, None);

    // BeanProduces preserved (factory fact)
    let bp = bean_produces(&edges);
    assert!(!bp.is_empty(), "BeanProduces should remain present");

    // No Binds edge
    let b = binds(&edges);
    assert!(
        b.is_empty(),
        "@Bean with explicit name must NOT produce a Binds edge"
    );
}

// -- Isolation: @Autowired unchanged -----------------------------------------

#[test]
fn provision_does_not_change_autowired_endpoint() {
    // Phase 21 must not alter the existing Autowired edge. The endpoint
    // remains spring/Service/<declared-type> (Phase 19 behavior).
    let source = r#"
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.web.bind.annotation.RestController;

@RestController
public class UserController {
    @Autowired
    private UserService userService;
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::High, None);
    let autowired = autowired(&edges);
    assert_eq!(autowired.len(), 1);
    assert_eq!(
        autowired[0].object,
        EntityRef::new("spring", "Service", "UserService")
    );
}

// -- Isolation: no @Component, no qualifier, no constructor injection -------

#[test]
fn provision_does_not_add_component_or_qualifier() {
    // @Component is not recognized; @Qualifier is not semantic data.
    let source = r#"
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.web.bind.annotation.RestController;

@RestController
public class UserController {
    @Autowired
    @Qualifier("specialUserService")
    private UserService userService;
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::High, None);

    // No qualifier-based edge
    assert!(!edges.iter().any(|e| e.object.name == "specialUserService"));
    // No component-based edge
    assert!(!edges.iter().any(|e| e.subject.entity_type == "Component"));
    // Autowired still works
    let autowired = autowired(&edges);
    assert_eq!(autowired.len(), 1);
}
