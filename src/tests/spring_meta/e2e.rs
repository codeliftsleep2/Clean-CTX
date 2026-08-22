// src/tests/spring_meta/e2e.rs
//
// E2E Meta-Layer Tests — Issue #4 (FAANG Audit Remediation)
// Verifies the full pipeline: raw Spring annotation source → extraction → Φ markers
// with proper abbreviations (Φrest:, Φsvc:, Φrepo:, etc.) and no old formats.
//
// Uses extract_annotations directly (not the full file compressor) because
// the meta-layer operates on raw class text, not complete Java source files.

use crate::compression::Fidelity;
use crate::spring_meta::annotations::extract_annotations;

// ── RestController E2E ─────────────────────────────

#[test]
fn spring_rest_controller_with_mappings_e2e() {
    let raw_class = r#"@RestController
@RequestMapping("/api/users")
public class UserController {

    @GetMapping
    public List<String> getUsers() {
        return new ArrayList<>();
    }

    @PostMapping
    public String createUser() {
        return "created";
    }
}"#;

    let result = extract_annotations(raw_class, Fidelity::Medium);
    assert!(
        result.is_some(),
        "should detect RestController with mappings"
    );
    let lines = &result.unwrap().lines;

    assert!(
        lines.iter().any(|l| l.starts_with("Φrest:UserController")),
        "should contain Φrest marker: {:?}",
        lines
    );
    assert!(
        lines.iter().any(|l| l.starts_with("Φmap:")),
        "should contain Φmap markers: {:?}",
        lines
    );
    assert!(
        !lines.iter().any(|l| l.starts_with("SP_REST_")),
        "should not contain old SP_ prefix"
    );
}

#[test]
fn spring_rest_controller_no_mappings_e2e() {
    let raw_class = r#"@RestController
public class HealthController {

    public String health() {
        return "OK";
    }
}"#;

    let result = extract_annotations(raw_class, Fidelity::Medium);
    assert!(
        result.is_some(),
        "should detect RestController without mappings"
    );
    let lines = &result.unwrap().lines;

    assert!(
        lines
            .iter()
            .any(|l| l.starts_with("Φrest:HealthController")),
        "should contain Φrest marker: {:?}",
        lines
    );
    assert!(
        !lines.iter().any(|l| l.starts_with("SP_REST_")),
        "should not contain old SP_ prefix"
    );
}

// ── Service E2E ────────────────────────────────────

#[test]
fn spring_service_e2e() {
    let raw_class = r#"@Service
public class UserService {

    public List<String> findAll() {
        return new ArrayList<>();
    }
}"#;

    let result = extract_annotations(raw_class, Fidelity::Medium);
    assert!(result.is_some(), "should detect @Service");
    let lines = &result.unwrap().lines;

    assert!(
        lines.iter().any(|l| l.starts_with("Φsvc:UserService")),
        "should contain Φsvc marker: {:?}",
        lines
    );
    assert!(
        !lines.iter().any(|l| l.starts_with("SP_SERVICE_")),
        "should not contain old SP_ prefix"
    );
}

// ── Repository E2E ─────────────────────────────────

#[test]
fn spring_repository_e2e() {
    let raw_class = r#"@Repository
public class UserRepository {

    public String findById(long id) {
        return "user";
    }
}"#;

    let result = extract_annotations(raw_class, Fidelity::Medium);
    assert!(result.is_some(), "should detect @Repository");
    let lines = &result.unwrap().lines;

    assert!(
        lines.iter().any(|l| l.starts_with("Φrepo:UserRepository")),
        "should contain Φrepo marker: {:?}",
        lines
    );
    assert!(
        !lines.iter().any(|l| l.starts_with("SP_REPOSITORY_")),
        "should not contain old SP_ prefix"
    );
}

// ── Configuration E2E ──────────────────────────────

#[test]
fn spring_configuration_with_bean_e2e() {
    let raw_class = r#"@Configuration
public class AppConfig {

    @Bean
    public String dataSource() {
        return "datasource";
    }
}"#;

    let result = extract_annotations(raw_class, Fidelity::Medium);
    assert!(result.is_some(), "should detect @Configuration");
    let lines = &result.unwrap().lines;

    assert!(
        lines.iter().any(|l| l.starts_with("Φconf:AppConfig")),
        "should contain Φconf marker: {:?}",
        lines
    );
    assert!(
        lines.iter().any(|l| l.starts_with("Φbean:dataSource")),
        "should contain Φbean marker: {:?}",
        lines
    );
}

