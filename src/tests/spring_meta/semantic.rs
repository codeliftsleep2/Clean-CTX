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
    // Phase 23: object role is now Token (source-level DI key).
    assert_eq!(
        autowired[0].object,
        EntityRef::new("spring", "Token", "UserService")
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
    // Phase 23: object role is now Token (source-level DI key).
    assert_eq!(
        autowired[0].object,
        EntityRef::new("spring", "Token", "UserService")
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
    // Phase 23: object role is now Token (source-level DI key).
    assert_eq!(
        autowired[0].object,
        EntityRef::new("spring", "Token", "com.example.UserService")
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
    // Phase 23: qualifier overrides the type-based token.
    assert_eq!(
        autowired[0].object,
        EntityRef::new("spring", "Token", "specialUserService")
    );
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
    // Phase 23: dual binding — explicit name produces both class token and name token.
    assert_eq!(b.len(), 2);
    assert!(
        b.iter()
            .any(|e| e.object == EntityRef::new("spring", "Token", "UserService"))
    );
    assert!(
        b.iter()
            .any(|e| e.object == EntityRef::new("spring", "Token", "userService"))
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
    // Phase 23: dual binding — explicit name produces both class token and name token.
    assert_eq!(b.len(), 2);
    assert!(
        b.iter()
            .any(|e| e.object == EntityRef::new("spring", "Token", "UserRepository"))
    );
    assert!(
        b.iter()
            .any(|e| e.object == EntityRef::new("spring", "Token", "userRepository"))
    );
}

// -- @Bean (fail closed on Binds) -------------------------------------------

#[test]
fn bean_preserves_bean_produces_with_binds() {
    // Phase 25: @Bean method with capturable return type produces both
    // BeanProduces (factory fact) and Binds (provision fact). Dual-binding:
    // return type token (for plain @Autowired) and method name token
    // (for @Qualifier).
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

    // Binds edge is created (return type captured)
    let b = binds(&edges);
    assert!(!b.is_empty(), "@Bean should produce Binds edges");
    // Dual binding: type token + method name token
    assert!(
        b.iter()
            .any(|e| e.object == EntityRef::new("spring", "Token", "UserService"))
    );
    assert!(
        b.iter()
            .any(|e| e.object == EntityRef::new("spring", "Token", "userService"))
    );
}

#[test]
fn bean_explicit_name_with_binds() {
    // Phase 25: @Bean("name") with capturable return type produces Binds
    // with explicit name as the token (plus return type token).
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

    // Binds edge with explicit name token
    let b = binds(&edges);
    assert!(
        !b.is_empty(),
        "@Bean with explicit name should produce Binds"
    );
    assert!(
        b.iter()
            .any(|e| e.object == EntityRef::new("spring", "Token", "specialUserService"))
    );
    assert!(
        b.iter()
            .any(|e| e.object == EntityRef::new("spring", "Token", "UserService"))
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
    // Phase 23: object role is now Token (source-level DI key).
    assert_eq!(
        autowired[0].object,
        EntityRef::new("spring", "Token", "UserService")
    );
}

// -- Isolation: no @Component, no constructor injection -----------------------

#[test]
fn provision_does_not_add_component() {
    // @Component is not recognized.
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

    // No component-based edge
    assert!(!edges.iter().any(|e| e.subject.entity_type == "Component"));
    // Autowired still works
    let autowired = autowired(&edges);
    assert_eq!(autowired.len(), 1);
}

// ── Phase 23: Consumer token migration ──────────────────────────────────────

#[test]
fn autowired_concrete_type_targets_token() {
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
        EntityRef::new("spring", "Token", "UserService")
    );
}

#[test]
fn autowired_interface_type_targets_token() {
    let source = r#"
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.web.bind.annotation.RestController;

@RestController
public class UserController {
    @Autowired
    private IUserService userService;
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::High, None);

    let autowired = autowired(&edges);
    assert_eq!(autowired.len(), 1);
    assert_eq!(
        autowired[0].object,
        EntityRef::new("spring", "Token", "IUserService")
    );
}

#[test]
fn autowired_required_false_still_targets_token() {
    let source = r#"
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.web.bind.annotation.RestController;

@RestController
public class UserController {
    @Autowired(required = false)
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
        EntityRef::new("spring", "Token", "UserService")
    );
}

#[test]
fn autowired_qualified_interface_targets_qualifier_token() {
    let source = r#"
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.web.bind.annotation.RestController;

@RestController
public class UserController {
    @Autowired
    @Qualifier("specialUserService")
    private IUserService userService;
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::High, None);

    let autowired = autowired(&edges);
    assert_eq!(autowired.len(), 1);
    assert_eq!(
        autowired[0].object,
        EntityRef::new("spring", "Token", "specialUserService")
    );
}

#[test]
fn autowired_qualified_concrete_type_targets_qualifier_token() {
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

    let autowired = autowired(&edges);
    assert_eq!(autowired.len(), 1);
    assert_eq!(
        autowired[0].object,
        EntityRef::new("spring", "Token", "specialUserService")
    );
}

#[test]
fn autowired_malformed_qualifier_falls_back_to_type() {
    let source = r#"
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.web.bind.annotation.RestController;

@RestController
public class UserController {
    @Autowired
    @Qualifier
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
        EntityRef::new("spring", "Token", "UserService")
    );
}

#[test]
fn autowired_unsupported_qualifier_expression_fails_closed() {
    let source = r#"
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.web.bind.annotation.RestController;

@RestController
public class UserController {
    @Autowired
    @Qualifier("a b")
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
        EntityRef::new("spring", "Token", "UserService")
    );
}

#[test]
fn autowired_multiple_fields_each_typed_token() {
    let source = r#"
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.web.bind.annotation.RestController;

@RestController
public class UserController {
    @Autowired
    private UserService userService;
    @Autowired
    private OrderService orderService;
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::High, None);

    let autowired = autowired(&edges);
    assert_eq!(autowired.len(), 2);
    assert!(
        autowired
            .iter()
            .any(|e| e.object == EntityRef::new("spring", "Token", "UserService"))
    );
    assert!(
        autowired
            .iter()
            .any(|e| e.object == EntityRef::new("spring", "Token", "OrderService"))
    );
}

#[test]
fn autowired_generic_type_exact_spelling() {
    let source = r#"
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.web.bind.annotation.RestController;

@RestController
public class UserController {
    @Autowired
    private Repository<Customer> repository;
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::High, None);

    let autowired = autowired(&edges);
    assert_eq!(autowired.len(), 1);
    assert_eq!(
        autowired[0].object,
        EntityRef::new("spring", "Token", "Repository<Customer>")
    );
}

#[test]
fn autowired_malformed_field_fails_closed() {
    let source = r#"
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.web.bind.annotation.RestController;

@RestController
public class UserController {
    @Autowired
    private;
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::High, None);

    let autowired = autowired(&edges);
    assert_eq!(autowired.len(), 0);
}

// ── Phase 23: Provider dual binding ────────────────────────────────────────

#[test]
fn service_default_name_one_binding() {
    let source = r#"
import org.springframework.stereotype.Service;

@Service
public class UserService {}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::Medium, None);

    let binds: Vec<&SemanticEdge> = edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::Binds)
        .collect();
    assert_eq!(binds.len(), 1);
    assert_eq!(
        binds[0].object,
        EntityRef::new("spring", "Token", "UserService")
    );
}

#[test]
fn service_explicit_name_dual_binding() {
    let source = r#"
import org.springframework.stereotype.Service;

@Service("specialUserService")
public class UserService {}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::Medium, None);

    let binds: Vec<&SemanticEdge> = edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::Binds)
        .collect();
    assert_eq!(binds.len(), 2);
    assert!(
        binds
            .iter()
            .any(|e| e.object == EntityRef::new("spring", "Token", "UserService"))
    );
    assert!(
        binds
            .iter()
            .any(|e| e.object == EntityRef::new("spring", "Token", "specialUserService"))
    );
}

