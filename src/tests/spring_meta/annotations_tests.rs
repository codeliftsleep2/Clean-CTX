// src/tests/spring_meta/annotations_tests.rs
// Tests for Spring Boot annotation extraction.

use crate::spring_meta::annotations::extract_annotations;
use crate::spring_meta::markers::PhiLineKind;

#[test]
fn test_rest_controller_extraction() {
    let raw_class = r#"@RestController
@RequestMapping("/api")
public class UserController {
    private UserService userService;
}"#;
    let result = extract_annotations(raw_class, crate::compression::Fidelity::Low);
    assert!(result.is_some());
    let lines = result.unwrap().lines;
    assert!(lines.iter().any(|l| l.starts_with("Φrest:")));
    assert!(lines.iter().any(|l| l.starts_with("Φmap:")));
}

#[test]
fn test_service_extraction() {
    let raw_class = r#"@Service
public class UserService {
    public List<User> findAll() {
        return userRepository.findAll();
    }
}"#;
    let result = extract_annotations(raw_class, crate::compression::Fidelity::Low);
    assert!(result.is_some());
    let lines = result.unwrap().lines;
    assert!(lines.iter().any(|l| l.starts_with("Φsvc:")));
}

#[test]
fn test_repository_extraction() {
    let raw_class = r#"@Repository
public class UserRepository {
    public List<User> findAll() {
        return jdbcTemplate.query("SELECT * FROM users");
    }
}"#;
    let result = extract_annotations(raw_class, crate::compression::Fidelity::Low);
    assert!(result.is_some());
    let lines = result.unwrap().lines;
    assert!(lines.iter().any(|l| l.starts_with("Φrepo:")));
}

#[test]
fn test_configuration_extraction() {
    let raw_class = r#"@Configuration
public class AppConfig {
    @Bean
    public UserService userService() {
        return new UserServiceImpl();
    }
}"#;
    let result = extract_annotations(raw_class, crate::compression::Fidelity::Low);
    assert!(result.is_some());
    let lines = result.unwrap().lines;
    assert!(lines.iter().any(|l| l.starts_with("Φconf:")));
}

#[test]
fn test_get_mapping_extraction() {
    let raw_class = r#"@RestController
public class UserController {
    @GetMapping("/users")
    public List<User> getUsers() {
        return userService.findAll();
    }
}"#;
    let result = extract_annotations(raw_class, crate::compression::Fidelity::Medium);
    assert!(result.is_some());
    let lines = result.unwrap().lines;
    assert!(lines.iter().any(|l| l.contains("GET /users")));
}

#[test]
fn test_high_fidelity_autowired() {
    let raw_class = r#"@Service
public class UserService {
    @Autowired
    private UserRepository userRepository;
    
    @Value("${app.name}")
    private String appName;
}"#;
    let result = extract_annotations(raw_class, crate::compression::Fidelity::High);
    assert!(result.is_some());
    let lines = result.unwrap().lines;
    assert!(lines.iter().any(|l| l.starts_with("Φaut:")));
    assert!(lines.iter().any(|l| l.starts_with("Φval:")));
}

#[test]
fn test_low_fidelity_no_autowired() {
    let raw_class = r#"@Service
public class UserService {
    @Autowired
    private UserRepository userRepository;
}"#;
    let result = extract_annotations(raw_class, crate::compression::Fidelity::Low);
    assert!(result.is_some());
    let lines = result.unwrap().lines;
    // Low fidelity should not include field-level @Autowired
    assert!(!lines.iter().any(|l| l.starts_with("Φaut:")));
}

#[test]
fn test_non_spring_class() {
    let raw_class = r#"public class PlainClass {
    private String name;
}"#;
    let result = extract_annotations(raw_class, crate::compression::Fidelity::Low);
    assert!(result.is_none());
}

#[test]
fn test_bean_method_extraction() {
    let raw_class = r#"@Configuration
public class AppConfig {
    @Bean
    public UserService userService() {
        return new UserServiceImpl();
    }
    
    @Bean
    public UserRepository userRepository() {
        return new UserRepositoryImpl();
    }
}"#;
    let result = extract_annotations(raw_class, crate::compression::Fidelity::High);
    assert!(result.is_some());
    let lines = result.unwrap().lines;
    assert!(lines.iter().any(|l| l.starts_with("Φbean:userService")));
    assert!(lines.iter().any(|l| l.starts_with("Φbean:userRepository")));
}

#[test]
fn test_configuration_properties_extraction() {
    let raw_class = r#"@ConfigurationProperties(prefix = "app")
public class AppProperties {
    private String name;
    private String version;
}"#;
    let result = extract_annotations(raw_class, crate::compression::Fidelity::High);
    assert!(result.is_some());
    let lines = result.unwrap().lines;
    assert!(lines.iter().any(|l| l.starts_with("Φprop:")));
}