// ── @Value Injection E2E ───────────────────────────

#[test]
fn spring_value_annotation_e2e() {
    let raw_class = r#"@Service
public class ConfigService {

    @Value("app.name")
    private String appName;

    public String getAppName() {
        return appName;
    }
}"#;

    let result = extract_annotations(raw_class, Fidelity::Medium);
    assert!(result.is_some(), "should detect @Service with @Value");
    let lines = &result.unwrap().lines;

    assert!(
        lines.iter().any(|l| l.starts_with("Φsvc:ConfigService")),
        "should contain Φsvc marker: {:?}",
        lines
    );
}

// ── Non-Spring Java (negative test) ────────────────

#[test]
fn plain_java_no_spring_markers_e2e() {
    let raw_class = r#"public class MathUtils {

    public static int add(int a, int b) {
        return a + b;
    }

    public static int multiply(int a, int b) {
        return a * b;
    }
}"#;

    let result = extract_annotations(raw_class, Fidelity::Medium);
    assert!(
        result.is_none(),
        "plain Java should not produce Spring annotations"
    );
}

// ── No old format regression ──────────────────────

#[test]
fn spring_e2e_no_old_format_markers() {
    let raw_class = r#"@RestController
public class UserController {
    public String get() { return "ok"; }
}"#;

    let result = extract_annotations(raw_class, Fidelity::Medium);
    assert!(result.is_some(), "should detect @RestController");
    let lines = &result.unwrap().lines;

    // Verify no old format markers
    let old_prefixes = [
        "SP_REST_",
        "SP_SERVICE_",
        "SP_REPOSITORY_",
        "SP_CONFIGURATION_",
        "SP_CONTROLLER_",
        "SP_BEAN_",
        "SP_COMPONENT_",
    ];
    for old in &old_prefixes {
        assert!(
            !lines.iter().any(|l| l.starts_with(old)),
            "should not contain old format {}: {:?}",
            old,
            lines
        );
    }

    // Verify Φ abbreviations are present
    assert!(
        lines.iter().any(|l| l.starts_with("Φrest:UserController")),
        "should contain Φrest: {:?}",
        lines
    );
}

// ── All Spring annotation types E2E ────────────────

#[test]
fn spring_all_annotation_types_e2e() {
    // Test each Spring stereotype annotation produces the correct Φ marker
    let test_cases = vec![
        (
            "@RestController\npublic class TestCtrl {}",
            "Φrest:TestCtrl",
        ),
        ("@Controller\npublic class TestCtrl {}", "Φctrl:TestCtrl"),
        ("@Service\npublic class TestService {}", "Φsvc:TestService"),
        ("@Repository\npublic class TestRepo {}", "Φrepo:TestRepo"),
        (
            "@Configuration\npublic class TestConfig {}",
            "Φconf:TestConfig",
        ),
    ];

    for (raw_class, expected_prefix) in test_cases {
        let result = extract_annotations(raw_class, Fidelity::Medium);
        assert!(
            result.is_some(),
            "should detect annotation in: {}",
            raw_class
        );
        let lines = &result.unwrap().lines;
        assert!(
            lines.iter().any(|l| l.starts_with(expected_prefix)),
            "expected {} marker in: {:?}",
            expected_prefix,
            lines
        );
    }
}

// ── Multiple annotations per class E2E ─────────────

#[test]
fn spring_multiple_annotations_per_class_e2e() {
    let raw_class = r#"@Service
@Transactional
public class UserService {

    public List<String> findAll() {
        return new ArrayList<>();
    }
}"#;

    let result = extract_annotations(raw_class, Fidelity::Medium);
    assert!(
        result.is_some(),
        "should detect @Service with @Transactional"
    );
    let lines = &result.unwrap().lines;

    assert!(
        lines.iter().any(|l| l.starts_with("Φsvc:UserService")),
        "should contain Φsvc marker: {:?}",
        lines
    );
}