#[test]
fn service_explicit_name_equals_class_name_no_duplicate() {
    let source = r#"
import org.springframework.stereotype.Service;

@Service("UserService")
public class UserService {}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::Medium, None);

    let binds: Vec<&SemanticEdge> = edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::Binds)
        .collect();
    assert_eq!(binds.len(), 1);
    assert_eq!(
        binds[0].object,
        EntityRef::new("spring", "Token", "UserService")
    );
}

#[test]
fn repository_default_name_one_binding() {
    let source = r#"
import org.springframework.stereotype.Repository;

@Repository
public class UserRepository {}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::Medium, None);

    let binds: Vec<&SemanticEdge> = edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::Binds)
        .collect();
    assert_eq!(binds.len(), 1);
    assert_eq!(
        binds[0].object,
        EntityRef::new("spring", "Token", "UserRepository")
    );
}

#[test]
fn repository_explicit_name_dual_binding() {
    let source = r#"
import org.springframework.stereotype.Repository;

@Repository("userRepo")
public class UserRepository {}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::Medium, None);

    let binds: Vec<&SemanticEdge> = edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::Binds)
        .collect();
    assert_eq!(binds.len(), 2);
    assert!(
        binds
            .iter()
            .any(|e| e.object == EntityRef::new("spring", "Token", "UserRepository"))
    );
    assert!(
        binds
            .iter()
            .any(|e| e.object == EntityRef::new("spring", "Token", "userRepo"))
    );
}

// ── Phase 23: Composition convergence ──────────────────────────────────────

#[test]
fn composition_service_and_autowired_converge() {
    let service_source = r#"
import org.springframework.stereotype.Service;

@Service
public class UserService {}
"#;
    let controller_source = r#"
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.web.bind.annotation.RestController;

@RestController
public class UserController {
    @Autowired
    private UserService userService;
}
"#;
    let layer = SpringBootMetaLayer::new();

    let service_edges = layer.extract_semantic_edges(
        service_source,
        &[service_source.to_string()],
        Fidelity::Medium,
        None,
    );
    let controller_edges = layer.extract_semantic_edges(
        controller_source,
        &[controller_source.to_string()],
        Fidelity::High,
        None,
    );

    let service_token = service_edges
        .iter()
        .find(|e| e.relation == SemanticRelation::Binds)
        .map(|e| e.object.clone())
        .unwrap();
    let consumer_token = controller_edges
        .iter()
        .find(|e| e.relation == SemanticRelation::Autowired)
        .map(|e| e.object.clone())
        .unwrap();

    assert_eq!(service_token, consumer_token);
    assert_eq!(
        service_token,
        EntityRef::new("spring", "Token", "UserService")
    );
}

#[test]
fn composition_explicit_service_and_qualified_autowired_converge() {
    let service_source = r#"
import org.springframework.stereotype.Service;

@Service("specialUserService")
public class UserService {}
"#;
    let controller_source = r#"
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.web.bind.annotation.RestController;

@RestController
public class UserController {
    @Autowired
    @Qualifier("specialUserService")
    private IUserService userService;
}
"#;
    let layer = SpringBootMetaLayer::new();

    let service_edges = layer.extract_semantic_edges(
        service_source,
        &[service_source.to_string()],
        Fidelity::Medium,
        None,
    );
    let controller_edges = layer.extract_semantic_edges(
        controller_source,
        &[controller_source.to_string()],
        Fidelity::High,
        None,
    );

    let service_token = service_edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::Binds)
        .find(|e| e.object.name == "specialUserService")
        .map(|e| e.object.clone())
        .unwrap();
    let consumer_token = controller_edges
        .iter()
        .find(|e| e.relation == SemanticRelation::Autowired)
        .map(|e| e.object.clone())
        .unwrap();

    assert_eq!(service_token, consumer_token);
    assert_eq!(
        service_token,
        EntityRef::new("spring", "Token", "specialUserService")
    );
}

#[test]
fn composition_explicit_service_and_type_autowired_converge_via_class_token() {
    let service_source = r#"
import org.springframework.stereotype.Service;

@Service("specialUserService")
public class UserService {}
"#;
    let controller_source = r#"
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.web.bind.annotation.RestController;

@RestController
public class UserController {
    @Autowired
    private UserService userService;
}
"#;
    let layer = SpringBootMetaLayer::new();

    let service_edges = layer.extract_semantic_edges(
        service_source,
        &[service_source.to_string()],
        Fidelity::Medium,
        None,
    );
    let controller_edges = layer.extract_semantic_edges(
        controller_source,
        &[controller_source.to_string()],
        Fidelity::High,
        None,
    );

    let service_class_token = service_edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::Binds)
        .find(|e| e.object.name == "UserService")
        .map(|e| e.object.clone())
        .unwrap();
    let consumer_token = controller_edges
        .iter()
        .find(|e| e.relation == SemanticRelation::Autowired)
        .map(|e| e.object.clone())
        .unwrap();

    assert_eq!(service_class_token, consumer_token);
    assert_eq!(
        service_class_token,
        EntityRef::new("spring", "Token", "UserService")
    );
}

// ── Phase 23: Isolation ────────────────────────────────────────────────────

#[test]
fn autowired_does_not_alter_bean_produces() {
    let source = r#"
import org.springframework.context.annotation.*;

@Configuration
public class AppConfig {
    @Bean
    public UserService userService() { return new UserService(); }
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::Medium, None);

    let bean_produces: Vec<&SemanticEdge> = edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::BeanProduces)
        .collect();
    assert_eq!(bean_produces.len(), 1);
    assert_eq!(
        bean_produces[0].object,
        EntityRef::new("spring", "Bean", "userService")
    );
}

// ── Phase 25: @Bean provision semantics ────────────────────────────────────

#[test]
fn bean_basic_emits_dual_binds() {
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

    let bp = bean_produces(&edges);
    assert!(!bp.is_empty(), "BeanProduces should remain present");

    let b = binds(&edges);
    assert_eq!(b.len(), 2, "dual binding: type token + method name token");
    assert!(
        b.iter()
            .any(|e| e.object == EntityRef::new("spring", "Token", "UserService"))
    );
    assert!(
        b.iter()
            .any(|e| e.object == EntityRef::new("spring", "Token", "userService"))
    );
}

#[test]
fn bean_explicit_name_emits_dual_binds() {
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

    let b = binds(&edges);
    assert_eq!(b.len(), 2, "dual binding: type token + explicit name token");
    assert!(
        b.iter()
            .any(|e| e.object == EntityRef::new("spring", "Token", "UserService"))
    );
    assert!(
        b.iter()
            .any(|e| e.object == EntityRef::new("spring", "Token", "specialUserService"))
    );
}

#[test]
fn bean_qualified_type_preserved() {
    let source = r#"
import org.springframework.context.annotation.*;

@Configuration
public class AppConfig {
    @Bean
    public com.example.UserService userService() {
        return new com.example.UserService();
    }
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::Medium, None);

    let b = binds(&edges);
    assert!(
        b.iter()
            .any(|e| e.object == EntityRef::new("spring", "Token", "com.example.UserService"))
    );
}

#[test]
fn bean_generic_return_type_no_binds() {
    let source = r#"
import org.springframework.context.annotation.*;

@Configuration
public class AppConfig {
    @Bean
    public List<UserService> userServices() {
        return new ArrayList<>();
    }
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::Medium, None);

    let bp = bean_produces(&edges);
    assert!(!bp.is_empty(), "BeanProduces should remain present");

    let b = binds(&edges);
    assert!(
        b.is_empty(),
        "@Bean with generic return type should NOT produce Binds"
    );
}

#[test]
fn bean_multiple_names_no_fabricated_name_token() {
    let source = r#"
import org.springframework.context.annotation.*;

@Configuration
public class AppConfig {
    @Bean({"a", "b"})
    public UserService userService() {
        return new UserService();
    }
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::Medium, None);

    let bp = bean_produces(&edges);
    assert!(!bp.is_empty(), "BeanProduces should remain present");

    let b = binds(&edges);
    // No fabricated name token from the array form. The type-based token from
    // the return type is still authoritative and must remain.
    assert!(
        !b.iter()
            .any(|e| e.object.name.contains('{') || e.object.name.contains(',')),
        "@Bean multiple names must NOT fabricate a name token"
    );
    assert!(
        b.iter()
            .any(|e| e.object == EntityRef::new("spring", "Token", "UserService"))
    );
}

#[test]
fn bean_malformed_return_type_fails_closed() {
    let source = r#"
import org.springframework.context.annotation.*;

@Configuration
public class AppConfig {
    @Bean
    public;
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::Medium, None);

    let b = binds(&edges);
    assert!(
        b.is_empty(),
        "@Bean with malformed return type should NOT produce Binds"
    );
}

#[test]
fn bean_convergence_plain_autowired() {
    let bean_source = r#"
import org.springframework.context.annotation.*;

@Configuration
public class AppConfig {
    @Bean
    public UserService userService() {
        return new UserService();
    }
}
"#;
    let autowired_source = r#"
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.web.bind.annotation.RestController;

@RestController
public class UserController {
    @Autowired
    private UserService userService;
}
"#;
    let layer = SpringBootMetaLayer::new();

    let bean_edges = layer.extract_semantic_edges(
        bean_source,
        &[bean_source.to_string()],
        Fidelity::Medium,
        None,
    );
    let autowired_edges = layer.extract_semantic_edges(
        autowired_source,
        &[autowired_source.to_string()],
        Fidelity::High,
        None,
    );

    let bean_type_token = bean_edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::Binds)
        .find(|e| e.object.name == "UserService")
        .map(|e| e.object.clone())
        .unwrap();
    let autowired_token = autowired_edges
        .iter()
        .find(|e| e.relation == SemanticRelation::Autowired)
        .map(|e| e.object.clone())
        .unwrap();

    assert_eq!(bean_type_token, autowired_token);
}

#[test]
fn bean_convergence_qualified_autowired() {
    let bean_source = r#"
import org.springframework.context.annotation.*;

@Configuration
public class AppConfig {
    @Bean("specialUserService")
    public UserService userService() {
        return new UserService();
    }
}
"#;
    let autowired_source = r#"
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
    let layer = SpringBootMetaLayer::new();

    let bean_edges = layer.extract_semantic_edges(
        bean_source,
        &[bean_source.to_string()],
        Fidelity::Medium,
        None,
    );
    let autowired_edges = layer.extract_semantic_edges(
        autowired_source,
        &[autowired_source.to_string()],
        Fidelity::High,
        None,
    );

    let bean_name_token = bean_edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::Binds)
        .find(|e| e.object.name == "specialUserService")
        .map(|e| e.object.clone())
        .unwrap();
    let autowired_token = autowired_edges
        .iter()
        .find(|e| e.relation == SemanticRelation::Autowired)
        .map(|e| e.object.clone())
        .unwrap();

    assert_eq!(bean_name_token, autowired_token);
}
// ── Phase 28: General DI consumption (per-role Autowired) ──────────────

#[test]
fn autowired_service_field_targets_declared_type() {
    let source = r#"
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Service;

@Service
public class OrderService {
    @Autowired
    private OrderRepository repository;
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::High, None);
    let autowired = autowired(&edges);
    assert_eq!(autowired.len(), 1, "one field, one role, one edge");
    assert_eq!(
        autowired[0].subject,
        EntityRef::new("spring", "Service", "OrderService")
    );
    assert_eq!(
        autowired[0].object,
        EntityRef::new("spring", "Token", "OrderRepository")
    );
}

#[test]
fn autowired_repository_field_targets_declared_type() {
    let source = r#"
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Repository;

@Repository
public class OrderRepositoryImpl {
    @Autowired
    private AuditService audit;
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::High, None);
    let autowired = autowired(&edges);
    assert_eq!(autowired.len(), 1);
    assert_eq!(
        autowired[0].subject,
        EntityRef::new("spring", "Repository", "OrderRepositoryImpl")
    );
    assert_eq!(
        autowired[0].object,
        EntityRef::new("spring", "Token", "AuditService")
    );
}

#[test]
fn autowired_configuration_field_targets_declared_type() {
    let source = r#"
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.context.annotation.Configuration;

@Configuration
public class AppConfig {
    @Autowired
    private SomeService service;
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::High, None);
    let autowired = autowired(&edges);
    assert_eq!(autowired.len(), 1);
    assert_eq!(
        autowired[0].subject,
        EntityRef::new("spring", "Configuration", "AppConfig")
    );
    assert_eq!(
        autowired[0].object,
        EntityRef::new("spring", "Token", "SomeService")
    );
    // Ordinary field injection into the configuration object itself;
    // no BeanProduces facts are implied by the field.
    assert!(
        edges
            .iter()
            .all(|e| e.relation != SemanticRelation::BeanProduces)
    );
}

#[test]
fn autowired_qualified_service_field_uses_qualifier_token() {
    let source = r#"
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Service;

@Service
public class OrderService {
    @Autowired
    @Qualifier("specialRepository")
    private Repository repository;
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::High, None);
    let autowired = autowired(&edges);
    assert_eq!(autowired.len(), 1);
    assert_eq!(
        autowired[0].subject,
        EntityRef::new("spring", "Service", "OrderService")
    );
    assert_eq!(
        autowired[0].object,
        EntityRef::new("spring", "Token", "specialRepository")
    );
    assert_ne!(
        autowired[0].object,
        EntityRef::new("spring", "Token", "Repository")
    );
}

#[test]
fn autowired_service_qualified_type_preserved() {
    let source = r#"
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Service;

@Service
public class OrderService {
    @Autowired
    private com.example.audit.AuditService audit;
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::High, None);
    let autowired = autowired(&edges);
    assert_eq!(autowired.len(), 1);
    assert_eq!(
        autowired[0].object,
        EntityRef::new("spring", "Token", "com.example.audit.AuditService")
    );
}

#[test]
fn autowired_service_generic_type_exact_spelling() {
    let source = r#"
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Service;

@Service
public class ReportService {
    @Autowired
    private List<OrderRepository> repositories;
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::High, None);
    let autowired = autowired(&edges);
    assert_eq!(autowired.len(), 1);
    assert_eq!(
        autowired[0].object,
        EntityRef::new("spring", "Token", "List<OrderRepository>")
    );
}

#[test]
fn autowired_service_multiple_dependencies_each_typed() {
    let source = r#"
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Service;

@Service
public class OrderService {
    @Autowired
    private OrderRepository repository;

    @Autowired
    private PaymentGateway gateway;
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::High, None);
    let autowired = autowired(&edges);
    assert_eq!(autowired.len(), 2);
    assert!(
        autowired
            .iter()
            .all(|e| e.subject == EntityRef::new("spring", "Service", "OrderService"))
    );
    let types: Vec<&str> = autowired.iter().map(|e| e.object.name.as_str()).collect();
    assert!(types.contains(&"OrderRepository"));
    assert!(types.contains(&"PaymentGateway"));
}

#[test]
fn autowired_service_duplicate_types_share_edge_identity() {
    let source = r#"
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Service;

@Service
public class ReportService {
    @Autowired
    private AuditService first;

    @Autowired
    private AuditService second;
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::High, None);
    let autowired = autowired(&edges);
    assert_eq!(autowired.len(), 2, "extractor emits per field occurrence");
    // Established contract: no per-extraction dedup; WorkspaceIndex::add_edges
    // deduplicates by (relation, subject identity, object identity), so these
    // two edges are ONE graph edge. Field names are not part of the edge
    // identity — no field-level multiplicity is introduced.
    assert_eq!(autowired[0].subject, autowired[1].subject);
    assert_eq!(autowired[0].relation, autowired[1].relation);
    assert_eq!(autowired[0].object, autowired[1].object);
}

#[test]
fn autowired_multi_role_class_emits_per_role() {
    let source = r#"
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Repository;
import org.springframework.stereotype.Service;

@Service
@Repository
public class Foo {
    @Autowired
    private BarService bar;
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::High, None);
    let autowired = autowired(&edges);
    assert_eq!(
        autowired.len(),
        2,
        "one edge per authoritative role identity"
    );
    assert_eq!(
        autowired[0].subject,
        EntityRef::new("spring", "Service", "Foo")
    );
    assert_eq!(
        autowired[1].subject,
        EntityRef::new("spring", "Repository", "Foo")
    );
    assert_eq!(
        autowired[0].object,
        EntityRef::new("spring", "Token", "BarService")
    );
    assert_eq!(autowired[0].object, autowired[1].object);
}

#[test]
fn autowired_service_high_fidelity_gate_preserved() {
    // The semantic High-fidelity threshold applies to every role (Phase 27).
    let source = r#"
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Service;

@Service
public class OrderService {
    @Autowired
    private OrderRepository repository;
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let medium = layer.extract_semantic_edges(source, &class_captures, Fidelity::Medium, None);
    assert!(autowired(&medium).is_empty(), "Medium must not emit");
    let low = layer.extract_semantic_edges(source, &class_captures, Fidelity::Low, None);
    assert!(autowired(&low).is_empty(), "Low must not emit");
}

#[test]
fn autowired_service_malformed_and_setter_fail_closed() {
    // Missing field name → no edge. Missing type → no edge.
    // Setter `@Autowired` → no edge. Role identity does not loosen the
    // Phase 19 fail-closed contract.
    let source = r#"
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Service;

@Service
public class BrokenService {
    @Autowired
    private;

    @Autowired
    UserService;

    @Autowired
    public void setThing(Thing thing) {}
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::High, None);
    assert!(
        autowired(&edges).is_empty(),
        "malformed declarations must fail closed for services too"
    );
    assert!(!edges.iter().any(|e| e.object.name == "?"));
    assert!(!edges.iter().any(|e| e.object.name == "private"));
    assert!(!edges.iter().any(|e| e.object.name == "public"));
}
// ── Phase 30-A: Implements ≠ Binds (Spring side) ─────────────────────────

/// A Spring @Repository implementing an interface binds ONLY its own class
/// token. The implemented interface is a language-level fact (projected by
/// the builtin layer, Phase 30-A) and must never receive a fabricated Binds
/// edge from the Spring semantic emitter.
#[test]
fn repository_implements_does_not_bind_interface_token() {
    let source = r#"
import org.springframework.stereotype.Repository;

@Repository
public class SqlUserRepository implements UserRepository {}
"#;
    let class_captures = vec![source.to_string()];
    let layer = SpringBootMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::High, None);

    let binds: Vec<&SemanticEdge> = edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::Binds)
        .collect();
    assert_eq!(
        binds.len(),
        1,
        "the repository binds exactly its own class token"
    );
    assert_eq!(
        binds[0].object,
        EntityRef::new("spring", "Token", "SqlUserRepository")
    );
    assert!(
        !binds.iter().any(|e| e.object.name == "UserRepository"),
        "no Binds edge to the implemented interface token may be invented"
    );
}
